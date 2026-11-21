#!/usr/bin/env python3
"""Minimal stdio MCP server for the Rust From Zero viewer.

The web UI (viewer/static/index.html) reports the document/section the user is
looking at to serve.py (`POST /api/view`), which mirrors it to a JSON state
file. This server exposes that state to agents (e.g. the Kimi Code CLI running
in the viewer's side panel) through MCP tools:

- `get_current_view`: where the user is (document, heading, scroll percent)
- `get_current_section`: the Markdown text of the section the user is reading

No third-party dependencies: MCP stdio transport is newline-delimited JSON-RPC.
"""

import hashlib
import json
import os
import re
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent  # project root


def _default_state_path():
    """Per-user, per-project default. Keep in sync with viewer/serve.py."""
    digest = hashlib.sha1(str(ROOT).encode("utf-8")).hexdigest()[:8]
    return Path(tempfile.gettempdir()) / f"rust-course-viewer-state-{os.getuid()}-{digest}.json"


STATE_PATH = Path(
    os.environ.get("RUST_COURSE_VIEW_STATE")
    or os.environ.get("LEARN_RUST_VIEW_STATE")  # legacy alias
    or _default_state_path()
)

DEFAULT_MAX_CHARS = 15000
HEADING_RE = re.compile(r"^(#{1,6})\s+(.*?)\s*#*\s*$")
FENCE_RE = re.compile(r"^\s*(```|~~~)")

TOOLS = [
    {
        "name": "get_current_view",
        "description": (
            "Return what the user is currently looking at in the Rust From Zero web "
            "viewer: the Markdown document path (relative to the project root), "
            "the section heading at the top of their screen, and how far they "
            "have scrolled. Call this whenever the user refers to 'this page', "
            "'what I'm reading', 'the current section', and similar."
        ),
        "inputSchema": {
            "type": "object",
            "properties": {},
            "additionalProperties": False,
        },
    },
    {
        "name": "get_current_section",
        "description": (
            "Return the Markdown text of the section the user is currently "
            "reading in the Rust From Zero web viewer — from the heading at the top "
            "of their screen to the next heading of the same or higher level. "
            "Prefer this over reading the whole document when the user asks "
            "about 'this section' / 'what I'm reading'. Use get_current_view "
            "first if you only need to know where the user is."
        ),
        "inputSchema": {
            "type": "object",
            "properties": {
                "maxChars": {
                    "type": "integer",
                    "description": (
                        f"Maximum characters to return (default {DEFAULT_MAX_CHARS}); "
                        "the section is truncated with a notice when longer."
                    ),
                }
            },
            "additionalProperties": False,
        },
    },
]


def load_state():
    """Return (state dict, error text). Exactly one of them is set."""
    if not STATE_PATH.is_file():
        return None, (
            "No view state recorded yet. The user may not have opened the web "
            "viewer, or the page has not reported its position."
        )
    try:
        return json.loads(STATE_PATH.read_text(encoding="utf-8")), None
    except (OSError, json.JSONDecodeError) as exc:
        return None, f"Failed to read view state: {exc}"


def state_age_note(state):
    age = time.time() - state.get("writtenAtEpoch", 0)
    if age > 3600:
        return (
            f"\n\n(warning: state is {int(age // 60)} minutes old; "
            "the user may have closed the viewer)"
        )
    return ""


def tool_get_current_view():
    state, err = load_state()
    if err:
        text = err
    else:
        text = json.dumps(state, ensure_ascii=False, indent=2) + state_age_note(state)
    return {"content": [{"type": "text", "text": text}]}


def normalize_heading(text):
    """Compare DOM heading text with Markdown source (strip inline markers)."""
    text = re.sub(r"[`*_~]", "", text)
    return re.sub(r"\s+", " ", text).strip()


def iter_headings(lines):
    """Yield (line_index, level, text) for ATX headings outside code fences."""
    in_fence = False
    for index, line in enumerate(lines):
        if FENCE_RE.match(line):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        match = HEADING_RE.match(line)
        if match:
            yield index, len(match.group(1)), match.group(2)


def tool_get_current_section(arguments):
    state, err = load_state()
    if err:
        return {"content": [{"type": "text", "text": err}]}
    document = state.get("currentDocument")
    if not document:
        return {
            "content": [
                {"type": "text", "text": "The user is not viewing any document right now."}
            ]
        }
    workspace = Path(state.get("workspace", "")).resolve()
    path = (workspace / document).resolve()
    if not path.is_relative_to(workspace) or path.suffix.lower() != ".md" or not path.is_file():
        return {
            "content": [
                {"type": "text", "text": f"Cannot read document: {document!r}"}
            ],
            "isError": True,
        }
    lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    headings = list(iter_headings(lines))

    # Locate the heading the user sees at the top of their screen.
    start, level = None, None
    target = normalize_heading(state.get("currentHeading") or "")
    if target:
        for index, heading_level, text in headings:
            if normalize_heading(text) == target:
                start, level = index, heading_level
                break
    if start is None:
        # Fall back to the scroll position, then walk up to the nearest heading.
        percent = state.get("scrollPercent") or 0
        approx = round(percent / 100 * max(0, len(lines) - 1))
        above = [h for h in headings if h[0] <= approx]
        if above:
            start, level = above[-1][0], above[-1][1]
        else:
            start, level = 0, 7  # no heading above: take everything up to the first h1..h6

    # The section ends at the next heading of the same or higher level.
    end = len(lines)
    for index, heading_level, _ in headings:
        if index > start and heading_level <= level:
            end = index
            break

    section = "\n".join(lines[start:end]).strip()
    max_chars = DEFAULT_MAX_CHARS
    try:
        max_chars = max(200, int(arguments.get("maxChars", max_chars)))
    except (TypeError, ValueError):
        pass
    truncated = ""
    if len(section) > max_chars:
        section = section[:max_chars]
        truncated = "\n\n(truncated: section is longer than maxChars)"
    header = (
        f"# view context\ndocument: {document}\n"
        f"section: lines {start + 1}-{end} of {len(lines)}\n\n"
    )
    text = header + section + truncated + state_age_note(state)
    return {"content": [{"type": "text", "text": text}]}


TOOL_HANDLERS = {
    "get_current_view": lambda args: tool_get_current_view(),
    "get_current_section": tool_get_current_section,
}


def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


def main():
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            request = json.loads(line)
        except json.JSONDecodeError:
            continue
        method = request.get("method")
        request_id = request.get("id")
        if request_id is None:  # notification, e.g. notifications/initialized
            continue
        if method == "initialize":
            result = {
                "protocolVersion": request.get("params", {}).get(
                    "protocolVersion", "2024-11-05"
                ),
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "rust-course-viewer", "version": "1.0.0"},
            }
        elif method == "ping":
            result = {}
        elif method == "tools/list":
            result = {"tools": TOOLS}
        elif method == "tools/call":
            params = request.get("params") or {}
            handler = TOOL_HANDLERS.get(params.get("name"))
            if handler:
                result = handler(params.get("arguments") or {})
            else:
                result = {
                    "content": [
                        {"type": "text", "text": f"unknown tool: {params.get('name')}"}
                    ],
                    "isError": True,
                }
        else:
            send(
                {
                    "jsonrpc": "2.0",
                    "id": request_id,
                    "error": {
                        "code": -32601,
                        "message": f"method not found: {method}",
                    },
                }
            )
            continue
        send({"jsonrpc": "2.0", "id": request_id, "result": result})


if __name__ == "__main__":
    main()
