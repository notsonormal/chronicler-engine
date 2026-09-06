"""Unit tests for build.py CLI parsing, the step registry, and log stamping.

Run via ``python -m unittest discover scripts/tests`` or via ``build.py``.
"""

from __future__ import annotations

import io
import os
import re
import sys
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT))

import build  # noqa: E402


class RegistryTests(unittest.TestCase):
    """The step registry is the single source of truth for step commands."""

    def test_expected_subcommands_exist(self):
        expected = {
            "fmt",
            "validate-data",
            "check",
            "clippy",
            "test-structure",
            "docstrings",
            "py-tests",
            "http-routes-check",
            "guardrails-doc-check",
            "validate-docs",
            "architecture",
            "guardrails",
            "unit",
            "integration",
            "nextest",
        }
        self.assertTrue(expected.issubset(build.REGISTRY), set(build.REGISTRY))

    def test_clippy_matches_full_gate(self):
        self.assertEqual(
            build.REGISTRY["clippy"].cmd,
            "cargo clippy --all-targets --all-features -- -D warnings",
        )

    def test_every_spec_has_label_and_cmd(self):
        for spec in build.REGISTRY.values():
            self.assertTrue(spec.label, spec.name)
            self.assertTrue(spec.cmd, spec.name)

    def test_gate_order_is_stable(self):
        gate_names = [name for name in build.GATE_ORDER if name in build.REGISTRY]
        self.assertEqual(gate_names[0], "fmt")
        self.assertIn("architecture", gate_names)
        self.assertIn("guardrails", gate_names)
        self.assertLess(
            gate_names.index("validate-docs"),
            gate_names.index("architecture"),
            "registry guardrail steps must run after the doc steps",
        )

    def test_nextest_pattern_spec_takes_pattern(self):
        self.assertTrue(build.REGISTRY["nextest"].pattern_arg)
        self.assertFalse(build.REGISTRY["clippy"].pattern_arg)


class ParseArgsTests(unittest.TestCase):
    """parse_args is pure and enforces mode exclusivity at parse time."""

    def test_no_subcommand_is_full_gate(self):
        self.assertIsNone(build.parse_args([]).command)

    def test_step_subcommand(self):
        self.assertEqual(build.parse_args(["clippy"]).command, "clippy")

    def test_unknown_subcommand_rejected(self):
        with self.assertRaises(SystemExit) as ctx:
            build.parse_args(["bogus"])
        self.assertEqual(ctx.exception.code, 2)

    def test_nextest_pattern_required(self):
        with self.assertRaises(SystemExit) as ctx:
            build.parse_args(["nextest"])
        self.assertEqual(ctx.exception.code, 2)

    def test_nextest_pattern_value(self):
        args = build.parse_args(["nextest", "behaviour::test_x"])
        self.assertEqual(args.pattern, "behaviour::test_x")

    def test_gate_flag_after_step_rejected(self):
        with self.assertRaises(SystemExit):
            build.parse_args(["clippy", "--coverage"])

    def test_gate_flag_before_step_rejected(self):
        with self.assertRaises(SystemExit):
            build.parse_args(["--coverage", "clippy"])

    def test_target_dir_before_step(self):
        args = build.parse_args(["--target-dir", "target/agent2", "clippy"])
        self.assertEqual(args.target_dir, "target/agent2")

    def test_target_dir_after_step(self):
        args = build.parse_args(["clippy", "--target-dir", "target/agent2"])
        self.assertEqual(args.target_dir, "target/agent2")

    def test_strict_after_step(self):
        self.assertTrue(build.parse_args(["clippy", "--strict"]).strict)

    def test_strict_before_step(self):
        self.assertTrue(build.parse_args(["--strict", "clippy"]).strict)


class StepCommandTests(unittest.TestCase):
    """Step command assembly, including nextest pattern quoting."""

    # Every registry command pinned verbatim: a typo in any spec's cmd fails
    # the suite instead of silently changing what a subcommand runs.
    EXPECTED_COMMANDS = {
        "fmt": "cargo fmt",
        "validate-data": "python scripts/validate_data.py",
        "check": "cargo check --all-targets --all-features",
        "clippy": "cargo clippy --all-targets --all-features -- -D warnings",
        "test-structure": "python scripts/check_test_structure.py",
        "docstrings": "python scripts/check_python_docstrings.py",
        "py-tests": "python -m unittest discover scripts/tests -v",
        "http-routes-check": "python scripts/extract_http_routes.py --check",
        "guardrails-doc-check": "python scripts/generate_guardrails_doc.py --check",
        "validate-docs": "python scripts/validate_docs.py",
        "architecture": "cargo nextest run --no-fail-fast --test architecture",
        "guardrails": "cargo nextest run --no-fail-fast --test guardrails",
        "unit": "cargo test --lib",
        "integration": "cargo nextest run --no-fail-fast --tests",
        "nextest": "cargo nextest run --no-fail-fast",
    }

    def test_every_spec_command_pinned(self):
        actual = {name: spec.cmd for name, spec in build.REGISTRY.items()}
        self.assertEqual(actual, self.EXPECTED_COMMANDS)

    def test_plain_spec_returns_cmd_verbatim(self):
        self.assertEqual(
            build._step_command(build.REGISTRY["clippy"]),
            "cargo clippy --all-targets --all-features -- -D warnings",
        )

    def test_pattern_is_shell_quoted(self):
        cmd = build._step_command(build.REGISTRY["nextest"], "my test")
        self.assertEqual(cmd, "cargo nextest run --no-fail-fast 'my test'")


class CargoEnvTests(unittest.TestCase):
    def test_target_dir_sets_env_var(self):
        env = build._cargo_env_for(SimpleNamespace(target_dir="target/agent2"))
        self.assertIn("CARGO_TARGET_DIR", env)

    def test_no_target_dir_keeps_env_clean(self):
        env = build._cargo_env_for(SimpleNamespace(target_dir=None))
        self.assertNotIn("CARGO_TARGET_DIR", env)
        self.assertEqual(env["NEXTEST_STATUS_LEVEL"], "fail")


class GatePlanTests(unittest.TestCase):
    """The gate plan derives its step count from GATE_ORDER expansion."""

    def gate_args(self, **overrides):
        defaults = dict(
            no_fmt=False,
            coverage=False,
            include_llm=False,
            test_timings=False,
            release=False,
            target_dir=None,
        )
        defaults.update(overrides)
        return SimpleNamespace(**defaults)

    def test_full_gate_has_fmt_first_and_14_steps(self):
        plan = build._plan_gate_steps(self.gate_args())
        self.assertEqual(plan[0].label, "Formatting...")
        self.assertEqual(len(plan), 14)

    def test_no_fmt_prunes_fmt_only(self):
        plan = build._plan_gate_steps(self.gate_args(no_fmt=True))
        labels = [step.label for step in plan]
        self.assertNotIn("Formatting...", labels)
        self.assertEqual(len(plan), 13)

    def test_coverage_mode_swaps_test_and_report_steps(self):
        plan = build._plan_gate_steps(self.gate_args(coverage=True))
        labels = [step.label for step in plan]
        self.assertIn("Running all tests with coverage...", labels)
        self.assertIn("Generating coverage report...", labels)
        self.assertNotIn(
            "Skipping coverage report (use --coverage to enable)", labels
        )

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


class SessionStampTests(unittest.TestCase):
    """Build logs carry the originating pi session id when known."""

    def setUp(self):
        self.buffer = io.StringIO()
        build._set_log_fh(self.buffer)

    def tearDown(self):
        build._set_log_fh(None)

    def test_stamp_written_when_session_id_present(self):
        with mock.patch.dict(
            os.environ, {"PI_SESSION_ID": "0123abcd-0000-1111-2222-333344445555"}
        ):
            build._stamp_session_id()
        self.assertIn(
            "Session-Id: 0123abcd-0000-1111-2222-333344445555",
            self.buffer.getvalue(),
        )

    def test_no_stamp_without_session_id(self):
        env = {key: value for key, value in os.environ.items() if key != "PI_SESSION_ID"}
        with mock.patch.dict(os.environ, env, clear=True):
            build._stamp_session_id()
        self.assertEqual(self.buffer.getvalue(), "")


class NextestSummaryLineTests(unittest.TestCase):
    """_nextest_summary_line renders nextest's Summary as a compact one-liner."""

    def test_single_test_pass_summary(self):
        output = "     Summary [  21.153s] 1 test run: 1 passed, 0 skipped\n"
        self.assertEqual(
            build._nextest_summary_line(output), "nextest: 1 passed, 0 failed"
        )

    def test_fail_summary_omits_zero_skipped(self):
        output = (
            "     Summary [   1.497s] 119 tests run: 118 passed, 1 failed, 0 skipped\n"
        )
        self.assertEqual(
            build._nextest_summary_line(output), "nextest: 118 passed, 1 failed"
        )

    def test_nonzero_skipped_is_kept(self):
        output = "     Summary [   2.000s] 26 tests run: 21 passed, 0 failed, 5 skipped\n"
        self.assertEqual(
            build._nextest_summary_line(output),
            "nextest: 21 passed, 0 failed, 5 skipped",
        )

    def test_failed_segment_absent_means_zero_failed(self):
        # Real nextest output omits the "failed" segment on clean runs.
        output = "     Summary [   0.174s] 21 tests run: 21 passed, 1462 skipped\n"
        self.assertEqual(
            build._nextest_summary_line(output),
            "nextest: 21 passed, 0 failed, 1462 skipped",
        )

    def test_last_summary_line_wins(self):
        output = (
            "     Summary [   1.000s] 2 tests run: 1 passed, 1 failed\n"
            "     Summary [   3.000s] 4 tests run: 4 passed, 0 failed\n"
        )
        self.assertEqual(
            build._nextest_summary_line(output), "nextest: 4 passed, 0 failed"
        )

    def test_no_summary_returns_none(self):
        self.assertIsNone(build._nextest_summary_line("error: could not compile\n"))

    def test_compilation_output_without_summary_returns_none(self):
        output = "   Compiling chronicler-engine v0.1.0\n     Running unittests\n"
        self.assertIsNone(build._nextest_summary_line(output))


class NextestStashTests(unittest.TestCase):
    """run() stashes the summary even when a checked failure sys.exits."""

    def setUp(self):
        build._NextestSummary.line = None

    def tearDown(self):
        build._NextestSummary.line = None

    @staticmethod
    def _fake_process(out, returncode):
        fake = mock.Mock()
        fake.communicate.return_value = (out, None)
        fake.returncode = returncode
        return fake

    def _run_silenced(self, out, returncode, check=True):
        fake = self._fake_process(out, returncode)
        with mock.patch.object(
            build.subprocess, "Popen", return_value=fake
        ), mock.patch("builtins.print"):
            return build.run("cargo nextest run", check=check)

    def test_run_stashes_summary_on_success(self):
        rc = self._run_silenced(
            "     Summary [  21.153s] 1 test run: 1 passed, 0 skipped\n", 0
        )
        self.assertEqual(rc, 0)
        self.assertEqual(build._NextestSummary.line, "nextest: 1 passed, 0 failed")

    def test_run_stashes_summary_on_checked_failure(self):
        with self.assertRaises(SystemExit):
            self._run_silenced(
                "     Summary [   1.497s] 3 tests run: 2 passed, 1 failed\n", 1
            )
        self.assertEqual(build._NextestSummary.line, "nextest: 2 passed, 1 failed")

    def test_run_without_summary_leaves_stash_unset(self):
        with self.assertRaises(SystemExit):
            self._run_silenced("error: could not compile\n", 101)
        self.assertIsNone(build._NextestSummary.line)


class NextestEpilogueTests(unittest.TestCase):
    """The epilogue prints the one-liner directly before the build banner."""

    def setUp(self):
        build._NextestSummary.line = None

    def tearDown(self):
        build._NextestSummary.line = None

    @staticmethod
    def _run_main(printed):
        memlog = _MemLog()
        gate_args = SimpleNamespace(
            command=None,
            cleanup=False,
            diagnostic_benchmark=False,
            llm_only=False,
        )
        with mock.patch.object(
            build, "parse_args", return_value=gate_args
        ), mock.patch.object(build, "run_gate"), mock.patch.object(
            build, "clean_old_logs"
        ), mock.patch(
            "builtins.print", side_effect=lambda msg="": printed.append(msg)
        ), mock.patch(
            "builtins.open", return_value=memlog
        ):
            return build.main()

    def test_summary_printed_between_step_summary_and_banner(self):
        printed = []
        build._NextestSummary.line = "nextest: 21 passed, 0 failed"
        self.assertEqual(self._run_main(printed), 0)
        idx = printed.index("nextest: 21 passed, 0 failed")
        self.assertEqual(printed[idx + 1], "=" * 60)
        self.assertEqual(printed[idx + 2], "=== Build Complete ===")

    def test_no_summary_prints_no_extra_line(self):
        printed = []
        self.assertEqual(self._run_main(printed), 0)
        self.assertFalse(any(msg.startswith("nextest:") for msg in printed))


class _MemLog(io.StringIO):
    """Log handle that survives main()'s close so tests can read it back."""

    def close(self):
        self.flush()


class MainStampTests(unittest.TestCase):
    """The session stamp is the first line written to every build log."""

    def test_stamp_is_first_log_line_in_main(self):
        memlog = _MemLog()
        gate_args = SimpleNamespace(
            command=None,
            cleanup=False,
            diagnostic_benchmark=False,
            llm_only=False,
        )
        with mock.patch.dict(
            os.environ, {"PI_SESSION_ID": "0123abcd-0000-1111-2222-333344445555"}
        ), mock.patch.object(build, "parse_args", return_value=gate_args), mock.patch.object(
            build, "run_gate"
        ), mock.patch.object(
            build, "clean_old_logs"
        ), mock.patch(
            "builtins.print"
        ), mock.patch(
            "builtins.open", return_value=memlog
        ):
            rc = build.main()
        self.assertEqual(rc, 0)
        first_line = memlog.getvalue().splitlines()[0]
        self.assertEqual(first_line, "Session-Id: 0123abcd-0000-1111-2222-333344445555")


if __name__ == "__main__":
    unittest.main()
