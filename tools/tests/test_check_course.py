import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.check_course import check_lesson


class CheckLessonTests(unittest.TestCase):
    def setUp(self):
        self.tempdir = tempfile.TemporaryDirectory()
        self.lessons = Path(self.tempdir.name)
        self.lesson = self.lessons / "01-basics"
        (self.lesson / "src").mkdir(parents=True)
        (self.lesson / "README.md").write_text(
            """# 第一课
## 本课定位
## 完整示例
## 编译错误与诊断
## 主练习
```bash
cargo test -p l01_basics
```
## 课后巩固
## 本课总结
## 下一课
""",
            encoding="utf-8",
        )
        (self.lesson / "HINTS.md").write_text(
            "一级提示\n二级提示\n三级提示\n课后巩固提示\n", encoding="utf-8"
        )
        (self.lesson / "Cargo.toml").write_text(
            """[package]
name = "l01_basics"
version = "0.1.0"
edition = "2024"
publish = false
""",
            encoding="utf-8",
        )
        (self.lesson / "src" / "main.rs").write_text(
            """fn main() {
    todo!()
}

#[cfg(test)]
mod tests {
    #[test]
    fn behavior_is_ready() {}
}
""",
            encoding="utf-8",
        )

    def tearDown(self):
        self.tempdir.cleanup()

    def check(self):
        with patch("tools.check_course.LESSONS", self.lessons), patch(
            "tools.check_course.ROOT", self.lessons.parent
        ):
            return check_lesson("01-basics", "l01_basics")

    def test_complete_lesson_passes(self):
        self.assertEqual(self.check(), [])

    def test_missing_consolidation_hints_are_reported(self):
        (self.lesson / "HINTS.md").write_text("一级提示\n二级提示\n", encoding="utf-8")
        errors = self.check()
        self.assertTrue(any("three hint levels" in error for error in errors))
        self.assertTrue(any("consolidation-task hints" in error for error in errors))

    def test_numbered_hint_levels_pass(self):
        (self.lesson / "HINTS.md").write_text(
            "第 1 级\n第 2 级\n第 3 级\n课后巩固\n", encoding="utf-8"
        )
        self.assertEqual(self.check(), [])

    def test_author_local_paths_are_reported(self):
        with (self.lesson / "README.md").open("a", encoding="utf-8") as readme:
            readme.write("\n/home/zhaochen/private/example\n")
        self.assertTrue(any("author-local path" in error for error in self.check()))


if __name__ == "__main__":
    unittest.main()
