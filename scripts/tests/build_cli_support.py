"""Shared fixtures for the build.py CLI unit tests: a log handle that survives close, a context
for running steps against a fake subprocess, and a runner for main()."""

from __future__ import annotations

import io
import shutil
import subprocess
import tempfile
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

import build

FIXTURES = Path(__file__).resolve().parent / "fixtures" / "build_output"


def fixture(name: str) -> str:
    return (FIXTURES / name).read_text(encoding="utf-8")


class _MemLog(io.StringIO):
    """Log handle that survives main()'s close so tests can read it back."""

    def close(self):
        self.flush()


def gate_args(**overrides) -> SimpleNamespace:
    """The namespace of a full-gate run with no flag set."""
    defaults = dict(
        command=None,
        cleanup=False,
        llm_only=False,
        no_fmt=False,
        no_browser=False,
        coverage=False,
        include_llm=False,
        test_timings=False,
        release=False,
        target_dir=None,
        fail_fast=None,
    )
    defaults.update(overrides)
    return SimpleNamespace(**defaults)


def make_ctx(testcase, target_dir=None, **args) -> build.Ctx:
    """A run context for a test.

    nextest writes its JUnit report under ``target/`` of the repo root, whatever the cargo
    target dir is, so the report directory is a temp dir and ``target_dir`` only goes into the
    environment, as ``--target-dir`` does.
    """
    tmp = tempfile.TemporaryDirectory()
    testcase.addCleanup(tmp.cleanup)
    patcher = mock.patch.object(build, "_NEXTEST_DIR", Path(tmp.name) / "nextest")
    patcher.start()
    testcase.addCleanup(patcher.stop)
    env = {"CARGO_TARGET_DIR": target_dir} if target_dir else {}
    return build.Ctx(gate_args(**args), env, [])


def fake_subprocess(stdout="", stderr="", rc=0, junit=None, junit_path=None):
    """A stand-in for ``subprocess.run`` that answers like cargo or nextest.

    ``junit`` names a fixture the fake copies to ``junit_path``, as nextest writes its report.
    """

    def run(cmd, **kwargs):
        if junit is not None:
            junit_path.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy(FIXTURES / junit, junit_path)
        return subprocess.CompletedProcess(cmd, rc, stdout, stderr)

    return mock.patch.object(build.subprocess, "run", side_effect=run)


def junit_path(profile="default") -> Path:
    return build._junit_path(profile)


def run_main(run_gate=None, baselines=None, **overrides):
    """Run main() as a full gate with every outside effect stubbed.

    ``run_gate(args, results)`` plays the gate. ``baselines`` is what the journal holds. Returns (printed lines, log text, exit code,
    exec argv).
    """
    printed = []
    memlog = _MemLog()
    args = gate_args(**overrides)
    with (
        mock.patch.object(build, "parse_args", return_value=args),
        mock.patch.object(build, "run_gate", side_effect=run_gate or (lambda a, r: None)),
        mock.patch.object(build, "clean_old_logs"),
        mock.patch.object(build.build_history, "tree_identity", return_value="abc"),
        mock.patch.object(build.build_history, "append"),
        mock.patch.object(build.build_history, "last_tier_sizes", return_value=baselines or {}),
        mock.patch("builtins.print", side_effect=lambda msg="": printed.append(msg)),
        mock.patch("builtins.open", return_value=memlog),
    ):
        code, exec_argv = build.main()
    return printed, memlog.getvalue(), code, exec_argv
