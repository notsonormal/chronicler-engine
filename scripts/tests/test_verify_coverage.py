"""Unit tests for scripts/verify_coverage.py: the function finder, the staleness check and the record.

Run via ``python -m unittest discover scripts/tests`` or via ``build.py``.
"""

from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))

import verify_coverage as vc  # noqa: E402

SOURCE = '''\
use std::fmt;

pub struct Gate;
pub struct Other;

impl Gate {
    /// Hold the game.
    pub fn hold(&self) -> u32 {
        let a = "}{ not a brace";
        1
    }

    fn release(&self) {
        // a stray } in a comment
        let c = '{';
    }
}

impl Other {
    fn hold(&self) -> u32 {
        2
    }
}

impl Drop for Gate {
    fn drop(&mut self) {
        let raw = r#"} "quoted" {"#;
    }
}

impl<T: Clone> From<T> for Other where T: fmt::Debug {
    fn from(_: T) -> Self {
        Other
    }
}

fn free_function(table: [u8; 4]) -> impl Sized {
    table
}
'''


def write(root: Path, rel: str, text: str) -> None:
    path = root / rel
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


class FindFunctionsTests(unittest.TestCase):
    def setUp(self):
        self.found = vc.find_functions(SOURCE)

    def test_a_function_is_keyed_by_its_impl_type(self):
        self.assertIn("Gate::hold", self.found)
        self.assertIn("Gate::release", self.found)

    def test_same_named_methods_of_two_impl_blocks_are_two_keys(self):
        self.assertEqual(len(self.found["Gate::hold"]), 1)
        self.assertEqual(len(self.found["Other::hold"]), 1)
        self.assertNotEqual(self.found["Gate::hold"], self.found["Other::hold"])

    def test_a_trait_impl_is_found_as_a_block_and_by_its_function(self):
        self.assertIn("fn drop(&mut self)", self.found["Drop for Gate"][0])
        self.assertIn("Drop for Gate::drop", self.found)
        self.assertIn("From for Other::from", self.found)

    def test_a_brace_in_a_string_comment_char_or_raw_string_does_not_move_the_bounds(self):
        self.assertTrue(self.found["Gate::hold"][0].rstrip().endswith("}"))
        self.assertIn("        1\n    }", self.found["Gate::hold"][0])
        self.assertIn("let c = '{';", self.found["Gate::release"][0])
        self.assertIn("r#\"} \"quoted\" {\"#", self.found["Drop for Gate"][0])
        self.assertEqual(self.found["Drop for Gate::drop"][0].count("fn "), 1)

    def test_the_text_starts_at_the_fn_line_and_leaves_the_doc_comment_out(self):
        self.assertTrue(self.found["Gate::hold"][0].startswith("    pub fn hold"))

    def test_a_free_function_with_an_impl_return_type_is_one_free_function(self):
        self.assertEqual(list(self.found["free_function"]), [self.found["free_function"][0]])
        self.assertNotIn("Sized", " ".join(self.found))

    def test_two_blocks_with_the_same_key_make_the_function_not_unique(self):
        found = vc.find_functions("impl A { fn x() {} }\nimpl A { fn x() {} }\n")
        self.assertEqual(len(found["A::x"]), 2)

    def test_a_lifetime_is_not_a_char_literal(self):
        found = vc.find_functions("impl<'a> A<'a> { fn x(&self) -> &'a str { '}'; \"\" } fn y() {} }\n")
        self.assertIn("A::x", found)
        self.assertIn("A::y", found)


class CheckTests(unittest.TestCase):
    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.root = Path(tmp.name)
        write(self.root, "src/gate.rs", SOURCE)
        self.coverage = self.root / ".agents/skills/verify-demo/coverage.toml"
        self.write_coverage(["Gate::hold", "Other::hold", "Drop for Gate"])

    def write_coverage(self, names, path="src/gate.rs"):
        lines = "".join(f'  "{n}",\n' for n in names)
        write(self.root, ".agents/skills/verify-demo/coverage.toml", f'[[files]]\npath = "{path}"\nfunctions = [\n{lines}]\n')

    def status(self):
        (status,) = vc.check(self.root)
        return status

    def edit_source(self, old, new):
        path = self.root / "src/gate.rs"
        path.write_text(path.read_text(encoding="utf-8").replace(old, new), encoding="utf-8")

    def test_a_function_with_no_hash_warns(self):
        self.assertEqual(self.status().unhashed, ["Gate::hold", "Other::hold", "Drop for Gate"])

    def test_record_then_check_gives_no_warning(self):
        self.assertEqual(vc.record(self.root), [])
        self.assertFalse(self.status().stale)
        self.assertEqual(vc.warning_lines(self.root), [])

    def test_a_changed_body_warns_for_that_function_only(self):
        vc.record(self.root)
        self.edit_source("        2\n", "        3\n")
        status = self.status()
        self.assertEqual(status.changed, ["Other::hold"])
        self.assertEqual(status.unhashed, [])

    def test_an_edit_outside_every_listed_function_gives_no_warning(self):
        vc.record(self.root)
        self.edit_source("pub struct Other;", "pub struct Other;\n\nfn added() {}\n")
        self.edit_source("    fn release(&self) {", "    fn release(&self) {\n        let x = 1;")
        self.assertFalse(self.status().stale)

    def test_a_renamed_function_warns_as_not_found(self):
        vc.record(self.root)
        self.edit_source("fn hold(&self) -> u32 {\n        2", "fn hold_all(&self) -> u32 {\n        2")
        self.assertEqual(self.status().missing, ["Other::hold"])

    def test_a_missing_file_warns_for_each_of_its_functions(self):
        vc.record(self.root)
        (self.root / "src/gate.rs").unlink()
        self.assertEqual(len(self.status().missing), 3)

    def test_a_function_defined_twice_warns_and_is_not_recorded(self):
        write(self.root, "src/gate.rs", "impl A { fn x() {} }\nimpl A { fn x() {} }\n")
        self.write_coverage(["A::x"])
        self.assertEqual(self.status().missing, ["A::x"])
        notes = vc.record(self.root)
        self.assertEqual(len(notes), 1)
        self.assertIn("not unique", notes[0])

    def test_a_malformed_coverage_file_gives_one_notice(self):
        self.coverage.write_text("[[files]\n", encoding="utf-8")
        lines = vc.warning_lines(self.root)
        self.assertTrue(lines[0].startswith("WARNING verify-demo: coverage.toml cannot be read"))

    def test_no_coverage_file_gives_no_warning(self):
        self.coverage.unlink()
        self.assertEqual(vc.warning_lines(self.root), [])

    def test_the_warning_is_three_lines_that_name_the_functions_and_the_rule(self):
        vc.record(self.root)
        self.edit_source("        2\n", "        3\n")
        lines = vc.warning_lines(self.root)
        self.assertEqual(len(lines), 3)
        self.assertEqual(lines[0], "WARNING verify-demo: the TLA+ model may not match the code (changed: Other::hold).")
        self.assertIn("/maintain-formal-verification-skill", lines[1])
        self.assertIn("scripts/verify_coverage.py record", lines[2])

    def test_a_long_list_of_names_is_cut(self):
        names = [f"n{i}" for i in range(7)]
        self.assertEqual(vc._names(names), "n0, n1, n2, n3 and 3 more")


class RecordTests(unittest.TestCase):
    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.root = Path(tmp.name)
        write(self.root, "src/gate.rs", SOURCE)
        write(
            self.root,
            ".agents/skills/verify-demo/coverage.toml",
            '# a comment\nsync_commit = "abc"\n\n[[files]]\npath = "src/gate.rs"\nfunctions = [\n'
            '  "Gate::hold",\n  "Other::hold",\n]\n\n[known_breaks]\nx = ["R"]\n',
        )
        self.path = self.root / ".agents/skills/verify-demo/coverage.toml"

    def test_record_writes_a_hash_next_to_each_name_and_keeps_the_rest(self):
        vc.record(self.root)
        text = self.path.read_text(encoding="utf-8")
        self.assertIn('{ name = "Gate::hold", hash = "', text)
        self.assertTrue(text.startswith('# a comment\nsync_commit = "abc"\n'))
        self.assertTrue(text.endswith('[known_breaks]\nx = ["R"]\n'))

    def test_record_twice_gives_the_same_file(self):
        vc.record(self.root)
        first = self.path.read_text(encoding="utf-8")
        vc.record(self.root)
        self.assertEqual(self.path.read_text(encoding="utf-8"), first)

    def test_record_only_a_named_function_leaves_the_others_with_no_hash(self):
        vc.record(self.root, only={"Gate::hold"})
        (status,) = vc.check(self.root)
        self.assertEqual(status.unhashed, ["Other::hold"])
        self.assertEqual(status.changed, [])

    def test_record_replaces_an_old_hash(self):
        vc.record(self.root)
        source = self.root / "src/gate.rs"
        source.write_text(source.read_text(encoding="utf-8").replace("        2\n", "        3\n"), encoding="utf-8")
        self.assertEqual(vc.check(self.root)[0].changed, ["Other::hold"])
        vc.record(self.root)
        self.assertFalse(vc.check(self.root)[0].stale)

    def test_record_names_a_function_of_an_inline_list_that_it_cannot_write(self):
        self.path.write_text('[[files]]\npath = "src/gate.rs"\nfunctions = ["Gate::hold"]\n', encoding="utf-8")
        notes = vc.record(self.root)
        self.assertEqual(len(notes), 1)
        self.assertIn("Gate::hold: not recorded", notes[0])


if __name__ == "__main__":
    unittest.main()
