"""Unit tests for build.py's nextest summary, epilogue and log stamps.

Run via ``python -m unittest discover scripts/tests`` or via ``build.py``.
"""

from __future__ import annotations

import io
import os
import sys
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT))
sys.path.insert(0, str(Path(__file__).resolve().parent))

import build  # noqa: E402
from build_cli_support import _MemLog, reset_epilogue_state  # noqa: E402

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

    def test_leaky_count_is_kept(self):
        # Recorded from logs/build_20261003_021636.log:309 (the browser tier).
        output = (
            "     Summary [  60.628s] 30 tests run: 30 passed (1 leaky), 0 skipped\n"
        )
        self.assertEqual(
            build._nextest_summary_line(output),
            "nextest: 30 passed, 0 failed, 1 leaky",
        )

    def test_leaky_count_survives_a_skipped_segment(self):
        output = "     Summary [  10.000s] 12 tests run: 9 passed (2 leaky), 1 failed, 3 skipped\n"
        self.assertEqual(
            build._nextest_summary_line(output),
            "nextest: 9 passed, 1 failed, 3 skipped, 2 leaky",
        )

    def test_flaky_count_is_kept(self):
        output = (
            "     Summary [  60.628s] 30 tests run: 28 passed, 1 flaky, 1 failed, 0 skipped\n"
        )
        self.assertEqual(
            build._nextest_summary_line(output),
            "nextest: 28 passed, 1 failed, 1 flaky",
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


class NextestResultLineTests(unittest.TestCase):
    """_NEXTEST_RESULT_RE drives the per-test timing report."""

    def test_leak_line_is_parsed_with_its_duration_and_name(self):
        line = "        LEAK [   0.229s] ( 2/2) leakprobe::leak leaks_a_child"
        match = build._NEXTEST_RESULT_RE.match(line)
        self.assertIsNotNone(match)
        self.assertEqual(build._nextest_duration_to_secs(match.group(1)), 0.229)
        self.assertEqual(match.group(2).strip(), "leakprobe::leak leaks_a_child")

    def test_pass_line_still_parses_after_adding_leak(self):
        line = "        PASS [  18.645s] ( 4/26) chronicler_engine::browser behaviour::test_x"
        match = build._NEXTEST_RESULT_RE.match(line)
        self.assertIsNotNone(match)
        self.assertEqual(match.group(2).strip(), "chronicler_engine::browser behaviour::test_x")


class NextestStashTests(unittest.TestCase):
    """run() stashes the summary even when a checked failure sys.exits."""

    def setUp(self):
        reset_epilogue_state()

    def tearDown(self):
        reset_epilogue_state()

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
        self.assertEqual(
            build._NextestSummary.lines, [("", "nextest: 1 passed, 0 failed")]
        )

    def test_run_stashes_summary_on_checked_failure(self):
        with self.assertRaises(SystemExit):
            self._run_silenced(
                "     Summary [   1.497s] 3 tests run: 2 passed, 1 failed\n", 1
            )
        self.assertEqual(
            build._NextestSummary.lines, [("", "nextest: 2 passed, 1 failed")]
        )

    def test_run_without_summary_leaves_stash_unset(self):
        with self.assertRaises(SystemExit):
            self._run_silenced("error: could not compile\n", 101)
        self.assertEqual(build._NextestSummary.lines, [])

    def test_run_names_a_leaky_test(self):
        build._NextestSummary.label = "Running browser tests..."
        self._run_silenced(
            "        LEAK [   1.229s] ( 2/63) chronicler_engine::browser behaviour::test_x\n"
            "     Summary [ 130.000s] 63 tests run: 63 passed (1 leaky), 0 skipped\n",
            0,
        )
        self.assertEqual(
            build._NextestLeaks.lines,
            [
                (
                    "Running browser tests...",
                    "LEAK",
                    "chronicler_engine::browser behaviour::test_x",
                )
            ],
        )

    def test_leak_printed_live_and_in_final_list_is_named_once(self):
        leak = "        LEAK [   1.229s] ( 2/63) chronicler_engine::browser behaviour::test_x\n"
        self._run_silenced(
            leak
            + "     Summary [ 130.000s] 63 tests run: 63 passed (1 leaky), 0 skipped\n"
            + leak,
            0,
        )
        self.assertEqual(len(build._NextestLeaks.lines), 1)

    def test_timing_report_counts_a_repeated_test_once(self):
        line = "        LEAK [   1.229s] ( 2/63) chronicler_engine::browser behaviour::test_x\n"
        result = mock.Mock(returncode=0, stdout="", stderr=line + line)
        printed = []
        with mock.patch.object(
            build.subprocess, "run", return_value=result
        ), mock.patch("builtins.print", side_effect=lambda msg="": printed.append(msg)):
            build.run_with_test_timings("cargo nextest run")
        self.assertIn("  Slowest tests (of 1 measured, top 30):", printed)

    def test_tier_command_records_its_test_set_size(self):
        build._NextestSummary.label = "Running integration tests..."
        fake = self._fake_process(
            "     Summary [  20.500s] 1570 tests run: 1570 passed, 2 skipped\n", 0
        )
        with mock.patch.object(build.subprocess, "Popen", return_value=fake), mock.patch(
            "builtins.print"
        ):
            build.run(build.get_integration_test_cmd())
        self.assertEqual(
            build._NextestSummary.sizes,
            {"Running integration tests...": build._TierSize("integration", 1572)},
        )
        # A NamedTuple equals a plain tuple, so check the type as well.
        self.assertIsInstance(
            build._NextestSummary.sizes["Running integration tests..."], build._TierSize
        )
        # The epilogue lines keep their (label, text) shape.
        self.assertEqual(
            build._NextestSummary.lines,
            [("Running integration tests...", "nextest: 1570 passed, 0 failed, 2 skipped")],
        )

    def test_other_command_records_no_size(self):
        self._run_silenced("     Summary [  1.0s] 1 test run: 1 passed, 0 skipped\n", 0)
        self.assertEqual(build._NextestSummary.sizes, {})
        self.assertEqual(len(build._NextestSummary.lines), 1)

    def test_run_ignores_pass_lines(self):
        self._run_silenced(
            "        PASS [  18.645s] ( 4/26) chronicler_engine::browser behaviour::test_x\n"
            "     Summary [  21.153s] 1 test run: 1 passed, 0 skipped\n",
            0,
        )
        self.assertEqual(build._NextestLeaks.lines, [])


class NextestEpilogueTests(unittest.TestCase):
    """The epilogue prints the one-liner directly before the build banner."""

    def setUp(self):
        reset_epilogue_state()

    def tearDown(self):
        reset_epilogue_state()

    @staticmethod
    def _run_main(printed, no_browser=False):
        memlog = _MemLog()
        gate_args = SimpleNamespace(
            command=None,
            cleanup=False,
            llm_only=False,
            no_browser=no_browser,
        )
        with mock.patch.object(
            build, "parse_args", return_value=gate_args
        ), mock.patch.object(build, "run_gate"), mock.patch.object(
            build, "clean_old_logs"
        ), mock.patch.object(
            build, "_tree_identity", return_value="abc"
        ), mock.patch.object(
            build, "_append_history"
        ), mock.patch.object(
            build, "_last_tier_sizes", return_value={}
        ), mock.patch(
            "builtins.print", side_effect=lambda msg="": printed.append(msg)
        ), mock.patch(
            "builtins.open", return_value=memlog
        ):
            return build.main()

    def test_summary_printed_between_step_summary_and_banner(self):
        printed = []
        build._NextestSummary.lines = [
            ("Running integration tests...", "nextest: 21 passed, 0 failed")
        ]
        self.assertEqual(self._run_main(printed), 0)
        idx = printed.index("nextest: 21 passed, 0 failed  (Running integration tests...)")
        self.assertEqual(printed[idx + 1], "=" * 60)
        self.assertEqual(printed[idx + 2], "=== Build Complete ===")

    def test_two_tiers_print_one_line_each_before_banner(self):
        """The gate runs integration and browser separately; both report."""
        printed = []
        build._NextestSummary.lines = [
            ("Running integration tests...", "nextest: 1587 passed, 0 failed"),
            ("Running browser tests...", "nextest: 27 passed, 0 failed"),
        ]
        self.assertEqual(self._run_main(printed), 0)
        block = (
            "nextest: 1587 passed, 0 failed  (Running integration tests...)\n"
            "nextest: 27 passed, 0 failed  (Running browser tests...)"
        )
        idx = printed.index(block)
        self.assertEqual(printed[idx + 1], "=" * 60)
        self.assertEqual(printed[idx + 2], "=== Build Complete ===")

    def test_no_summary_prints_no_extra_line(self):
        printed = []
        self.assertEqual(self._run_main(printed), 0)
        self.assertFalse(any(msg.startswith("nextest:") for msg in printed))
        self.assertFalse(any(msg.startswith("skipped:") for msg in printed))

    def test_tier_line_carries_the_count_change(self):
        printed = []
        build._NextestSummary.lines = [
            ("Running integration tests...", "nextest: 1563 passed, 0 failed, 2 skipped")
        ]
        build._NextestSummary.sizes = {
            "Running integration tests...": build._TierSize("integration", 1565)
        }
        self.assertEqual(self._run_main(printed), 0)
        self.assertIn(
            "nextest: 1563 passed, 0 failed, 2 skipped  (Running integration tests...)"
            "  tests: 1565 (no earlier record)",
            printed,
        )

    def test_no_browser_gate_names_the_skipped_tier(self):
        printed = []
        self.assertEqual(self._run_main(printed, no_browser=True), 0)
        idx = printed.index('skipped: browser tier (run "python build.py browser")')
        self.assertEqual(printed[idx + 1], "=" * 60)

    def test_leaky_test_name_is_printed_before_the_banner(self):
        printed = []
        build._NextestLeaks.lines = [
            (
                "Running browser tests...",
                "LEAK",
                "chronicler_engine::browser behaviour::test_x",
            )
        ]
        self.assertEqual(self._run_main(printed), 0)
        idx = printed.index(
            "leak test: chronicler_engine::browser behaviour::test_x  (Running browser tests...)"
        )
        self.assertEqual(printed[idx + 1], "=" * 60)
        self.assertEqual(printed[idx + 2], "=== Build Complete ===")


class MainStampTests(unittest.TestCase):
    """The session stamp is the first line written to every build log."""

    def test_stamp_is_first_log_line_in_main(self):
        memlog = _MemLog()
        gate_args = SimpleNamespace(
            command=None,
            cleanup=False,
            llm_only=False,
            no_browser=False,
        )
        with mock.patch.dict(
            os.environ, {"PI_SESSION_ID": "0123abcd-0000-1111-2222-333344445555"}
        ), mock.patch.object(build, "parse_args", return_value=gate_args), mock.patch.object(
            build, "run_gate"
        ), mock.patch.object(
            build, "_tree_identity", return_value="abc"
        ), mock.patch.object(
            build, "_append_history"
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
