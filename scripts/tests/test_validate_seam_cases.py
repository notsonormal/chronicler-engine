"""Regression tests for `scripts/validate_seam_cases.py`.

The checker reads one strict form: the tag above a test carries the case id and
its description, `// CASE <id>: <description>`. Each test below drives one of its
failure modes over synthetic Rust text. Run from the repo root via
``python -m unittest discover scripts/tests`` or via ``build.py``.
"""

from __future__ import annotations

import contextlib
import io
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / "scripts"))

import validate_seam_cases as vsc  # noqa: E402

CONTRACT = Path("src/application/agents/quantifier/agent_tests.rs")
OTHER = Path("tests/storage/transaction.rs")

DECLARING = """// CASE quantifier-1: a usable answer sets the scene
#[test]
fn test_usable_answer_sets_the_scene() {
    let _ = 1;
}

// CASE quantifier-2 [failure]: an unusable answer keeps the current scene
#[tokio::test]
async fn test_unusable_answer_keeps_the_scene() {
    let _ = 1;
}
"""


def errors_for(text: str, path: Path = CONTRACT) -> list[str]:
    """Validate one file's text against an otherwise empty tree."""
    return vsc.validate({path: text})


class CleanContractTests(unittest.TestCase):
    def test_tagged_tests_are_clean(self) -> None:
        self.assertEqual(errors_for(DECLARING), [])

    def test_empty_tree_is_clean(self) -> None:
        self.assertEqual(vsc.validate({}), [])

    def test_file_without_tags_is_clean(self) -> None:
        text = "#[test]\nfn test_something() {\n    let _ = 1;\n}\n"
        self.assertEqual(errors_for(text), [])


class UntaggedTestTests(unittest.TestCase):
    def test_test_without_a_tag_is_reported(self) -> None:
        text = DECLARING + ("\n#[test]\nfn test_extra_behaviour() {\n    let _ = 1;\n}\n")
        errors = errors_for(text)
        self.assertTrue(
            any("test in a declaring file has no `// CASE <id>:` tag" in e for e in errors),
            errors,
        )

    def test_tag_without_an_anchor_is_reported(self) -> None:
        text = DECLARING + "\n// CASE quantifier-3: a case with no test\n"
        errors = errors_for(text)
        self.assertTrue(
            any("has no test attribute within" in e for e in errors),
            errors,
        )

    def test_two_tags_on_one_test_are_reported(self) -> None:
        text = DECLARING.replace(
            "// CASE quantifier-2 [failure]: an unusable answer keeps the current scene",
            "// CASE quantifier-2 [failure]: an unusable answer keeps the current scene\n"
            "// CASE quantifier-3: another case",
        )
        errors = errors_for(text)
        self.assertTrue(
            any("carries 2 tags" in e and "one test pins one case" in e for e in errors),
            errors,
        )


class DuplicateIdTests(unittest.TestCase):
    def test_duplicate_across_files_is_reported(self) -> None:
        errors = vsc.validate({CONTRACT: DECLARING, OTHER: DECLARING})
        self.assertTrue(
            any("duplicate case id `quantifier-1`" in e for e in errors),
            errors,
        )


class MalformedTagTests(unittest.TestCase):
    def assert_problem(self, text: str, needle: str) -> None:
        errors = errors_for(text)
        self.assertTrue(any(needle in e for e in errors), errors)

    def test_missing_colon_is_reported(self) -> None:
        self.assert_problem(
            "// CASE quantifier-1\n#[test]\nfn test_x() {}\n",
            "the tag has no `:` after the id",
        )

    def test_missing_description_is_reported(self) -> None:
        self.assert_problem(
            "// CASE quantifier-1:\n#[test]\nfn test_x() {}\n",
            "the tag has no description",
        )

    def test_uppercase_id_is_reported(self) -> None:
        self.assert_problem(
            "// CASE Quantifier-1: a scene\n#[test]\nfn test_x() {}\n",
            "the case id is missing or bad",
        )

    def test_bad_label_is_reported(self) -> None:
        self.assert_problem(
            "// CASE quantifier-1 [Failure]: a scene\n#[test]\nfn test_x() {}\n",
            "the label `[Failure]` is bad",
        )

    def test_unclosed_label_is_reported(self) -> None:
        self.assert_problem(
            "// CASE quantifier-1 [fail: a scene\n#[test]\nfn test_x() {}\n",
            "the label has no closing bracket",
        )


class OldFormTests(unittest.TestCase):
    def test_module_doc_list_is_reported(self) -> None:
        text = "//! CASE quantifier-1: a usable answer\n#[test]\nfn test_x() {}\n"
        errors = errors_for(text)
        self.assertTrue(any("old module-doc form `//! CASE`" in e for e in errors), errors)

    def test_bare_tag_is_reported(self) -> None:
        text = "// CASE: quantifier-1\n#[test]\nfn test_x() {}\n"
        errors = errors_for(text)
        self.assertTrue(any("old bare form `// CASE: <id>`" in e for e in errors), errors)


class ParserTests(unittest.TestCase):
    def test_lowercase_identifier_is_required(self) -> None:
        self.assertEqual(vsc.parse_tags("// CASE Quantifier-1: x\n")[0], [])
        self.assertEqual(vsc.parse_tags("// CASE quantifier_1: x\n")[0], [])

    def test_tag_needs_a_description(self) -> None:
        self.assertEqual(vsc.parse_tags("// CASE quantifier-1:\n")[0], [])

    def test_the_description_is_captured_verbatim(self) -> None:
        tags, faults = vsc.parse_tags("// CASE quantifier-1: a usable answer sets the scene\n")
        self.assertEqual(faults, [])
        self.assertEqual(
            tags,
            [vsc.CaseTag(1, "quantifier-1", "a usable answer sets the scene", 0)],
        )

    def test_optional_label_is_accepted(self) -> None:
        tags, _ = vsc.parse_tags("// CASE quantifier-2 [failure]: keeps the scene\n")
        self.assertEqual(tags, [vsc.CaseTag(1, "quantifier-2", "keeps the scene", 0)])

    def test_tag_anchors_on_a_test_attribute(self) -> None:
        text = "// CASE quantifier-1: a scene\n#[test]\nfn test_x() {}\n"
        self.assertEqual(
            vsc.parse_tags(text)[0],
            [vsc.CaseTag(1, "quantifier-1", "a scene", 2)],
        )

    def test_proptest_block_is_an_anchor(self) -> None:
        text = "// CASE rules-1: the rules hold\nproptest! {\n    #[test]\n    fn prop_x() {}\n}\n"
        self.assertEqual(
            vsc.parse_tags(text)[0],
            [vsc.CaseTag(1, "rules-1", "the rules hold", 2)],
        )


class ListOutputTests(unittest.TestCase):
    def test_the_case_list_names_every_case_and_its_description(self) -> None:
        declared = vsc.declaring_files({CONTRACT: DECLARING})
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            vsc.print_cases(declared)
        output = buffer.getvalue()
        self.assertIn(str(CONTRACT), output)
        self.assertIn("quantifier-1: a usable answer sets the scene", output)
        self.assertIn("quantifier-2: an unusable answer keeps the current scene", output)


if __name__ == "__main__":
    unittest.main()
