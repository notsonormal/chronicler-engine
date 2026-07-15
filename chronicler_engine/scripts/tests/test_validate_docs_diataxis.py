"""Regression tests for `scripts/validate_docs_diataxis.py`.

Exercises each docs-diataxis-only rule in isolation by invoking the validator's
parser/check helpers directly with synthetic text. Run from the
``chronicler_engine/`` directory via ``python -m unittest discover scripts/tests``
or via ``build.py``.

Pinned at 14 fixtures (one per rule + two clean docs). Adding a new rule to the
validator without a fixture here will show up in code review as a missing test.
"""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

REPO_ROOT = Path("/home/moridin84/projects/mrn-general")
sys.path.insert(0, str(REPO_ROOT / "chronicler_engine" / "scripts"))

import validate_docs_diataxis as vd  # noqa: E402


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
    *,
    is_architecture_shaped: bool = True,
) -> list[str]:
    """Parse + check synthetic body; return the rule names found."""
    path = tmp_path / "fixture.md"
    path.write_text(text, encoding="utf-8")
    report = _FakeReport(path)
    vd.check_diataxis_frontmatter(
        report,  # type: ignore[arg-type]
        is_architecture_shaped=is_architecture_shaped,
    )
    fm = vd.parse_frontmatter(text)
    if fm.present and fm.parsed is not None:
        vd.check_mode_content_heuristic(report, fm)  # type: ignore[arg-type]
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

    def test_arc52_out_of_place(self) -> None:
        # Warning only; fires when arc52: appears on a non-architecture doc.
        rules = _run_check(
            self._make_tmp(),
            "---\ndiataxis: reference\ntitle: Foo\narc52: [§3, §5]\n---\n# Body\n",
            is_architecture_shaped=False,
        )
        self.assertEqual(rules, ["FRONTMATTER_ARC52_OUT_OF_PLACE"])

    def test_mode_content_mismatch_reference_with_procedural(self) -> None:
        # Reference doc containing reader-directing prose triggers the heuristic.
        rules = _run_check(
            self._make_tmp(),
            "---\ndiataxis: reference\ntitle: Foo\n---\n# Body\n\n"
            "Let's walk through the steps. You should first set up the engine, "
            "then run the tests.\n",
        )
        self.assertEqual(rules, ["MODE_CONTENT_MISMATCH"])

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


if __name__ == "__main__":
    unittest.main()
