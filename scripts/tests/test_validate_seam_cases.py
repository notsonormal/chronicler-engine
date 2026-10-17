"""Regression tests for `scripts/validate_seam_cases.py`.

The checker pairs a seam contract's `//! CASE <id>:` declarations with the
`// CASE: <id>` tag on the test that pins the case. Each test below drives one
of its five failure modes over synthetic Rust text. Run from the repo root via
``python -m unittest discover scripts/tests`` or via ``build.py``.
"""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / "scripts"))

import validate_seam_cases as vsc  # noqa: E402

CONTRACT = Path("src/application/agents/quantifier/agent_tests.rs")
OTHER = Path("tests/storage/transaction.rs")

DECLARING = """//! CASE quantifier-1: a usable answer sets the scene
//! CASE quantifier-2 [failure]: an unusable answer keeps the current scene

// CASE: quantifier-1
#[test]
fn test_usable_answer_sets_the_scene() {
    let _ = 1;
}

// CASE: quantifier-2
#[tokio::test]
async fn test_unusable_answer_keeps_the_scene() {
    let _ = 1;
}
"""


def errors_for(text: str, path: Path = CONTRACT) -> list[str]:
    """Validate one file's text against an otherwise empty tree."""
    return vsc.validate({path: text})


class CleanContractTests(unittest.TestCase):
    def test_declared_cases_with_tags_are_clean(self) -> None:
        self.assertEqual(errors_for(DECLARING), [])

    def test_empty_tree_is_clean(self) -> None:
        self.assertEqual(vsc.validate({}), [])

    def test_file_without_declarations_is_clean(self) -> None:
        text = "#[test]\nfn test_something() {\n    let _ = 1;\n}\n"
        self.assertEqual(errors_for(text), [])


class GapTests(unittest.TestCase):
    def test_declared_case_without_a_tag_is_reported(self) -> None:
        text = DECLARING.replace(
            "// CASE: quantifier-2\n#[tokio::test]\nasync fn "
            "test_unusable_answer_keeps_the_scene() {\n    let _ = 1;\n}\n",
            "",
        )
        errors = errors_for(text)
        self.assertTrue(
            any("declared case `quantifier-2` has no tagged test" in e for e in errors),
            errors,
        )


class OrphanTagTests(unittest.TestCase):
    def test_tag_without_a_declaration_is_reported(self) -> None:
        text = DECLARING.replace("//! CASE quantifier-2 [failure]", "//! CASE other-9 [failure]")
        errors = errors_for(text)
        self.assertTrue(
            any("tag `quantifier-2` has no declared case" in e for e in errors),
            errors,
        )

    def test_tag_in_a_non_declaring_file_is_reported(self) -> None:
        text = "// CASE: quantifier-1\n#[test]\nfn test_something() {\n    let _ = 1;\n}\n"
        errors = errors_for(text)
        self.assertTrue(
            any("declares no case" in e for e in errors),
            errors,
        )


class UntaggedTestTests(unittest.TestCase):
    def test_untagged_test_is_reported(self) -> None:
        text = DECLARING + (
            "\n#[test]\nfn test_extra_behaviour() {\n    let _ = 1;\n}\n"
        )
        errors = errors_for(text)
        self.assertTrue(
            any("test in a declaring file has no `// CASE:` tag" in e for e in errors),
            errors,
        )

    def test_tag_without_an_anchor_is_reported(self) -> None:
        text = DECLARING + "\n// CASE: quantifier-1\n"
        errors = errors_for(text)
        self.assertTrue(
            any("has no test attribute within" in e for e in errors),
            errors,
        )


class DuplicateIdTests(unittest.TestCase):
    def test_duplicate_across_files_is_reported(self) -> None:
        errors = vsc.validate({CONTRACT: DECLARING, OTHER: DECLARING})
        self.assertTrue(
            any("duplicate case id `quantifier-1`" in e for e in errors),
            errors,
        )


class ParserTests(unittest.TestCase):
    def test_lowercase_identifier_is_required(self) -> None:
        self.assertEqual(vsc.parse_declarations("//! CASE Quantifier-1: x\n"), [])
        self.assertEqual(vsc.parse_declarations("//! CASE quantifier_1: x\n"), [])

    def test_declaration_needs_a_description(self) -> None:
        self.assertEqual(vsc.parse_declarations("//! CASE quantifier-1:\n"), [])

    def test_optional_label_is_accepted(self) -> None:
        self.assertEqual(
            vsc.parse_declarations("//! CASE quantifier-2 [failure]: keeps the scene\n"),
            [vsc.CaseDeclaration(1, "quantifier-2")],
        )

    def test_tag_anchors_on_a_test_attribute(self) -> None:
        text = "// CASE: quantifier-1\n#[test]\nfn test_x() {}\n"
        self.assertEqual(vsc.parse_tags(text), [vsc.CaseTag(1, "quantifier-1", 2)])

    def test_proptest_block_is_an_anchor(self) -> None:
        text = "// CASE: rules-1\nproptest! {\n    #[test]\n    fn prop_x() {}\n}\n"
        self.assertEqual(vsc.parse_tags(text), [vsc.CaseTag(1, "rules-1", 2)])


class OneTestPerCaseTests(unittest.TestCase):
    """One test pins one case: a second test on the same case restates it."""

    def test_case_with_two_tagged_tests_is_reported(self) -> None:
        text = DECLARING + (
            "\n// CASE: quantifier-1\n#[test]\n"
            "fn test_another_angle() {\n    let _ = 1;\n}\n"
        )
        errors = errors_for(text)
        self.assertTrue(
            any("has more than one tagged test" in e for e in errors), errors
        )

    def test_one_tag_per_case_is_clean(self) -> None:
        self.assertEqual(errors_for(DECLARING), [])


if __name__ == "__main__":
    unittest.main()
