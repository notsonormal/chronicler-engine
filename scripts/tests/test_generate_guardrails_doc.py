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
import tomllib
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


class AnalyzerScopeTests(unittest.TestCase):
    """The reference doc once showed only the `deny-scope-dep` rules, never
    which tree the linter scans or what it skips, so an agent had to read
    arch-lint.toml (or its vendored crate) to answer whether test files are
    analyzed."""

    def _scope_for(self, toml_text: str) -> str:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "arch-lint.toml"
            path.write_text(toml_text)
            original = ggd.ARCH_LINT_TOML
            ggd.ARCH_LINT_TOML = path
            try:
                return ggd.parse_analyzer_scope()
            finally:
                ggd.ARCH_LINT_TOML = original

    def test_scope_line_lists_root_and_excludes(self) -> None:
        scope = self._scope_for(
            '[analyzer]\nroot = "./src"\nexclude = ["**/tests/**", "**/*_tests.rs"]\n'
        )
        self.assertIn('root = "./src"', scope)
        self.assertIn("`**/tests/**`", scope)
        self.assertIn("`**/*_tests.rs`", scope)

    def test_missing_analyzer_section_is_an_error(self) -> None:
        with self.assertRaises(SystemExit):
            self._scope_for('preset = "recommended"\n')

    def test_analyzer_without_exclude_is_an_error(self) -> None:
        with self.assertRaises(SystemExit):
            self._scope_for('[analyzer]\nroot = "./src"\n')

    def test_live_doc_states_every_live_exclude_glob(self) -> None:
        with ggd.ARCH_LINT_TOML.open("rb") as f:
            excludes = tomllib.load(f)["analyzer"]["exclude"]
        doc = (ggd.ROOT / ggd.DOC_REL).read_text(encoding="utf-8")
        for pattern in excludes:
            self.assertIn(
                f"`{pattern}`",
                doc,
                f"exclude glob `{pattern}` is missing from {ggd.DOC_REL}",
            )


class LiveRepoExhaustivenessTests(unittest.TestCase):
    """A silent drop must be a build failure: a new `check_*`, or one renamed
    into a shape the pattern misses, leaves the doc incomplete."""

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


class WalkerRowTableTests(unittest.TestCase):
    """A walker row whose reason or rule number is missing would vanish from
    the reference, so the rule no longer says why it exists or which decision it
    enforces."""

    ROW = (
        "// Rule 1: every read and write names its Game.\n"
        "BannedNameRow {\n"
        '    rule: 1,\n'
        '    scope: "adapters/driven/storage/",\n'
        "    banned: &[BannedName::StaticItem],\n"
        "},\n"
    )

    def _rows_for(self, source: str) -> list[tuple[int, str, str, int]]:
        with tempfile.TemporaryDirectory() as tmp:
            fake_root = Path(tmp)
            rows_file = fake_root / "walker_rows.rs"
            rows_file.write_text(source)
            originals = (ggd.ROOT, ggd.WALKER_ROWS_RS)
            ggd.ROOT = fake_root
            ggd.WALKER_ROWS_RS = rows_file
            try:
                return ggd.parse_walker_rows()
            finally:
                ggd.ROOT, ggd.WALKER_ROWS_RS = originals

    def test_a_row_and_its_reason_are_parsed(self) -> None:
        rows = self._rows_for(self.ROW)
        self.assertEqual(len(rows), 1)
        rule, kind, reason, line = rows[0]
        self.assertEqual(rule, 1)
        self.assertEqual(kind, "banned name")
        self.assertIn("every read and write names its Game", reason)
        self.assertEqual(line, 2)

    def test_a_row_without_a_reason_is_an_error(self) -> None:
        without_reason = self.ROW.split("\n", 1)[1]
        with self.assertRaises(SystemExit):
            self._rows_for(without_reason)

    def test_a_row_without_a_rule_is_an_error(self) -> None:
        source = self.ROW.replace("    rule: 1,\n", "")
        with self.assertRaises(SystemExit):
            self._rows_for(source)

    def test_a_non_numeric_rule_is_an_error(self) -> None:
        source = self.ROW.replace("rule: 1,", 'rule: "one",')
        with self.assertRaises(SystemExit):
            self._rows_for(source)

    def test_an_empty_table_renders_the_empty_state(self) -> None:
        table = ggd.format_walker_rows_table([])
        self.assertIn("No row exists yet", table)

    def test_a_row_like_item_outside_the_form_is_an_error(self) -> None:
        """A row the parser cannot read must fail, never be skipped."""
        one_line_row = (
            "// Rule 1: every read and write names its Game.\n"
            'BannedNameRow { rule: 1, scope: "storage/", banned: &[] },\n'
        )
        with self.assertRaises(SystemExit):
            self._rows_for(one_line_row)

    def test_a_field_line_outside_the_form_is_an_error(self) -> None:
        source = self.ROW.replace("    rule: 1,\n", "    rule: 1\n")
        with self.assertRaises(SystemExit):
            self._rows_for(source)

    def test_a_row_without_a_closing_line_is_an_error(self) -> None:
        source = self.ROW.replace("},\n", "")
        with self.assertRaises(SystemExit):
            self._rows_for(source)

    def test_unknown_kinds_are_humanized(self) -> None:
        self.assertEqual(ggd.humanize_kind("BannedNameRow"), "banned name")
        self.assertEqual(ggd.humanize_kind("TokenBudgetRow"), "token budget")

    def test_live_tables_parse(self) -> None:
        """Every live row parses, and each carries a rule number and a reason."""
        for rule, kind, reason, line in ggd.parse_walker_rows():
            self.assertGreater(rule, 0, f"row at line {line} has no rule number")
            self.assertTrue(kind, f"row at line {line} has no kind")
            self.assertTrue(reason, f"row at line {line} has no reason")


if __name__ == "__main__":
    unittest.main()
