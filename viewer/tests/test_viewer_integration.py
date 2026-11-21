#!/usr/bin/env python3
"""Stdlib-only tests for the Rust From Zero viewer agent integration.

Covers:
- safe default bind address and env-var branding (RUST_COURSE_*, legacy aliases)
- agent detection, preference order and custom commands (fake executables)
- MCP current-section reading and rejection of illegal paths
- MCP stdio protocol (initialize / tools/list / tools/call) without any model
- agent session lifecycle with a fake executable (no real agent CLI needed)
- HTTP API behavior with no agent and with a fake agent

Run:  python3 -m unittest discover -s viewer/tests -v
"""

import importlib
import json
import os
import socket
import stat
import subprocess
import sys
import tempfile
import time
import unittest
import urllib.error
import urllib.request
from pathlib import Path

VIEWER_DIR = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(VIEWER_DIR))

ENV_KEYS = [
    "RUST_COURSE_AGENT",
    "LEARN_RUST_AGENT",
    "RUST_COURSE_HOST",
    "LEARN_RUST_HOST",
    "RUST_COURSE_VIEW_STATE",
    "LEARN_RUST_VIEW_STATE",
    "PATH",
]


def fresh_import(module_name, **env):
    """Import viewer/<module_name>.py with a controlled environment."""
    saved = {key: os.environ.get(key) for key in ENV_KEYS}
    for key in ENV_KEYS:
        os.environ.pop(key, None)
    os.environ.update(env)
    try:
        sys.modules.pop(module_name, None)
        return importlib.import_module(module_name)
    finally:
        for key, value in saved.items():
            if value is None:
                os.environ.pop(key, None)
            else:
                os.environ[key] = value


def make_bin(directory, *names):
    """Create trivially-executable fake CLIs and return the PATH string."""
    for name in names:
        path = directory / name
        path.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
        path.chmod(0o755)
    return str(directory)


class ConfigTests(unittest.TestCase):
    def test_default_host_is_loopback(self):
        serve = fresh_import("serve")
        self.assertEqual(serve.SERVER_HOST, "127.0.0.1")

    def test_host_env_override_and_legacy_alias(self):
        serve = fresh_import("serve", LEARN_RUST_HOST="0.0.0.0")
        self.assertEqual(serve.SERVER_HOST, "0.0.0.0")
        serve = fresh_import(
            "serve", LEARN_RUST_HOST="0.0.0.0", RUST_COURSE_HOST="192.168.1.5"
        )
        self.assertEqual(serve.SERVER_HOST, "192.168.1.5")  # new name wins

    def test_default_view_state_path_is_per_user_per_project(self):
        serve = fresh_import("serve")
        name = serve.VIEW_STATE_PATH.name
        self.assertTrue(name.startswith(f"rust-course-viewer-state-{os.getuid()}-"), name)
        mcp = fresh_import("mcp_server")
        self.assertEqual(mcp.STATE_PATH, serve.VIEW_STATE_PATH)  # both sides agree

    def test_view_state_env_override_and_legacy_alias(self):
        serve = fresh_import("serve", RUST_COURSE_VIEW_STATE="/tmp/custom-state.json")
        self.assertEqual(str(serve.VIEW_STATE_PATH), "/tmp/custom-state.json")
        serve = fresh_import("serve", LEARN_RUST_VIEW_STATE="/tmp/legacy-state.json")
        self.assertEqual(str(serve.VIEW_STATE_PATH), "/tmp/legacy-state.json")


class AgentDetectionTests(unittest.TestCase):
    def test_kimi_is_preferred_when_installed(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = make_bin(Path(tmp), "kimi", "codex", "claude")
            serve = fresh_import("serve", PATH=path)
            self.assertEqual(set(serve.AVAILABLE_AGENTS), {"kimi", "codex", "claude"})
            self.assertEqual(serve.default_agent_id(), "kimi")

    def test_falls_back_to_next_available_agent(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = make_bin(Path(tmp), "codex")
            serve = fresh_import("serve", PATH=path)
            self.assertEqual(serve.default_agent_id(), "codex")

    def test_no_agents_detected(self):
        with tempfile.TemporaryDirectory() as tmp:
            serve = fresh_import("serve", PATH=tmp)
            self.assertEqual(serve.AVAILABLE_AGENTS, {})
            self.assertIsNone(serve.default_agent_id())

    def test_custom_command_wins(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = make_bin(Path(tmp), "kimi", "mycli")
            serve = fresh_import(
                "serve", PATH=path, RUST_COURSE_AGENT="mycli --fast --model x"
            )
            self.assertIn("configured", serve.AVAILABLE_AGENTS)
            self.assertEqual(
                serve.AVAILABLE_AGENTS["configured"]["command"], "mycli --fast --model x"
            )
            self.assertEqual(serve.default_agent_id(), "configured")

    def test_custom_command_with_missing_executable_is_ignored(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = make_bin(Path(tmp), "kimi")
            serve = fresh_import("serve", PATH=path, RUST_COURSE_AGENT="no-such-cli --go")
            self.assertNotIn("configured", serve.AVAILABLE_AGENTS)
            self.assertEqual(serve.default_agent_id(), "kimi")


class McpToolTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.workspace = Path(self.tmp.name) / "repo"
        lesson = self.workspace / "lessons" / "01-basics"
        lesson.mkdir(parents=True)
        (lesson / "README.md").write_text(
            "# 第一课\n\n导入。\n\n## 安装工具\n\n安装这一节的内容。\n\n"
            "## 第一个程序\n\n程序这一节的内容。\n",
            encoding="utf-8",
        )
        self.state_path = Path(self.tmp.name) / "state.json"

    def write_state(self, **overrides):
        payload = {
            "currentDocument": "lessons/01-basics/README.md",
            "currentHeading": "安装工具",
            "scrollPercent": 40,
            "workspace": str(self.workspace),
            "writtenAtEpoch": time.time(),
        }
        payload.update(overrides)
        self.state_path.write_text(json.dumps(payload), encoding="utf-8")
        return fresh_import("mcp_server", RUST_COURSE_VIEW_STATE=str(self.state_path))

    def test_get_current_view_reports_state(self):
        mcp = self.write_state()
        result = mcp.tool_get_current_view()
        text = result["content"][0]["text"]
        self.assertIn("lessons/01-basics/README.md", text)
        self.assertIn("安装工具", text)

    def test_get_current_section_stops_at_next_same_level_heading(self):
        mcp = self.write_state()
        result = mcp.tool_get_current_section({})
        text = result["content"][0]["text"]
        self.assertIn("## 安装工具", text)
        self.assertIn("安装这一节的内容。", text)
        self.assertNotIn("第一个程序", text)

    def test_missing_state_file_is_explained(self):
        mcp = fresh_import(
            "mcp_server", RUST_COURSE_VIEW_STATE=str(self.state_path)  # never written
        )
        text = mcp.tool_get_current_view()["content"][0]["text"]
        self.assertIn("No view state recorded yet", text)

    def test_document_outside_workspace_is_rejected(self):
        mcp = self.write_state(currentDocument="../outside.md")
        result = mcp.tool_get_current_section({})
        self.assertTrue(result.get("isError"))

    def test_non_markdown_document_is_rejected(self):
        secret = self.workspace / "secret.txt"
        secret.write_text("nope", encoding="utf-8")
        mcp = self.write_state(currentDocument="secret.txt")
        result = mcp.tool_get_current_section({})
        self.assertTrue(result.get("isError"))


class McpStdioTests(unittest.TestCase):
    """Drive the real MCP server process over stdio JSON-RPC (no model needed)."""

    def test_initialize_list_and_call(self):
        with tempfile.TemporaryDirectory() as tmp:
            state = Path(tmp) / "state.json"
            state.write_text(
                json.dumps(
                    {
                        "currentDocument": None,
                        "workspace": tmp,
                        "writtenAtEpoch": time.time(),
                    }
                ),
                encoding="utf-8",
            )
            env = os.environ.copy()
            env["RUST_COURSE_VIEW_STATE"] = str(state)
            requests = "\n".join(
                [
                    json.dumps({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}),
                    json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized"}),
                    json.dumps({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}),
                    json.dumps(
                        {
                            "jsonrpc": "2.0",
                            "id": 3,
                            "method": "tools/call",
                            "params": {"name": "get_current_view", "arguments": {}},
                        }
                    ),
                ]
            )
            proc = subprocess.run(
                [sys.executable, str(VIEWER_DIR / "mcp_server.py")],
                input=requests,
                capture_output=True,
                text=True,
                env=env,
                timeout=30,
            )
            self.assertEqual(proc.returncode, 0, proc.stderr)
            replies = {}
            for line in proc.stdout.splitlines():
                message = json.loads(line)
                replies[message["id"]] = message
            self.assertEqual(
                replies[1]["result"]["serverInfo"]["name"], "rust-course-viewer"
            )
            tools = {tool["name"] for tool in replies[2]["result"]["tools"]}
            self.assertEqual(tools, {"get_current_view", "get_current_section"})
            self.assertIn("content", replies[3]["result"])


class AgentSessionTests(unittest.TestCase):
    def test_session_lifecycle_with_fake_executable(self):
        with tempfile.TemporaryDirectory() as tmp:
            fake = Path(tmp) / "fake-agent"
            fake.write_text(
                '#!/bin/sh\necho "fake-agent ready ctx=$RUST_COURSE_AGENT_CONTEXT"\n'
                "exec sleep 300\n",
                encoding="utf-8",
            )
            fake.chmod(0o755)
            serve = fresh_import(
                "serve",
                PATH=tmp,
                RUST_COURSE_VIEW_STATE=str(Path(tmp) / "state.json"),
            )
            session = serve.AgentSession(str(fake), current_document=None)
            try:
                collected = b""
                offset = 0
                deadline = time.monotonic() + 10
                while b"fake-agent ready" not in collected and time.monotonic() < deadline:
                    chunk, offset = session.read(offset, timeout=1.0)
                    collected += chunk
                self.assertIn(b"fake-agent ready", collected)
                # The child was told where its per-session context file lives.
                self.assertIn(str(session.context_path).encode(), collected)
                self.assertTrue(session.context_path.is_file())
                mode = stat.S_IMODE(session.context_path.stat().st_mode)
                self.assertEqual(mode, 0o600)
                payload = json.loads(session.context_path.read_text(encoding="utf-8"))
                self.assertIsNone(payload["currentDocument"])
                session.update_context("lessons/01-basics/README.md")
                payload = json.loads(session.context_path.read_text(encoding="utf-8"))
                self.assertEqual(
                    payload["currentDocument"], "lessons/01-basics/README.md"
                )
            finally:
                session.close()
            self.assertFalse(session.context_path.exists())
            deadline = time.monotonic() + 5
            while not session.exited and time.monotonic() < deadline:
                time.sleep(0.05)
            self.assertTrue(session.exited, "fake agent should be reaped after close()")


def free_port():
    sock = socket.socket()
    sock.bind(("127.0.0.1", 0))
    port = sock.getsockname()[1]
    sock.close()
    return port


class HttpApiTests(unittest.TestCase):
    """Start the real server as a subprocess and exercise it over HTTP."""

    def start_server(self, **env):
        port = free_port()
        full_env = os.environ.copy()
        for key in ENV_KEYS:
            full_env.pop(key, None)
        full_env.update(env)
        proc = subprocess.Popen(
            [sys.executable, str(VIEWER_DIR / "serve.py"), str(port)],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=full_env,
        )
        self.addCleanup(self._stop, proc)
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            if proc.poll() is not None:
                self.fail(f"server exited early: {proc.stderr.read().decode()}")
            try:
                self.request(port, "/api/files")
                return proc, port
            except OSError:
                time.sleep(0.1)
        self.fail("server did not start listening in time")

    @staticmethod
    def _stop(proc):
        proc.terminate()
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait(timeout=10)
        for stream in (proc.stdout, proc.stderr):
            if stream:
                stream.close()

    @staticmethod
    def request(port, path, method="GET", body=None):
        data = json.dumps(body).encode() if body is not None else None
        req = urllib.request.Request(
            f"http://127.0.0.1:{port}{path}",
            data=data,
            method=method,
            headers={"Content-Type": "application/json"} if data else {},
        )
        try:
            with urllib.request.urlopen(req, timeout=10) as resp:
                return resp.status, json.loads(resp.read() or b"null")
        except urllib.error.HTTPError as exc:
            return exc.code, json.loads(exc.read() or b"null")

    def test_no_agent_api_behavior(self):
        with tempfile.TemporaryDirectory() as tmp:
            _, port = self.start_server(PATH=tmp)
            status, data = self.request(port, "/api/agents")
            self.assertEqual(status, 200)
            self.assertEqual(data["agents"], [])
            self.assertIsNone(data["default"])
            self.assertIn("docs/agent-integration.md", data["hint"])
            status, data = self.request(port, "/api/agent/session", "POST", {})
            self.assertEqual(status, 400)
            self.assertIn("no supported agent CLI", data["error"])

    def test_index_and_files_served_without_agents(self):
        with tempfile.TemporaryDirectory() as tmp:
            _, port = self.start_server(PATH=tmp)
            req = urllib.request.Request(f"http://127.0.0.1:{port}/")
            with urllib.request.urlopen(req, timeout=10) as resp:
                html = resp.read().decode()
            self.assertIn("Rust From Zero", html)
            status, data = self.request(port, "/api/files")
            self.assertEqual(status, 200)
            self.assertGreater(data["total"], 0)

    def test_view_state_written_and_validated(self):
        with tempfile.TemporaryDirectory() as tmp:
            state = Path(tmp) / "view.json"
            _, port = self.start_server(PATH=tmp, RUST_COURSE_VIEW_STATE=str(state))
            status, _ = self.request(
                port,
                "/api/view",
                "POST",
                {"document": "README.md", "heading": None, "scrollPercent": 10},
            )
            self.assertEqual(status, 200)
            payload = json.loads(state.read_text(encoding="utf-8"))
            self.assertEqual(payload["currentDocument"], "README.md")
            status, _ = self.request(
                port, "/api/view", "POST", {"document": "../escape.md"}
            )
            self.assertEqual(status, 400)
            status, _ = self.request(
                port, "/api/view", "POST", {"document": "Cargo.toml"}
            )
            self.assertEqual(status, 400)

    def test_custom_agent_session_over_http(self):
        with tempfile.TemporaryDirectory() as tmp:
            fake = Path(tmp) / "myagent"
            fake.write_text(
                '#!/bin/sh\necho "http-fake-agent ready"\nexec sleep 300\n',
                encoding="utf-8",
            )
            fake.chmod(0o755)
            _, port = self.start_server(
                PATH=tmp, RUST_COURSE_AGENT="myagent --turbo"
            )
            status, data = self.request(port, "/api/agents")
            self.assertEqual(status, 200)
            self.assertEqual(data["default"], "configured")
            self.assertIsNone(data["hint"])
            status, data = self.request(port, "/api/agent/session", "POST", {})
            self.assertEqual(status, 201, data)
            session_id = data["id"]
            self.assertEqual(data["command"], "myagent --turbo")
            output = ""
            deadline = time.monotonic() + 10
            offset = 0
            while "http-fake-agent ready" not in output and time.monotonic() < deadline:
                status, chunk = self.request(
                    port, f"/api/agent/output?id={session_id}&offset={offset}"
                )
                self.assertEqual(status, 200)
                output += chunk["data"]
                offset = chunk["offset"]
            self.assertIn("http-fake-agent ready", output)
            status, _ = self.request(
                port, f"/api/agent/session?id={session_id}", "DELETE"
            )
            self.assertEqual(status, 200)
            status, _ = self.request(
                port, f"/api/agent/output?id={session_id}&offset=0"
            )
            self.assertEqual(status, 404)


if __name__ == "__main__":
    unittest.main()
