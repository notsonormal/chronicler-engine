"""Unit tests for build.py's gate plan, tier sizes and run journal.

Run via ``python -m unittest discover scripts/tests`` or via ``build.py``.
"""

from __future__ import annotations

import os
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT))

import build  # noqa: E402

class GatePlanTests(unittest.TestCase):
    """The gate plan derives its step count from GATE_ORDER expansion."""

    def gate_args(self, **overrides):
        defaults = dict(
            no_fmt=False,
            no_browser=False,
            coverage=False,
            include_llm=False,
            test_timings=False,
            release=False,
            target_dir=None,
        )
        defaults.update(overrides)
        return SimpleNamespace(**defaults)

    def test_full_gate_installs_hooks_first_and_has_19_steps(self):
        plan = build._plan_gate_steps(self.gate_args())
        self.assertEqual(plan[0].kind, "hooks")
        self.assertEqual(plan[0].label, "Installing git hooks...")
        self.assertEqual(plan[1].kind, "worktrees")
        self.assertEqual(plan[2].label, "Formatting...")
        self.assertEqual(len(plan), 19)

    def test_worktree_removal_is_best_effort(self):
        with mock.patch.object(build, "run", return_value=1) as runner, mock.patch.object(
            build, "both_print"
        ) as printer:
            build._run_remove_worktrees({})
        # A failed removal warns; it does not raise or record a step failure.
        self.assertFalse(runner.call_args.kwargs["check"])
        self.assertEqual(
            runner.call_args.args[0], "python scripts/remove_worktrees.py --apply"
        )
        printer.assert_called_once()

    def test_gate_splits_browser_from_integration(self):
        """The gate runs the browser binary as its own step, not merged in."""
        plan = build._plan_gate_steps(self.gate_args())
        labels = [step.label for step in plan]
        self.assertIn("Running integration tests...", labels)
        self.assertIn("Running browser tests...", labels)
        by_label = {step.label: step for step in plan}
        integration_cmd = by_label["Running integration tests..."].cmd
        self.assertIn("-E 'not binary(browser)", integration_cmd)
        self.assertIn("not binary(architecture)", integration_cmd)
        self.assertIn("not binary(guardrails)", integration_cmd)
        self.assertIn("-E 'binary(browser)'", by_label["Running browser tests..."].cmd)
        self.assertNotIn(
            "not binary(browser)", by_label["Running browser tests..."].cmd
        )

    def test_no_browser_drops_only_the_browser_tier(self):
        plan = build._plan_gate_steps(self.gate_args(no_browser=True))
        labels = [step.label for step in plan]
        self.assertNotIn("Running browser tests...", labels)
        self.assertIn("Running integration tests...", labels)
        self.assertIn("Running architecture tests...", labels)
        self.assertIn("Running guardrail tests...", labels)
        self.assertEqual(len(plan), 18)

    def test_no_fmt_prunes_fmt_only(self):
        plan = build._plan_gate_steps(self.gate_args(no_fmt=True))
        labels = [step.label for step in plan]
        self.assertNotIn("Formatting...", labels)
        self.assertIn("Installing git hooks...", labels)
        self.assertEqual(len(plan), 18)

    def test_coverage_mode_swaps_test_and_report_steps(self):
        plan = build._plan_gate_steps(self.gate_args(coverage=True))
        labels = [step.label for step in plan]
        self.assertIn("Running integration tests with coverage...", labels)
        self.assertIn("Running browser tests with coverage...", labels)
        self.assertIn("Generating coverage report...", labels)
        self.assertNotIn(
            "Skipping coverage report (use --coverage to enable)", labels
        )
        # Coverage keeps the full non-browser filter.
        by_label = {step.label: step for step in plan}
        cov_cmd = by_label["Running integration tests with coverage..."].cmd
        self.assertIn("-E 'not binary(browser)'", cov_cmd)
        self.assertNotIn("binary(architecture)", cov_cmd)
        self.assertNotIn("binary(guardrails)", cov_cmd)

    def test_architecture_runs_after_asset_copy(self):
        plan = build._plan_gate_steps(self.gate_args())
        labels = [step.label for step in plan]
        self.assertLess(
            labels.index("Copying data and assets for deployment..."),
            labels.index("Running architecture tests..."),
        )

    def test_llm_note_printed_only_without_include_llm(self):
        for include_llm, expect_note in ((False, True), (True, False)):
            with self.subTest(include_llm=include_llm):
                args = self.gate_args(include_llm=include_llm)
                plan = build._plan_gate_steps(args)
                printed = self.run_plan(plan, args)
                note = any("LLM tests were skipped" in msg for msg in printed)
                self.assertEqual(note, expect_note)

    def run_plan(self, plan, args):
        """Execute a plan with all external effects mocked; return stdout lines."""
        record = build.StepRecord()
        printed = []

        def fake_timed_run(counter, _record, label, _cmd, **_kwargs):
            counter.next(label)

        with mock.patch.object(
            build, "_timed_run", side_effect=fake_timed_run
        ), mock.patch.object(build, "_copy_deployment_assets"), mock.patch.object(
            build, "_generate_coverage_report"
        ), mock.patch.object(
            build, "run", return_value=0
        ), mock.patch.object(
            build, "both_print", side_effect=lambda msg="": printed.append(msg)
        ):
            build._execute_gate_plan(plan, args, {}, record)
        return printed

    def test_counter_total_matches_counted_steps(self):
        """Regression: every plan entry must advance the counter exactly once."""
        args = self.gate_args()
        plan = build._plan_gate_steps(args)
        progress = [
            msg for msg in self.run_plan(plan, args) if re.match(r"^\[\d+/\d+\]", msg)
        ]
        self.assertEqual(len(progress), len(plan))
        totals = {msg.split("]")[0].split("/")[1] for msg in progress}
        self.assertEqual(totals, {str(len(plan))})

    def test_duplicates_step_is_report_only(self):
        plan = build._plan_gate_steps(self.gate_args())
        dup_steps = [step for step in plan if step.kind == "duplicates"]
        self.assertEqual(len(dup_steps), 1)
        self.assertEqual(dup_steps[0].label, "Checking for duplicate code...")
        with mock.patch.object(build, "run", return_value=1) as runner, mock.patch.object(
            build, "both_print"
        ) as printer:
            build._run_duplicates_report({})
        # Report-only: a failed command warns, it does not raise or record.
        self.assertFalse(runner.call_args.kwargs["check"])
        self.assertEqual(runner.call_args.args[0], "python scripts/healthcheck.py duplicates")
        printer.assert_called_once()


class TierForCmdTests(unittest.TestCase):
    """Only the four fixed tier commands record a test-set size."""

    def test_tier_commands_map_to_their_tier(self):
        expected = {
            build.REGISTRY["architecture"].cmd: "architecture",
            build.REGISTRY["guardrails"].cmd: "guardrails",
            build.get_integration_test_cmd(): "integration",
            build.get_browser_test_cmd(): "browser",
        }
        for cmd, tier in expected.items():
            with self.subTest(cmd=cmd):
                self.assertEqual(build._tier_for_cmd(cmd), tier)

    def test_other_test_sets_map_to_none(self):
        for cmd in (
            build.get_coverage_cmd(browser_only=False),
            build.get_coverage_cmd(browser_only=True),
            build.get_integration_test_cmd(include_llm=True),
            build._NEXTEST_RUN,
            build._step_command(build.REGISTRY["test-pattern"], "x"),
            None,
        ):
            with self.subTest(cmd=cmd):
                self.assertIsNone(build._tier_for_cmd(cmd))

    def test_summary_size_counts_run_and_skipped(self):
        self.assertEqual(
            build._summary_size(
                "     Summary [  20.5s] 1570 tests run: 1570 passed, 2 skipped\n"
            ),
            1572,
        )
        self.assertEqual(
            build._summary_size("     Summary [ 1.0s] 1 test run: 1 passed\n"), 1
        )
        self.assertIsNone(build._summary_size("error: could not compile\n"))


class JournalTests(unittest.TestCase):
    """The journal records the tree and per-tier test counts of each run."""

    OLD_HEADER = "timestamp | duration_s | exit_code | args\n"

    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.path = Path(tmp.name) / "build_history.txt"

    def lines(self):
        return self.path.read_text(encoding="utf-8").splitlines()

    def test_record_has_six_columns(self):
        build._append_history(
            self.path, "--no-browser", 51.2, 0, "abc+12345678", "integration=1565"
        )
        header, record = self.lines()
        self.assertEqual(header, build._HISTORY_HEADER)
        parts = record.split(" | ")
        self.assertEqual(len(parts), 6)
        self.assertEqual(parts[3:], ["--no-browser", "abc+12345678", "integration=1565"])

    def test_old_header_is_replaced_not_duplicated(self):
        old_record = "2026-10-08T19:46:00 | 1.0 | 0 | clippy\n"
        self.path.write_text(self.OLD_HEADER + old_record, encoding="utf-8")
        build._append_history(self.path, "clippy", 1.0, 0, "unknown", "-")
        lines = self.lines()
        self.assertEqual(lines[0], build._HISTORY_HEADER)
        self.assertEqual(lines[1], old_record.rstrip("\n"))
        self.assertEqual(len(lines), 3)
        self.assertTrue(lines[2].endswith(" | clippy | unknown | -"))

    def test_trim_keeps_the_header(self):
        with mock.patch.object(build, "_HISTORY_MAX_LINES", 3):
            for n in range(5):
                build._append_history(self.path, f"run{n}", 1.0, 0, "abc", "-")
        lines = self.lines()
        self.assertEqual(lines[0], build._HISTORY_HEADER)
        self.assertEqual([line.split(" | ")[3] for line in lines[1:]], ["run3", "run4"])

    def test_last_tier_sizes_takes_each_tiers_newest_record(self):
        self.path.write_text(
            build._HISTORY_HEADER + "\n"
            "2026-10-08T19:00:00 | 1.0 | 0 | clippy\n"
            "2026-10-08T19:10:00 | 90.0 | 0 | (full gate) | aaa | "
            "architecture=1 guardrails=165 integration=1572 browser=68\n"
            "2026-10-08T19:20:00 | 20.0 | 0 | integration | bbb+1234abcd | integration=1565\n"
            "2026-10-08T19:30:00 | 1.0 | 0 | clippy | bbb | -\n",
            encoding="utf-8",
        )
        self.assertEqual(
            build._last_tier_sizes(self.path),
            {
                "architecture": ("2026-10-08T19:10:00", "aaa", 1),
                "guardrails": ("2026-10-08T19:10:00", "aaa", 165),
                "integration": ("2026-10-08T19:20:00", "bbb+1234abcd", 1565),
                "browser": ("2026-10-08T19:10:00", "aaa", 68),
            },
        )

    def test_last_tier_sizes_without_journal_or_tier(self):
        self.assertEqual(build._last_tier_sizes(self.path), {})
        self.path.write_text(self.OLD_HEADER + "2026-10-08T19:00:00 | 1.0 | 0 | x\n")
        self.assertEqual(build._last_tier_sizes(self.path), {})

    def test_tests_column_follows_tier_order(self):
        sizes = {
            "Running integration tests...": ("integration", 1565),
            "Running architecture tests...": ("architecture", 1),
        }
        self.assertEqual(build._tests_column(sizes), "architecture=1 integration=1565")
        self.assertEqual(build._tests_column({}), "-")


class SizeSuffixTests(unittest.TestCase):
    """The epilogue suffix compares a tier's count with its newest record."""

    BASE = ("2026-10-08T19:46:00", "a1b2c3d4e5", 1572)

    def test_decrease(self):
        self.assertEqual(
            build._size_suffix(1565, self.BASE),
            "tests: 1565 (-7 vs 2026-10-08T19:46:00, tree a1b2c3d4e5)",
        )

    def test_increase(self):
        self.assertEqual(
            build._size_suffix(1575, self.BASE),
            "tests: 1575 (+3 vs 2026-10-08T19:46:00, tree a1b2c3d4e5)",
        )

    def test_same(self):
        self.assertEqual(
            build._size_suffix(1572, self.BASE),
            "tests: 1572 (same as 2026-10-08T19:46:00, tree a1b2c3d4e5)",
        )

    def test_no_record(self):
        self.assertEqual(build._size_suffix(68, None), "tests: 68 (no earlier record)")


class TreeIdentityTests(unittest.TestCase):
    """The tree name is the short HEAD, plus a content digest when dirty."""

    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.repo = Path(tmp.name)
        self.git("init", "-q")
        (self.repo / "a.txt").write_text("one\n")
        self.git("add", "a.txt")
        self.git(
            "-c", "user.email=t@example.com", "-c", "user.name=t",
            "commit", "-q", "-m", "init",
        )

    def git(self, *argv):
        return subprocess.run(
            ["git", "-C", str(self.repo), *argv], check=True, capture_output=True, text=True
        ).stdout.strip()

    def test_clean_tree_is_the_short_head(self):
        head = self.git("rev-parse", "--short=10", "HEAD")
        self.assertEqual(build._tree_identity(self.repo), head)

    def test_tracked_change_adds_a_digest(self):
        head = self.git("rev-parse", "--short=10", "HEAD")
        (self.repo / "a.txt").write_text("two\n")
        self.assertRegex(build._tree_identity(self.repo), rf"^{head}\+[0-9a-f]{{8}}$")

    def test_untracked_file_changes_the_digest(self):
        (self.repo / "a.txt").write_text("two\n")
        before = build._tree_identity(self.repo)
        (self.repo / "new.txt").write_text("x\n")
        with_new = build._tree_identity(self.repo)
        self.assertNotEqual(before, with_new)
        (self.repo / "new.txt").write_text("y\n")
        self.assertNotEqual(with_new, build._tree_identity(self.repo))

    def test_failed_blob_hash_keeps_a_dirty_name(self):
        head = self.git("rev-parse", "--short=10", "HEAD")
        (self.repo / "new.txt").write_text("x\n")
        real_run = subprocess.run

        def fail_hash_object(argv, **kwargs):
            if "hash-object" in argv:
                raise subprocess.CalledProcessError(128, argv)
            return real_run(argv, **kwargs)

        with mock.patch.object(build.subprocess, "run", side_effect=fail_hash_object):
            self.assertRegex(build._tree_identity(self.repo), rf"^{head}\+[0-9a-f]{{8}}$")

    def test_outside_a_repo_is_unknown(self):
        with tempfile.TemporaryDirectory() as plain:
            with mock.patch.dict(os.environ, {"GIT_CEILING_DIRECTORIES": plain}):
                self.assertEqual(build._tree_identity(Path(plain)), "unknown")
