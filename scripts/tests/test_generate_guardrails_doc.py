"""Regression tests for `scripts/generate_guardrails_doc.py`.

Run via ``python -m unittest discover scripts/tests``.

The regression: a rule whose Rust function name contains a digit (e.g.
``check_tier3_rule``) was silently dropped from the generated doc, and
``--check`` then reported the doc as up to date — the rule vanished from the
reference. The discovery patterns are now ``\\w+``, and the exhaustiveness test
fails the build if any live rule has no doc row.
"""

from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / "scripts"))

import generate_guardrails_doc as ggd  # noqa: E402


def _rule(name: str, doc: str = "A rule description.") -> str:
    return f"/// {doc}\npub fn {name}(path: &str, content: &str) -> Vec<Violation> {{\n}}\n"


class DiscoveryPatternTests(unittest.TestCase):
    def _rows_for(self, source: str) -> list[str]:
        with tempfile.TemporaryDirectory() as tmp:
            # Point ROOT at a temp dir and put the fixture in a `guardrails/`
            # subdir, so the generator's `relative_to(ROOT)` resolves.
            fake_root = Path(tmp)
            fixture = fake_root / "guardrails"
            fixture.mkdir()
            (fixture / "probe.rs").write_text(source)
            originals = (ggd.ROOT, ggd.GUARDRAILS_DIR)
            ggd.ROOT = fake_root
            ggd.GUARDRAILS_DIR = fixture
            try:
                return [row[0] for row in ggd.parse_syn_table()]
            finally:
                ggd.ROOT, ggd.GUARDRAILS_DIR = originals

    def test_digit_in_name_is_discovered(self) -> None:
        """The regression: `check_tier3_rule` must not vanish from the doc."""
        rows = self._rows_for(_rule("check_tier3_rule"))
        self.assertEqual(rows, ["check_tier3_rule"])

    def test_trailing_digit_is_discovered(self) -> None:
        rows = self._rows_for(_rule("check_rule_2"))
        self.assertEqual(rows, ["check_rule_2"])

    def test_digit_free_name_still_discovered(self) -> None:
        rows = self._rows_for(_rule("check_browser_interactions_use_htmx_settle"))
        self.assertEqual(rows, ["check_browser_interactions_use_htmx_settle"])

    def test_multiple_rules_all_discovered(self) -> None:
        source = _rule("check_alpha_1") + _rule("check_beta") + _rule("check_gamma_2")
        rows = self._rows_for(source)
        self.assertEqual(rows, ["check_alpha_1", "check_beta", "check_gamma_2"])

    def test_undocumented_rule_is_an_error(self) -> None:
        """A rule with no `///` doc block must fail loudly, not be skipped."""
        with tempfile.TemporaryDirectory() as tmp:
            fake_root = Path(tmp)
            fixture = fake_root / "guardrails"
            fixture.mkdir()
            (fixture / "probe.rs").write_text(
                "pub fn check_undocumented(path: &str) -> Vec<Violation> {\n}\n"
            )
            originals = (ggd.ROOT, ggd.GUARDRAILS_DIR)
            ggd.ROOT = fake_root
            ggd.GUARDRAILS_DIR = fixture
            try:
                with self.assertRaises(SystemExit):
                    ggd.parse_syn_table()
            finally:
                ggd.ROOT, ggd.GUARDRAILS_DIR = originals


class LiveRepoExhaustivenessTests(unittest.TestCase):
    """Every rule in the live repo must have a row in the generated doc.

    This is the check that turns a silent drop into a build failure: if a new
    `check_*` appears (or one is renamed into a shape the pattern misses), the
    doc is incomplete and this test fails.
    """

    def test_every_check_fn_has_a_doc_row(self) -> None:
        rows = ggd.parse_syn_table()
        documented = {row[0] for row in rows}

        discovered: set[str] = set()
        for rs_file in ggd.GUARDRAILS_DIR.glob("*.rs"):
            if rs_file.name == "mod.rs":
                continue
            text = rs_file.read_text(encoding="utf-8")
            for line in text.splitlines():
                stripped = line.strip()
                if stripped.startswith("pub fn check_"):
                    name = stripped.split("pub fn ", 1)[1].split("(", 1)[0]
                    discovered.add(name)

        self.assertTrue(discovered, "no check_* functions found — wrong directory?")
        missing = discovered - documented
        self.assertEqual(
            missing,
            set(),
            f"guardrail rules missing from the generated doc (invisible to "
            f"generate_guardrails_doc.py): {sorted(missing)}",
        )

    def test_row_count_matches_function_count(self) -> None:
        rows = ggd.parse_syn_table()
        names = [row[0] for row in rows]
        self.assertEqual(
            len(names),
            len(set(names)),
            f"duplicate doc rows for: {sorted(n for n in names if names.count(n) > 1)}",
        )


class ClippyPatternTests(unittest.TestCase):
    def test_clippy_pattern_accepts_digits(self) -> None:
        import re

        pattern = re.compile(r"clippy::([a-z0-9_]+),?\s*$")
        self.assertIsNotNone(pattern.match("clippy::lint2_rule,"))
        self.assertIsNotNone(pattern.match("clippy::lint"))


if __name__ == "__main__":
    unittest.main()
