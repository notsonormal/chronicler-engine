"""Unit tests for build.py's compile-error capture and its epilogue block.

Run via ``python -m unittest discover scripts/tests`` or via ``build.py``.
"""

from __future__ import annotations

import sys
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT))
sys.path.insert(0, str(Path(__file__).resolve().parent))

import build  # noqa: E402
from build_cli_support import _MemLog  # noqa: E402

# Captured from logs/build_20261009_211753.log (cargo check --all-targets).
_E0599_OUTPUT = """\
error[E0599]: no method named `latest_llm_message_per_agent` found for struct `Storage` in the current scope
   --> tests/storage/llm_message_storage.rs:152:26
    |
152 |     let latest = storage.latest_llm_message_per_agent().unwrap();
    |                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
help: there is a method `save_llm_message` with a similar name, but with different arguments
   --> /workspace/chronicler-engine/src/adapters/driven/storage/llm_messages.rs:14:5

For more information about this error, try `rustc --explain E0599`.
error: could not compile `chronicler_engine` (test "storage") due to 1 previous error
"""

_E0599_RENDERED = (
    "error[E0599]: no method named `latest_llm_message_per_agent` found for struct"
    " `Storage` in the current scope  (tests/storage/llm_message_storage.rs:152:26)"
)


def _reset():
    build._CompileErrors.lines = []
    build._NextestSummary.lines = []
    build._NextestSummary.label = ""
    build._NextestSummary.sizes = {}
    build._NextestLeaks.lines = []


class StashCompileErrorsTests(unittest.TestCase):
    """_stash_compile_errors keeps each distinct error with its location."""

    def setUp(self):
        _reset()

    def tearDown(self):
        _reset()

    def test_rustc_error_keeps_header_and_location_without_cargo_noise(self):
        build._stash_compile_errors(_E0599_OUTPUT)
        self.assertEqual(build._CompileErrors.lines, [_E0599_RENDERED])

    def test_error_repeated_for_another_target_is_kept_once(self):
        build._stash_compile_errors(_E0599_OUTPUT + _E0599_OUTPUT)
        self.assertEqual(build._CompileErrors.lines, [_E0599_RENDERED])

    def test_clippy_error_without_code_is_captured(self):
        build._stash_compile_errors(
            "error: this `if` has identical blocks\n"
            "  --> src/lib.rs:3:5\n"
            "   |\n"
        )
        self.assertEqual(
            build._CompileErrors.lines,
            ["error: this `if` has identical blocks  (src/lib.rs:3:5)"],
        )

    def test_error_without_location_is_kept_bare(self):
        build._stash_compile_errors("error: linking with `cc` failed: exit status: 1\n\n")
        self.assertEqual(
            build._CompileErrors.lines,
            ["error: linking with `cc` failed: exit status: 1"],
        )

    def test_failed_run_stashes_errors_and_passing_run_does_not(self):
        fake = mock.Mock()
        fake.communicate.return_value = (_E0599_OUTPUT, None)
        fake.returncode = 0
        with mock.patch.object(build.subprocess, "Popen", return_value=fake), mock.patch(
            "builtins.print"
        ):
            build.run("cargo check", check=False)
            self.assertEqual(build._CompileErrors.lines, [])
            fake.returncode = 101
            with self.assertRaises(SystemExit):
                build.run("cargo check")
        self.assertEqual(build._CompileErrors.lines, [_E0599_RENDERED])


class CompileErrorEpilogueTests(unittest.TestCase):
    """main() prints the captured errors just before the closing banner."""

    def setUp(self):
        _reset()

    def tearDown(self):
        _reset()

    @staticmethod
    def _run_main(printed):
        gate_args = SimpleNamespace(
            command=None, cleanup=False, llm_only=False, no_browser=False
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
            "builtins.open", return_value=_MemLog()
        ):
            return build.main()

    def test_errors_are_printed_before_the_banner(self):
        printed = []
        build._CompileErrors.lines = [_E0599_RENDERED]
        self._run_main(printed)
        idx = printed.index("compile errors (first 1 of 1):")
        self.assertEqual(printed[idx + 1], f"  {_E0599_RENDERED}")
        self.assertEqual(printed[idx + 2], "=" * 60)

    def test_block_is_capped_at_the_limit(self):
        printed = []
        build._CompileErrors.lines = [f"error: e{n}" for n in range(7)]
        self._run_main(printed)
        idx = printed.index("compile errors (first 5 of 7):")
        self.assertEqual(printed[idx + 5], "  error: e4")
        self.assertEqual(printed[idx + 6], "=" * 60)

    def test_no_block_without_errors(self):
        printed = []
        self._run_main(printed)
        self.assertFalse(any(p.startswith("compile errors") for p in printed))


if __name__ == "__main__":
    unittest.main()
