#!/usr/bin/env python3
"""
Rust From Zero markdown viewer.

Auto-discovers every *.md file in the project (excluding target/, .git/,
node_modules/ and this viewer/ directory) and serves them to the web UI.

The file list is re-scanned on EVERY /api/files request, so documents added
or modified later show up automatically — no restart needed.

Usage:  python3 viewer/serve.py [port]     (default port 8000, binds 127.0.0.1)

Environment variables (the old LEARN_RUST_* names still work as aliases):
  RUST_COURSE_AGENT       agent CLI command to launch in the side panel
                          (default: auto-detect, Kimi preferred)
  RUST_COURSE_HOST        bind host (default: 127.0.0.1 — loopback only;
                          set 0.0.0.0 to expose the UI on the LAN, which is
                          unsafe: there is no authentication)
  RUST_COURSE_VIEW_STATE  path of the JSON file the MCP server
                          (viewer/mcp_server.py) reads; defaults to a
                          per-user, per-project path in the temp dir
"""

import fcntl
import hashlib
import json
import os
import pty
import re
import select
import shlex
import shutil
import signal
import socket
import struct
import sys
import tempfile
import termios
import threading
import time
import uuid
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, unquote, urlparse

ROOT = Path(__file__).resolve().parent.parent  # project root
STATIC = Path(__file__).resolve().parent / "static"

EXCLUDE_DIRS = {"target", ".git", "node_modules", "viewer"}
GROUP_ORDER = {"lessons": 0, "notes": 1, "docs": 2}  # everything else after, root last
GROUP_LABELS = {"lessons": "Lessons", "notes": "Notes", "docs": "Docs", "": "Project"}

MIME = {
    ".html": "text/html; charset=utf-8",
    ".js": "text/javascript; charset=utf-8",
    ".css": "text/css; charset=utf-8",
    ".svg": "image/svg+xml",
    ".png": "image/png",
    ".md": "text/markdown; charset=utf-8",
    ".json": "application/json; charset=utf-8",
}

H1_RE = re.compile(r"^#\s+(.+?)\s*#*\s*$", re.M)


def _env(primary, legacy, default=""):
    """Read the RUST_COURSE_* variable, falling back to the LEARN_RUST_* alias."""
    return os.environ.get(primary) or os.environ.get(legacy) or default


AGENT_COMMAND = _env("RUST_COURSE_AGENT", "LEARN_RUST_AGENT")
SERVER_HOST = _env("RUST_COURSE_HOST", "LEARN_RUST_HOST", "127.0.0.1")
MAX_AGENT_BUFFER = 2 * 1024 * 1024

# Insertion order is the preference order: the first installed CLI becomes
# the default agent. Kimi reads this repo's .kimi-code/mcp.json and the
# rust-course-mentor agent profile, so it needs no extra setup here.
BUILTIN_AGENTS = {
    "kimi": {"label": "Kimi", "command": "kimi --agent rust-course-mentor"},
    "codex": {"label": "Codex", "command": "codex"},
    "claude": {"label": "Claude", "command": "claude"},
    "grok": {"label": "Grok", "command": "grok"},
    "omp": {"label": "OMP", "command": "omp"},
}

NO_AGENT_HINT = (
    "no supported agent CLI was found on PATH. Install one (Kimi Code is the "
    "zero-config option for this course: https://www.kimi.com/code), log in, "
    "then restart the viewer. Setup guide: docs/agent-integration.md"
)

LOOPBACK_HOSTS = {"127.0.0.1", "localhost", "::1"}


def available_agents():
    agents = {}
    for agent_id, config in BUILTIN_AGENTS.items():
        executable = shlex.split(config["command"])[0]
        if shutil.which(executable):
            agents[agent_id] = config
    if AGENT_COMMAND:
        argv = shlex.split(AGENT_COMMAND)
        already_listed = AGENT_COMMAND in {item["command"] for item in agents.values()}
        if argv and shutil.which(argv[0]) and not already_listed:
            agents["configured"] = {"label": "Custom", "command": AGENT_COMMAND}
    return agents


AVAILABLE_AGENTS = available_agents()


def default_agent_id():
    if "configured" in AVAILABLE_AGENTS:
        return "configured"
    return next(iter(AVAILABLE_AGENTS), None)


def default_view_state_path():
    """Per-user, per-project default so viewers of different checkouts never
    share state. Keep in sync with viewer/mcp_server.py."""
    digest = hashlib.sha1(str(ROOT).encode("utf-8")).hexdigest()[:8]
    return Path(tempfile.gettempdir()) / f"rust-course-viewer-state-{os.getuid()}-{digest}.json"


# Latest view state reported by the web UI, also mirrored to a JSON file so the
# MCP server (viewer/mcp_server.py) can expose it to agents as a tool.
VIEW_STATE_PATH = Path(
    _env("RUST_COURSE_VIEW_STATE", "LEARN_RUST_VIEW_STATE") or default_view_state_path()
)


def _atomic_write(path: Path, text: str):
    """Write text to path atomically with 0600 permissions (temp files live in
    a shared /tmp, so keep them private to this user)."""
    fd, tmp_name = tempfile.mkstemp(dir=str(path.parent), prefix=path.name + ".")
    try:
        with os.fdopen(fd, "wb") as handle:
            os.fchmod(handle.fileno(), 0o600)
            handle.write(text.encode("utf-8"))
        os.replace(tmp_name, path)
    except BaseException:
        try:
            os.unlink(tmp_name)
        except OSError:
            pass
        raise


class AgentSession:
    """One interactive agent process connected through a Unix PTY."""

    def __init__(self, command: str, cols=100, rows=30, current_document=None):
        self.id = uuid.uuid4().hex
        self.command = command
        self.context_path = (
            Path(tempfile.gettempdir()) / f"rust-course-agent-{os.getuid()}-{self.id}.json"
        )
        self.current_document = current_document
        self.cols = cols
        self.rows = rows
        self.master_fd = None
        self.pid = None
        self.output = bytearray()
        self.base_offset = 0
        self.exited = False
        self.exit_code = None
        self.lock = threading.Condition()
        self.update_context(current_document)
        self._spawn()

    def _spawn(self):
        argv = shlex.split(self.command)
        if not argv:
            raise ValueError("agent command is empty")
        if Path(argv[0]).name == "grok":
            rules = (
                f"The Rust course workspace is {ROOT}. All course chapters are under "
                f"{ROOT / 'lessons'} and you may read any of them. Before answering each "
                f"user message, read {self.context_path}; its currentDocument field is the "
                "lesson or project document currently visible in the Web UI. Use that document "
                "as the immediate teaching context when it is not null."
            )
            argv.extend(["--rules", rules])
        pid, master_fd = pty.fork()
        if pid == 0:
            os.chdir(ROOT)
            env = os.environ.copy()
            env.update(
                {
                    "TERM": "xterm-256color",
                    "COLORTERM": "truecolor",
                    "RUST_COURSE_WORKSPACE": str(ROOT),
                    # Agents that launch viewer/mcp_server.py (e.g. Kimi via
                    # .kimi-code/mcp.json) inherit this and read the same
                    # state file this server writes.
                    "RUST_COURSE_VIEW_STATE": str(VIEW_STATE_PATH),
                    # Generic hook: per-session JSON with the document open in
                    # the UI, for agents without MCP support.
                    "RUST_COURSE_AGENT_CONTEXT": str(self.context_path),
                }
            )
            try:
                os.execvpe(argv[0], argv, env)
            except OSError as exc:
                print(f"Unable to start {argv[0]}: {exc}", file=sys.stderr)
                os._exit(127)
        self.pid = pid
        self.master_fd = master_fd
        self.resize(self.cols, self.rows)
        threading.Thread(target=self._read_loop, daemon=True).start()

    def _read_loop(self):
        while True:
            try:
                ready, _, _ = select.select([self.master_fd], [], [], 0.25)
                if not ready:
                    waited, status = os.waitpid(self.pid, os.WNOHANG)
                    if not waited:
                        continue
                    self._mark_exited(os.waitstatus_to_exitcode(status))
                    return
                chunk = os.read(self.master_fd, 65536)
                if not chunk:
                    break
                with self.lock:
                    self.output.extend(chunk)
                    if len(self.output) > MAX_AGENT_BUFFER:
                        trim = len(self.output) - MAX_AGENT_BUFFER
                        del self.output[:trim]
                        self.base_offset += trim
                    self.lock.notify_all()
            except OSError:
                break
        try:
            _, status = os.waitpid(self.pid, 0)
            code = os.waitstatus_to_exitcode(status)
        except ChildProcessError:
            code = self.exit_code
        self._mark_exited(code)

    def _mark_exited(self, code):
        with self.lock:
            self.exited = True
            self.exit_code = code
            self.lock.notify_all()
        if self.master_fd is not None:
            try:
                os.close(self.master_fd)
            except OSError:
                pass
            self.master_fd = None

    def read(self, offset, timeout=1.0):
        deadline = time.monotonic() + timeout
        with self.lock:
            while offset >= self.base_offset + len(self.output) and not self.exited:
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    break
                self.lock.wait(remaining)
            start = max(offset - self.base_offset, 0)
            data = bytes(self.output[start:])
            return data, self.base_offset + len(self.output)

    def write(self, data):
        if self.exited or self.master_fd is None:
            raise BrokenPipeError("agent session has exited")
        os.write(self.master_fd, data.encode("utf-8"))

    def resize(self, cols, rows):
        self.cols = max(20, min(int(cols), 400))
        self.rows = max(5, min(int(rows), 200))
        if self.master_fd is not None:
            size = struct.pack("HHHH", self.rows, self.cols, 0, 0)
            fcntl.ioctl(self.master_fd, termios.TIOCSWINSZ, size)

    def update_context(self, current_document):
        self.current_document = current_document
        payload = {
            "currentDocument": current_document,
            "lessonsDirectory": str(ROOT / "lessons"),
            "workspace": str(ROOT),
        }
        _atomic_write(self.context_path, json.dumps(payload, ensure_ascii=False, indent=2) + "\n")

    def close(self):
        """Terminate the agent (escalating if it ignores SIGHUP), reap it and
        remove the per-session context file."""
        if self.pid:
            if not self.exited:
                for sig in (signal.SIGHUP, signal.SIGTERM, signal.SIGKILL):
                    try:
                        os.kill(self.pid, sig)
                    except (ProcessLookupError, PermissionError):
                        break
                    deadline = time.monotonic() + 0.8
                    while not self.exited and time.monotonic() < deadline:
                        time.sleep(0.05)
                    if self.exited:
                        break
            try:
                os.waitpid(self.pid, os.WNOHANG)
            except (ChildProcessError, OSError):
                pass  # the reader thread already reaped it
        try:
            self.context_path.unlink()
        except FileNotFoundError:
            pass


class AgentSessions:
    def __init__(self):
        self.sessions = {}
        self.lock = threading.Lock()

    def create(self, command, cols=100, rows=30, current_document=None):
        session = AgentSession(command, cols, rows, current_document)
        with self.lock:
            self.sessions[session.id] = session
        return session

    def get(self, session_id):
        with self.lock:
            return self.sessions.get(session_id)

    def remove(self, session_id):
        with self.lock:
            session = self.sessions.pop(session_id, None)
        if session:
            session.close()
        return session

    def close_all(self):
        with self.lock:
            sessions = list(self.sessions.values())
            self.sessions.clear()
        for session in sessions:
            session.close()


AGENTS = AgentSessions()


def update_view_state(document, heading, scroll_percent):
    payload = {
        "currentDocument": document,
        "currentHeading": heading,
        "scrollPercent": scroll_percent,
        "workspace": str(ROOT),
        "lessonsDirectory": str(ROOT / "lessons"),
        "updatedAt": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
        "writtenAtEpoch": time.time(),
    }
    try:
        _atomic_write(
            VIEW_STATE_PATH, json.dumps(payload, ensure_ascii=False, indent=2) + "\n"
        )
    except OSError:
        pass
    return payload


def read_title(path: Path):
    """First H1 heading of the file, used as the display subtitle."""
    try:
        head = path.read_text(encoding="utf-8", errors="replace")[:8192]
    except OSError:
        return None
    m = H1_RE.search(head)
    return m.group(1).strip() if m else None


def iter_markdown():
    """Walk the project, pruning excluded/hidden dirs, yielding *.md paths."""
    for dirpath, dirnames, filenames in os.walk(ROOT):
        dirnames[:] = sorted(
            d for d in dirnames if d not in EXCLUDE_DIRS and not d.startswith(".")
        )
        for name in sorted(filenames):
            if name.lower().endswith(".md"):
                yield Path(dirpath, name)


def discover():
    """Group all markdown files by their top-level directory."""
    groups = {}
    for path in iter_markdown():
        rel = path.relative_to(ROOT)
        top = rel.parts[0] if len(rel.parts) > 1 else ""
        stat = path.stat()
        groups.setdefault(top, []).append(
            {
                "path": rel.as_posix(),
                "name": rel.stem,
                "title": read_title(path) or rel.stem,
                "mtime": stat.st_mtime,
                "size": stat.st_size,
            }
        )

    ordered = sorted(groups, key=lambda g: (GROUP_ORDER.get(g, 50), g))
    if "" in ordered:  # project-root files go last
        ordered.remove("")
        ordered.append("")

    return {
        "groups": [
            {
                "key": key or "root",
                "label": GROUP_LABELS.get(key, key.capitalize()),
                "files": sorted(groups[key], key=lambda f: f["path"]),
            }
            for key in ordered
        ],
        "total": sum(len(v) for v in groups.values()),
    }


class Handler(BaseHTTPRequestHandler):
    server_version = "RustCourseViewer/1.0"
    protocol_version = "HTTP/1.1"

    def log_message(self, fmt, *args):  # keep the log short
        sys.stderr.write("  %s\n" % (fmt % args))

    # ---------- helpers ----------

    def _send(self, code, body, ctype="text/plain; charset=utf-8", cache=False):
        if isinstance(body, str):
            body = body.encode("utf-8")
        self.send_response(code)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "max-age=3600" if cache else "no-store")
        self.end_headers()
        self.wfile.write(body)

    def _json(self, obj, code=200):
        self._send(code, json.dumps(obj, ensure_ascii=False), MIME[".json"])

    def _read_json(self):
        try:
            length = int(self.headers.get("Content-Length", "0"))
            if length > 1024 * 1024:
                raise ValueError("request body is too large")
            return json.loads(self.rfile.read(length) or b"{}")
        except (ValueError, json.JSONDecodeError) as exc:
            self._json({"error": str(exc)}, 400)
            return None

    def _agent(self, session_id):
        session = AGENTS.get(session_id)
        if session is None:
            self._json({"error": "agent session not found"}, 404)
        return session

    def _document(self, value):
        if value is None:
            return None
        if not isinstance(value, str) or not value.lower().endswith(".md"):
            raise ValueError("document must be a project Markdown path or null")
        target = (ROOT / value).resolve()
        if not target.is_relative_to(ROOT) or not target.is_file():
            raise ValueError("document does not exist in this project")
        return target.relative_to(ROOT).as_posix()

    def _serve_path(self, base: Path, rel: str, cache=False):
        target = (base / rel).resolve()
        if not target.is_relative_to(base) or not target.is_file():
            return self._send(404, "not found")
        ctype = MIME.get(target.suffix.lower(), "application/octet-stream")
        self._send(200, target.read_bytes(), ctype, cache=cache)

    # ---------- routes ----------

    def do_GET(self):
        url = urlparse(self.path)
        route = unquote(url.path)

        if route == "/":
            return self._serve_path(STATIC, "index.html")
        if route == "/api/files":  # re-scans the disk on every call
            return self._json(discover())
        if route == "/api/agents":
            return self._json(
                {
                    "agents": [
                        {"id": key, "label": value["label"]}
                        for key, value in AVAILABLE_AGENTS.items()
                    ],
                    "default": default_agent_id(),
                    "guide": "docs/agent-integration.md",
                    "hint": None if AVAILABLE_AGENTS else NO_AGENT_HINT,
                }
            )
        if route == "/api/file":
            rel = parse_qs(url.query).get("path", [""])[0]
            if not rel:
                return self._json({"error": "missing ?path="}, 400)
            if not rel.lower().endswith(".md"):
                return self._json({"error": "only .md files are served"}, 403)
            return self._serve_path(ROOT, rel)
        if route == "/api/agent/output":
            query = parse_qs(url.query)
            session = self._agent(query.get("id", [""])[0])
            if session is None:
                return
            try:
                offset = max(0, int(query.get("offset", ["0"])[0]))
            except ValueError:
                return self._json({"error": "invalid offset"}, 400)
            data, next_offset = session.read(offset)
            return self._json(
                {
                    "data": data.decode("latin1"),
                    "offset": next_offset,
                    "baseOffset": session.base_offset,
                    "exited": session.exited,
                    "exitCode": session.exit_code,
                }
            )
        if route == "/favicon.ico":
            return self._send(204, b"")
        return self._serve_path(STATIC, route.lstrip("/"), cache=True)

    def do_POST(self):
        route = unquote(urlparse(self.path).path)
        if not self.headers.get("Content-Type", "").startswith("application/json"):
            return self._json({"error": "Content-Type must be application/json"}, 415)
        body = self._read_json()
        if body is None:
            return
        if route == "/api/view":
            try:
                document = self._document(body.get("document"))
                heading = body.get("heading")
                if heading is not None and not isinstance(heading, str):
                    raise ValueError("heading must be a string or null")
                scroll = body.get("scrollPercent")
                scroll = max(0, min(100, round(float(scroll)))) if scroll is not None else None
            except (TypeError, ValueError) as exc:
                return self._json({"error": str(exc)}, 400)
            return self._json({"ok": True, "view": update_view_state(document, heading, scroll)})
        if route == "/api/agent/session":
            try:
                agent_id = body.get("agent") or default_agent_id()
                config = AVAILABLE_AGENTS.get(agent_id)
                if config is None:
                    if not AVAILABLE_AGENTS:
                        raise ValueError(NO_AGENT_HINT)
                    raise ValueError(
                        f"agent {agent_id!r} is not available on this machine; "
                        "pick one of: " + ", ".join(sorted(AVAILABLE_AGENTS))
                    )
                document = self._document(body.get("document"))
                session = AGENTS.create(
                    config["command"], body.get("cols", 100), body.get("rows", 30), document
                )
            except ValueError as exc:
                return self._json({"error": str(exc)}, 400)
            except OSError as exc:
                return self._json({"error": f"failed to start agent: {exc}"}, 500)
            return self._json(
                {"id": session.id, "agent": agent_id, "command": session.command}, 201
            )
        session = self._agent(body.get("id", ""))
        if session is None:
            return
        if route == "/api/agent/input":
            data = body.get("data")
            if not isinstance(data, str):
                return self._json({"error": "data must be a string"}, 400)
            try:
                session.write(data)
            except (BrokenPipeError, OSError) as exc:
                return self._json({"error": str(exc)}, 409)
            return self._json({"ok": True})
        if route == "/api/agent/resize":
            try:
                session.resize(body["cols"], body["rows"])
            except (KeyError, TypeError, ValueError):
                return self._json({"error": "cols and rows must be integers"}, 400)
            return self._json({"ok": True, "cols": session.cols, "rows": session.rows})
        if route == "/api/agent/context":
            try:
                document = self._document(body.get("document"))
                session.update_context(document)
            except (OSError, ValueError) as exc:
                return self._json({"error": str(exc)}, 400)
            return self._json({"ok": True, "document": document})
        return self._json({"error": "not found"}, 404)

    def do_DELETE(self):
        url = urlparse(self.path)
        if unquote(url.path) != "/api/agent/session":
            return self._json({"error": "not found"}, 404)
        session_id = parse_qs(url.query).get("id", [""])[0]
        if not AGENTS.remove(session_id):
            return self._json({"error": "agent session not found"}, 404)
        return self._json({"ok": True})


def lan_ip():
    try:
        s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        s.connect(("8.8.8.8", 80))  # UDP: no traffic is actually sent
        ip = s.getsockname()[0]
        s.close()
        return ip
    except OSError:
        return "127.0.0.1"


def main():
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8000
    server = ThreadingHTTPServer((SERVER_HOST, port), Handler)
    print("Rust From Zero markdown viewer")
    print(f"  local:   http://localhost:{port}")
    if SERVER_HOST not in LOOPBACK_HOSTS:
        print(f"  lan/wsl: http://{lan_ip()}:{port}")
        print("  WARNING: no authentication — anyone who can reach this port")
        print("           can read project files and drive agent sessions.")
    print(f"  root:    {ROOT}")
    default_id = default_agent_id()
    if default_id is None:
        print(f"  agent:   none detected — {NO_AGENT_HINT}")
    else:
        print(f"  agent:   {AVAILABLE_AGENTS[default_id]['command']} (default)")
    print("watching for *.md changes — no restart needed. Ctrl+C to stop.")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nstopped.")
    finally:
        AGENTS.close_all()
        server.server_close()


if __name__ == "__main__":
    main()
