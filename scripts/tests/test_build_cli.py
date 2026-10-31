"""Unit tests for build.py CLI parsing, the step registry, and log stamping.

Run via ``python -m unittest discover scripts/tests`` or via ``build.py``.
"""

from __future__ import annotations

import io
import os
import socket
import sys
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

try:
    import fcntl
except ImportError:  # Windows has no fcntl; cargo target locking is skipped there.
    fcntl = None

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT))
sys.path.insert(0, str(Path(__file__).resolve().parent))

import build  # noqa: E402
from build_cli_support import gate_args, run_main  # noqa: E402


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
            "duplicates",
            "http-routes-check",
            "http-routes",
            "guardrails-doc-check",
            "guardrails-doc",
            "validate-docs",
            "guardrails",
            "integration",
            "test-pattern",
            "run",
            "remove-worktrees",
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

    def test_gate_plan_order_is_stable(self):
        labels = [step.label for step in build._plan_gate_steps(gate_args())]
        self.assertEqual(labels[2], build.REGISTRY["fmt"].label)
        self.assertLess(
            labels.index(build.REGISTRY["validate-docs"].label),
            labels.index(build.REGISTRY["guardrails"].label),
            "registry guardrail steps must run after the doc steps",
        )

    def test_test_pattern_spec_takes_pattern(self):
        self.assertTrue(build.REGISTRY["test-pattern"].pattern_arg)
        self.assertFalse(build.REGISTRY["clippy"].pattern_arg)

    def test_run_is_not_a_gate_step(self):
        """`run` has its own dispatcher and must not join the full gate."""
        labels = [step.label for step in build._plan_gate_steps(gate_args())]
        self.assertNotIn(build.REGISTRY["run"].label, labels)


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

    def test_test_pattern_required(self):
        with self.assertRaises(SystemExit) as ctx:
            build.parse_args(["test-pattern"])
        self.assertEqual(ctx.exception.code, 2)

    def test_test_pattern_value(self):
        args = build.parse_args(["test-pattern", "behaviour::test_x"])
        self.assertEqual(args.pattern, "behaviour::test_x")
        self.assertIsNone(args.test_target)

    def test_test_pattern_accepts_a_test_target(self):
        args = build.parse_args(["test-pattern", "x", "--test-target", "http"])
        self.assertEqual(args.test_target, "http")

    def test_duplicates_step_forwards_extra_args(self):
        args = build.parse_args(["duplicates", "--all"])
        self.assertTrue(args.duplicates_all)
        extra = build._step_extra_args(build.REGISTRY["duplicates"], args)
        self.assertEqual(
            build._step_command(build.REGISTRY["duplicates"], None, extra),
            "python scripts/healthcheck.py duplicates --all",
        )

    def test_duplicates_step_forwards_ref(self):
        args = build.parse_args(["duplicates", "--ref", "develop"])
        self.assertEqual(args.duplicates_ref, "develop")
        extra = build._step_extra_args(build.REGISTRY["duplicates"], args)
        self.assertEqual(
            build._step_command(build.REGISTRY["duplicates"], None, extra),
            "python scripts/healthcheck.py duplicates --ref develop",
        )

    def test_non_duplicates_step_has_no_extra(self):
        args = build.parse_args(["clippy"])
        self.assertIsNone(build._step_extra_args(build.REGISTRY["clippy"], args))

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

    def test_strict_flag_rejected(self):
        with self.assertRaises(SystemExit) as ctx:
            build.parse_args(["clippy", "--strict"])
        self.assertEqual(ctx.exception.code, 2)

    def test_strict_flag_before_step_rejected(self):
        with self.assertRaises(SystemExit) as ctx:
            build.parse_args(["--strict", "clippy"])
        self.assertEqual(ctx.exception.code, 2)

    def test_no_browser_is_a_gate_flag(self):
        args = build.parse_args(["--no-browser"])
        self.assertIsNone(args.command)
        self.assertTrue(args.no_browser)

    def test_no_browser_rejected_with_step_in_both_orders(self):
        for argv in (["clippy", "--no-browser"], ["--no-browser", "clippy"]):
            with self.subTest(argv=argv), self.assertRaises(SystemExit) as ctx:
                with mock.patch("sys.stderr", new_callable=io.StringIO):
                    build.parse_args(argv)
            self.assertEqual(ctx.exception.code, 2)

    def test_no_browser_rejected_with_coverage(self):
        """Coverage without the browser tier would under-report dashboard code."""
        with self.assertRaises(SystemExit) as ctx:
            with mock.patch("sys.stderr", new_callable=io.StringIO):
                build.parse_args(["--coverage", "--no-browser"])
        self.assertEqual(ctx.exception.code, 2)


class RunPassthroughTests(unittest.TestCase):
    """`python build.py run [-- <server flags>]` forwards the server's own flags."""

    def test_strip_drops_exactly_one_double_dash(self):
        self.assertEqual(
            build._strip_passthrough(["--", "--port", "9"]), ["--port", "9"]
        )
        self.assertEqual(build._strip_passthrough(["--", "--", "x"]), ["--", "x"])
        self.assertEqual(build._strip_passthrough(["--port", "9"]), ["--port", "9"])
        self.assertEqual(build._strip_passthrough([]), [])

    def test_parse_args_keeps_the_passthrough(self):
        args = build.parse_args(["run", "--", "--world", "x", "--port", "9"])
        self.assertEqual(args.command, "run")
        self.assertEqual(args.server_flags, ["--", "--world", "x", "--port", "9"])

    def test_no_passthrough_is_empty(self):
        self.assertEqual(build.parse_args(["run"]).server_flags, [])

    def test_target_dir_before_run(self):
        args = build.parse_args(["--target-dir", "t/a", "run", "--", "--port", "9"])
        self.assertEqual(args.target_dir, "t/a")
        self.assertEqual(build._strip_passthrough(args.server_flags), ["--port", "9"])

    def test_target_dir_after_run(self):
        args = build.parse_args(["run", "--target-dir", "t/a", "--", "--port", "9"])
        self.assertEqual(args.target_dir, "t/a")
        self.assertEqual(build._strip_passthrough(args.server_flags), ["--port", "9"])

    def test_release_before_run_rejected(self):
        with self.assertRaises(SystemExit) as ctx:
            build.parse_args(["--release", "run"])
        self.assertEqual(ctx.exception.code, 2)

    def test_run_without_separator_rejects_server_flag(self):
        with self.assertRaises(SystemExit):
            build.parse_args(["run", "--world", "x"])


class BindTargetTests(unittest.TestCase):
    """The pre-build port check mirrors the server's clap defaults."""

    def test_defaults_match_server_cli(self):
        self.assertEqual(build._parse_bind_target([]), (3000, "0.0.0.0"))

    def test_space_form(self):
        self.assertEqual(build._parse_bind_target(["--port", "3099"]), (3099, "0.0.0.0"))
        self.assertEqual(
            build._parse_bind_target(["--host", "127.0.0.1"]), (3000, "127.0.0.1")
        )

    def test_equals_form(self):
        self.assertEqual(build._parse_bind_target(["--port=3099"]), (3099, "0.0.0.0"))
        self.assertEqual(
            build._parse_bind_target(["--host=127.0.0.1"]), (3000, "127.0.0.1")
        )

    def test_later_flag_wins(self):
        self.assertEqual(
            build._parse_bind_target(["--port", "1", "--port=2"]), (2, "0.0.0.0")
        )

    def test_skip_flags_disable_the_check(self):
        for flag in ("--list-worlds", "--help", "-h", "--version", "-V"):
            with self.subTest(flag=flag):
                self.assertIsNone(build._parse_bind_target([flag]))

    def test_unusable_values_skip_the_check(self):
        self.assertIsNone(build._parse_bind_target(["--port", "abc"]))
        self.assertIsNone(build._parse_bind_target(["--port"]))
        self.assertIsNone(build._parse_bind_target(["--host"]))

    def test_out_of_range_port_skips_the_check(self):
        # The probe would raise OverflowError rather than report a conflict.
        for value in ("70000", "-1", "65536"):
            with self.subTest(value=value):
                self.assertIsNone(build._parse_bind_target(["--port", value]))


class PortProbeTests(unittest.TestCase):
    """The bind probe matches tokio's SO_REUSEADDR semantics."""

    @staticmethod
    def _free_port():
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as probe:
            probe.bind(("127.0.0.1", 0))
            return probe.getsockname()[1]

    def test_free_port_reads_as_free(self):
        self.assertFalse(build._port_in_use("127.0.0.1", self._free_port()))

    def test_bound_port_reads_as_in_use(self):
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as held:
            held.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            held.bind(("127.0.0.1", 0))
            held.listen(1)
            self.assertTrue(build._port_in_use("127.0.0.1", held.getsockname()[1]))

    def test_busy_port_stops_with_port_is_in_use(self):
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as held:
            held.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            held.bind(("127.0.0.1", 0))
            held.listen(1)
            port = held.getsockname()[1]
            with mock.patch.object(build, "both_print") as printer:
                with self.assertRaises(SystemExit) as ctx:
                    build._require_free_port((port, "127.0.0.1"))
        self.assertEqual(ctx.exception.code, 1)
        messages = " ".join(str(call.args[0]) for call in printer.call_args_list)
        self.assertIn(f"port {port} is in use", messages)

    def test_none_target_is_a_noop(self):
        build._require_free_port(None)


class RunServerTests(unittest.TestCase):
    """run_server builds through the plan and returns the argv; it never execs."""

    def setUp(self):
        patcher = mock.patch.dict(
            os.environ, {"CHRONICLER_NO_LLD": "1"}
        )  # keep ~/.cache untouched
        patcher.start()
        self.addCleanup(patcher.stop)

    @staticmethod
    def _args(flags, target_dir=None):
        return SimpleNamespace(
            command="run", server_flags=flags, target_dir=target_dir, release=False
        )

    def _serve(self, args, port_check=None):
        """Run run_server with the toolchain, the port check and the plan run stubbed."""
        with mock.patch.object(build, "_prelude"), mock.patch.object(
            build, "_require_free_port", side_effect=port_check
        ), mock.patch.object(build, "_run_plan") as plan:
            argv = build.run_server(args, [])
        return argv, plan

    def test_returns_binary_and_stripped_flags(self):
        argv, plan = self._serve(self._args(["--", "--world", "x", "--port", "3099"]))
        self.assertEqual(
            argv, ["target/debug/chronicler_engine", "--world", "x", "--port", "3099"]
        )
        self.assertEqual(plan.call_args.args[0][0].cmd, "cargo build --bin chronicler_engine")

    def test_target_dir_after_run_points_at_that_binary(self):
        argv, _ = self._serve(self._args(["--port", "3099"], target_dir="target/agent2"))
        self.assertEqual(argv[0], "target/agent2/debug/chronicler_engine")

    def test_busy_port_fails_before_building(self):
        with mock.patch.object(build, "_prelude"), mock.patch.object(
            build, "_require_free_port", side_effect=SystemExit(1)
        ), mock.patch.object(build, "_run_plan") as plan:
            with self.assertRaises(SystemExit):
                build.run_server(self._args(["--", "--port", "3099"]), [])
        plan.assert_not_called()


class ExecPendingTests(unittest.TestCase):
    """The __main__ exec replaces the process only after a clean build."""

    ARGV = ["/bin/echo", "--port", "3099"]

    def test_execs_when_clean(self):
        with mock.patch.object(build, "_require_free_port"), mock.patch.object(
            build, "os"
        ) as os_mod:
            build._exec_pending(0, self.ARGV)
        os_mod.execv.assert_called_once_with("/bin/echo", self.ARGV)

    def test_rechecks_the_port_before_the_exec(self):
        with mock.patch.object(
            build, "_require_free_port", side_effect=SystemExit(1)
        ), mock.patch.object(build, "os") as os_mod:
            with self.assertRaises(SystemExit):
                build._exec_pending(0, self.ARGV)
        os_mod.execv.assert_not_called()

    def test_does_not_exec_on_failure_or_without_argv(self):
        with mock.patch.object(build, "os") as os_mod:
            build._exec_pending(1, self.ARGV)
            build._exec_pending(0, None)
        os_mod.execv.assert_not_called()


class MainRunTests(unittest.TestCase):
    """main() routes `run` to run_server and hands the exec argv back to __main__."""

    def _run_main(self, run_server):
        with mock.patch.object(build, "run_server", side_effect=run_server):
            return run_main(command="run")

    def test_returns_the_argv_to_exec(self):
        _, _, code, exec_argv = self._run_main(lambda args, results: ["/bin/echo", "--port", "9"])
        self.assertEqual(code, 0)
        self.assertEqual(exec_argv, ["/bin/echo", "--port", "9"])

    def test_a_failed_build_returns_no_argv(self):
        def failed_build(args, results):
            results.append(build.StepResult("Building dev server...", rc=101, fatal=True))
            return ["/bin/echo"]

        _, _, code, exec_argv = self._run_main(failed_build)
        self.assertEqual(code, 101)
        self.assertIsNone(exec_argv)


class StepCommandTests(unittest.TestCase):
    # Every registry command pinned verbatim: a typo in any spec's cmd fails
    # the suite instead of silently changing what a subcommand runs.
    EXPECTED_COMMANDS = {
        "install-hooks": "python scripts/install_git_hooks.py",
        "remove-worktrees": "python scripts/remove_worktrees.py --apply --prune-targets",
        "fmt": "cargo fmt",
        "validate-data": "python scripts/validate_data.py",
        "check": "cargo check --all-targets --all-features",
        "clippy": "cargo clippy --all-targets --all-features -- -D warnings",
        "test-structure": "python scripts/check_test_structure.py",
        "spec-coverage": "python scripts/validate_feature_spec.py",
        "seam-cases": "python scripts/validate_seam_cases.py",
        "docstrings": "python scripts/check_python_docstrings.py",
        "py-tests": "python -m unittest discover scripts/tests -v",
        "duplicates": "python scripts/healthcheck.py duplicates",
        "http-routes-check": "python scripts/extract_http_routes.py --check",
        "http-routes": "python scripts/extract_http_routes.py",
        "guardrails-doc-check": "python scripts/generate_guardrails_doc.py --check",
        "guardrails-doc": "python scripts/generate_guardrails_doc.py",
        "validate-docs": "python scripts/validate_docs.py",
        "guardrails": "cargo nextest run --test architecture --test guardrails",
        "integration": "cargo nextest run -E 'not binary(browser) and not binary(architecture) and not binary(guardrails)'",
        "browser": "cargo nextest run -E 'binary(browser)'",
        "test-pattern": "cargo nextest run",
        "run": "cargo build --bin chronicler_engine",
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
        cmd = build._step_command(build.REGISTRY["test-pattern"], "my test")
        self.assertEqual(cmd, "cargo nextest run 'my test'")

    def test_test_target_selects_one_binary(self):
        cmd = build._step_command(build.REGISTRY["test-pattern"], "x", None, "http")
        self.assertEqual(cmd, "cargo nextest run --test http x")

    def test_test_target_lib_selects_the_lib_unit_binary(self):
        cmd = build._step_command(build.REGISTRY["test-pattern"], "", None, "lib")
        self.assertEqual(cmd, "cargo nextest run --lib ''")

    def test_no_test_target_keeps_every_binary(self):
        cmd = build._step_command(build.REGISTRY["test-pattern"], "", None, None)
        self.assertEqual(cmd, "cargo nextest run ''")

    def test_test_target_is_ignored_by_a_non_test_step(self):
        command = build._step_command(build.REGISTRY["clippy"], None, None, "http")
        self.assertEqual(command, build.REGISTRY["clippy"].cmd)


class FailModeTests(unittest.TestCase):
    """A step run stops at the first failing test. The gate reports every failure."""

    @staticmethod
    def step_plan(argv):
        """The steps a step run plans."""
        return build._step_plan(build.parse_args(argv))

    def test_step_run_stops_at_the_first_failure(self):
        (step,) = self.step_plan(["test-pattern", "x", "--test-target", "browser"])
        self.assertEqual(step.cmd, "cargo nextest run --test browser x --fail-fast")

    def test_browser_step_stops_at_the_first_failure(self):
        (step,) = self.step_plan(["browser"])
        self.assertEqual(step.cmd, "cargo nextest run -E 'binary(browser)' --fail-fast")

    def test_explicit_no_fail_fast_wins_in_a_step(self):
        (step,) = self.step_plan(["browser", "--no-fail-fast"])
        self.assertEqual(step.cmd, "cargo nextest run -E 'binary(browser)' --no-fail-fast")

    def test_gate_reports_every_failure(self):
        by_label = {step.label: step for step in build._plan_gate_steps(gate_args())}
        for label in (
            "Running browser tests...",
            "Running integration tests...",
            "Running guardrail tests...",
        ):
            with self.subTest(label=label):
                self.assertTrue(by_label[label].cmd.endswith("--no-fail-fast"), label)

    def test_explicit_fail_fast_wins_in_the_gate(self):
        by_label = {
            step.label: step for step in build._plan_gate_steps(gate_args(fail_fast=True))
        }
        self.assertTrue(by_label["Running browser tests..."].cmd.endswith("--fail-fast"))

    def test_flag_is_accepted_before_and_after_a_step(self):
        self.assertIs(build.parse_args(["--fail-fast", "browser"]).fail_fast, True)
        self.assertIs(build.parse_args(["browser", "--fail-fast"]).fail_fast, True)
        self.assertIs(build.parse_args(["browser", "--no-fail-fast"]).fail_fast, False)

    def test_last_flag_wins(self):
        args = build.parse_args(["browser", "--fail-fast", "--no-fail-fast"])
        self.assertIs(args.fail_fast, False)

    def test_no_flag_leaves_the_mode_default(self):
        self.assertIsNone(build.parse_args([]).fail_fast)
        self.assertIsNone(build.parse_args(["browser"]).fail_fast)

    def test_fail_fast_is_not_gate_only(self):
        """The flag is not gate-only: `_reject_gate_flags_with_step` must pass it."""
        args = build.parse_args(["clippy", "--fail-fast"])
        self.assertEqual(args.command, "clippy")
        self.assertIs(args.fail_fast, True)

    def test_a_non_nextest_command_keeps_its_text(self):
        args = build.parse_args(["validate-docs"])
        self.assertEqual(
            build._with_fail_flag("python scripts/validate_docs.py", args),
            "python scripts/validate_docs.py",
        )

    def test_coverage_refuses_fail_fast(self):
        """The coverage report needs every test, so `--coverage` refuses the flag."""
        with self.assertRaises(SystemExit) as ctx:
            with mock.patch("sys.stderr", new_callable=io.StringIO) as stderr:
                build.parse_args(["--coverage", "--fail-fast"])
        self.assertEqual(ctx.exception.code, 2)
        self.assertIn("--coverage", stderr.getvalue())

    def test_llm_only_keeps_the_gate_fail_mode(self):
        """`--llm-only` is a gate mode, so its nextest command reports every failure."""
        args = build.parse_args(["--llm-only"])
        with mock.patch.object(build, "_prelude"), mock.patch.object(
            build, "_run_plan"
        ) as plan:
            build.run_llm_only(args, [])
        building, llm, done = plan.call_args.args[0]
        self.assertEqual(building.cmd, "cargo build")
        self.assertEqual(
            llm.cmd,
            "cargo nextest run --profile llm --run-ignored all --no-fail-fast --test llm",
        )
        self.assertEqual(done.cmd, "")


class ClippyFormatCheckTests(unittest.TestCase):
    """The narrow `clippy` step runs `cargo fmt --check` first. The full gate runs `fmt` instead."""

    def test_clippy_runs_the_fmt_check_first(self):
        cmds = [step.cmd for step in build._step_plan(build.parse_args(["clippy"]))]
        self.assertEqual(
            cmds,
            [
                "cargo fmt --check -- -l",
                "cargo clippy --all-targets --all-features -- -D warnings",
            ],
        )

    def test_other_steps_do_not_run_the_fmt_check(self):
        self.assertEqual(len(build._step_plan(build.parse_args(["check"]))), 1)

    def test_the_full_gate_has_no_fmt_check(self):
        labels = [step.label for step in build._plan_gate_steps(gate_args())]
        self.assertNotIn("Checking formatting...", labels)
        self.assertIn("Formatting...", labels)


class CargoEnvTests(unittest.TestCase):
    def setUp(self):
        patcher = mock.patch.dict(os.environ, {"CHRONICLER_NO_LLD": "1"})  # keep ~/.cache untouched
        patcher.start()
        self.addCleanup(patcher.stop)

    def test_target_dir_sets_env_var(self):
        env = build._cargo_env_for(SimpleNamespace(target_dir="target/agent2"))
        self.assertIn("CARGO_TARGET_DIR", env)

    def test_no_target_dir_keeps_env_clean(self):
        env = build._cargo_env_for(SimpleNamespace(target_dir=None))
        self.assertNotIn("CARGO_TARGET_DIR", env)
        # `leak` names a passing-but-leaking test without the PASS flood.
        self.assertEqual(env["NEXTEST_STATUS_LEVEL"], "leak")



class LldLinkerEnvTests(unittest.TestCase):
    """The linker wrapper is installed at one fixed path, whichever checkout runs the build."""

    HOST = "x86_64-unknown-linux-gnu"
    VAR = "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER"

    def setUp(self):
        import tempfile

        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.cache = Path(tmp.name)
        patcher = mock.patch.dict(os.environ, {}, clear=False)
        patcher.start()
        self.addCleanup(patcher.stop)
        os.environ.pop("CHRONICLER_NO_LLD", None)
        os.environ.pop(self.VAR, None)

    def test_installs_the_wrapper_and_names_the_per_triple_variable(self):
        env = build._lld_linker_env(self.HOST, self.cache)
        path = Path(env[self.VAR])
        self.assertEqual(path, self.cache / "chronicler-engine" / "lld-linker.sh")
        self.assertTrue(os.access(path, os.X_OK))
        self.assertEqual(path.read_bytes(), (REPO_ROOT / "scripts" / "lld-linker.sh").read_bytes())

    def test_the_path_does_not_depend_on_the_checkout(self):
        first = build._lld_linker_env(self.HOST, self.cache)
        other = self.cache / "other-checkout"
        (other / "scripts").mkdir(parents=True)
        (other / "scripts" / "lld-linker.sh").write_text("#!/bin/sh\nexec cc \"$@\"\n")
        with mock.patch.object(build, "__file__", str(other / "build.py")):
            second = build._lld_linker_env(self.HOST, self.cache)
        self.assertEqual(first, second)
        self.assertIn("exec cc", Path(second[self.VAR]).read_text())  # the newer wrapper replaced it

    def test_an_existing_linker_setting_wins(self):
        os.environ[self.VAR] = "/opt/my-linker"
        self.assertEqual(build._lld_linker_env(self.HOST, self.cache), {})

    def test_opt_out(self):
        os.environ["CHRONICLER_NO_LLD"] = "1"
        self.assertEqual(build._lld_linker_env(self.HOST, self.cache), {})

    def test_other_platforms_get_nothing(self):
        with mock.patch.object(build.sys, "platform", "win32"):
            self.assertEqual(build._lld_linker_env(self.HOST, self.cache), {})


class IsTargetLockedTests(unittest.TestCase):
    """is_target_locked scans a profile lock with the same shared mode the seed uses."""

    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.target = Path(tmp.name)
        profile = self.target / "debug"
        profile.mkdir()
        self.lock = profile / ".cargo-lock"
        self.lock.write_text("")

    def test_unheld_lock_reads_as_free(self):
        self.assertFalse(build.is_target_locked(self.target))

    def test_a_missing_profile_dir_reads_as_free(self):
        self.assertFalse(build.is_target_locked(self.target / "nope"))

    @unittest.skipIf(fcntl is None, "the target lock probe is an flock")
    def test_a_shared_seed_holder_is_not_a_cargo_build(self):
        # A seed in progress holds the lock shared; build.py must not warn.
        with open(self.lock, "rb") as seed:
            fcntl.flock(seed, fcntl.LOCK_SH | fcntl.LOCK_NB)
            self.assertFalse(build.is_target_locked(self.target))

    @unittest.skipIf(fcntl is None, "the target lock probe is an flock")
    def test_an_exclusive_cargo_holder_reads_as_locked(self):
        with open(self.lock, "rb") as cargo:
            fcntl.flock(cargo, fcntl.LOCK_EX | fcntl.LOCK_NB)
            self.assertTrue(build.is_target_locked(self.target))


if __name__ == "__main__":
    unittest.main()
