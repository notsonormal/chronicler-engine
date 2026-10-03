"""Unit tests for healthcheck.py duplicate scoping.

Run via ``python -m unittest discover scripts/tests`` or via ``build.py py-tests``.
"""

from __future__ import annotations

import contextlib
import io
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / "scripts"))

import healthcheck  # noqa: E402


def _clone(first: str, second: str, lines: int = 6) -> dict:
    """One jscpd clone entry spanning both named files."""
    return {
        "firstFile": {"name": first, "start": 1, "end": lines},
        "secondFile": {"name": second, "start": 1, "end": lines},
        "lines": lines,
        "fragment": "fn duplicated() {}",
    }


def _report(*clones: dict) -> dict:
    return {"duplicates": list(clones)}


class SummarizeScopeTests(unittest.TestCase):
    """The changed-files filter keeps a pair when either side changed."""

    def test_pair_with_one_changed_side_is_kept(self):
        text = healthcheck.summarize_report(
            _report(_clone("src/a.rs", "src/b.rs")), changed={"src/a.rs"}
        )
        self.assertIn("src/a.rs", text)
        self.assertIn("src/b.rs", text)
        self.assertIn("- Scope: changed files only", text)

    def test_pair_with_neither_side_changed_is_dropped(self):
        text = healthcheck.summarize_report(
            _report(_clone("src/a.rs", "src/b.rs")), changed={"src/other.rs"}
        )
        self.assertNotIn("src/a.rs", text)
        self.assertIn("- Total clone pairs: 0", text)
        self.assertIn("No pair survived the scope filter", text)

    def test_scope_line_reports_the_changed_file_count(self):
        text = healthcheck.summarize_report(
            _report(_clone("src/a.rs", "src/b.rs")), changed={"src/a.rs", "src/b.rs"}
        )
        self.assertIn("- Scope: changed files only (2 in scope)", text)

    def test_empty_changed_set_is_called_out(self):
        text = healthcheck.summarize_report(
            _report(_clone("src/a.rs", "src/b.rs")), changed=set()
        )
        self.assertIn("empty by construction", text)

    def test_changed_none_keeps_every_pair(self):
        text = healthcheck.summarize_report(
            _report(_clone("src/a.rs", "src/b.rs")), changed=None
        )
        self.assertIn("src/a.rs", text)
        self.assertIn("- Scope: whole repository", text)

    def test_absolute_report_paths_are_normalized(self):
        changed = {
            healthcheck.normalize_report_path(
                str(healthcheck.WORKSPACE_ROOT / "src/a.rs")
            )
        }
        self.assertIn("src/a.rs", changed)
        text = healthcheck.summarize_report(
            _report(
                _clone(str(healthcheck.WORKSPACE_ROOT / "src/a.rs"), "src/b.rs")
            ),
            changed=changed,
        )
        self.assertIn("src/a.rs", text)


class ResolveScopeTests(unittest.TestCase):
    """Scope resolution decides between changed files and whole-repo."""

    def test_all_flag_bypasses_the_filter(self):
        changed, note = healthcheck.resolve_scope(all_files=True, ref=None)
        self.assertIsNone(changed)
        self.assertIsNone(note)

    def test_git_unavailable_falls_back_with_note(self):
        with mock.patch.object(healthcheck, "is_git_available", return_value=False):
            changed, note = healthcheck.resolve_scope(all_files=False, ref=None)
        self.assertIsNone(changed)
        self.assertIn("git unavailable", note)

    def test_detached_head_falls_back_with_note(self):
        with mock.patch.object(
            healthcheck, "is_git_available", return_value=True
        ), mock.patch.object(healthcheck, "is_head_detached", return_value=True):
            changed, note = healthcheck.resolve_scope(all_files=False, ref=None)
        self.assertIsNone(changed)
        self.assertIn("detached", note)

    def test_unresolvable_base_falls_back_with_note(self):
        with mock.patch.object(
            healthcheck, "is_git_available", return_value=True
        ), mock.patch.object(
            healthcheck, "is_head_detached", return_value=False
        ), mock.patch.object(
            healthcheck, "resolve_base_ref", return_value=None
        ):
            changed, note = healthcheck.resolve_scope(all_files=False, ref=None)
        self.assertIsNone(changed)
        self.assertIn("resolves", note)

    def test_explicit_ref_is_passed_through(self):
        with mock.patch.object(
            healthcheck, "is_git_available", return_value=True
        ), mock.patch.object(
            healthcheck, "is_head_detached", return_value=False
        ), mock.patch.object(
            healthcheck, "resolve_base_ref", return_value="develop"
        ) as base, mock.patch.object(
            healthcheck, "list_changed_files", return_value={"src/a.rs"}
        ) as lister:
            changed, note = healthcheck.resolve_scope(all_files=False, ref="develop")
        self.assertEqual(changed, {"src/a.rs"})
        self.assertIsNone(note)
        base.assert_called_once_with("develop")
        lister.assert_called_once_with("develop")

    def test_unresolvable_explicit_ref_note_names_only_that_ref(self):
        with mock.patch.object(
            healthcheck, "is_git_available", return_value=True
        ), mock.patch.object(
            healthcheck, "is_head_detached", return_value=False
        ), mock.patch.object(healthcheck, "resolve_base_ref", return_value=None):
            changed, note = healthcheck.resolve_scope(all_files=False, ref="develop")
        self.assertIsNone(changed)
        self.assertIn("develop does not resolve", note)
        self.assertNotIn("origin/main", note)


class CheckDuplicatesScopeTests(unittest.TestCase):
    """check_duplicates wires the resolved scope into the summary."""

    def _run(self, changed: set[str]) -> str:
        """Run the check over a two-pair report with a stubbed git state."""
        with tempfile.TemporaryDirectory() as tmp:
            report = Path(tmp) / "report.json"
            report.write_text(
                json.dumps(
                    _report(
                        _clone("src/a.rs", "src/b.rs"),
                        _clone("src/c.rs", "src/d.rs"),
                    )
                ),
                encoding="utf-8",
            )
            args = healthcheck.build_parser().parse_args(
                ["duplicates", "--skip-run", "--report", str(report)]
            )
            stdout = io.StringIO()
            with mock.patch.object(
                healthcheck, "is_git_available", return_value=True
            ), mock.patch.object(
                healthcheck, "is_head_detached", return_value=False
            ), mock.patch.object(
                healthcheck, "resolve_base_ref", return_value="main"
            ), mock.patch.object(
                healthcheck, "list_changed_files", return_value=changed
            ), contextlib.redirect_stderr(
                io.StringIO()
            ), contextlib.redirect_stdout(
                stdout
            ):
                result = healthcheck.check_duplicates(args)
        self.assertTrue(result.ok)
        return stdout.getvalue()

    def test_only_pairs_with_a_changed_side_survive(self):
        text = self._run({"src/a.rs"})
        self.assertIn("src/a.rs", text)
        self.assertIn("src/b.rs", text)
        self.assertNotIn("src/c.rs", text)
        self.assertNotIn("src/d.rs", text)
        self.assertIn("- Scope: changed files only (1 in scope)", text)
        self.assertIn("- Total clone pairs: 1 ", text)

    def test_empty_changed_set_reports_an_empty_scope(self):
        text = self._run(set())
        self.assertIn("empty by construction", text)
        self.assertIn("- Total clone pairs: 0 ", text)


class ParserScopeTests(unittest.TestCase):
    """--all and --ref parse into the duplicates check's scope flags."""

    def test_defaults_scope_to_changed_files(self):
        args = healthcheck.build_parser().parse_args(["duplicates"])
        self.assertFalse(args.all)
        self.assertIsNone(args.ref)

    def test_all_flag(self):
        args = healthcheck.build_parser().parse_args(["duplicates", "--all"])
        self.assertTrue(args.all)

    def test_ref_flag(self):
        args = healthcheck.build_parser().parse_args(["duplicates", "--ref", "develop"])
        self.assertEqual(args.ref, "develop")


class FallbackNoteTests(unittest.TestCase):
    """A forced whole-repo fallback prints its note during a run."""

    def test_check_duplicates_prints_fallback_note(self):
        with tempfile.TemporaryDirectory() as tmp:
            report = Path(tmp) / "report.json"
            report.write_text(
                json.dumps(_report(_clone("src/a.rs", "src/b.rs"))), encoding="utf-8"
            )
            args = healthcheck.build_parser().parse_args(
                ["duplicates", "--skip-run", "--report", str(report)]
            )
            stderr = io.StringIO()
            with mock.patch.object(
                healthcheck, "is_git_available", return_value=False
            ), contextlib.redirect_stderr(stderr), contextlib.redirect_stdout(
                io.StringIO()
            ):
                result = healthcheck.check_duplicates(args)
        self.assertTrue(result.ok)
        self.assertIn("whole-repo", stderr.getvalue())


if __name__ == "__main__":
    unittest.main()
