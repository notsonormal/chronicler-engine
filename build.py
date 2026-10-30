"""Full build, validate, and test for Chronicler Engine.

Uses cargo-nextest for parallel test execution.

Two invocation modes:

- Full gate (default): ``python build.py`` runs fmt, validation, clippy,
  guardrails, the full test suite, and packaging.
- Gate without the browser tier: ``python build.py --no-browser`` runs the same
  gate minus the browser (Playwright) tier.
- Step mode: ``python build.py <step>`` runs one registry step (e.g.
  ``clippy``, ``fmt``, ``test-pattern <pattern>``) with a minimal prelude — no
  asset copy, no SQLite cleanup. ``--target-dir`` and ``--fail-fast`` are
  accepted on either side of the step; all other top-level flags are gate-only
  and rejected next to a step.

Test failure mode:

- A step run stops at the first failing test.
- The full gate runs every test, so one red gate names every failure.
- ``--fail-fast`` and ``--no-fail-fast`` override either default, before or
  after the step name. The last one on the command line wins. ``--coverage``
  refuses ``--fail-fast``, because the coverage report needs every test.

``python build.py run [-- <server flags>]`` builds the binary and then replaces
this process with it, so the dev server links in the ``build.py`` environment
(lld linker, chosen target dir) and switching between a build step and the
server costs no recompile. The build runs under the machine-wide slot; the
server itself runs without any lock. The port is checked before the build and
again just before the exec. ``run`` is dev-profile only.

Structured output: every step is a ``Step`` that returns a ``StepResult``. A step run stops
at the first failing step. The full gate stops there too, except for the test tiers, which
record their failure and let the gate go on. ``main`` renders the epilogue from the results.
cargo runs with ``--message-format json`` (nextest with ``--cargo-message-format json``), and
nextest writes a JUnit report (``[profile.default.junit]`` in ``.config/nextest.toml``).
``scripts/build_report.py`` reads both: the compile errors, the failing tests with the first
line of their message, and the per-test durations of ``--test-timings``. Only nextest's
``Summary`` line and its ``LEAK``/``FLAKY`` lines are read as text, because the JUnit report
has no count of ignored tests and does not mark a leaking test.

Stdout carries the agent-facing decision signal + tailable progress (banner,
step labels, ``$ cmd`` echoes, failure signals, Step Timing Summary, closing
banner with log path). Full output is written to ``logs/build_*.log``; each
log's first line is a session stamp (see ``_stamp_session_id``). Every
completed run also appends one pipe-delimited summary line (timestamp,
duration, exit code, args, tree, per-tier test counts) to the append-only
journal ``logs/build_history.txt`` — see ``scripts/build_history.py``. Each test tier's
epilogue line compares its test count with that tier's newest journal record.

Concurrent builds:

Cargo compile and test steps queue on a machine-wide lock (``scripts/build_slot.py``), so builds
from different checkouts run one at a time. A cold target dir is seeded from a warm sibling
(``scripts/target_seed.py``); cargo links with the toolchain's lld (``scripts/lld-linker.sh``).
The lock does not cover ``cargo fmt``, the Python checks or ``cargo llvm-cov report``.
A step waits up to 30 minutes for it, then runs anyway.
Each script's docstring lists its ``CHRONICLER_*`` switches. Use one target dir per checkout::

    python build.py --target-dir target/agent2

The full gate also removes linked worktrees that are safe to lose: clean, pushed, and with no
live process inside (``scripts/remove_worktrees.py``). It also prunes the stale ``target/``
dirs of the linked worktrees it keeps, so a surviving tree does not hold a multi-GiB one for
good. It warns and continues if either fails.

``--cleanup`` deletes this checkout's whole cargo target dir (the next build is cold) and the
machine-wide port-lock dir ``<tmp>/chronicler_test_ports`` that every checkout's tests share.
Do not run it while another checkout is testing.

A full gate takes about 2 minutes on a warm target dir, about 15 on a cold unseeded one, plus any
wait for the build lock. Use a tool timeout of at least 1200 seconds. ``--coverage`` takes longer.
"""

from __future__ import annotations

import argparse
import contextlib
import errno
import io
import json
import os
import re
import shlex
import shutil
import socket
import sqlite3
import subprocess
import sys
import tempfile
import time
from collections import defaultdict
from dataclasses import dataclass, field
from pathlib import Path
from typing import Callable, NamedTuple

sys.path.insert(0, str(Path(__file__).resolve().parent / "scripts"))
import build_history  # noqa: E402
import build_report  # noqa: E402
import build_slot  # noqa: E402
import target_seed  # noqa: E402

# Force UTF-8 for stdout/stderr on Windows to handle cargo's Unicode output
if sys.platform == "win32":
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding="utf-8", errors="replace")


class _LogState:
    fh = None


def _set_log_fh(fh):
    _LogState.fh = fh


def log_status(msg: str = "") -> None:
    """Write a status line to the build log only (not stdout).

    Use for internal bookkeeping, warnings, and notices the agent doesn't need
    to see on stdout. The newline is appended automatically. Safe to call before
    ``_set_log_fh`` is called — the log write is skipped silently in that case.
    """
    if _LogState.fh is not None:
        _LogState.fh.write(msg + "\n")
        _LogState.fh.flush()


def both_print(msg: str = "") -> None:
    """Write a line to both stdout and the build log.

    Use for the agent-facing decision signal + tailable progress (header
    banner, step labels, ``$ cmd`` echoes, failure signals, Step Timing Summary,
    closing banner). Newlines embedded in ``msg`` are preserved.
    """
    print(msg)
    sys.stdout.flush()  # a build run detached with its output redirected stays pollable
    log_status(msg)


# Cargo progress lines are pure noise for an agent caller — every incremental
# build emits hundreds of "   Compiling foo v1.0.X" / "    Checking foo" lines.
# Strip them (and cargo-llvm-cov's info lines) before writing to the log, so the log
# carries only actionable content (warnings, errors, test results, build summary).
_LOG_NOISE_RE = re.compile(
    r"^\s*(?:(?:Compiling|Checking|Downloading|Updating|Adding|Building|Running `rustc`)\s+\S+"
    r"|info: cargo-llvm-cov)"
)


def _log_output(text: str) -> None:
    kept = [line for line in text.splitlines() if not _LOG_NOISE_RE.match(line)]
    if kept:
        log_status("\n".join(kept))


class StepCounter:
    def __init__(self, total: int):
        self.current = 0
        self.total = total

    def next(self, label: str):
        self.current += 1
        both_print(f"[{self.current}/{self.total}] {label}")


def check_rust_version():
    """Ensure rustc >= 1.85."""
    result = subprocess.run(
        ["rustc", "--version"],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        both_print("ERROR: Could not determine Rust version.")
        sys.exit(1)
    match = re.search(r"rustc (\d+)\.(\d+)", result.stdout)
    if not match:
        both_print("ERROR: Could not parse Rust version.")
        sys.exit(1)
    major, minor = int(match.group(1)), int(match.group(2))
    if major < 1 or (major == 1 and minor < 85):
        both_print(f"ERROR: Rust {major}.{minor} found, but >= 1.85 is required.")
        sys.exit(1)
    log_status(f"Rust version: {major}.{minor} (OK)")


def require_nextest():
    result = subprocess.run(
        ["cargo", "nextest", "--version"],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        both_print(
            "ERROR: The nextest library is required. Install it with: cargo install cargo-nextest --locked"
        )
        sys.exit(1)


# The browser binary runs the Playwright tier: each test spawns a real engine
# server plus Chromium. Splitting it from the CPU-light tiers keeps
# `python build.py integration` to ~20s and gives each tier its own epilogue
# verdict.
_BROWSER_FILTER = "binary(browser)"
_NON_BROWSER_FILTER = "not binary(browser)"
# The gate's guardrails step runs the architecture and guardrails binaries,
# so this filter drops them. Coverage uses _NON_BROWSER_FILTER to still
# instrument them.
_INTEGRATION_FILTER = (
    "not binary(browser) and not binary(architecture) and not binary(guardrails)"
)

# The nextest command line without a fail mode. `_with_fail_flag` adds the mode where
# a run knows it, so the gate and a step share one registry string.
_NEXTEST_RUN = "cargo nextest run"
_FAIL_FAST = "--fail-fast"
_NO_FAIL_FAST = "--no-fail-fast"
# The suffix of every run on the slow LLM tests. It also names the nextest profile.
_LLM_ARGS = " --profile llm --run-ignored all"
# The engine, http, storage, bootstrap and llm tiers are CPU-light; excluding the browser
# binary keeps this step to ~20s, so it works as a standalone check.
_INTEGRATION_CMD = f"{_NEXTEST_RUN} -E '{_INTEGRATION_FILTER}'"
_BROWSER_CMD = f"{_NEXTEST_RUN} -E '{_BROWSER_FILTER}'"


def _coverage_cmd(filter_expr: str) -> str:
    """The coverage command for one half of the gate's tier split.

    The coverage step runs twice, once per half (the non-browser tier and the browser binary),
    so both profiling runs share the single merged report that ``--no-report`` defers.
    """
    return f"cargo llvm-cov nextest --no-report --no-fail-fast -E '{filter_expr}'"


# How a failed command's output is read. cargo compile and test commands have JSON messages
# and need no entry. rustfmt has none, so its syntax errors are read from the text; the check
# also lists the files that need formatting.
OUTPUT_PLAIN = "plain"
OUTPUT_RUSTFMT = "rustfmt"
OUTPUT_RUSTFMT_CHECK = "rustfmt-check"


class StepSpec(NamedTuple):
    """One runnable build step, shared by the full gate and step subcommands.

    The registry is the single source of truth for step commands: the full
    gate and the step subcommands consume the same spec, so the two modes
    cannot drift into different command strings. ``tier`` names the journal
    column of a step that runs one whole test tier. ``output`` names how a failed
    run is read, see ``Step``.
    """

    name: str
    label: str
    cmd: str
    needs_nextest: bool = False
    pattern_arg: bool = False
    help: str = ""
    tier: str | None = None
    output: str = "plain"


# The `--test-target` name that selects the package's lib unit-test binary
# (`cargo nextest run --lib`) instead of one integration test binary.
LIB_TEST_TARGET = "lib"


def _fail_flag(args) -> str:
    """The fail mode of this run: an explicit flag wins, else the mode default.

    A step run answers whether the code is still red, and the rest of the tier
    cannot change a failure it already found. The gate runs every test, so one
    cycle names every failure.
    """
    explicit = getattr(args, "fail_fast", None)
    if explicit is not None:
        return _FAIL_FAST if explicit else _NO_FAIL_FAST
    return _FAIL_FAST if getattr(args, "command", None) else _NO_FAIL_FAST


def _with_fail_flag(cmd: str, args) -> str:
    """Append this run's fail mode to a nextest command. Others pass through."""
    if not cmd.startswith(_NEXTEST_RUN):
        return cmd
    return f"{cmd} {_fail_flag(args)}"


REGISTRY: dict[str, StepSpec] = {
    spec.name: spec
    for spec in [
        StepSpec(
            "install-hooks",
            "Installing git hooks...",
            "python scripts/install_git_hooks.py",
            help="Install or update git hooks; differing hooks are backed up first.",
        ),
        StepSpec(
            "remove-worktrees",
            "Removing safe worktrees and pruning stale target dirs...",
            "python scripts/remove_worktrees.py --apply --prune-targets",
            help=(
                "Remove linked worktrees that are clean, pushed, and have no live process"
                " inside; every other tree is refused with a reason. Also prunes the stale"
                " target dirs of the linked trees that survive, which is what keeps a"
                " multi-GiB target/ around; the main worktree's target dir is never pruned."
            ),
        ),
        StepSpec(
            "fmt",
            "Formatting...",
            "cargo fmt",
            help="Format sources in place (same as the full gate's fmt step).",
            output=OUTPUT_RUSTFMT,
        ),
        StepSpec(
            "validate-data",
            "Validating JSON data...",
            "python scripts/validate_data.py",
            help="Validate JSON data files against schemas.",
        ),
        StepSpec(
            "check",
            "Checking compilation...",
            "cargo check --all-targets --all-features",
            help="Fast compile check across all targets; no lint (lighter than clippy).",
        ),
        StepSpec(
            "clippy",
            "Running clippy...",
            "cargo clippy --all-targets --all-features -- -D warnings",
            help="Check formatting (cargo fmt --check), then lint Rust sources; warnings are errors.",
        ),
        StepSpec(
            "test-structure",
            "Running test structure guardrail...",
            "python scripts/check_test_structure.py",
            help="Enforce Rust unit-test structure rules.",
        ),
        StepSpec(
            "spec-coverage",
            "Running spec-coverage guardrail...",
            "python scripts/validate_feature_spec.py",
            help="Enforce spec-scenario coverage and SCENARIO-tag rules.",
        ),
        StepSpec(
            "seam-cases",
            "Running seam-case guardrail...",
            "python scripts/validate_seam_cases.py",
            help=(
                "Enforce the seam-contract rule: every test in a declaring file"
                " carries its CASE tag, and every tag carries a description."
            ),
        ),
        StepSpec(
            "docstrings",
            "Running Python docstring guardrail...",
            "python scripts/check_python_docstrings.py",
            help="Check Python scripts for missing module docstrings.",
        ),
        StepSpec(
            "py-tests",
            "Running Python tests...",
            "python -m unittest discover scripts/tests -v",
            help="Run the Python unit tests under scripts/tests.",
        ),
        StepSpec(
            "duplicates",
            "Checking for duplicate code...",
            "python scripts/healthcheck.py duplicates",
            help=(
                "Report duplicate-code clones, scoped to files changed vs main"
                " (report-only; pass --all for the whole repo or --ref <ref>)."
            ),
        ),
        StepSpec(
            "http-routes-check",
            "Checking http_routes.md freshness...",
            "python scripts/extract_http_routes.py --check",
            help="Fail when the generated http_routes.md is stale.",
        ),
        StepSpec(
            "guardrails-doc-check",
            "Checking guardrails.md freshness...",
            "python scripts/generate_guardrails_doc.py --check",
            help="Fail when the generated guardrails doc is stale.",
        ),
        StepSpec(
            "validate-docs",
            "Validating markdown docs...",
            "python scripts/validate_docs.py",
            help="Validate markdown docs and DOC anchors under docs/.",
        ),
        StepSpec(
            "guardrails",
            "Running guardrail tests...",
            f"{_NEXTEST_RUN} --test architecture --test guardrails",
            needs_nextest=True,
            tier="guardrails",
            help="Run the architecture and guardrails test binaries.",
        ),
        StepSpec(
            "integration",
            "Running integration tests...",
            _INTEGRATION_CMD,
            needs_nextest=True,
            tier="integration",
            help=(
                "Run every test binary except browser, architecture and guardrails,"
                " the lib unit tests included (~20s warm)."
            ),
        ),
        StepSpec(
            "browser",
            "Running browser tests...",
            _BROWSER_CMD,
            needs_nextest=True,
            tier="browser",
            help="Run only the browser (Playwright) test binary (~1.5 min warm).",
        ),
        StepSpec(
            "test-pattern",
            "Running test-pattern step...",
            _NEXTEST_RUN,
            needs_nextest=True,
            pattern_arg=True,
            help=(
                "Run the tests whose name matches a pattern. With"
                " `--test-target <name>` only that test binary is built and run"
                f" (`--test <name>`, or `--test-target {LIB_TEST_TARGET}` for"
                " `--lib`); with no target, every test binary runs."
            ),
        ),
        StepSpec(
            "run",
            "Building dev server...",
            "cargo build --bin chronicler_engine",
            help=(
                "Build the dev server and replace this process with the binary."
                " Server flags go after `--`, e.g. `run -- --port 3099`."
            ),
        ),
    ]
}

# Top-level flags that only make sense for the full gate. Rejected next to a
# step subcommand so invalid combinations fail at parse time.
_GATE_ONLY_FLAGS = (
    "coverage",
    "release",
    "include_llm",
    "llm_only",
    "no_fmt",
    "no_browser",
    "cleanup",
    "test_timings",
)


def _add_fail_mode_flags(parser, default=None):
    """Add the two fail-mode flags, accepted before or after a step name.

    Both flags write one dest, so the last one on the command line wins.
    ``default=None`` keeps "not given" distinct from either value. A step
    subparser passes ``SUPPRESS``, so it does not overwrite a flag typed before
    the step name.
    """
    parser.add_argument(
        "--fail-fast",
        dest="fail_fast",
        action="store_const",
        const=True,
        default=default,
        help=(
            "Stop the test tier at the first failing test (a step run does this"
            " by default; the full gate reports every failure)"
        ),
    )
    parser.add_argument(
        "--no-fail-fast",
        dest="fail_fast",
        action="store_const",
        const=False,
        default=default,
        help=(
            "Run every test and report every failure (the full gate does this"
            " by default; a step run stops at the first failure)"
        ),
    )


def parse_args(argv=None):
    """Parse CLI arguments into a namespace. Pure: no filesystem or subprocess
    side effects.

    Step subcommands share the top-level ``--target-dir`` flag; the shared copy
    uses a ``SUPPRESS`` default so a value given before the subcommand is not
    clobbered by the subparser.
    """
    parser = argparse.ArgumentParser(description="Chronicler Engine build script")
    parser.add_argument(
        "--coverage",
        action="store_true",
        help="Run tests with coverage instrumentation (slower, useful for CI)",
    )
    parser.add_argument(
        "--release",
        action="store_true",
        help="Build and package in release mode",
    )
    parser.add_argument(
        "--include-llm",
        action="store_true",
        dest="include_llm",
        help="Include slow LLM tests in the test suite",
    )
    parser.add_argument(
        "--llm-only",
        action="store_true",
        dest="llm_only",
        help=(
            "Run only the slow LLM tests (skips formatting, clippy, guardrails, and other tests)"
        ),
    )
    parser.add_argument(
        "--target-dir",
        dest="target_dir",
        default=None,
        help="Custom cargo target directory for isolated builds (e.g., target/agent2)",
    )
    parser.add_argument(
        "--no-fmt",
        action="store_true",
        dest="no_fmt",
        help="Skip cargo fmt",
    )
    parser.add_argument(
        "--no-browser",
        action="store_true",
        dest="no_browser",
        help=(
            "Run the full gate without the browser (Playwright) tier"
        ),
    )
    parser.add_argument(
        "--cleanup",
        action="store_true",
        dest="cleanup",
        help="Delete this checkout's target dir and the machine-wide test port-lock dir. "
        "Do not run while another checkout is testing.",
    )
    parser.add_argument(
        "--test-timings",
        action="store_true",
        dest="test_timings",
        help=(
            "Print a per-test timing report after the test step: the slowest tests"
            " sorted by wall-clock duration, plus per-binary totals."
            " Surfaces nextest's per-test durations without a separate command."
        ),
    )
    _add_fail_mode_flags(parser)

    sub = parser.add_subparsers(dest="command", metavar="<step>")
    for spec in REGISTRY.values():
        step_parser = sub.add_parser(spec.name, help=spec.help)
        _add_fail_mode_flags(step_parser, default=argparse.SUPPRESS)
        step_parser.add_argument(
            "--target-dir",
            dest="target_dir",
            default=argparse.SUPPRESS,
            help=argparse.SUPPRESS,
        )
        if spec.pattern_arg:
            step_parser.add_argument(
                "pattern", help="Test-name pattern passed to cargo nextest"
            )
            step_parser.add_argument(
                "--test-target",
                dest="test_target",
                default=None,
                metavar="NAME",
                help=(
                    "Build and run only one test binary (`--test NAME`), or the"
                    f" lib unit tests (`{LIB_TEST_TARGET}`). Default: every test binary."
                ),
            )
        if spec.name == "run":
            # REMAINDER keeps everything after the step (including a leading
            # ``--``) so the flags reach the server binary unparsed. Strip the
            # ``--`` once in _strip_passthrough, not here.
            step_parser.add_argument(
                "server_flags",
                nargs=argparse.REMAINDER,
                help="Flags forwarded verbatim to the server binary",
            )
        if spec.name == "duplicates":
            step_parser.add_argument(
                "--all",
                action="store_true",
                dest="duplicates_all",
                help="Report the whole repo instead of only changed files",
            )
            step_parser.add_argument(
                "--ref",
                dest="duplicates_ref",
                default=None,
                help="Base ref for the changed-files scope (default: main)",
            )

    args = parser.parse_args(argv)
    _reject_gate_flags_with_step(parser, args)
    if args.coverage and args.no_browser:
        parser.error(
            "--no-browser cannot be combined with --coverage: coverage needs the browser tier"
        )
    if args.coverage and getattr(args, "fail_fast", None):
        parser.error(
            "--fail-fast cannot be combined with --coverage: the coverage report"
            " needs the result of every test"
        )
    return args


def _reject_gate_flags_with_step(parser, args):
    """Fail at parse time, before any step runs."""
    if args.command is None:
        return
    offenders = [
        f"--{name.replace('_', '-')}"
        for name in _GATE_ONLY_FLAGS
        if getattr(args, name, False)
    ]
    if offenders:
        parser.error(
            f"{', '.join(offenders)} only apply to the full gate and cannot be"
            f" combined with the '{args.command}' step"
        )


def _stamp_session_id():
    """Stamp the log with the originating pi session id, when known.

    Written as the first log line. mrn-context (.pi/extensions/mrn-context)
    reads this header and only surfaces the build-log pointer to the session
    that produced the build (unstamped logs stay visible to everyone).
    """
    session_id = os.environ.get("PI_SESSION_ID", "").strip()
    if session_id:
        log_status(f"Session-Id: {session_id}")


def _target_paths(args) -> tuple[Path, Path]:
    """Return (cargo_target_dir, profile target dir) for the given args."""
    cargo_target_dir = (
        Path(args.target_dir) if getattr(args, "target_dir", None) else Path("target")
    )
    build_profile = "release" if args.release else "debug"
    return cargo_target_dir, cargo_target_dir / build_profile


def _lld_linker_env(host=None, cache_dir=None) -> dict:
    """Point cargo at the lld wrapper, installed at one path shared by every checkout."""
    if os.environ.get("CHRONICLER_NO_LLD") or not sys.platform.startswith("linux"):
        return {}
    if host is None:
        try:
            host = target_seed.rustc_host_triple()
        except RuntimeError:
            return {}
    var = "CARGO_TARGET_" + re.sub(r"[^A-Z0-9]", "_", host.upper()) + "_LINKER"
    if os.environ.get(var):
        return {}
    cache = cache_dir or Path(os.environ.get("XDG_CACHE_HOME") or Path.home() / ".cache")
    source = Path(__file__).resolve().parent / "scripts" / "lld-linker.sh"
    installed = Path(cache) / "chronicler-engine" / "lld-linker.sh"
    try:
        installed.parent.mkdir(parents=True, exist_ok=True)
        if not installed.exists() or installed.read_bytes() != source.read_bytes():
            staging = installed.with_name(f"lld-linker.sh.{os.getpid()}")
            shutil.copyfile(source, staging)
            staging.chmod(0o755)
            os.replace(staging, installed)
    except OSError:
        return {}
    return {var: str(installed)}


def _cargo_env_for(args) -> dict:
    """Cargo environment shared by gate and step runs.

    ``leak`` prints FAIL/RETRY/SLOW/LEAK lines but suppresses the PASS flood, so
    a passing-but-leaking test is named instead of only counted. The JUnit report,
    not the PASS lines, gives the ``--test-timings`` durations.
    """
    env = {"NEXTEST_STATUS_LEVEL": "leak", **_lld_linker_env()}
    if getattr(args, "target_dir", None):
        env["CARGO_TARGET_DIR"] = str(Path(args.target_dir).resolve())
    return env


def _warn_if_target_locked(cargo_target_dir: Path, custom: bool):
    """Warn when cargo holds the target directory (another agent may be building)."""
    if not is_target_locked(cargo_target_dir):
        return
    if custom:
        both_print(
            f"WARNING: Target directory {cargo_target_dir} appears to be "
            "locked by another cargo process."
        )
        both_print("         Another agent may be building in this directory.")
    else:
        both_print(
            "WARNING: Default target directory (target/) appears to be "
            "locked by another cargo process."
        )
        both_print(
            "         Use --target-dir to build in a unique folder and avoid conflicts:"
        )
        both_print("         python build.py --target-dir target/<unique-name>")


def _step_command(
    spec: StepSpec,
    pattern: str | None = None,
    extra: list[str] | None = None,
    test_target: str | None = None,
) -> str:
    """Assemble the shell command for a step spec (quotes the nextest pattern).

    ``test_target`` narrows a ``pattern_arg`` step to one test binary, so a
    targeted run compiles only that binary (``--test <NAME>``, or ``--lib``
    for ``LIB_TEST_TARGET``). No target keeps the all-binaries command.
    """
    if spec.pattern_arg:
        target_flags = _test_target_flags(test_target)
        cmd = f"{spec.cmd}{target_flags} {shlex.quote(pattern)}"
    else:
        cmd = spec.cmd
    if extra:
        cmd = " ".join([cmd, *(shlex.quote(arg) for arg in extra)])
    return cmd


def _test_target_flags(test_target: str | None) -> str:
    if not test_target:
        return ""
    if test_target == LIB_TEST_TARGET:
        return " --lib"
    return f" --test {shlex.quote(test_target)}"


def clean_sqlite_dbs(data_dir: Path):
    if not data_dir.exists():
        return
    removed = []
    for pattern in ["*.db", "*.db-journal", "*.db-wal", "*.db-shm"]:
        for f in data_dir.glob(pattern):
            f.unlink()
            removed.append(f.name)
    if removed:
        log_status(f"  Removed stale SQLite DBs: {', '.join(removed)}")


def clean_old_logs(log_dir: Path, max_age_days: int = 3):
    if not log_dir.exists():
        return
    now = time.time()
    max_age_sec = max_age_days * 86400
    removed = []
    for f in log_dir.iterdir():
        if (
            f.is_file()
            and f.name.startswith("build_")
            and f.suffix == ".log"
            and (now - f.stat().st_mtime) > max_age_sec
        ):
            f.unlink()
            removed.append(f.name)
    if removed:
        log_status(f"  Removed old build logs (> {max_age_days} days): {', '.join(removed)}")


def clean_tmp_dirs(tmp_dirs: list[Path], max_age_days: int = 30):
    """Recurses into subdirectories, which are themselves left in place."""
    now = time.time()
    max_age_sec = max_age_days * 86400
    for tmp_dir in tmp_dirs:
        if not tmp_dir.exists():
            continue
        removed = []
        for root, _dirs, files in os.walk(tmp_dir):
            for name in files:
                p = Path(root) / name
                try:
                    if (now - p.stat().st_mtime) <= max_age_sec:
                        continue
                    p.unlink()
                    removed.append(str(p))
                except Exception as e:
                    log_status(f"  Warning: Could not remove {p}: {e}")
        if removed:
            log_status(f"  Removed stale entries (> {max_age_days} days) in {tmp_dir}: {', '.join(removed)}")


def dump_sqlite_to_jsonl(db_path: Path, output_dir: Path):
    """Dump all tables from a SQLite database to JSONL files.

    One file per table: {output_dir}/{table_name}.jsonl
    Each line is a JSON object representing one row.
    """
    if not db_path.exists():
        return
    output_dir.mkdir(parents=True, exist_ok=True)
    try:
        conn = sqlite3.connect(str(db_path))
        conn.row_factory = sqlite3.Row
        cursor = conn.cursor()


        cursor.execute(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"
        )
        tables = [row[0] for row in cursor.fetchall()]

        dumped = []
        for table in tables:
            cursor.execute(f'SELECT * FROM "{table}"')
            rows = cursor.fetchall()
            if not rows:
                continue
            out_file = output_dir / f"{table}.jsonl"
            with open(out_file, "w", encoding="utf-8") as f:
                for row in rows:
                    record = dict(row)
                    f.write(json.dumps(record, ensure_ascii=False, default=str) + "\n")
            dumped.append(f"{table} ({len(rows)} rows)")

        conn.close()
        if dumped:
            log_status(f"  Dumped tables to {output_dir}/: {', '.join(dumped)}")
        else:
            log_status(f"  No data to dump in {db_path}")
    except Exception as e:
        log_status(f"  Warning: Could not dump SQLite DB: {e}")


def is_target_locked(target_dir: Path) -> bool:
    """Check if cargo holds a build lock on a profile dir under ``target_dir``.

    Delegates to target_seed's shared-flock probe, so a target seed in progress (which holds the
    lock shared) does not read as a cargo build.
    """
    for profile in ["debug", "release"]:
        handle = target_seed.probe_shared_lock(target_dir / profile / ".cargo-lock")
        if handle is None:
            return True  # An exclusive holder (cargo) has the lock
        handle.close()  # We got the shared lock, so cargo doesn't hold it
    return False  # No lock file, or shared holders only, assume not locked



@dataclass
class StepResult:
    """What one step left behind: the outcome, and the facts the epilogue and the journal print.

    ``fatal`` marks the failure that stops the run. ``tail`` holds the cause of a failed step
    that is not a nextest run and has no compile error: its last output lines.
    """

    label: str
    rc: int = 0
    elapsed: float = 0.0
    report_only: bool = False
    fatal: bool = False
    tier: str | None = None
    counts: build_report.Counts | None = None
    leaks: list[tuple[str, str]] = field(default_factory=list)
    failures: list[tuple[str, str]] = field(default_factory=list)
    compile_errors: list[str] = field(default_factory=list)
    tail: list[str] = field(default_factory=list)

    @property
    def failed(self) -> bool:
        """A report-only step that did not complete only warns."""
        return self.rc != 0 and not self.report_only


class Step(NamedTuple):
    """One entry of a run plan: a shell command, or an in-process action.

    A step with neither is a counted marker. ``check=False`` lets the run go on after a
    failure; the failure still shows and still fails the run's exit code. ``report_only`` makes a
    failure a warning. ``tier`` names the journal column of a step that runs a whole test tier.
    ``profile`` is the nextest profile of a nextest run (empty for any other step): the run
    leaves a JUnit report for that profile. ``output`` is how a failed run is read (``OUTPUT_*``).
    ``before`` and ``after`` are notes printed around the step.
    """

    label: str
    cmd: str = ""
    action: Callable[[Ctx], int | None] | None = None
    tier: str | None = None
    profile: str = ""
    output: str = OUTPUT_PLAIN
    check: bool = True
    report_only: bool = False
    timings: bool = False
    before: str = ""
    after: str = ""


@dataclass
class Ctx:
    """The state one run shares between its steps."""

    args: argparse.Namespace
    env: dict
    results: list[StepResult]
    seeded: bool = False


def _prelude(args, results, nextest: bool = True) -> Ctx:
    """Check the toolchain, warn about a busy target dir, and build the run context."""
    check_rust_version()
    if nextest:
        require_nextest()
    cargo_target_dir, _ = _target_paths(args)
    custom = bool(getattr(args, "target_dir", None))
    if custom:
        log_status(f"Using custom target directory: {cargo_target_dir}")
    _warn_if_target_locked(cargo_target_dir, custom=custom)
    return Ctx(args, _cargo_env_for(args), results)


def _seed_cold_target(ctx: Ctx) -> None:
    """Seed a cold target dir once per run, inside the slot so the source is idle."""
    if ctx.seeded:
        return
    ctx.seeded = True
    cargo_target_dir, profile_dir = _target_paths(ctx.args)
    try:
        target_seed.seed_if_cold(Path.cwd(), cargo_target_dir.resolve(), profile_dir.name, both_print)
    except Exception as err:  # seeding is an optimisation and must never fail the build
        both_print(f"Target seeding skipped: {err}")


def _stamp_target(ctx: Ctx) -> None:
    _, profile_dir = _target_paths(ctx.args)
    try:
        target_seed.stamp(profile_dir.resolve(), target_seed.build_signature(Path.cwd()))
    except (OSError, subprocess.SubprocessError):
        pass


@contextlib.contextmanager
def _cargo_slot(cmd: str, label: str, ctx: Ctx):
    heavy = build_slot.is_heavy(cmd)
    with build_slot.maybe_hold(cmd, label, both_print):
        if heavy:
            _seed_cold_target(ctx)
        yield
        if heavy:
            _stamp_target(ctx)


# cargo prints its diagnostics as JSON lines on stdout and its progress on stderr. The flag
# goes right after the subcommand, so it stays before clippy's `--`.
_JSON_OUTPUT_FLAGS = (
    (re.compile(r"^cargo (?:check|clippy|build)\b"), " --message-format json"),
    (re.compile(r"^cargo (?:nextest run|llvm-cov nextest)\b"), " --cargo-message-format json"),
)


def _with_json_output(cmd: str) -> str | None:
    """The command with cargo's JSON message flag, or None when it is not a cargo compile."""
    for pattern, flag in _JSON_OUTPUT_FLAGS:
        match = pattern.match(cmd)
        if match:
            return cmd[: match.end()] + flag + cmd[match.end() :]
    return None


class _Run(NamedTuple):
    rc: int
    text: str  # the output as logged: rendered compiler output, then plain lines
    errors: list[str]  # compile errors read from cargo's JSON stream


def _run_command(cmd: str, env: dict, note: str = "") -> _Run:
    """Run a shell command and write its output to the log only (not stdout).

    The ``$ {command}`` echo, with the exact command that runs, goes to both. A cargo compile or test command runs with JSON
    messages: stdout and stderr stay apart, and the rendered compiler output goes to the log
    in place of the JSON. Any other command has stderr merged into stdout.
    """
    structured = _with_json_output(cmd)
    both_print(f"$ {structured or cmd}{note}")
    proc = subprocess.run(
        structured or cmd,
        shell=True,
        cwd=os.getcwd(),
        env={**os.environ, **env},
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE if structured else subprocess.STDOUT,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if structured:
        cargo = build_report.read_cargo_stream(proc.stdout)
        text, errors = f"{cargo.text}\n{proc.stderr}", cargo.errors
    else:
        text, errors = proc.stdout, []
    _log_output(text)
    return _Run(proc.returncode, text, errors)


# nextest writes its JUnit report under the workspace root, whatever CARGO_TARGET_DIR or
# --target-dir says (also under cargo-llvm-cov), and build.py runs from the repo root. So the
# target dirs of one checkout share one report file per profile. That is safe because the
# build slot serializes every nextest step: each step deletes the file, runs, and reads it
# before the next step starts. (Slot off, or two steps at once, can mix reports.)
_NEXTEST_DIR = Path("target") / "nextest"


def _junit_path(profile: str) -> Path:
    """Where nextest leaves its JUnit report for a profile."""
    return _NEXTEST_DIR / profile / "junit.xml"


_FORMAT_CHECK_CMD = "cargo fmt --check -- -l"
_FORMAT_CHECK_LABEL = "Checking formatting..."
_FMT_FILE_LIMIT = 10


def _unformatted_detail(text: str) -> list[str]:
    files = build_report.unformatted_files(text)
    detail = [f"unformatted: {name}" for name in files[:_FMT_FILE_LIMIT]]
    if len(files) > _FMT_FILE_LIMIT:
        detail.append(f"... {len(files) - _FMT_FILE_LIMIT} more files")
    return (detail or build_report.tail(text)) + ["fix: python build.py fmt"]


def _run_cmd_step(step: Step, ctx: Ctx, result: StepResult) -> None:
    """Run a command step and read what it left: counts, leaks, failing tests, compile errors.

    The JUnit report is a file that earlier runs leave behind, so it is deleted before the run
    and read only when this run wrote it.
    """
    junit = _junit_path(step.profile) if step.profile else None
    if junit is not None:
        junit.unlink(missing_ok=True)
    run = _run_command(step.cmd, ctx.env, "  (with --test-timings)" if step.timings else "")
    result.rc = run.rc
    result.counts = build_report.parse_summary(run.text)
    result.leaks = build_report.find_leaks(run.text)
    result.compile_errors = run.errors
    report = None
    if junit is not None and (run.rc != 0 or step.timings):
        report = build_report.read_junit(junit)
        result.failures = report.failures if report is not None else []
    if step.timings:
        _print_timing_report(report)
    if step.output != OUTPUT_PLAIN:
        run.errors.extend(build_report.text_errors(run.text))
        result.compile_errors = run.errors
    if step.output == OUTPUT_RUSTFMT_CHECK and run.rc != 0 and not run.errors:
        result.tail = _unformatted_detail(run.text)
    elif run.rc != 0 and result.counts is None and not run.errors:
        result.tail = build_report.tail(run.text)


def _run_step(step: Step, ctx: Ctx) -> StepResult:
    """Run one step under the build slot. The recorded time excludes the slot wait."""
    result = StepResult(step.label, report_only=step.report_only, tier=step.tier)
    with _cargo_slot(step.cmd, step.label, ctx):
        start = time.time()
        if step.action:
            result.rc = step.action(ctx) or 0
        else:
            _run_cmd_step(step, ctx, result)
        result.elapsed = round(time.time() - start, 2)
    if result.rc != 0 and step.report_only:
        both_print(
            f"    Warning: {step.label.rstrip('.')} did not complete (report-only step);"
            " the build continues."
        )
    elif result.rc != 0 and step.check:
        result.fatal = True
        both_print(f"FAILED with code {result.rc}")
    return result


def _run_plan(plan: list[Step], ctx: Ctx) -> bool:
    """Run the steps in order, one result each. Returns False when a fatal failure stopped it.

    The counter total is len(plan), so the count cannot drift from the steps actually run.
    """
    counter = StepCounter(len(plan))
    for step in plan:
        counter.next(step.label)
        if not (step.cmd or step.action):
            continue
        if step.before:
            both_print(step.before)
        result = _run_step(step, ctx)
        ctx.results.append(result)
        if step.after:
            both_print(step.after)
        if result.fatal:
            return False
    return True


def _print_timing_report(report: build_report.JunitReport | None) -> None:
    """The slowest tests (top 30), per-binary totals, and the slowest stub checks inside runners."""
    both_print("")
    both_print("--- Test Timing Report ---")
    if report is None or not report.times:
        both_print("  No per-test durations in the nextest JUnit report.")
    else:
        # A test name is "<binary_id> <test_path>"; group by binary_id.
        per_binary = defaultdict(float)
        for secs, name in report.times:
            per_binary[name.split(" ", 1)[0]] += secs
        both_print(f"  Slowest tests (of {len(report.times)} measured, top 30):")
        for secs, name in sorted(report.times, reverse=True)[:30]:
            both_print(f"    {secs:>8.3f}s  {name}")
        both_print("")
        both_print("  Per-binary totals:")
        for binary, total in sorted(per_binary.items(), key=lambda kv: kv[1], reverse=True):
            both_print(f"    {total:>8.2f}s  {binary}")
        both_print(f"    {'':>8}   Sum of measured: {sum(t[0] for t in report.times):.2f}s")
    if report is not None and report.stub_checks:
        both_print("")
        both_print(f"  Slowest stub checks (of {len(report.stub_checks)}, top 30):")
        for secs, runner, name, status in sorted(report.stub_checks, reverse=True)[:30]:
            both_print(f"    {secs:>8.3f}s  [{status}]  {name}  ({runner.rsplit('::', 1)[-1]})")
    both_print("---")


def _spec_step(name: str, args, *, report_only: bool = False, cmd: str | None = None) -> Step:
    """The plan step of a registry spec. A report-only step never stops the run.

    ``cmd`` replaces the spec's command, for a step run that adds a pattern or flags.
    """
    spec = REGISTRY[name]
    return Step(
        spec.label,
        _with_fail_flag(cmd or spec.cmd, args),
        tier=spec.tier,
        profile="default" if spec.needs_nextest else "",
        output=spec.output,
        check=not report_only,
        report_only=report_only,
    )


# The gate's checks between the formatting and the asset copy, in order.
_GATE_CHECKS = (
    "validate-data",
    "clippy",
    "test-structure",
    "spec-coverage",
    "seam-cases",
    "docstrings",
    "py-tests",
    "http-routes-check",
    "guardrails-doc-check",
    "validate-docs",
)
_LLM_SKIPPED_NOTE = (
    "    NOTE: 2 LLM tests were skipped. Run 'python build.py --llm-only' to execute them."
)


def _test_steps(args) -> list[Step]:
    """The gate runs the test suite as two nextest invocations — the integration tier, then
    the browser tier — so the browser binary gets its own epilogue verdict and its own
    fail-fast exit. A tier with coverage or the LLM tests runs a different test set than
    the fixed tier, so it records no size.
    """
    if args.coverage:
        tiers = [
            ("integration", _coverage_cmd(_NON_BROWSER_FILTER), None, "default"),
            ("browser", _coverage_cmd(_BROWSER_FILTER), None, "default"),
        ]
        suffix = " with coverage"
    else:
        integration = (
            ("integration", _INTEGRATION_CMD + _LLM_ARGS, None, "llm")
            if args.include_llm
            else ("integration", _INTEGRATION_CMD, "integration", "default")
        )
        tiers = [integration, ("browser", _BROWSER_CMD, "browser", "default")]
        suffix = ""
    tail = " and timings..." if args.test_timings else "..."
    steps = [
        Step(
            f"Running {name} tests{suffix}{tail}",
            _with_fail_flag(cmd, args),
            tier=tier,
            profile=profile,
            check=False,
            timings=args.test_timings,
        )
        for name, cmd, tier, profile in tiers
        if not (name == "browser" and args.no_browser)
    ]
    if not args.coverage and not args.include_llm:
        steps[0] = steps[0]._replace(after=_LLM_SKIPPED_NOTE)
    return steps


def _plan_gate_steps(args) -> list[Step]:
    """The concrete gate plan.

    The order is load-bearing: the guardrails step (the architecture and guardrails binaries)
    runs before the ~2-minute full suite and fails the build immediately — otherwise a
    guardrail failure only surfaces after the full suite has run.
    """
    plan = [
        _spec_step("install-hooks", args, report_only=True),
        _spec_step("remove-worktrees", args, report_only=True),
    ]
    if not args.no_fmt:
        plan.append(_spec_step("fmt", args))
    plan += [_spec_step(name, args) for name in _GATE_CHECKS]
    plan += [
        _spec_step("duplicates", args, report_only=True),
        Step(
            "Copying data and assets for deployment...",
            action=lambda ctx: _copy_deployment_assets(ctx.args),
        ),
        _spec_step("guardrails", args),
        *_test_steps(args),
    ]
    if args.coverage:
        plan.append(
            Step(
                "Generating coverage report...",
                action=_generate_coverage_report,
                check=False,
                report_only=True,
            )
        )
    else:
        plan.append(Step("Skipping coverage report (use --coverage to enable)"))
    return plan


def _copy_deployment_assets(args):
    _, target_dir = _target_paths(args)
    target_dir.mkdir(parents=True, exist_ok=True)

    for src_name in ("data", "assets"):
        src = Path(src_name)
        if not src.exists():
            continue
        dest = target_dir / src_name
        if dest.exists():
            shutil.rmtree(dest)
        shutil.copytree(src, dest)
        log_status(f"  Copied {src_name}/ -> {dest}")

    (target_dir / "logs").mkdir(exist_ok=True)
    log_status("  Created logs/")

    log_status(f"  Package ready in {target_dir}/")
    log_status(f"  Deployment: copy {target_dir}/ folder to your target machine")

    # DB lives inside the target folder so each build profile has its own instance.
    clean_sqlite_dbs(target_dir / "data")


def _generate_coverage_report(ctx: Ctx) -> int:
    cargo_target_dir, _ = _target_paths(ctx.args)
    json_path = cargo_target_dir / "llvm-cov" / "coverage.json"
    json_path.parent.mkdir(parents=True, exist_ok=True)
    # Excluded: code that no test reaches (the process entry, the logging setup, the
    # real-HTTP LLM client), and test support.
    ignore_regex = (
        r"src[\\/]main\.rs|bootstrap[\\/]logging\.rs"
        r"|llm[\\/]transport[\\/]utils[\\/]client\.rs|test_support[\\/].*\.rs"
    )
    report = _run_command(
        f'cargo llvm-cov report --json --output-path "{json_path}" --ignore-filename-regex "{ignore_regex}"',
        ctx.env,
    )
    if not json_path.exists():
        both_print("Warning: Could not generate coverage JSON.")
        return report.rc or 1
    summary = _run_command(f'python scripts/parse_coverage.py --json "{json_path}"', ctx.env)
    return report.rc or summary.rc


def run_gate(args, results):
    ctx = _prelude(args, results)
    if args.no_fmt:
        both_print("Skipping formatting (--no-fmt set).")
    if args.no_browser:
        both_print("Skipping the browser tier (--no-browser set).")
    if _run_plan(_plan_gate_steps(args), ctx):
        _gate_tail(args)


def _step_plan(args) -> list[Step]:
    """One registry step. The narrow ``clippy`` step runs ``cargo fmt --check`` first; the
    full gate runs ``fmt``, which rewrites the files, so it needs no check.
    """
    spec = REGISTRY[args.command]
    command = _step_command(
        spec,
        getattr(args, "pattern", None),
        _step_extra_args(spec, args),
        getattr(args, "test_target", None),
    )
    step = _spec_step(spec.name, args, cmd=command)
    if spec.name == "clippy":
        return [Step(_FORMAT_CHECK_LABEL, _FORMAT_CHECK_CMD, output=OUTPUT_RUSTFMT_CHECK), step]
    return [step]


def run_step(args, results):
    """Single-step mode: run one registry step with a minimal prelude.

    Skips the gate-only prelude (asset copy, SQLite cleanup)
    so quick iteration does not disturb a running dev server.
    """
    ctx = _prelude(args, results, nextest=REGISTRY[args.command].needs_nextest)
    _run_plan(_step_plan(args), ctx)


def _gate_tail(args):
    project_root_tmp = Path(__file__).resolve().parent.parent / "tmp"
    engine_tmp = Path("tmp")
    clean_tmp_dirs([project_root_tmp, engine_tmp], max_age_days=30)

    _, target_dir = _target_paths(args)
    db_path = target_dir / "data" / "chronicler.db"
    if db_path.exists():
        dump_dir = Path("tmp") / "db_dumps"
        dump_sqlite_to_jsonl(db_path, dump_dir)


def _step_extra_args(spec: StepSpec, args) -> list[str] | None:
    if spec.name == "duplicates":
        extra: list[str] = []
        if getattr(args, "duplicates_all", False):
            extra.append("--all")
        ref = getattr(args, "duplicates_ref", None)
        if ref:
            extra.extend(["--ref", ref])
        return extra
    return None


# Server bind defaults, mirrored from src/utils/cli.rs (--port 3000, --host 0.0.0.0).
# Kept here so the pre-build port check can run before any cargo command.
_DEFAULT_PORT = 3000
_DEFAULT_HOST = "0.0.0.0"


def _strip_passthrough(flags: list[str]) -> list[str]:
    """Drop exactly one leading ``--`` that separates build.py flags from server flags."""
    if flags and flags[0] == "--":
        return flags[1:]
    return list(flags)


def _parse_bind_target(flags: list[str]) -> tuple[int, str] | None:
    """Return the (port, host) the server will bind, or None to skip the check.

    Accepts ``--port N``/``--port=N`` and ``--host H``/``--host=H``; later flags
    win. Returns None when the flags ask for a non-serving mode
    (``--list-worlds``, ``--help``, ``--version``) or carry a value the parser
    cannot understand — the server's own clap parser produces the authoritative
    error in those cases.
    """
    if any(
        flag in ("--list-worlds", "--help", "-h", "--version", "-V")
        for flag in flags
    ):
        return None
    port, host = _DEFAULT_PORT, _DEFAULT_HOST
    for index, flag in enumerate(flags):
        value = None
        if flag in ("--port", "--host"):
            if index + 1 >= len(flags):
                return None
            value = flags[index + 1]
        elif flag.startswith("--port="):
            value = flag.split("=", 1)[1]
        elif flag.startswith("--host="):
            value = flag.split("=", 1)[1]
        if value is None:
            continue
        if flag.startswith("--port"):
            try:
                port = int(value)
            except ValueError:
                return None
            if not 0 <= port <= 65535:
                return None
        else:
            host = value
    return port, host


def _port_in_use(host: str, port: int) -> bool:
    """Probe whether the server's bind address is already taken.

    Sets SO_REUSEADDR, as tokio's Unix listener does, so the probe reports the
    same conflict the real bind would. Other bind failures (e.g. a host the
    kernel rejects) read as free: the server's own bind decides. mio omits
    SO_REUSEADDR on Windows, so there a TIME_WAIT port reads free and the
    server's own bind stays authoritative.
    """
    family = socket.AF_INET6 if ":" in host else socket.AF_INET
    with socket.socket(family, socket.SOCK_STREAM) as probe:
        probe.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        try:
            probe.bind((host, port))
        except OSError as exc:
            return exc.errno == errno.EADDRINUSE
    return False


def _require_free_port(bind_target: tuple[int, str] | None) -> None:
    if bind_target is None:
        return
    port, host = bind_target
    if _port_in_use(host, port):
        both_print(f"ERROR: port {port} is in use.")
        both_print(
            f"       Stop the process using port {port}, or pass"
            " `-- --port <other>` to run on a different port."
        )
        sys.exit(1)



def run_server(args, results):
    """Build the dev server and return the argv to exec once main() finishes.

    No exec happens here: main() must first run its epilogue (summary, banner,
    log close, history line). The port is checked before the build so a busy
    port fails fast without invoking cargo.
    """
    server_flags = _strip_passthrough(getattr(args, "server_flags", []))
    ctx = _prelude(args, results, nextest=False)
    _require_free_port(_parse_bind_target(server_flags))
    spec = REGISTRY["run"]
    _run_plan([Step(spec.label, spec.cmd)], ctx)
    _, profile_dir = _target_paths(args)
    return [str(profile_dir / "chronicler_engine"), *server_flags]


def _exec_pending(exit_code: int, argv: list[str] | None) -> None:
    """Replace this process with the server when a ``run`` step built it.

    Called only from the ``__main__`` block, after main() has written the
    epilogue and history line. Re-checks the port because the build took
    time; a port taken meanwhile fails here with the same message.
    """
    if exit_code != 0 or not argv:
        return
    _require_free_port(_parse_bind_target(argv[1:]))
    sys.stdout.flush()
    sys.stderr.flush()
    os.execv(argv[0], argv)


def run_cleanup(args):
    both_print("=== Cleanup Mode ===")

    lock_dir = Path(tempfile.gettempdir()) / "chronicler_test_ports"
    if lock_dir.exists():
        log_status(f"Cleaning stale port locks from {lock_dir}...")
        shutil.rmtree(lock_dir)

    cargo_target_dir, _ = _target_paths(args)
    if cargo_target_dir.exists():
        log_status(f"Removing build directory: {cargo_target_dir}")
        shutil.rmtree(cargo_target_dir)
    else:
        log_status(f"Build directory does not exist: {cargo_target_dir}")

    both_print("=== Cleanup Complete ===")


_LLM_BANNER = "\n".join(
    [
        "=" * 60,
        "NOTE: LLM tests contact the real OpenRouter API.",
        "      Each test takes 1-3 minutes. Total: ~3-9 minutes.",
        "      Do not interrupt. Set your tool timeout to >= 1200s.",
        "=" * 60,
    ]
)


def run_llm_only(args, results):
    ctx = _prelude(args, results)
    build_cmd = f"cargo build {'--release' if args.release else ''}".strip()
    # The `--test llm` filter selects the LLM binary inside the llm profile run.
    llm_cmd = _with_fail_flag(f"{_NEXTEST_RUN}{_LLM_ARGS}", args) + " --test llm"
    _run_plan(
        [
            Step("Building...", build_cmd),
            Step("Running LLM tests only...", llm_cmd, profile="llm", before=_LLM_BANNER),
            Step("Done"),
        ],
        ctx,
    )


_FAILURE_LIMIT = 20
_COMPILE_ERROR_LIMIT = 5


def _tier_sizes(results) -> dict[str, int]:
    """The test-set size of each tier the run covered whole."""
    return {r.tier: r.counts.size for r in results if r.tier and r.counts}


def _exit_code(results) -> int:
    """A fatal failure exits with its own code; a recorded test failure exits 1."""
    fatal = next((r for r in results if r.fatal), None)
    if fatal is not None:
        return fatal.rc
    return 1 if any(r.failed for r in results) else 0


def _print_step_summary(results, log_path):
    if not results:
        return
    both_print("")
    both_print("--- Step Timing Summary ---")
    for r in results:
        status = "FAILED" if r.failed else "WARN" if r.rc != 0 else "OK"
        note = "  (report-only)" if r.report_only else ""
        both_print(f"  {r.elapsed:>6.2f}s  [{status}]  {r.label}{note}")
    both_print(f"  {'':>6}   Total: {sum(r.elapsed for r in results):.2f}s")
    failed = [r.label for r in results if r.failed]
    if failed:
        both_print(f"\n  Failed steps: {', '.join(failed)}")
    both_print(f"\n  Full log: {log_path}")
    both_print("---")


def _print_failures(results):
    """Name the failing tests and the causes of the failed non-nextest steps.

    A report-only step that warned is not a failure. The first ``_FAILURE_LIMIT`` tests print;
    the rest are counted.
    """
    failed = [r for r in results if r.failed]
    tests = [(r.label, name, message) for r in failed for name, message in r.failures]
    steps = [(r.label, r.tail) for r in failed if r.tail]
    if not tests and not steps:
        return
    both_print("")
    both_print("--- Failures ---")
    last_label = None
    for label, name, message in tests[:_FAILURE_LIMIT]:
        if label != last_label:
            both_print(f"  {label}")
            last_label = label
        both_print(f"    FAIL {name}")
        if message:
            both_print(f"      {message}")
    if len(tests) > _FAILURE_LIMIT:
        both_print(f"  ... {len(tests) - _FAILURE_LIMIT} more, see the log")
    for label, tail in steps:
        both_print(f"  {label}")
        for line in tail:
            both_print(f"    {line}")
    both_print("---")


def _epilogue_baselines(sizes: dict[str, int]) -> dict | None:
    """Load the per-tier baselines for the epilogue; None hides the count change."""
    if not sizes:
        return None
    try:
        return build_history.last_tier_sizes(build_history.HISTORY_FILE)
    except Exception:
        return None  # Best-effort: the plain nextest line still prints.


def _print_epilogue(results, log_path, browser_skipped: bool) -> None:
    """The step table, then the verdict lines an agent tailing stdout reads before the banner."""
    _print_step_summary(results, log_path)
    sizes = _tier_sizes(results)
    baselines = _epilogue_baselines(sizes)
    for r in results:
        if r.counts is None:
            continue
        line = f"{r.counts.line}  ({r.label})"
        if baselines is not None and r.tier in sizes:
            line += f"  {build_history.size_suffix(sizes[r.tier], baselines.get(r.tier))}"
        both_print(line)
    for r in results:
        for status, name in r.leaks:
            both_print(f"{status.lower()} test: {name}  ({r.label})")
    _print_failures(results)
    if browser_skipped:
        both_print('skipped: browser tier (run "python build.py browser")')
    errors = list(dict.fromkeys(e for r in results for e in r.compile_errors))
    if errors:
        both_print(f"compile errors (first {min(len(errors), _COMPILE_ERROR_LIMIT)} of {len(errors)}):")
        for line in errors[:_COMPILE_ERROR_LIMIT]:
            both_print(f"  {line}")


def main(argv=None) -> tuple[int, list[str] | None]:
    """Run the build. Returns the exit code and the argv of the server to exec, if any."""
    args = parse_args(argv)

    # Journal inputs, captured before any chdir so the record reflects
    # exactly what the caller invoked.
    run_start = time.time()
    history_args = " ".join(sys.argv[1:] if argv is None else argv).strip() or "(full gate)"

    os.chdir(os.path.dirname(os.path.abspath(__file__)) or os.getcwd())
    # Taken at the start: an edit made during the run is not covered.
    tree = build_history.tree_identity(Path.cwd())

    log_dir = Path("logs")
    log_dir.mkdir(exist_ok=True)
    clean_old_logs(log_dir, max_age_days=3)
    log_path = log_dir / f"build_{time.strftime('%Y%m%d_%H%M%S')}.log"
    log_fh = open(log_path, "w", encoding="utf-8")
    _set_log_fh(log_fh)
    _stamp_session_id()

    results: list[StepResult] = []
    exit_code = 0
    exec_argv = None
    browser_skipped = False
    try:
        both_print("=" * 60)
        both_print("=== Chronicler Engine Build ===")
        both_print(f"Full build log: {log_path}")
        both_print("=" * 60)

        if args.command == "run":
            exec_argv = run_server(args, results)
        elif args.command:
            run_step(args, results)
        elif args.cleanup:
            run_cleanup(args)
        elif args.llm_only:
            run_llm_only(args, results)
        else:
            browser_skipped = bool(args.no_browser)
            run_gate(args, results)
    except SystemExit as e:
        # A failed precondition (toolchain, busy port). Capture the code so
        # the finally can print the summary, then propagate.
        exit_code = int(e.code) if e.code is not None else 1
        raise
    finally:
        # Print the epilogue BEFORE closing the log so the summary + banner are
        # written into the log file, not only to stdout.
        _print_epilogue(results, log_path, browser_skipped)

        both_print("=" * 60)
        both_print("=== Build Complete ===")
        both_print(f"Full build log: {log_path}")
        both_print("=" * 60)

        try:
            log_fh.flush()
            log_fh.close()
        except Exception:
            pass
        _set_log_fh(None)

        # The finally runs on every exit path, so the journal can never disagree with
        # the process exit code.
        exit_code = exit_code or _exit_code(results)

        try:
            build_history.append(
                build_history.HISTORY_FILE,
                history_args,
                time.time() - run_start,
                exit_code,
                tree,
                build_history.tests_column(_tier_sizes(results)),
            )
        except Exception:
            pass  # Best-effort bookkeeping; never fail a run over it.

    return exit_code, (exec_argv if exit_code == 0 else None)


if __name__ == "__main__":
    _code, _argv = main()
    _exec_pending(_code, _argv)
    sys.exit(_code)
