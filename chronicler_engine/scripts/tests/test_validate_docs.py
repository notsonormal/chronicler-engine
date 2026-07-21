"""Regression tests for `scripts/validate_docs.py`.

Exercises each rule in isolation by invoking the validator's parser/check
helpers directly with synthetic text. Run from the ``chronicler_engine/``
directory via ``python -m unittest discover scripts/tests`` or via ``build.py``.

Pinned at 13 markdown-front-matter fixtures + 13 DOC-anchor fixtures. Adding a
new rule to the validator without a fixture here will show up in code review as
a missing test.
"""

from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path("/home/moridin84/projects/mrn-general")
sys.path.insert(0, str(REPO_ROOT / "chronicler_engine" / "scripts"))

import validate_docs as vd  # noqa: E402


class _FakeReport:
    """Stand-in for FileReport so check_* helpers work in isolation.

    The helpers write ``read_text(report)`` to a temp file under ``tmp_path``
    via the real implementation; we only need a path + violations list.
    """

    def __init__(self, path: Path) -> None:
        self.path = path
        self.violations: list[vd.Violation] = []


def _run_check(
    tmp_path: Path,
    text: str,
) -> list[str]:
    """Parse + check synthetic body; return the rule names found."""
    path = tmp_path / "fixture.md"
    path.write_text(text, encoding="utf-8")
    report = _FakeReport(path)
    vd.check_diataxis_frontmatter(
        report,  # type: ignore[arg-type]
    )
    return sorted({v.rule for v in report.violations})


class TestDiataxisValidator(unittest.TestCase):
    """One test per rule plus two clean-doc baselines."""

    def test_missing_frontmatter(self) -> None:
        rules = _run_check(
            self._make_tmp(),
            "# A heading\n\nJust body, no front-matter.\n",
        )
        self.assertEqual(rules, ["MISSING_FRONTMATTER"])

    def test_empty_frontmatter(self) -> None:
        rules = _run_check(
            self._make_tmp(),
            "---\n---\n# Body\n",
        )
        self.assertEqual(rules, ["EMPTY_FRONTMATTER"])

    def test_unterminated_frontmatter(self) -> None:
        rules = _run_check(
            self._make_tmp(),
            "---\ndiataxis: reference\ntitle: Foo\n# Body without closing fence\n",
        )
        self.assertEqual(rules, ["FRONTMATTER_PARSE_ERROR"])

    def test_yaml_parse_error(self) -> None:
        rules = _run_check(
            self._make_tmp(),
            "---\ndiataxis: [unclosed\n---\n# Body\n",
        )
        self.assertEqual(rules, ["FRONTMATTER_PARSE_ERROR"])

    def test_frontmatter_not_mapping(self) -> None:
        rules = _run_check(
            self._make_tmp(),
            "---\n- one\n- two\n---\n# Body\n",
        )
        self.assertEqual(rules, ["FRONTMATTER_NOT_MAPPING"])

    def test_missing_diataxis_key(self) -> None:
        rules = _run_check(
            self._make_tmp(),
            "---\ntitle: Foo\n---\n# Body\n",
        )
        self.assertEqual(rules, ["FRONTMATTER_MISSING_KEY"])

    def test_missing_title_key(self) -> None:
        rules = _run_check(
            self._make_tmp(),
            "---\ndiataxis: reference\n---\n# Body\n",
        )
        self.assertEqual(rules, ["FRONTMATTER_MISSING_KEY"])

    def test_invalid_mode_vocab(self) -> None:
        rules = _run_check(
            self._make_tmp(),
            "---\ndiataxis: walkthrough\ntitle: Foo\n---\n# Body\n",
        )
        self.assertEqual(rules, ["FRONTMATTER_INVALID_MODE"])

    def test_arc52_not_a_list(self) -> None:
        rules = _run_check(
            self._make_tmp(),
            "---\ndiataxis: reference\ntitle: Foo\narc52: §3\n---\n# Body\n",
        )
        self.assertEqual(rules, ["FRONTMATTER_INVALID_ARC52"])

    def test_arc52_bad_entries(self) -> None:
        rules = _run_check(
            self._make_tmp(),
            "---\ndiataxis: reference\ntitle: Foo\narc52: [§3, §6, §11]\n---\n# Body\n",
        )
        self.assertEqual(rules, ["FRONTMATTER_INVALID_ARC52"])

    def test_reference_clean(self) -> None:
        # Reference doc without procedural markers + correct front-matter: zero violations.
        rules = _run_check(
            self._make_tmp(),
            "---\ndiataxis: reference\ntitle: Foo\n---\n# Body\n\n"
            "Tables exist with these columns. The schema is enumerated below.\n",
        )
        self.assertEqual(rules, [])

    def test_all_clean(self) -> None:
        # Bare-minimum clean doc: front-matter valid, mode reference, body neutral.
        rules = _run_check(
            self._make_tmp(),
            "---\ndiataxis: reference\ntitle: Foo\n---\n# Body\n\n"
            "Table A: columns are id, key, name.\n",
        )
        self.assertEqual(rules, [])

    @staticmethod
    def _make_tmp() -> Path:
        """Return a fresh tmp directory rooted under the system temp dir."""
        import tempfile

        return Path(tempfile.mkdtemp(prefix="diataxis-fix-"))


class _AnchorFixture:
    """Builds a synthetic engine-root tree under a tmp dir for anchor checks."""

    def __init__(self) -> None:
        self.root = Path(tempfile.mkdtemp(prefix="anchor-fix-"))
        self.engine = self.root / "chronicler_engine"
        (self.engine / "src").mkdir(parents=True)
        (self.engine / "tests").mkdir(parents=True)
        (self.engine / "docs" / "diataxis" / "reference").mkdir(parents=True)

    def write_storage_md(self) -> Path:
        path = self.engine / "docs" / "diataxis" / "reference" / "storage.md"
        path.write_text("---\ndiataxis: reference\ntitle: Storage\n---\n# Storage\n", encoding="utf-8")
        return path

    def write_src_file(self, rel: str, text: str) -> Path:
        path = self.engine / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
        return path

    def write_test_support_file(self, name: str, text: str) -> Path:
        return self.write_src_file(f"src/test_support/{name}", text)

    def scan(self, path: Path) -> list[vd.Violation]:
        report = vd.FileReport(path)
        vd.check_doc_anchors(report, path, self.engine)
        vd.check_test_support_rules(report, path, self.engine)
        return report.violations


class TestDocAnchors(unittest.TestCase):
    """Fixtures A-L exercising the DOC anchor rules."""

    def test_fixture_a_valid_full_path_anchor(self) -> None:
        # Fixture A: valid chronicler_engine/.../storage.md anchor → 0 violations.
        fix = _AnchorFixture()
        fix.write_storage_md()
        path = fix.write_src_file(
            "src/foo.rs",
            "//! [DOC: chronicler_engine/docs/diataxis/reference/storage.md]\n//! Summary.\n",
        )
        self.assertEqual(fix.scan(path), [])

    def test_fixture_b_bogus_target_missing(self) -> None:
        # Fixture B: target file doesn't exist → BROKEN_DOC_ANCHOR (target-missing).
        fix = _AnchorFixture()
        path = fix.write_src_file(
            "src/foo.rs",
            "//! [DOC: chronicler_engine/docs/diataxis/reference/missing.md]\n//! Summary.\n",
        )
        rules = {v.rule for v in fix.scan(path)}
        self.assertEqual(rules, {"BROKEN_DOC_ANCHOR"})

    def test_fixture_c_short_form_prefix_missing(self) -> None:
        # Fixture C: short-form docs/diataxis/... (no prefix) → BROKEN_DOC_ANCHOR (path-form).
        fix = _AnchorFixture()
        fix.write_storage_md()
        path = fix.write_src_file(
            "src/foo.rs",
            "//! [DOC: docs/diataxis/reference/storage.md]\n//! Summary.\n",
        )
        violations = fix.scan(path)
        self.assertEqual(len(violations), 1)
        self.assertEqual(violations[0].rule, "BROKEN_DOC_ANCHOR")
        self.assertIn("must resolve under chronicler_engine/docs/diataxis/reference/", violations[0].message)

    def test_fixture_d_non_doc_comment_line_still_detected(self) -> None:
        # Fixture D: `// [DOC: ...]` (no `!`) with short-form target still fires.
        fix = _AnchorFixture()
        path = fix.write_src_file(
            "src/foo.rs",
            "// [DOC: docs/diataxis/reference/storage.md]\n",
        )
        rules = {v.rule for v in fix.scan(path)}
        self.assertEqual(rules, {"BROKEN_DOC_ANCHOR"})

    def test_fixture_e_plain_path_form_parsed_normally(self) -> None:
        # Fixture E: a non-section-suffix form is detected and parsed.
        fix = _AnchorFixture()
        path = fix.write_src_file(
            "src/foo.rs",
            "//! [DOC: docs/system/storage.md]\n//! Summary.\n",
        )
        rules = {v.rule for v in fix.scan(path)}
        self.assertEqual(rules, {"BROKEN_DOC_ANCHOR"})

    def test_fixture_f_test_support_anchor_forbidden(self) -> None:
        # Fixture F: src/test_support/x.rs with [DOC:] → TEST_SUPPORT_ANCHOR_FORBIDDEN.
        fix = _AnchorFixture()
        path = fix.write_test_support_file(
            "x.rs",
            "//! [DOC: chronicler_engine/docs/diataxis/reference/storage.md]\n//! Summary.\n",
        )
        rules = {v.rule for v in fix.scan(path)}
        self.assertEqual(rules, {"TEST_SUPPORT_ANCHOR_FORBIDDEN"})

    def test_fixture_g_test_support_clean_summary(self) -> None:
        # Fixture G: src/test_support/x.rs no anchor + non-empty //! line 1 → 0.
        fix = _AnchorFixture()
        path = fix.write_test_support_file("x.rs", "//! Test helper summary.\n")
        self.assertEqual(fix.scan(path), [])

    def test_fixture_h_test_support_empty_summary_required(self) -> None:
        # Fixture H: src/test_support/x.rs with empty //! line 1 → TEST_SUPPORT_SUMMARY_REQUIRED.
        fix = _AnchorFixture()
        path = fix.write_test_support_file("x.rs", "//!\n")
        rules = {v.rule for v in fix.scan(path)}
        self.assertEqual(rules, {"TEST_SUPPORT_SUMMARY_REQUIRED"})

    def test_fixture_i_tests_file_anchor_forbidden(self) -> None:
        # Fixture I: tests/x.rs with [DOC:] → TEST_FILES_ANCHOR_FORBIDDEN.
        fix = _AnchorFixture()
        path = fix.engine / "tests" / "x.rs"
        path.write_text(
            "//! [DOC: chronicler_engine/docs/diataxis/reference/storage.md]\n",
            encoding="utf-8",
        )
        rules = {v.rule for v in fix.scan(path)}
        self.assertEqual(rules, {"TEST_FILES_ANCHOR_FORBIDDEN"})

    def test_fixture_j_no_anchors_flag_absent(self) -> None:
        # Fixture J: per plan, there is no `--no-anchors` opt-out flag. The
        # argparse schema does not declare it, so parse_args rejects it.
        with self.assertRaises(SystemExit):
            vd.parse_args(["--no-anchors"])

    def test_fixture_k_violation_message_has_lineno_and_path(self) -> None:
        # Fixture K: violation message includes line number + path.
        fix = _AnchorFixture()
        path = fix.write_src_file(
            "src/foo.rs",
            "//! [DOC: docs/system/storage.md]\n//! Summary.\n",
        )
        violations = fix.scan(path)
        self.assertEqual(len(violations), 1)
        msg = violations[0].message
        self.assertIn("Line 1:", msg)
        self.assertIn("src/foo.rs", msg)

    def test_fixture_l_live_anchor_scan_smoke(self) -> None:
        # Fixture L: collect_anchor_files on live engine-root returns files and
        # scanning them produces ZERO anchor-related violations post-migration
        # (proves the full cutover succeeded end-to-end).
        engine_root = REPO_ROOT / "chronicler_engine"
        files = vd.collect_anchor_files(engine_root)
        self.assertGreater(len(files), 100)
        anchor_violations: list[vd.Violation] = []
        for f in files:
            report = vd.scan_anchor_file(f, engine_root)
            anchor_violations.extend(
                v for v in report.violations
                if v.rule in {
                    "BROKEN_DOC_ANCHOR",
                    "TEST_SUPPORT_ANCHOR_FORBIDDEN",
                    "TEST_FILES_ANCHOR_FORBIDDEN",
                    "TEST_SUPPORT_SUMMARY_REQUIRED",
                }
            )
        self.assertEqual(
            anchor_violations,
            [],
            f"Expected zero anchor violations on live tree, got {len(anchor_violations)}",
        )

    def test_fixture_m_explanation_target_rejected(self) -> None:
        # Fixture M: anchor pointing at diataxis/explanation/ is rejected —
        # only `chronicler_engine/docs/diataxis/reference/` is allowed.
        fix = _AnchorFixture()
        explanation_md = fix.engine / "docs" / "diataxis" / "explanation" / "prompt_system_design.md"
        explanation_md.parent.mkdir(parents=True, exist_ok=True)
        explanation_md.write_text("---\ndiataxis: explanation\ntitle: P\n---\n# P\n", encoding="utf-8")
        path = fix.write_src_file(
            "src/foo.rs",
            "//! [DOC: chronicler_engine/docs/diataxis/explanation/prompt_system_design.md]\n//! Summary.\n",
        )
        violations = fix.scan(path)
        self.assertEqual(len(violations), 1)
        self.assertEqual(violations[0].rule, "BROKEN_DOC_ANCHOR")
        self.assertIn("must resolve under chronicler_engine/docs/diataxis/reference/", violations[0].message)

    def test_fixture_n_path_traversal_rejected(self) -> None:
        # Fixture N: `reference/../../explanation/foo.md` escapes via `..` and
        # is rejected even though the string starts with the canonical prefix.
        fix = _AnchorFixture()
        explanation_md = fix.engine / "docs" / "diataxis" / "explanation" / "foo.md"
        explanation_md.parent.mkdir(parents=True, exist_ok=True)
        explanation_md.write_text("---\ndiataxis: explanation\ntitle: F\n---\n# F\n", encoding="utf-8")
        path = fix.write_src_file(
            "src/foo.rs",
            "//! [DOC: chronicler_engine/docs/diataxis/reference/../../explanation/foo.md]\n//! Summary.\n",
        )
        rules = {v.rule for v in fix.scan(path)}
        self.assertEqual(rules, {"BROKEN_DOC_ANCHOR"})

    def test_fixture_o_tests_file_clean_no_anchor(self) -> None:
        # Fixture O: negative fixture for TEST_FILES_ANCHOR_FORBIDDEN — a
        # tests/*.rs file with a plain `//! <summary>` line is clean.
        fix = _AnchorFixture()
        path = fix.engine / "tests" / "clean.rs"
        path.write_text("//! A clean test summary.\n", encoding="utf-8")
        self.assertEqual(fix.scan(path), [])

    def test_fixture_p_format_string_literal_not_detected(self) -> None:
        # Fixture P: a Rust format string like `format!("... [DOC: ...] ...")`
        # must not be mistaken for a real anchor — the strict regex anchors to
        # line-start with an optional comment prefix only.
        fix = _AnchorFixture()
        path = fix.write_src_file(
            "src/foo.rs",
            "            format!(\"Module `{path}` lacks a module-level DOC anchor. Add `//! [DOC: ...]` at the top of the file.\")\n",
        )
        self.assertEqual(fix.scan(path), [])

    def test_fixture_q_suffix_form_in_test_support_still_forbidden(self) -> None:
        # Fixture Q: legacy `— section \"...\"` suffix form in test_support is
        # still caught by the forbidden rule (uses the permissive DOC_ANCHOR_LINE
        # regex, not the strict path-only regex).
        fix = _AnchorFixture()
        path = fix.write_test_support_file(
            "x.rs",
            "//! [DOC: chronicler_engine/docs/diataxis/reference/storage.md \u2014 section \"Foo\"]\n//! Summary.\n",
        )
        rules = {v.rule for v in fix.scan(path)}
        self.assertEqual(rules, {"TEST_SUPPORT_ANCHOR_FORBIDDEN"})


if __name__ == "__main__":
    unittest.main()
