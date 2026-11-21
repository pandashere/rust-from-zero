import tempfile
import unittest
from pathlib import Path

from tools.check_markdown_links import check_markdown_links, destination


class MarkdownLinkTests(unittest.TestCase):
    def test_ignores_external_and_anchor_links(self):
        self.assertIsNone(destination("https://example.com/guide"))
        self.assertIsNone(destination("#section"))
        self.assertIsNone(destination("mailto:course@example.com"))

    def test_accepts_existing_relative_target(self):
        with tempfile.TemporaryDirectory() as tempdir:
            root = Path(tempdir)
            (root / "docs").mkdir()
            (root / "docs" / "guide.md").write_text("guide", encoding="utf-8")
            (root / "README.md").write_text("[guide](docs/guide.md#start)", encoding="utf-8")
            self.assertEqual(check_markdown_links(root), [])

    def test_reports_missing_target_and_line(self):
        with tempfile.TemporaryDirectory() as tempdir:
            root = Path(tempdir)
            (root / "README.md").write_text("ok\n[missing](docs/none.md)", encoding="utf-8")
            errors = check_markdown_links(root)
            self.assertEqual(len(errors), 1)
            self.assertIn("README.md:2", errors[0])
            self.assertIn("docs/none.md", errors[0])

    def test_ignores_rust_indexing_and_fenced_code(self):
        with tempfile.TemporaryDirectory() as tempdir:
            root = Path(tempdir)
            (root / "README.md").write_text(
                "`operations[0](3)`\n```rust\nlet x = values[1](2);\n```\n",
                encoding="utf-8",
            )
            self.assertEqual(check_markdown_links(root), [])


if __name__ == "__main__":
    unittest.main()
