"""Unit tests for build.py's gate plan and the run of a plan.

Run via ``python -m unittest discover scripts/tests`` or via ``build.py``.
"""

from __future__ import annotations

import re
import sys
import unittest
from pathlib import Path
from unittest import mock

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT))
sys.path.insert(0, str(Path(__file__).resolve().parent))

import build  # noqa: E402
from build_cli_support import gate_args, make_ctx  # noqa: E402


_REGENERATE_DOC_LABELS = ("Regenerating http_routes.md...", "Regenerating guardrails.md...")
_CHECK_DOC_LABELS = ("Checking http_routes.md freshness...", "Checking guardrails.md freshness...")


def labels(plan):
    return [step.label for step in plan]


class GatePlanTests(unittest.TestCase):
    """The gate plan is a list of typed steps; its length is the counter total."""

    def plan(self, **overrides):
        return build._plan_gate_steps(gate_args(**overrides))

    def by_label(self, **overrides):
        return {step.label: step for step in self.plan(**overrides)}

    def test_full_gate_has_20_steps_and_starts_with_the_housekeeping(self):
        plan = self.plan()
        self.assertEqual(len(plan), 20)
        self.assertEqual(
            labels(plan)[:3],
            [
                "Installing git hooks...",
                "Removing safe worktrees and pruning stale target dirs...",
                "Formatting...",
            ],
        )

    def test_housekeeping_and_duplicates_are_report_only(self):
        report_only = [step.label for step in self.plan() if step.report_only]
        self.assertEqual(
            report_only,
            [
                "Installing git hooks...",
                "Removing safe worktrees and pruning stale target dirs...",
                "Checking for duplicate code...",
            ],
        )
        # A report-only step never stops the run.
        self.assertFalse(any(s.check for s in self.plan() if s.report_only))

    def test_only_the_test_tiers_let_the_gate_go_on_after_a_failure(self):
        keep_going = [step.label for step in self.plan() if not step.check and not step.report_only]
        self.assertEqual(keep_going, ["Running integration tests...", "Running browser tests..."])

    def test_gate_splits_browser_from_integration(self):
        by_label = self.by_label()
        integration = by_label["Running integration tests..."].cmd
        for excluded in ("browser", "architecture", "guardrails"):
            self.assertIn(f"not binary({excluded})", integration)
        browser = by_label["Running browser tests..."].cmd
        self.assertIn("-E 'binary(browser)'", browser)
        self.assertNotIn("not binary(browser)", browser)

    def test_no_browser_drops_only_the_browser_tier(self):
        plan = self.plan(no_browser=True)
        self.assertNotIn("Running browser tests...", labels(plan))
        self.assertIn("Running integration tests...", labels(plan))
        self.assertEqual(len(plan), 19)

    def test_no_fmt_prunes_fmt_only(self):
        plan = self.plan(no_fmt=True)
        self.assertNotIn("Formatting...", labels(plan))
        self.assertIn("Installing git hooks...", labels(plan))
        self.assertEqual(len(plan), 19)

    def test_fmt_regenerates_the_two_generated_docs(self):
        self.assert_doc_steps(self.plan(), present=_REGENERATE_DOC_LABELS, absent=_CHECK_DOC_LABELS)

    def test_no_fmt_checks_the_two_generated_docs(self):
        self.assert_doc_steps(
            self.plan(no_fmt=True), present=_CHECK_DOC_LABELS, absent=_REGENERATE_DOC_LABELS
        )

    def assert_doc_steps(self, plan, *, present, absent):
        order = labels(plan)
        for label in present:
            self.assertIn(label, order)
            self.assertLess(order.index(label), order.index("Validating markdown docs..."))
        for label in absent:
            self.assertNotIn(label, order)

    def test_coverage_mode_swaps_test_and_report_steps(self):
        by_label = self.by_label(coverage=True)
        self.assertIn("Generating coverage report...", by_label)
        self.assertNotIn("Skipping coverage report (use --coverage to enable)", by_label)
        cov = by_label["Running integration tests with coverage..."].cmd
        self.assertIn("-E 'not binary(browser)'", cov)
        self.assertNotIn("binary(architecture)", cov)  # coverage keeps the full non-browser filter
        self.assertIn("Running browser tests with coverage...", by_label)

    def test_guardrails_run_after_the_asset_copy_and_before_the_tests(self):
        order = labels(self.plan())
        self.assertLess(
            order.index("Copying data and assets for deployment..."),
            order.index("Running guardrail tests..."),
        )
        self.assertLess(
            order.index("Running guardrail tests..."), order.index("Running integration tests...")
        )

    def test_timings_name_the_label_and_set_the_flag(self):
        tiers = [s for s in self.plan(test_timings=True) if s.timings]
        self.assertEqual(
            labels(tiers),
            [
                "Running integration tests and timings...",
                "Running browser tests and timings...",
            ],
        )

    def test_a_whole_tier_names_its_journal_column(self):
        tiers = {s.label: s.tier for s in self.plan() if s.tier}
        self.assertEqual(
            tiers,
            {
                "Running guardrail tests...": "guardrails",
                "Running integration tests...": "integration",
                "Running browser tests...": "browser",
            },
        )

    def test_other_test_sets_name_no_column(self):
        """Coverage and the LLM tests select another set, so the size would mislead."""
        coverage = [s.tier for s in self.plan(coverage=True) if s.timings is False and s.profile]
        self.assertEqual(coverage, ["guardrails", None, None])
        llm = {s.label: s.tier for s in self.plan(include_llm=True) if s.profile}
        self.assertIsNone(llm["Running integration tests..."])
        self.assertEqual(llm["Running browser tests..."], "browser")

    def test_a_nextest_step_names_the_profile_of_its_report(self):
        profiles = {s.label: s.profile for s in self.plan(include_llm=True) if s.profile}
        self.assertEqual(profiles["Running integration tests..."], "llm")
        self.assertEqual(profiles["Running browser tests..."], "default")
        self.assertEqual(profiles["Running guardrail tests..."], "default")
        self.assertEqual({s.profile for s in self.plan() if s.profile}, {"default"})
        self.assertEqual({s.profile for s in self.plan() if "tests" not in s.label}, {""})

    def test_the_fmt_steps_are_read_as_rustfmt_output(self):
        self.assertEqual(
            [s.output for s in self.plan() if s.label == "Formatting..."], [build.OUTPUT_RUSTFMT]
        )
        check = build._step_plan(build.parse_args(["clippy"]))[0]
        self.assertEqual(check.output, build.OUTPUT_RUSTFMT_CHECK)

    def test_llm_note_follows_the_first_tier_only_without_include_llm(self):
        for overrides, expect_note in (({}, True), ({"include_llm": True}, False), ({"coverage": True}, False)):
            with self.subTest(overrides=overrides):
                notes = [s for s in self.plan(**overrides) if s.after]
                self.assertEqual(len(notes), 1 if expect_note else 0)
                if expect_note:
                    self.assertIn("LLM tests were skipped", notes[0].after)
                    self.assertTrue(notes[0].label.startswith("Running integration"))


def fake_result(step, rc=0):
    return build.StepResult(step.label, rc=rc, report_only=step.report_only)


class RunPlanTests(unittest.TestCase):
    """_run_plan runs the steps in order, one result each, and stops at a fatal failure."""

    def run_plan(self, plan, failing=None):
        """Run a plan with step execution stubbed. ``failing`` maps a label to an exit code."""
        ctx = make_ctx(self)
        printed = []

        def fake_run(step, _ctx):
            rc = (failing or {}).get(step.label, 0)
            result = fake_result(step, rc)
            result.fatal = rc != 0 and step.check
            return result

        with mock.patch.object(build, "_run_step", side_effect=fake_run), mock.patch.object(
            build, "both_print", side_effect=lambda msg="": printed.append(msg)
        ):
            completed = build._run_plan(plan, ctx)
        return completed, ctx.results, printed

    def test_every_step_advances_the_counter_once_with_the_plan_length_as_total(self):
        plan = build._plan_gate_steps(gate_args())
        completed, results, printed = self.run_plan(plan)
        progress = [msg for msg in printed if re.match(r"^\[\d+/\d+\]", msg)]
        self.assertTrue(completed)
        self.assertEqual(len(progress), len(plan))
        self.assertEqual({msg.split("]")[0].split("/")[1] for msg in progress}, {str(len(plan))})
        # The skip note is counted but has no result.
        self.assertEqual(len(results), len(plan) - 1)

    def test_a_fatal_failure_stops_the_run(self):
        plan = build._plan_gate_steps(gate_args())
        completed, results, _ = self.run_plan(plan, failing={"Running clippy...": 101})
        self.assertFalse(completed)
        self.assertEqual(results[-1].label, "Running clippy...")
        self.assertTrue(results[-1].fatal)

    def test_a_failing_test_tier_lets_the_gate_go_on(self):
        plan = build._plan_gate_steps(gate_args())
        completed, results, _ = self.run_plan(
            plan, failing={"Running integration tests...": 100}
        )
        self.assertTrue(completed)
        self.assertEqual(results[-1].label, "Running browser tests...")
        self.assertEqual(build._exit_code(results), 1)

    def test_a_failing_guardrails_step_fails_the_gate_before_the_suite(self):
        plan = build._plan_gate_steps(gate_args())
        completed, results, _ = self.run_plan(plan, failing={"Running guardrail tests...": 100})
        self.assertFalse(completed)
        self.assertNotIn("Running integration tests...", [r.label for r in results])
        self.assertEqual(build._exit_code(results), 100)

    def test_step_notes_print_around_the_step(self):
        plan = [build.Step("Tests", "cargo nextest run", before="BEFORE", after="AFTER", check=False)]
        _, _, printed = self.run_plan(plan)
        self.assertEqual(printed, ["[1/1] Tests", "BEFORE", "AFTER"])


class RunStepTests(unittest.TestCase):
    """_run_step turns an exit code into a fatal failure, a warning, or a recorded failure."""

    def run_step(self, step, rc):
        ctx = make_ctx(self)
        printed = []
        with mock.patch.object(
            build, "_run_command", return_value=build._Run(rc, "", [])
        ), mock.patch.object(
            build, "both_print", side_effect=lambda msg="": printed.append(msg)
        ):
            return build._run_step(step, ctx), printed

    def test_a_checked_failure_is_fatal_and_says_so(self):
        result, printed = self.run_step(build.Step("Lint", "python lint.py"), 3)
        self.assertTrue(result.fatal)
        self.assertTrue(result.failed)
        self.assertIn("FAILED with code 3", printed)

    def test_an_unchecked_failure_is_recorded_without_stopping(self):
        result, _ = self.run_step(build.Step("Tests", "python t.py", check=False), 1)
        self.assertFalse(result.fatal)
        self.assertTrue(result.failed)

    def test_a_report_only_failure_warns_and_keeps_the_run_green(self):
        step = build.Step("Checking for duplicate code...", "python d.py", check=False, report_only=True)
        result, printed = self.run_step(step, 1)
        self.assertFalse(result.fatal)
        self.assertFalse(result.failed)
        self.assertTrue(any(msg.startswith("    Warning: Checking for duplicate code did not") for msg in printed))
        self.assertEqual(build._exit_code([result]), 0)
