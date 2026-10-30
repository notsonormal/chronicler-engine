"""Unit tests for scripts/build_report.py, against output recorded from cargo and nextest.

The fixtures in ``fixtures/build_output/`` come from a throwaway crate: ``junit_failed.xml`` and
``nextest_failed.stderr`` from one run with a panicking test, a timing-out test and a leaking
test; ``nextest_cancelled.stderr`` from a ``--fail-fast`` run; ``cargo_check_errors.json`` from
``cargo check --all-targets`` with one error in two targets and a warning;
``cargo_clippy_error.json`` from ``cargo clippy -- -D warnings``; ``fmt_list.txt`` from
``cargo fmt --check -- -l``.
"""

from __future__ import annotations

import os
import sys
import unittest
from pathlib import Path
from unittest import mock

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))

import build_report  # noqa: E402

FIXTURES = Path(__file__).resolve().parent / "fixtures" / "build_output"


def fixture(name: str) -> str:
    return (FIXTURES / name).read_text(encoding="utf-8")


class SummaryTests(unittest.TestCase):
    def test_summary_of_a_failed_run_counts_every_kind(self):
        counts = build_report.parse_summary(fixture("nextest_failed.stderr"))
        self.assertEqual(
            counts,
            build_report.Counts(
                passed=3, failed=1, skipped=1, leaky=1, flaky=0, timed_out=1, reached=5, selected=5
            ),
        )
        self.assertEqual(
            counts.line,
            "nextest: 3 passed, 1 failed, 1 skipped, 1 leaky, 1 timed out",
        )
        self.assertEqual(counts.size, 6)

    def test_cancelled_run_names_the_tests_it_never_reached(self):
        counts = build_report.parse_summary(fixture("nextest_cancelled.stderr"))
        self.assertEqual(counts.line, "nextest: 1 passed, 1 failed, 1 skipped, 3 not run")
        # The tier is the 5 selected tests and the 1 ignored one, however far the run got.
        self.assertEqual(counts.size, 6)

    def test_a_clean_run_shows_zero_failed_and_no_other_zero(self):
        counts = build_report.parse_summary("     Summary [   0.174s] 21 tests run: 21 passed\n")
        self.assertEqual(counts.line, "nextest: 21 passed, 0 failed")

    def test_flaky_count_is_kept(self):
        counts = build_report.parse_summary(
            "     Summary [  60.6s] 30 tests run: 28 passed, 1 flaky, 1 failed, 0 skipped\n"
        )
        self.assertEqual(counts.line, "nextest: 28 passed, 1 failed, 1 flaky")

    def test_the_last_summary_wins(self):
        counts = build_report.parse_summary(
            "     Summary [ 1.0s] 2 tests run: 1 passed, 1 failed\n"
            "     Summary [ 3.0s] 4 tests run: 4 passed\n"
        )
        self.assertEqual(counts.line, "nextest: 4 passed, 0 failed")

    def test_output_without_a_summary_has_no_counts(self):
        self.assertIsNone(build_report.parse_summary("error: could not compile `probe`\n"))


class LeakTests(unittest.TestCase):
    def test_a_leak_line_gives_the_test_name_once(self):
        leaks = build_report.find_leaks(fixture("nextest_failed.stderr"))
        self.assertEqual(leaks, [("LEAK", "probe::leak leaks_a_child")])

    def test_a_flaky_line_without_a_progress_counter(self):
        leaks = build_report.find_leaks("       FLAKY [   0.229s] bin::t\n")
        self.assertEqual(leaks, [("FLAKY", "bin::t")])

    def test_pass_and_fail_lines_are_not_leaks(self):
        self.assertEqual(build_report.find_leaks(fixture("nextest_cancelled.stderr")), [])


class JunitTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report = build_report.read_junit(FIXTURES / "junit_failed.xml")

    def test_a_panic_gives_its_message_line(self):
        self.assertIn(
            ("probe t::fails", "assertion `left == right` failed: the real message"),
            self.report.failures,
        )

    def test_a_timeout_without_a_body_gives_its_type(self):
        self.assertIn(("probe t::times_out", "test timeout"), self.report.failures)

    def test_passing_tests_are_not_failures(self):
        self.assertEqual(len(self.report.failures), 2)

    def test_every_test_has_its_duration(self):
        times = dict((name, secs) for secs, name in self.report.times)
        self.assertEqual(len(times), 5)
        self.assertGreater(times["probe t::times_out"], 1.0)
        self.assertIn("probe::leak leaks_a_child", times)

    def test_a_stub_check_line_is_attributed_to_its_runner(self):
        self.assertEqual(
            self.report.stub_checks,
            [(0.902, "probe stubs::run_checks", "check_x", "OK")],
        )

    def test_a_failure_with_only_a_header_line_keeps_the_header(self):
        import xml.etree.ElementTree as ET

        failure = ET.fromstring("<failure>thread 'x' panicked at src/a.rs:1:1:</failure>")
        self.assertEqual(
            build_report._failure_message(failure), "thread 'x' panicked at src/a.rs:1:1:"
        )


class CargoStreamTests(unittest.TestCase):
    E0308 = "error[E0308]: mismatched types  (src/lib.rs:6:5)"

    def test_an_error_repeated_for_two_targets_is_kept_once(self):
        out = build_report.read_cargo_stream(fixture("cargo_check_errors.json"))
        self.assertEqual(out.errors, [self.E0308])

    def test_the_log_text_is_the_rendered_compiler_output(self):
        out = build_report.read_cargo_stream(fixture("cargo_check_errors.json"))
        self.assertIn("error[E0308]: mismatched types", out.text)
        self.assertIn("warning: unused variable: `unused`", out.text)
        self.assertNotIn('"reason"', out.text)

    def test_a_clippy_error_has_no_code_in_its_header(self):
        out = build_report.read_cargo_stream(fixture("cargo_clippy_error.json"))
        self.assertEqual(out.errors, ["error: this `if` has identical blocks  (src/lib.rs:2:10)"])

    def test_a_line_that_is_not_json_stays_in_the_log(self):
        out = build_report.read_cargo_stream("plain output\n{not json\n")
        self.assertEqual(out.text, "plain output\n{not json")
        self.assertEqual(out.errors, [])

    def test_an_error_without_a_span_has_no_location(self):
        message = {"level": "error", "rendered": "error: linking with `cc` failed\n\n", "spans": []}
        self.assertEqual(build_report._error_line(message), "error: linking with `cc` failed")


class UnformattedFilesTests(unittest.TestCase):
    def test_paths_are_made_relative_to_the_working_directory(self):
        with mock.patch.object(build_report.os.path, "relpath", side_effect=lambda p: p[len("/work/"):]):
            files = build_report.unformatted_files(fixture("fmt_list.txt") * 2)
        self.assertEqual(files, ["probe/src/lib.rs"])

    def test_other_lines_are_not_files(self):
        self.assertEqual(
            build_report.unformatted_files("error: expected item\n --> src/a.rs:1:2\n"), []
        )


class TextErrorsTests(unittest.TestCase):
    def test_a_rustfmt_syntax_error_keeps_its_location(self):
        with mock.patch.object(build_report.os.path, "relpath", side_effect=lambda p: p[len("/work/") :]):
            errors = build_report.text_errors(fixture("fmt_syntax_error.txt"))
        self.assertEqual(
            errors, ["error: this file contains an unclosed delimiter  (probe/src/lib.rs:5:3)"]
        )

    def test_an_error_without_a_location_is_kept_bare(self):
        self.assertEqual(
            build_report.text_errors("error: linking with `cc` failed\n\n"),
            ["error: linking with `cc` failed"],
        )

    def test_a_listed_path_is_not_an_error(self):
        self.assertEqual(build_report.text_errors(fixture("fmt_list.txt")), [])


class TailTests(unittest.TestCase):
    def test_the_last_three_non_empty_lines(self):
        self.assertEqual(
            build_report.tail("one\n\ntwo\nthree\n\nfour\n\n"), ["two", "three", "four"]
        )


if __name__ == "__main__":
    unittest.main()
