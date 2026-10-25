"""Regression tests for the comment-fixer skill's `comment_finder.py`.

The JavaScript scanner must find every comment form and must not take a `//`
or `/*` inside a string, a template literal or a regex literal for a comment.
Run from the repo root via ``python -m unittest discover scripts/tests`` or via
``build.py``.
"""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / ".agents" / "skills" / "chronicler-comment-fixer" / "scripts"))

import comment_finder as cf  # noqa: E402


def js_lines(source: str) -> list[int]:
    return [n for n, _ in cf._js_comment_lines(source.splitlines(keepends=True))]


class JavaScriptCommentTests(unittest.TestCase):
    def test_whole_line_and_trailing_line_comments(self):
        src = "// head\nconst a = 1; // tail\nconst b = 2;\n"
        self.assertEqual(js_lines(src), [1, 2])

    def test_block_comments_cover_every_line_including_jsdoc(self):
        src = "/**\n * Doc.\n */\nfunction f() {}\nlet x = /* inline */ 3;\n"
        self.assertEqual(js_lines(src), [1, 2, 3, 5])

    def test_comment_markers_inside_strings_are_code(self):
        src = "const u = 'http://x';\nconst v = \"/* no */\";\nconst w = 'a\\'// still';\n"
        self.assertEqual(js_lines(src), [])

    def test_comment_markers_inside_template_literals_are_code(self):
        src = "const t = `//${ {a: '/*'}.a }// ${`//`}`;\nconst z = 1; // real\n"
        self.assertEqual(js_lines(src), [2])

    def test_regex_literal_is_code_and_division_is_not_a_regex(self):
        src = "const r = /\\/\\/[/*]/g;\nconst q = a / b; // half\nreturn /x\\//.test(s);\n"
        self.assertEqual(js_lines(src), [2])


class ChangeWordTests(unittest.TestCase):
    def flag(self, text: str) -> str:
        return cf.CommentMatch("src/x.rs", 7, text).format_rich()

    def test_each_change_word_is_flagged(self):
        for word in ("rename", "renamed", "became", "stays", "no longer", "used to", "was changed"):
            self.assertIn(f"[change-word: {word}]", self.flag(f"// the key {word} here"), word)

    def test_flag_is_case_insensitive_and_lowercased(self):
        self.assertTrue(self.flag("// Renamed in v31").endswith("[change-word: renamed]"))

    def test_multiword_phrase_tolerates_extra_whitespace(self):
        self.assertIn("[change-word: no longer]", self.flag("// is no  longer used"))

    def test_words_must_be_whole_words(self):
        self.assertEqual(self.flag("// the renamer unbecame stayside"), "src/x.rs:7 - // the renamer unbecame stayside")

    def test_plain_comment_has_no_flag(self):
        self.assertEqual(self.flag("// keeps order"), "src/x.rs:7 - // keeps order")

    def test_report_keeps_every_comment_and_lists_each_flag(self):
        import io, contextlib, tempfile
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "a.rs"
            path.write_text("// stays and became\n// plain\n", encoding="utf-8")
            old_root = cf.WORKSPACE_ROOT
            cf.WORKSPACE_ROOT = Path(tmp)
            try:
                matches = cf.find_comments_in_file(path)
                out = io.StringIO()
                with contextlib.redirect_stdout(out):
                    cf.print_results("t", [path], [(path, matches)])
            finally:
                cf.WORKSPACE_ROOT = old_root
        lines = out.getvalue().splitlines()
        self.assertIn("a.rs:1 - // stays and became  [change-word: stays, became]", lines)
        self.assertIn("a.rs:2 - // plain", lines)
        self.assertIn("# Files: 1, Comments: 2", lines)


class FileSelectionTests(unittest.TestCase):
    def test_minified_bundles_are_not_source(self):
        self.assertTrue(cf._is_source("assets/dashboard-panels.js"))
        self.assertFalse(cf._is_source("assets/htmx.min.js"))

    def test_all_mode_covers_python_and_javascript(self):
        files = {f.relative_to(cf.WORKSPACE_ROOT).as_posix() for f in cf.get_all_source_files()}
        self.assertIn("build.py", files)
        self.assertIn("scripts/validate_docs.py", files)
        self.assertIn("assets/dashboard-panels.js", files)
        self.assertNotIn("assets/htmx.min.js", files)


if __name__ == "__main__":
    unittest.main()
