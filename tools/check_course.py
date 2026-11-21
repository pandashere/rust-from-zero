#!/usr/bin/env python3
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parent.parent
LESSONS = ROOT / "lessons"

EXPECTED = {
    "01-basics": "l01_basics",
    "02-data-modeling": "l02_data_modeling",
    "03-ownership-borrowing": "l03_ownership_borrowing",
    "04-option-result": "l04_option_result",
    "05-vec-slices": "l05_vec_slices",
    "06-hash-collections": "l06_hash_collections",
    "07-modules-crates": "l07_modules_crates",
    "08-testing-quality": "l08_testing_quality",
    "09-closures": "l09_closures",
    "10-iterators": "l10_iterators",
    "11-generics": "l11_generics",
    "12-traits": "l12_traits",
    "13-box-recursion": "l13_box_recursion",
    "14-lifetimes": "l14_lifetimes",
    "15-smart-pointers": "l15_smart_pointers",
    "16-threads-channels": "l16_threads_channels",
    "17-shared-concurrency": "l17_shared_concurrency",
    "18-files-cli": "l18_files_cli",
    "19-async": "l19_async",
    "20-macros": "l20_macros",
    "21-unsafe-ffi": "l21_unsafe_ffi",
    "22-capstone": "l22_capstone",
}


def read(path, errors):
    try:
        return path.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as exc:
        errors.append(f"{path.relative_to(ROOT)}: cannot read UTF-8 text: {exc}")
        return ""


def check_lesson(directory, package):
    lesson = LESSONS / directory
    errors = []
    prefix = f"lessons/{directory}"
    if not lesson.is_dir():
        return [f"{prefix}: missing lesson directory"]

    readme_path = lesson / "README.md"
    hints_path = lesson / "HINTS.md"
    manifest_path = lesson / "Cargo.toml"
    for path in (readme_path, hints_path, manifest_path):
        if not path.is_file():
            errors.append(f"{prefix}: missing {path.name}")

    readme = read(readme_path, errors) if readme_path.is_file() else ""
    hints = read(hints_path, errors) if hints_path.is_file() else ""
    for label, pattern in {
        "positioning/prerequisites": r"本课定位|前置知识|学习准备",
        "complete example": r"完整示例|综合示例",
        "diagnostics": r"编译错误|诊断|排错",
        "main assignment": r"主练习",
        "consolidation homework": r"课后巩固|课后作业",
        "acceptance command": rf"cargo\s+test\s+-p\s+{re.escape(package)}",
        "summary": r"本课总结|课程总结|项目总结",
    }.items():
        if not re.search(pattern, readme):
            errors.append(f"{prefix}/README.md: missing {label}")

    if directory != "22-capstone" and not re.search(r"下一课|下一步", readme):
        errors.append(f"{prefix}/README.md: missing next-lesson connection")

    levels = [
        re.search(pattern, hints)
        for pattern in (
            r"一级|第一级|第\s*1\s*级|级别\s*1|提示\s*1",
            r"二级|第二级|第\s*2\s*级|级别\s*2|提示\s*2",
            r"三级|第三级|第\s*3\s*级|级别\s*3|提示\s*3",
        )
    ]
    if not all(levels):
        errors.append(f"{prefix}/HINTS.md: expected three hint levels")
    if not re.search(r"巩固|课后", hints):
        errors.append(f"{prefix}/HINTS.md: missing consolidation-task hints")

    if manifest_path.is_file():
        manifest = read(manifest_path, errors)
        section_match = re.search(
            r"(?ms)^\[package\]\s*\n(.*?)(?=^\[|\Z)", manifest
        )
        if section_match is None:
            errors.append(f"{prefix}/Cargo.toml: missing [package] section")
        else:
            section = section_match.group(1)
            name = re.search(r'(?m)^name\s*=\s*"([^"]+)"', section)
            edition = re.search(r'(?m)^edition\s*=\s*"([^"]+)"', section)
            if name is None or name.group(1) != package:
                errors.append(f"{prefix}/Cargo.toml: package name must be {package}")
            if edition is None or edition.group(1) != "2024":
                errors.append(f"{prefix}/Cargo.toml: edition must be 2024")
            if not re.search(r"(?m)^publish\s*=\s*false\s*(?:#.*)?$", section):
                errors.append(f"{prefix}/Cargo.toml: publish must be false")

    source = lesson / "src"
    if not source.is_dir():
        errors.append(f"{prefix}: missing src directory")
    rust_files = sorted(source.rglob("*.rs")) + sorted((lesson / "tests").rglob("*.rs"))
    if not rust_files:
        errors.append(f"{prefix}: no Rust exercise or test files")
    rust_text = "\n".join(read(path, errors) for path in rust_files)
    if "todo!(" not in rust_text and "unimplemented!(" not in rust_text:
        errors.append(f"{prefix}: no learner-owned todo!()/unimplemented!() exercise")
    if not re.search(r"#\[(?:[A-Za-z0-9_]+::)?test\]", rust_text):
        errors.append(f"{prefix}: no automatic tests found")

    for path in lesson.rglob("*"):
        if path.is_file() and re.search(r"(?:^|[-_])(solution|answer)(?:[-_.]|$)", path.name, re.I):
            errors.append(f"{prefix}: possible published answer file: {path.relative_to(lesson)}")

    for path in [readme_path, hints_path, manifest_path, *rust_files]:
        if not path.is_file():
            continue
        text = read(path, errors)
        if "/home/zhaochen" in text or "learning/learn_rust" in text:
            errors.append(f"{path.relative_to(ROOT)}: contains an author-local path")
        if re.search(r"(?:\.\./){2,}src/|learn_rust::", text):
            errors.append(f"{path.relative_to(ROOT)}: references the old shared package")

    return errors


def main():
    errors = []
    for directory, package in EXPECTED.items():
        errors.extend(check_lesson(directory, package))

    unexpected = sorted(
        path.name
        for path in LESSONS.iterdir()
        if path.is_dir() and path.name not in EXPECTED
    )
    if unexpected:
        errors.append(f"lessons: unexpected course directories: {', '.join(unexpected)}")

    if errors:
        print(f"Course checks failed with {len(errors)} problem(s):", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print(f"Course checks passed: {len(EXPECTED)} lessons, exercises, tests, and hints are present.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
