#!/usr/bin/env python3
from pathlib import Path
import re
import sys
from urllib.parse import unquote

ROOT = Path(__file__).resolve().parent.parent
LINK = re.compile(r"(?<!!)\[[^\]]*\]\(([^)]+)\)")
SKIPPED_DIRS = {".git", "target", "__pycache__"}
FENCE = re.compile(r"^\s*(```|~~~)")
INLINE_CODE = re.compile(r"`[^`]*`")


def content_lines(text):
    in_fence = False
    for line_number, line in enumerate(text.splitlines(), 1):
        if FENCE.match(line):
            in_fence = not in_fence
            yield line_number, ""
        elif in_fence:
            yield line_number, ""
        else:
            yield line_number, INLINE_CODE.sub("", line)


def destination(raw):
    target = raw.strip()
    if target.startswith("<") and ">" in target:
        target = target[1 : target.index(">")]
    else:
        target = target.split(maxsplit=1)[0] if target else ""
    if not target or target.startswith(("#", "//")):
        return None
    if re.match(r"^[A-Za-z][A-Za-z0-9+.-]*:", target):
        return None
    return unquote(target.split("#", 1)[0].split("?", 1)[0])


def check_markdown_links(root):
    errors = []
    markdown_files = [
        path
        for path in root.rglob("*.md")
        if not any(part in SKIPPED_DIRS for part in path.relative_to(root).parts)
    ]
    for document in sorted(markdown_files):
        text = document.read_text(encoding="utf-8")
        for line_number, line in content_lines(text):
            for match in LINK.finditer(line):
                target = destination(match.group(1))
                if not target:
                    continue
                resolved = (root / target.lstrip("/")) if target.startswith("/") else (document.parent / target)
                resolved = resolved.resolve()
                if not resolved.is_relative_to(root.resolve()):
                    errors.append(f"{document.relative_to(root)}:{line_number}: link leaves repository: {target}")
                elif not resolved.exists():
                    errors.append(f"{document.relative_to(root)}:{line_number}: missing link target: {target}")
    return errors


def main():
    errors = check_markdown_links(ROOT)
    if errors:
        print(f"Markdown link checks failed with {len(errors)} problem(s):", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print("Markdown link checks passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
