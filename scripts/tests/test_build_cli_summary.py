"""Unit tests for how build.py reads a finished step, and for its epilogue and log stamp.

The step tests run ``_run_step`` against a fake ``subprocess.run`` that answers with output
recorded from cargo and nextest (``fixtures/build_output/``).

Run via ``python -m unittest discover scripts/tests`` or via ``build.py``.
"""

from __future__ import annotations

import io
import os
import sys
import unittest
from pathlib import Path
from unittest import mock

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT))
sys.path.insert(0, str(Path(__file__).resolve().parent))

import build  # noqa: E402
from build_cli_support import (  # noqa: E402
    fake_subprocess,
    fixture,
    junit_path,
    make_ctx,
    run_main,
)


class SessionStampTests(unittest.TestCase):
    """Build logs carry the originating pi session id when known."""

    def test_stamp_written_when_session_id_present(self):
        buffer = io.StringIO()
        build._set_log_fh(buffer)
        self.addCleanup(build._set_log_fh, None)
        with mock.patch.dict(os.environ, {"PI_SESSION_ID": "0123abcd-0000-1111-2222-333344445555"}):
            build._stamp_session_id()
        self.assertEqual(buffer.getvalue(), "Session-Id: 0123abcd-0000-1111-2222-333344445555\n")

    def test_no_stamp_without_session_id(self):
        buffer = io.StringIO()
        build._set_log_fh(buffer)
        self.addCleanup(build._set_log_fh, None)
        env = {key: value for key, value in os.environ.items() if key != "PI_SESSION_ID"}
        with mock.patch.dict(os.environ, env, clear=True):
            build._stamp_session_id()
        self.assertEqual(buffer.getvalue(), "")

    def test_stamp_is_first_log_line_in_main(self):
        with mock.patch.dict(os.environ, {"PI_SESSION_ID": "0123abcd-0000-1111-2222-333344445555"}):
            _, log, code, _ = run_main()
        self.assertEqual(code, 0)
        self.assertEqual(log.splitlines()[0], "Session-Id: 0123abcd-0000-1111-2222-333344445555")


class JsonOutputTests(unittest.TestCase):
    """cargo compile and test commands run with JSON messages; the rest run unchanged."""

    def test_the_flag_goes_right_after_the_subcommand(self):
        cases = {
            "cargo clippy --all-targets -- -D warnings": (
                "cargo clippy --message-format json --all-targets -- -D warnings"
            ),
            "cargo check --all-targets": "cargo check --message-format json --all-targets",
            "cargo build --bin chronicler_engine": (
                "cargo build --message-format json --bin chronicler_engine"
            ),
            "cargo nextest run -E 'x' --fail-fast": (
                "cargo nextest run --cargo-message-format json -E 'x' --fail-fast"
            ),
            "cargo llvm-cov nextest --no-report": (
                "cargo llvm-cov nextest --cargo-message-format json --no-report"
            ),
        }
        for cmd, expected in cases.items():
            with self.subTest(cmd=cmd):
                self.assertEqual(build._with_json_output(cmd), expected)

    def test_other_commands_are_left_alone(self):
        for cmd in (
            "cargo fmt --check -- -l",
            "cargo llvm-cov report --json",
            "python scripts/validate_docs.py",
        ):
            with self.subTest(cmd=cmd):
                self.assertIsNone(build._with_json_output(cmd))


class RunStepReadsOutputTests(unittest.TestCase):
    """What _run_step keeps from the output of the command it ran."""

    def setUp(self):
        self.ctx = make_ctx(self)
        patcher = mock.patch.object(build, "both_print")
        patcher.start()
        self.addCleanup(patcher.stop)

    def nextest_step(self, cmd="cargo nextest run --no-fail-fast", profile="default"):
        return build.Step("Running tests...", cmd, profile=profile, check=False)

    def test_a_red_nextest_run_names_its_tests_counts_and_leaks(self):
        with fake_subprocess(
            stderr=fixture("nextest_failed.stderr"),
            rc=100,
            junit="junit_failed.xml",
            junit_path=junit_path(),
        ):
            result = build._run_step(self.nextest_step(), self.ctx)
        self.assertEqual(result.rc, 100)
        self.assertEqual(
            result.counts.line, "nextest: 3 passed, 1 failed, 1 skipped, 1 leaky, 1 timed out"
        )
        self.assertEqual(result.leaks, [("LEAK", "probe::leak leaks_a_child")])
        self.assertEqual(
            result.failures,
            [
                ("probe t::fails", "assertion `left == right` failed: the real message"),
                ("probe t::times_out", "test timeout"),
            ],
        )
        self.assertEqual(result.tail, [])  # nextest names its own cause

    def test_a_report_left_by_an_earlier_run_is_not_read(self):
        stale = junit_path()
        stale.parent.mkdir(parents=True)
        stale.write_text(fixture("junit_failed.xml"))
        with fake_subprocess(stderr=fixture("nextest_failed.stderr"), rc=100):
            result = build._run_step(self.nextest_step(), self.ctx)
        self.assertEqual(result.failures, [])
        self.assertFalse(stale.exists())

    def test_target_dir_does_not_move_the_report(self):
        """nextest ignores --target-dir for the report, so build.py reads the same file."""
        ctx = make_ctx(self, target_dir="/elsewhere/target/agent2")
        with fake_subprocess(
            stderr=fixture("nextest_failed.stderr"),
            rc=100,
            junit="junit_failed.xml",
            junit_path=junit_path(),
        ):
            result = build._run_step(self.nextest_step(), ctx)
        self.assertEqual(len(result.failures), 2)
        self.assertEqual(build._junit_path("default"), build._NEXTEST_DIR / "default" / "junit.xml")
        self.assertNotIn("elsewhere", str(build._junit_path("default")))

    def test_the_llm_profile_reads_its_own_report(self):
        cmd = "cargo nextest run --profile llm --run-ignored all --no-fail-fast --test llm"
        with fake_subprocess(
            stderr=fixture("nextest_failed.stderr"),
            rc=100,
            junit="junit_failed.xml",
            junit_path=junit_path("llm"),
        ):
            result = build._run_step(self.nextest_step(cmd, "llm"), self.ctx)
        self.assertEqual(len(result.failures), 2)

    def test_a_green_run_keeps_counts_without_reading_the_report(self):
        with fake_subprocess(
            stderr="     Summary [ 1.0s] 4 tests run: 4 passed, 1 skipped\n",
            junit="junit_failed.xml",
            junit_path=junit_path(),
        ):
            result = build._run_step(self.nextest_step(), self.ctx)
        self.assertEqual(result.counts.line, "nextest: 4 passed, 0 failed, 1 skipped")
        self.assertEqual(result.failures, [])

    def test_a_compile_error_gives_each_error_once_and_the_rendered_text_to_the_log(self):
        log = []
        step = build.Step("Checking compilation...", "cargo check --all-targets")
        with fake_subprocess(
            stdout=fixture("cargo_check_errors.json"),
            stderr=fixture("cargo_check_errors.stderr"),
            rc=101,
        ), mock.patch.object(build, "log_status", side_effect=log.append):
            result = build._run_step(step, self.ctx)
        self.assertEqual(result.compile_errors, ["error[E0308]: mismatched types  (src/lib.rs:6:5)"])
        self.assertTrue(result.fatal)
        self.assertEqual(result.tail, [])
        logged = "\n".join(log)
        self.assertIn("error[E0308]: mismatched types", logged)
        self.assertIn("error: could not compile `probe` (lib)", logged)
        self.assertNotIn('"reason"', logged)
        self.assertNotIn("Checking probe", logged)  # cargo progress stays out of the log

    def test_the_echo_names_the_command_that_runs(self):
        printed = []
        with fake_subprocess(), mock.patch.object(
            build, "both_print", side_effect=lambda msg="": printed.append(msg)
        ):
            build._run_command("cargo nextest run -E 'x'", {})
            build._run_command("cargo fmt", {})
        self.assertEqual(
            printed,
            [
                "$ cargo nextest run --cargo-message-format json -E 'x'",
                "$ cargo fmt",
            ],
        )

    def test_cargo_runs_with_split_streams_and_other_commands_with_merged_ones(self):
        calls = []
        with mock.patch.object(
            build.subprocess,
            "run",
            side_effect=lambda cmd, **kw: calls.append((cmd, kw["stderr"])) or mock.Mock(
                returncode=0, stdout="", stderr=""
            ),
        ):
            build._run_command("cargo check", {})
            build._run_command("python x.py", {})
        self.assertEqual(calls[0], ("cargo check --message-format json", build.subprocess.PIPE))
        self.assertEqual(calls[1], ("python x.py", build.subprocess.STDOUT))

    def test_a_failed_plain_step_keeps_its_last_three_lines(self):
        step = build.Step("Checking guardrails.md freshness...", "python scripts/x.py")
        with fake_subprocess(stdout="one\n\ntwo\nthree\nfour\nfix: python scripts/x.py\n\n", rc=1):
            result = build._run_step(step, self.ctx)
        self.assertEqual(result.tail, ["three", "four", "fix: python scripts/x.py"])

    def test_unformatted_files_are_named_with_the_fix(self):
        step = build.Step(
            build._FORMAT_CHECK_LABEL, build._FORMAT_CHECK_CMD, output=build.OUTPUT_RUSTFMT_CHECK
        )
        with fake_subprocess(stdout=fixture("fmt_list.txt"), rc=1), mock.patch.object(
            build.build_report.os.path, "relpath", side_effect=lambda p: p[len("/work/") :]
        ):
            result = build._run_step(step, self.ctx)
        self.assertEqual(result.tail, ["unformatted: probe/src/lib.rs", "fix: python build.py fmt"])

    def test_a_rustfmt_syntax_error_joins_the_compile_errors_with_its_location(self):
        for output, step_cmd in (
            (build.OUTPUT_RUSTFMT_CHECK, build._FORMAT_CHECK_CMD),
            (build.OUTPUT_RUSTFMT, "cargo fmt"),
        ):
            with self.subTest(output=output):
                step = build.Step("Formatting...", step_cmd, output=output)
                with fake_subprocess(stdout=fixture("fmt_syntax_error.txt"), rc=1), mock.patch.object(
                    build.build_report.os.path, "relpath", side_effect=lambda p: p[len("/work/") :]
                ):
                    result = build._run_step(step, self.ctx)
                self.assertEqual(
                    result.compile_errors,
                    ["error: this file contains an unclosed delimiter  (probe/src/lib.rs:5:3)"],
                )
                self.assertEqual(result.tail, [])

    def test_a_plain_step_is_not_scanned_for_error_lines(self):
        step = build.Step("Validating...", "python v.py")
        with fake_subprocess(stdout="error: bad data\n", rc=1):
            result = build._run_step(step, self.ctx)
        self.assertEqual((result.compile_errors, result.tail), ([], ["error: bad data"]))

    def test_more_than_ten_unformatted_files_are_counted(self):
        text = "".join(f"/work/probe/src/f{n}.rs\n" for n in range(12))
        with mock.patch.object(build.build_report.os.path, "relpath", side_effect=lambda p: p):
            detail = build._unformatted_detail(text)
        self.assertEqual(len(detail), 12)
        self.assertEqual(detail[10:], ["... 2 more files", "fix: python build.py fmt"])

    def test_a_passing_step_leaves_no_failure_facts(self):
        with fake_subprocess(stdout="all good\n"):
            result = build._run_step(build.Step("Validating...", "python v.py"), self.ctx)
        self.assertEqual((result.rc, result.tail, result.failures, result.counts), (0, [], [], None))


class TestTimingReportTests(unittest.TestCase):
    """--test-timings prints the durations and the stub checks from the JUnit report."""

    def test_the_report_lists_the_slowest_tests_binaries_and_stub_checks(self):
        ctx = make_ctx(self)
        printed = []
        step = build.Step(
            "Running tests...", "cargo nextest run", profile="default", check=False, timings=True
        )
        with fake_subprocess(
            stderr=fixture("nextest_failed.stderr"),
            rc=100,
            junit="junit_failed.xml",
            junit_path=junit_path(),
        ), mock.patch.object(build, "both_print", side_effect=lambda msg="": printed.append(msg)):
            build._run_step(step, ctx)
        self.assertIn("$ cargo nextest run --cargo-message-format json  (with --test-timings)", printed)
        self.assertIn("  Slowest tests (of 5 measured, top 30):", printed)
        slowest = printed[printed.index("  Slowest tests (of 5 measured, top 30):") + 1]
        self.assertTrue(slowest.endswith("s  probe t::times_out"), slowest)
        self.assertTrue(any(line.endswith("s  probe::leak") for line in printed))
        self.assertIn("  Slowest stub checks (of 1, top 30):", printed)
        self.assertIn("       0.902s  [OK]  check_x  (run_checks)", printed)

    def test_a_missing_report_says_so(self):
        printed = []
        with mock.patch.object(build, "both_print", side_effect=lambda msg="": printed.append(msg)):
            build._print_timing_report(None)
        self.assertIn("  No per-test durations in the nextest JUnit report.", printed)


def failed_tests_result(label="Running guardrail tests...", **kwargs):
    return build.StepResult(
        label,
        rc=100,
        failures=[("chronicler_engine::guardrails long_comment_runs", "gate.rs:338 [WARN] Long comment run")],
        **kwargs,
    )


class EpilogueTests(unittest.TestCase):
    """main() prints the verdict lines in order, right before the closing banner."""

    def epilogue(self, results, baselines=None, **overrides):
        def gate(_args, out):
            out.extend(results)

        return run_main(gate, baselines=baselines, **overrides)

    def test_counts_print_between_the_step_table_and_the_banner(self):
        counts = build.build_report.parse_summary(fixture("nextest_cancelled.stderr"))
        result = build.StepResult("Running integration tests...", counts=counts, tier="integration")
        printed, _, code, _ = self.epilogue([result])
        self.assertEqual(code, 0)
        idx = printed.index(
            "nextest: 1 passed, 1 failed, 1 skipped, 3 not run  (Running integration tests...)"
            "  tests: 6 (no earlier record)"
        )
        self.assertEqual(printed[idx + 1 : idx + 3], ["=" * 60, "=== Build Complete ==="])

    def test_a_run_that_is_not_a_whole_tier_has_no_count_change(self):
        counts = build.build_report.parse_summary(fixture("nextest_cancelled.stderr"))
        printed, *_ = self.epilogue([build.StepResult("Running tests...", counts=counts)])
        self.assertIn("nextest: 1 passed, 1 failed, 1 skipped, 3 not run  (Running tests...)", printed)

    def test_the_count_change_compares_with_the_newest_record(self):
        counts = build.build_report.parse_summary(
            "     Summary [ 1s] 1563 tests run: 1563 passed, 2 skipped\n"
        )
        baseline = build.build_history.TierRecord("2026-10-08T19:46:00", "a1b2c3d4e5", 1572)
        result = build.StepResult("Running integration tests...", counts=counts, tier="integration")
        printed, *_ = self.epilogue([result], baselines={"integration": baseline})
        self.assertIn(
            "nextest: 1563 passed, 0 failed, 2 skipped  (Running integration tests...)"
            "  tests: 1565 (-7 vs 2026-10-08T19:46:00, tree a1b2c3d4e5)",
            printed,
        )

    def test_a_leaking_test_is_named(self):
        result = build.StepResult("Running browser tests...", leaks=[("LEAK", "probe::leak leaks_a_child")])
        printed, *_ = self.epilogue([result])
        self.assertIn("leak test: probe::leak leaks_a_child  (Running browser tests...)", printed)

    def test_the_failures_block_names_each_test_with_its_message(self):
        result = failed_tests_result()
        printed, _, code, _ = self.epilogue([result])
        self.assertEqual(code, 1)
        idx = printed.index("--- Failures ---")
        self.assertEqual(
            printed[idx + 1 : idx + 4],
            [
                "  Running guardrail tests...",
                "    FAIL chronicler_engine::guardrails long_comment_runs",
                "      gate.rs:338 [WARN] Long comment run",
            ],
        )

    def test_the_failures_block_gives_the_cause_of_a_failed_plain_step(self):
        result = build.StepResult("Checking guardrails.md freshness...", rc=1, tail=["stale", "run: x"])
        printed, *_ = self.epilogue([result])
        idx = printed.index("--- Failures ---")
        self.assertEqual(
            printed[idx + 1 : idx + 4],
            ["  Checking guardrails.md freshness...", "    stale", "    run: x"],
        )

    def test_the_failures_block_is_bounded(self):
        result = build.StepResult(
            "Running integration tests...", rc=100, failures=[(f"bin::t{n}", "") for n in range(25)]
        )
        printed, *_ = self.epilogue([result])
        self.assertEqual(sum(1 for msg in printed if msg.startswith("    FAIL ")), 20)
        self.assertIn("  ... 5 more, see the log", printed)

    def test_a_green_run_and_a_report_only_warning_print_no_failures_block(self):
        warned = build.StepResult("Checking for duplicate code...", rc=1, report_only=True, tail=["x"])
        printed, _, code, _ = self.epilogue([warned, build.StepResult("Formatting...")])
        self.assertEqual(code, 0)
        self.assertNotIn("--- Failures ---", printed)
        self.assertIn("    0.00s  [WARN]  Checking for duplicate code...  (report-only)", printed)

    def test_compile_errors_print_before_the_banner_without_repeats_and_bounded(self):
        results = [
            build.StepResult("Checking compilation...", rc=101, compile_errors=[f"error: e{n}" for n in range(7)]),
            build.StepResult("Running clippy...", rc=101, compile_errors=["error: e0"]),
        ]
        printed, *_ = self.epilogue(results)
        idx = printed.index("compile errors (first 5 of 7):")
        self.assertEqual(printed[idx + 5], "  error: e4")
        self.assertEqual(printed[idx + 6], "=" * 60)

    def test_no_browser_gate_names_the_skipped_tier(self):
        printed, *_ = self.epilogue([], no_browser=True)
        idx = printed.index('skipped: browser tier (run "python build.py browser")')
        self.assertEqual(printed[idx + 1], "=" * 60)

    def test_a_clean_empty_run_prints_no_verdict_lines(self):
        printed, *_ = self.epilogue([])
        self.assertFalse(any(msg.startswith(("nextest:", "skipped:", "compile errors")) for msg in printed))

    def test_step_table_marks_ok_failed_and_the_total(self):
        results = [
            build.StepResult("Formatting...", elapsed=1.5),
            failed_tests_result(elapsed=2.0),
        ]
        printed, *_ = self.epilogue(results)
        self.assertIn("    1.50s  [OK]  Formatting...", printed)
        self.assertIn("    2.00s  [FAILED]  Running guardrail tests...", printed)
        self.assertIn("           Total: 3.50s", printed)
        self.assertIn("\n  Failed steps: Running guardrail tests...", printed)


class ExitCodeTests(unittest.TestCase):
    def test_a_fatal_failure_exits_with_its_own_code(self):
        results = [build.StepResult("a", rc=100), build.StepResult("b", rc=101, fatal=True)]
        self.assertEqual(build._exit_code(results), 101)

    def test_a_recorded_test_failure_exits_one(self):
        self.assertEqual(build._exit_code([build.StepResult("a", rc=100)]), 1)

    def test_a_report_only_failure_exits_zero(self):
        self.assertEqual(build._exit_code([build.StepResult("a", rc=1, report_only=True)]), 0)

    def test_tier_sizes_cover_only_the_whole_tiers(self):
        counts = build.build_report.parse_summary("     Summary [ 1s] 1570 tests run: 1570 passed, 2 skipped\n")
        results = [
            build.StepResult("integration", counts=counts, tier="integration"),
            build.StepResult("pattern", counts=counts),
        ]
        self.assertEqual(build._tier_sizes(results), {"integration": 1572})
