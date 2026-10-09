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
from build_cli_support import _MemLog
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
            "guardrails-doc-check",
            "validate-docs",
            "architecture",
            "guardrails",
            "unit",
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

    def test_test_pattern_spec_takes_pattern(self):
        self.assertTrue(build.REGISTRY["test-pattern"].pattern_arg)
        self.assertFalse(build.REGISTRY["clippy"].pattern_arg)

    def test_run_is_not_a_gate_step(self):
        """`run` has its own dispatcher and must not join the full gate."""
        self.assertIn("run", build.REGISTRY)
        self.assertNotIn("run", build.GATE_ORDER)


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
    """run_server builds under the slot and returns the argv; it never execs."""

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

    def test_returns_binary_and_stripped_flags(self):
        args = self._args(["--", "--world", "x", "--port", "3099"])
        with mock.patch.object(build, "check_rust_version"), mock.patch.object(
            build, "_require_free_port"
        ), mock.patch.object(build, "_warn_if_target_locked"), mock.patch.object(
            build, "_timed_run"
        ) as timed:
            argv = build.run_server(args, build.StepRecord())
        self.assertEqual(
            argv,
            ["target/debug/chronicler_engine", "--world", "x", "--port", "3099"],
        )
        self.assertEqual(timed.call_args.args[3], "cargo build --bin chronicler_engine")

    def test_target_dir_after_run_points_at_that_binary(self):
        args = self._args(["--port", "3099"], target_dir="target/agent2")
        with mock.patch.object(build, "check_rust_version"), mock.patch.object(
            build, "_require_free_port"
        ), mock.patch.object(build, "_warn_if_target_locked"), mock.patch.object(
            build, "_timed_run"
        ):
            argv = build.run_server(args, build.StepRecord())
        self.assertEqual(argv[0], "target/agent2/debug/chronicler_engine")

    def test_busy_port_fails_before_building(self):
        args = self._args(["--", "--port", "3099"])
        with mock.patch.object(build, "check_rust_version"), mock.patch.object(
            build, "_require_free_port", side_effect=SystemExit(1)
        ), mock.patch.object(build, "_timed_run") as timed:
            with self.assertRaises(SystemExit):
                build.run_server(args, build.StepRecord())
        timed.assert_not_called()


class ExecPendingTests(unittest.TestCase):
    """The __main__ exec replaces the process only after a clean build."""

    def tearDown(self):
        build._pending_exec = None

    def test_execs_when_pending_and_clean(self):
        build._pending_exec = ["/bin/echo", "--port", "3099"]
        with mock.patch.object(build, "_require_free_port"), mock.patch.object(
            build, "os"
        ) as os_mod:
            build._exec_pending(0)
        os_mod.execv.assert_called_once_with(
            "/bin/echo", ["/bin/echo", "--port", "3099"]
        )

    def test_rechecks_the_port_before_the_exec(self):
        build._pending_exec = ["/bin/echo", "--port", "3099"]
        with mock.patch.object(
            build, "_require_free_port", side_effect=SystemExit(1)
        ), mock.patch.object(build, "os") as os_mod:
            with self.assertRaises(SystemExit):
                build._exec_pending(0)
        os_mod.execv.assert_not_called()

    def test_does_not_exec_on_failure(self):
        build._pending_exec = ["/bin/echo"]
        with mock.patch.object(build, "os") as os_mod:
            build._exec_pending(1)
        os_mod.execv.assert_not_called()

    def test_does_not_exec_when_nothing_pending(self):
        build._pending_exec = None
        with mock.patch.object(build, "os") as os_mod:
            build._exec_pending(0)
        os_mod.execv.assert_not_called()

    def test_clears_pending_after_one_call(self):
        build._pending_exec = ["/bin/echo"]
        with mock.patch.object(build, "_require_free_port"), mock.patch.object(build, "os"):
            build._exec_pending(0)
        self.assertIsNone(build._pending_exec)


class MainRunTests(unittest.TestCase):
    """main() routes `run` to run_server and stores the exec argv for __main__."""

    def tearDown(self):
        build._pending_exec = None

    @staticmethod
    def _run_args():
        return SimpleNamespace(
            command="run", cleanup=False, llm_only=False, no_browser=False
        )

    def _run_main(self, run_server):
        memlog = _MemLog()
        with (
            mock.patch.object(build, "parse_args", return_value=self._run_args()),
            mock.patch.object(build, "run_server", side_effect=run_server) as server,
            mock.patch.object(build, "run_step") as step,
            mock.patch.object(build, "_append_history"),
            mock.patch.object(build, "_tree_identity", return_value="abc"),
            mock.patch.object(build, "clean_old_logs"),
            mock.patch("builtins.print"),
            mock.patch("builtins.open", return_value=memlog),
        ):
            rc = build.main()
        return rc, server, step

    def test_stores_argv_and_never_calls_run_step(self):
        rc, server, step = self._run_main(
            lambda args, record: ["/bin/echo", "--port", "9"]
        )
        self.assertEqual(rc, 0)
        server.assert_called_once()
        step.assert_not_called()
        self.assertEqual(build._pending_exec, ["/bin/echo", "--port", "9"])

    def test_failed_build_leaves_nothing_pending(self):
        def boom(args, record):
            raise SystemExit(101)

        with self.assertRaises(SystemExit) as ctx:
            self._run_main(boom)
        self.assertEqual(ctx.exception.code, 101)
        self.assertIsNone(build._pending_exec)


class StepCommandTests(unittest.TestCase):
    """Step command assembly, including nextest pattern quoting."""

    # Every registry command pinned verbatim: a typo in any spec's cmd fails
    # the suite instead of silently changing what a subcommand runs.
    EXPECTED_COMMANDS = {
        "install-hooks": "python scripts/install_git_hooks.py",
        "remove-worktrees": "python scripts/remove_worktrees.py --apply",
        "fmt": "cargo fmt",
        "validate-data": "python scripts/validate_data.py",
        "check": "cargo check --all-targets --all-features",
        "clippy": "cargo clippy --all-targets --all-features -- -D warnings",
        "test-structure": "python scripts/check_test_structure.py",
        "spec-coverage": "python scripts/validate_feature_spec.py",
        "docstrings": "python scripts/check_python_docstrings.py",
        "py-tests": "python -m unittest discover scripts/tests -v",
        "duplicates": "python scripts/healthcheck.py duplicates",
        "http-routes-check": "python scripts/extract_http_routes.py --check",
        "guardrails-doc-check": "python scripts/generate_guardrails_doc.py --check",
        "validate-docs": "python scripts/validate_docs.py",
        "architecture": "cargo nextest run --no-fail-fast --test architecture",
        "guardrails": "cargo nextest run --no-fail-fast --test guardrails",
        "unit": "cargo test --lib",
        "integration": "cargo nextest run --no-fail-fast -E 'not binary(browser) and not binary(architecture) and not binary(guardrails)'",
        "browser": "cargo nextest run --no-fail-fast -E 'binary(browser)'",
        "test-pattern": "cargo nextest run --no-fail-fast",
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
        self.assertEqual(cmd, "cargo nextest run --no-fail-fast 'my test'")


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

    def test_test_timings_restores_pass_level(self):
        # The timing report needs PASS lines, which the leak level suppresses.
        env = build._cargo_env_for(SimpleNamespace(target_dir=None, test_timings=True))
        self.assertEqual(env["NEXTEST_STATUS_LEVEL"], "pass")


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
