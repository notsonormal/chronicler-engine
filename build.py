"""Full build, validate, and test for Chronicler Engine.

Uses cargo-nextest for parallel test execution.

Two invocation modes:

- Full gate (default): ``python build.py`` runs fmt, validation, clippy,
  guardrails, the full test suite, and packaging.
- Gate without the browser tier: ``python build.py --no-browser`` runs the same
  gate minus the browser (Playwright) tier — the iteration default. The last
  run before reporting a task done is the bare full gate.
- Step mode: ``python build.py <step>`` runs one registry step (e.g.
  ``clippy``, ``fmt``, ``test-pattern <pattern>``) with a minimal prelude — no
  asset copy, no SQLite cleanup. ``--target-dir`` is
  accepted on either side of the step; all other top-level flags are gate-only
  and rejected next to a step.

``python build.py run [-- <server flags>]`` builds the binary and then replaces
this process with it, so the dev server links in the ``build.py`` environment
(lld linker, chosen target dir) and switching between a build step and the
server costs no recompile. The build runs under the machine-wide slot; the
server itself runs without any lock. The port is checked before the build and
again just before the exec. ``run`` is dev-profile only.

Stdout carries the agent-facing decision signal + tailable progress (banner,
step labels, ``$ cmd`` echoes, failure signals, Step Timing Summary, closing
banner with log path). Full output is written to ``logs/build_*.log``; each
log's first line is a session stamp (see ``_stamp_session_id``). Every
completed run also appends one pipe-delimited summary line (timestamp,
duration, exit code, args, tree, per-tier test counts) to the append-only
journal ``logs/build_history.txt`` — see ``_append_history``. Each test tier's
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
live process inside (``scripts/remove_worktrees.py``). It warns and continues if that fails.

``--cleanup`` deletes this checkout's whole cargo target dir (the next build is cold) and the
machine-wide port-lock dir ``<tmp>/chronicler_test_ports`` that every checkout's tests share.
Do not run it while another checkout is testing.

A full gate takes about 2 minutes on a warm target dir, about 15 on a cold unseeded one, plus any
wait for the build lock. Use a tool timeout of at least 1200 seconds. ``--coverage`` takes longer.
"""

import argparse
import contextlib
import errno
import hashlib
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
from pathlib import Path
from typing import NamedTuple

sys.path.insert(0, str(Path(__file__).resolve().parent / "scripts"))
import build_slot  # noqa: E402
import target_seed  # noqa: E402

try:
    import fcntl
except ImportError:  # Windows has no fcntl; journal locking is skipped there.
    fcntl = None

# Force UTF-8 for stdout/stderr on Windows to handle cargo's Unicode output
if sys.platform == "win32":
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding="utf-8", errors="replace")


class _LogState:
    """Module-level log handle for the current build run."""

    fh = None


class _TierSize(NamedTuple):
    """One step's test-set size, for a run whose command is a known tier."""

    tier: str
    size: int


class _TierRecord(NamedTuple):
    """A tier's newest test-set size in the build-history journal."""

    timestamp: str
    tree: str
    size: int


class _NextestSummary:
    """nextest summary lines captured this run, re-printed at the epilogue.

    One entry per test tier: the gate runs the integration and browser tiers as
    separate nextest invocations. Module state (like _LogState) because run()
    sys.exits on a checked failure before its caller could inspect the captured
    output; main()'s finally must still print the counts at the tail.

    ``label`` names the step the next captured summary belongs to; ``_timed_run``
    sets it before dispatch. ``sizes`` maps a step label to its ``_TierSize``
    for the runs whose command is a known tier (see ``_tier_for_cmd``).
    """

    lines: list[tuple[str, str]] = []
    label = ""
    sizes: dict[str, _TierSize] = {}

    @classmethod
    def line_text(cls, baselines: dict | None) -> str:
        """Render every captured summary as the epilogue's one-liner block.

        With ``baselines`` (tier -> journal record, see ``_last_tier_sizes``),
        a line whose step has a recorded size also gets the count change.
        """
        rendered = []
        for label, text in cls.lines:
            line = f"{text}  ({label})" if label else text
            entry = cls.sizes.get(label)
            if entry is not None and baselines is not None:
                line += f"  {_size_suffix(entry.size, baselines.get(entry.tier))}"
            rendered.append(line)
        return "\n".join(rendered)


class _NextestLeaks:
    """nextest LEAK/FLAKY lines, captured so the epilogue can name the test.

    The Summary counts a leak without naming it, and the default status level
    hides the per-test line. ``lines`` holds
    ``(step label, status, test name)``.
    """

    lines: list[tuple[str, str, str]] = []


class _CompileErrors:
    """rustc/clippy errors from failed commands, re-printed at the epilogue.

    Cargo output goes only to the log, so without this a failed compile shows
    an agent the step name but not the cause. ``lines`` holds rendered errors,
    deduplicated (``--all-targets`` reports one error once per target), in
    first-seen order. The epilogue prints the first ``LIMIT``.
    """

    LIMIT = 5
    lines: list[str] = []


def _set_log_fh(fh):
    """Register the log file handle for the current build run."""
    _LogState.fh = fh


def _log_write(text):
    """Write text to the log file. No newline is added."""
    fh = _LogState.fh
    if fh is None:
        return
    fh.write(text)
    fh.flush()


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
    if _LogState.fh is not None:
        _LogState.fh.write(msg + "\n")
        _LogState.fh.flush()


# Cargo progress lines are pure noise for an agent caller — every incremental
# build emits hundreds of "   Compiling foo v1.0.X" / "    Checking foo" lines.
# Strip them before writing to the log so the log carries only actionable content
# (warnings, errors, test results, build summary). The line counter still advances
# so existing log-range pointers stay accurate.
_CARGO_PROGRESS_RE = re.compile(
    r"^\s*(?:Compiling|Checking|Downloading|Updating|Adding|Building|Running `rustc`)\s+\S+"
)


class StepCounter:
    """Simple step counter for build progress output."""

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
    """Ensure cargo-nextest is installed; exit with error if not."""
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
# The gate runs architecture and guardrails as their own steps, so this filter
# drops them. Coverage uses _NON_BROWSER_FILTER to still instrument them.
_INTEGRATION_FILTER = (
    "not binary(browser) and not binary(architecture) and not binary(guardrails)"
)


def get_test_cmd(include_llm=False):
    """Return the test command using nextest."""
    cmd = _NEXTEST_RUN
    if include_llm:
        cmd += " --profile llm --run-ignored all"
    return cmd


def get_browser_test_cmd():
    """Return the command running only the browser (Playwright) binary."""
    return f"{_NEXTEST_RUN} -E '{_BROWSER_FILTER}'"


def get_integration_test_cmd(include_llm=False):
    """Return the command running every test binary except browser, architecture
    and guardrails.

    The engine, http, storage, bootstrap and llm tiers are CPU-light; excluding
    the browser binary keeps this step to ~20s, so it works as a standalone
    check.
    """
    cmd = f"{_NEXTEST_RUN} -E '{_INTEGRATION_FILTER}'"
    if include_llm:
        cmd += " --profile llm --run-ignored all"
    return cmd


# nextest per-test result line, e.g.:
#   "        PASS [  18.645s] ( 4/26) chronicler_engine::browser behaviour::test_x"
#   "        LEAK [   0.229s] ( 2/2) leakprobe::leak leaks_a_child"
#   "       FLAKY [   0.229s] ( 2/2) leakprobe::leak leaks_a_child"
# The progress counter is optional: nextest only prints it in some
# --show-progress modes, and the timing report must survive config drift.
# LEAK and FLAKY are passing tests, so they must not read as failures.
_NEXTEST_RESULT_RE = re.compile(
    r"^\s*(?:PASS|FAIL|SKIP|LEAK|FLAKY)\s*\[([^\]]*)\]\s*(?:\([^)]*\))?\s*(\S.*)$"
)

# nextest final summary line, e.g.:
#   "     Summary [  21.153s] 1 test run: 1 passed, 0 skipped"
#   "     Summary [   1.497s] 119 tests run: 118 passed, 1 failed, 0 skipped"
# Counts are matched independently of segment order so a nextest format
# change cannot silently break the epilogue one-liner.
_NEXTEST_SUMMARY_RE = re.compile(r"^\s*Summary \[[^\]]*\]\s+\d+ tests? run:")


def _summary_count(line: str, label: str) -> str | None:
    """Return the count preceding ``label`` in a Summary line, or None."""
    m = re.search(rf"(\d+) {label}", line)
    return m.group(1) if m else None


def _nextest_summary_line(output: str) -> str | None:
    """Render nextest's final Summary line as a compact epilogue one-liner.

    Returns e.g. "nextest: 118 passed, 1 failed", or None when the output
    carries no nextest summary (compile error, non-nextest command).
    nextest omits the "failed" segment when nothing failed, but the epilogue
    line always shows it (0 when absent) so a clean run still reads as clean.
    A zero "skipped" or "leaky" segment is omitted; a nonzero one is kept so a
    passing-but-leaking run cannot read as fully clean.
    """
    rendered = None
    for line in output.splitlines():
        if not _NEXTEST_SUMMARY_RE.match(line):
            continue
        passed = _summary_count(line, "passed") or "?"
        failed = _summary_count(line, "failed") or "0"
        skipped = _summary_count(line, "skipped")
        leaky = _summary_count(line, "leaky")
        flaky = _summary_count(line, "flaky")
        rendered = f"nextest: {passed} passed, {failed} failed"
        if skipped and skipped != "0":
            rendered += f", {skipped} skipped"
        if leaky and leaky != "0":
            rendered += f", {leaky} leaky"
        if flaky and flaky != "0":
            rendered += f", {flaky} flaky"
    return rendered


# Test tiers in journal column order. A tier is one fixed nextest command, so
# a step run and the gate's run of the same tier share one baseline.
_TIERS = ("architecture", "guardrails", "integration", "browser")


def _tier_for_cmd(cmd: str | None) -> str | None:
    """Return the tier a nextest command runs, or None for any other command.

    Coverage, --include-llm, test-pattern and LLM runs select a different test
    set, so they map to None and record no size.
    """
    commands = {
        REGISTRY["architecture"].cmd: "architecture",
        REGISTRY["guardrails"].cmd: "guardrails",
        get_integration_test_cmd(): "integration",
        get_browser_test_cmd(): "browser",
    }
    return commands.get(cmd)


def _summary_size(output: str) -> int | None:
    """Return the test-set size (tests run + skipped) from nextest's Summary."""
    size = None
    for line in output.splitlines():
        if not _NEXTEST_SUMMARY_RE.match(line):
            continue
        run_count = _summary_count(line, "tests? run")
        skipped = _summary_count(line, "skipped") or "0"
        if run_count is not None:
            size = int(run_count) + int(skipped)
    return size


def _stash_nextest_summary(output: str, cmd: str | None = None) -> None:
    """Capture nextest's summary line, the tier's test-set size, and any
    leaky/flaky test names."""
    line = _nextest_summary_line(output)
    if line is not None:
        _NextestSummary.lines.append((_NextestSummary.label, line))
        tier = _tier_for_cmd(cmd)
        size = _summary_size(output)
        if tier is not None and size is not None:
            _NextestSummary.sizes[_NextestSummary.label] = _TierSize(tier, size)
    for raw in output.splitlines():
        m = _NEXTEST_RESULT_RE.match(raw)
        if m is None:
            continue
        status = raw.strip().split(maxsplit=1)[0]
        entry = (_NextestSummary.label, status, m.group(2).strip())
        # nextest prints a LEAK or FLAKY line live and again in the final list.
        if status in ("LEAK", "FLAKY") and entry not in _NextestLeaks.lines:
            _NextestLeaks.lines.append(entry)


# A rustc or clippy diagnostic header: "error[E0599]: ..." or "error: ...".
_COMPILE_ERROR_RE = re.compile(r"^error(?:\[E\d{4}\])?: ")
# Cargo's closing lines restate the failure and carry no cause. nextest's
# "test run failed", "no tests to run" and its "command `cargo test --no-run`
# exited" wrapper have the same header shape, but they are not compiler errors.
_COMPILE_ERROR_NOISE = (
    "error: could not compile",
    "error: aborting due to",
    "error: test run failed",
    "error: no tests to run",
    "error: command `",
)


def _stash_compile_errors(output: str) -> None:
    """Capture each compiler error header with its ``--> file:line:col``."""
    lines = output.splitlines()
    for i, raw in enumerate(lines):
        if not _COMPILE_ERROR_RE.match(raw) or raw.startswith(_COMPILE_ERROR_NOISE):
            continue
        rendered = raw.strip()
        for follow in lines[i + 1 : i + 4]:
            stripped = follow.strip()
            if stripped.startswith("--> "):
                rendered += f"  ({stripped[4:]})"
                break
            if not stripped:
                break
        if rendered not in _CompileErrors.lines:
            _CompileErrors.lines.append(rendered)


def _nextest_duration_to_secs(token: str) -> float:
    """Convert a nextest duration token (e.g. "18.645s", "1m23s") to seconds."""
    token = token.strip()
    total = 0.0
    for value, unit in re.findall(r"([0-9]+(?:\.[0-9]+)?)([hms])", token):
        n = float(value)
        if unit == "h":
            total += n * 3600.0
        elif unit == "m":
            total += n * 60.0
        else:
            total += n
    return total


# One line of a stub runner's per-check summary (`StubRunner::finish` in
# tests/browser/stub/support.rs), e.g. "  0.902s  [OK]  check_x".
_STUB_CHECK_RE = re.compile(r"^\s*([0-9]+(?:\.[0-9]+)?)s\s+\[(OK|FAIL)\]\s+(check_\w+)\s*$")


def _parse_stub_check_timings(lines):
    """Return (secs, runner test, check name, status) for every stub check.

    A runner test holds many checks, so its own nextest duration hides which
    check is slow. Each check line follows its runner's nextest result line
    in the success output, so the latest result line names the runner.
    """
    checks = []
    runner = None
    for line in lines:
        result = _NEXTEST_RESULT_RE.match(line)
        if result:
            runner = result.group(2).strip()
            continue
        check = _STUB_CHECK_RE.match(line)
        if check and runner:
            checks.append(
                (float(check.group(1)), runner, check.group(3), check.group(2))
            )
    return checks


def _timed_test_cmd(cmd):
    """Return `cmd` with the flags the timing report needs.

    The browser tier also shows passing tests' output, which carries each
    stub runner's per-check summary. Other commands keep it hidden: the
    integration tier's passing output would grow the log about 14 times.
    """
    timed = f"{cmd} --final-status-level pass"
    if _tier_for_cmd(cmd) == "browser":
        timed += " --success-output final"
    return timed


def _combined_output(result) -> str:
    """Return a captured run's stdout and stderr as one text to scan."""
    return (result.stdout or "") + "\n" + (result.stderr or "")


def run_with_test_timings(cmd, env=None, check=True):
    """Run nextest and print a per-test timing report: slowest tests (top 30)
    and per-binary totals, plus the slowest stub checks inside runner tests.
    Returns the exit code; exits on failure when check=True.
    """
    both_print(f"$ {cmd}  (with --test-timings)")
    timed_cmd = _timed_test_cmd(cmd)
    merged_env = os.environ.copy()
    if env:
        merged_env.update(env)
    result = subprocess.run(
        timed_cmd,
        shell=True,
        cwd=os.getcwd(),
        env=merged_env,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    # nextest writes its per-test report on stderr; log both streams and scan both.
    for stream in (result.stdout, result.stderr):
        if stream:
            for line in stream.splitlines():
                _log_write(line + "\n")

    output = _combined_output(result)
    all_lines = output.splitlines()
    # Keyed by test name: at the `leak` status level every non-PASS result line
    # (FAIL, LEAK) is printed live and again in the final status list.
    durations = {}
    for line in all_lines:
        m = _NEXTEST_RESULT_RE.match(line)
        if not m:
            continue
        secs = _nextest_duration_to_secs(m.group(1))
        name = m.group(2).strip()
        # SKIP lines carry no duration measurement.
        if secs == 0.0 and "SKIP" in line:
            continue
        durations[name] = secs
    timings = [(secs, name) for name, secs in durations.items()]
    _stash_nextest_summary(output, cmd)

    both_print("")
    both_print("--- Test Timing Report ---")
    if not timings:
        both_print("  No per-test durations parsed from nextest output.")
    else:
        # nextest test path is "<binary_id> <test_path>"; group by binary_id.
        per_binary = defaultdict(float)
        for secs, name in timings:
            binary = name.split(" ", 1)[0] if " " in name else name
            per_binary[binary] += secs

        sorted_timings = sorted(timings, key=lambda t: t[0], reverse=True)
        both_print(f"  Slowest tests (of {len(timings)} measured, top 30):")
        for secs, name in sorted_timings[:30]:
            both_print(f"    {secs:>8.3f}s  {name}")
        both_print("")
        both_print("  Per-binary totals:")
        for binary, total in sorted(per_binary.items(), key=lambda kv: kv[1], reverse=True):
            both_print(f"    {total:>8.2f}s  {binary}")
        both_print(
            f"    {'':>8}   Sum of measured: {sum(t[0] for t in timings):.2f}s"
        )
    stub_checks = _parse_stub_check_timings(all_lines)
    if stub_checks:
        both_print("")
        both_print(f"  Slowest stub checks (of {len(stub_checks)}, top 30):")
        for secs, runner, name, status in sorted(stub_checks, reverse=True)[:30]:
            runner_fn = runner.rsplit("::", 1)[-1]
            both_print(f"    {secs:>8.3f}s  [{status}]  {name}  ({runner_fn})")
    both_print("---")

    if result.returncode != 0:
        _stash_compile_errors(output)
        both_print(f"FAILED with code {result.returncode}")
        if check:
            sys.exit(result.returncode)
    return result.returncode


def get_coverage_cmd(browser_only=False):
    """Return the coverage test command using nextest.

    ``browser_only`` selects which half the gate's split runs: False yields
    the non-browser tier (everything except the browser binary), True the
    browser binary alone. The coverage step runs twice, once per half, so both
    profiling runs share the single merged report that ``--no-report`` defers.
    """
    cmd = "cargo llvm-cov nextest --no-report --no-fail-fast"
    if browser_only:
        cmd += f" -E '{_BROWSER_FILTER}'"
    else:
        cmd += f" -E '{_NON_BROWSER_FILTER}'"
    return cmd


class StepSpec(NamedTuple):
    """One runnable build step, shared by the full gate and step subcommands.

    The registry is the single source of truth for step commands: the full
    gate and the step subcommands consume the same spec, so the two modes
    cannot drift into different command strings.
    """

    name: str
    label: str
    cmd: str
    needs_nextest: bool = False
    pattern_arg: bool = False
    help: str = ""


_NEXTEST_RUN = "cargo nextest run --no-fail-fast"

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
            "Removing worktrees that are safe to remove...",
            "python scripts/remove_worktrees.py --apply",
            help=(
                "Remove linked worktrees that are clean, pushed, and have no live process"
                " inside; every other tree is refused with a reason."
            ),
        ),
        StepSpec(
            "fmt",
            "Formatting...",
            "cargo fmt",
            help="Format sources in place (same as the full gate's fmt step).",
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
            help="Lint Rust sources; warnings are errors.",
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
            "architecture",
            "Running architecture tests...",
            f"{_NEXTEST_RUN} --test architecture",
            needs_nextest=True,
            help="Run the architecture integration tests.",
        ),
        StepSpec(
            "guardrails",
            "Running guardrail tests...",
            f"{_NEXTEST_RUN} --test guardrails",
            needs_nextest=True,
            help="Run the guardrail integration tests.",
        ),
        StepSpec(
            "unit",
            "Running unit tests...",
            "cargo test --lib",
            help="Run the Rust unit tests (lib target only).",
        ),
        StepSpec(
            "integration",
            "Running integration tests...",
            get_integration_test_cmd(),
            needs_nextest=True,
            help=(
                "Run every test binary except browser, architecture and guardrails,"
                " the lib unit tests included (~20s warm)."
            ),
        ),
        StepSpec(
            "browser",
            "Running browser tests...",
            get_browser_test_cmd(),
            needs_nextest=True,
            help="Run only the browser (Playwright) test binary (~1.5 min warm).",
        ),
        StepSpec(
            "test-pattern",
            "Running test-pattern step...",
            _NEXTEST_RUN,
            needs_nextest=True,
            pattern_arg=True,
            help="Run the tests whose name matches a pattern, across all test binaries.",
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

# Full-gate step order. "HOOKS" expands to the git-hook install step,
# "WORKTREES" to the best-effort worktree removal, "COPY"
# to the deployment-asset copy step, "TESTS" to the composite test step
# (coverage/timings variants), and "REPORT" to the coverage report (or the
# skip note when coverage is off).
#
# The order is load-bearing: the architecture/guardrail binaries run before
# the ~2-minute full suite and fail the build immediately (check=True) —
# otherwise a guardrail failure only surfaces after the full suite has run.
GATE_ORDER = [
    "HOOKS",
    "WORKTREES",
    "fmt",
    "validate-data",
    "clippy",
    "test-structure",
    "spec-coverage",
    "docstrings",
    "py-tests",
    "http-routes-check",
    "guardrails-doc-check",
    "validate-docs",
    "DUPLICATES",
    "COPY",
    "architecture",
    "guardrails",
    "TESTS",
    "REPORT",
]

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
            "Run the full gate without the browser tier (the iteration default;"
            " the final run is bare `python build.py`)"
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

    sub = parser.add_subparsers(dest="command", metavar="<step>")
    for spec in REGISTRY.values():
        step_parser = sub.add_parser(spec.name, help=spec.help)
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
            # Forward the duplicate scope flags through to healthcheck.py.
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
    return args


def _reject_gate_flags_with_step(parser, args):
    """Fail at parse time when gate-only flags are mixed with a step subcommand."""
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
    a passing-but-leaking test is named instead of only counted. The
    ``--test-timings`` report gets its PASS lines from ``--final-status-level
    pass`` (see ``_timed_test_cmd``); a live ``pass`` level would print each one
    twice.
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
    spec: StepSpec, pattern: str | None = None, extra: list[str] | None = None
) -> str:
    """Assemble the shell command for a step spec (quotes the nextest pattern)."""
    if spec.pattern_arg:
        cmd = f"{spec.cmd} {shlex.quote(pattern)}"
    else:
        cmd = spec.cmd
    if extra:
        cmd = " ".join([cmd, *(shlex.quote(arg) for arg in extra)])
    return cmd


def clean_sqlite_dbs(data_dir: Path):
    """Remove any SQLite database files from the data directory."""
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
    """Remove log files older than max_age_days from the log directory."""
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
    """Remove files older than max_age_days from the given tmp directories (recursing
    into subdirectories). Subdirectories themselves are left in place."""
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


def run(cmd, cwd=None, check=True, show_output=True, env=None):
    """Run a command, writing its output to the log file only (not stdout).

    Returns the exit code. When ``check`` is True and the command fails, calls
    ``sys.exit`` with the return code. Status messages (the ``$ {cmd}`` echo and
    any ``FAILED`` notice) go to both stdout and the log.
    """
    both_print(f"$ {cmd}")
    merged_env = os.environ.copy()
    if env:
        merged_env.update(env)

    if show_output:
        # Use communicate() rather than manual line iteration: the line-by-line
        # loop loses the final lines of stdout when the child process closes its
        # pipe before the kernel has flushed its output buffer. communicate()
        # waits for EOF and returns the full output deterministically.
        process = subprocess.Popen(
            cmd,
            shell=True,
            cwd=cwd or os.getcwd(),
            env=merged_env,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        out, _ = process.communicate()
        if out:
            for line in out.splitlines(keepends=True):
                # Filter out noisy cargo-llvm-cov info messages from the log too.
                if line.strip().startswith("info: cargo-llvm-cov"):
                    continue
                # Skip cargo progress spam; the log retains everything else.
                if _CARGO_PROGRESS_RE.match(line):
                    continue
                _log_write(line)
        # Stash before the check: a checked failure sys.exits below, and the
        # epilogue must still be able to print the pass/fail counts.
        _stash_nextest_summary(out or "", cmd)
        if process.returncode != 0:
            _stash_compile_errors(out or "")
        if check and process.returncode != 0:
            both_print(f"FAILED with code {process.returncode}")
            sys.exit(process.returncode)
        return process.returncode
    else:
        result = subprocess.run(
            cmd,
            shell=True,
            cwd=cwd or os.getcwd(),
            env=merged_env,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        if result.stdout:
            for line in result.stdout.splitlines(keepends=True):
                if _CARGO_PROGRESS_RE.match(line):
                    continue
                _log_write(line)
        if result.stderr:
            for line in result.stderr.splitlines(keepends=True):
                if _CARGO_PROGRESS_RE.match(line):
                    continue
                _log_write(line)
        output = _combined_output(result)
        _stash_nextest_summary(output, cmd)
        if result.returncode != 0:
            _stash_compile_errors(output)
        if check and result.returncode != 0:
            both_print(f"FAILED with code {result.returncode}")
            sys.exit(result.returncode)
        return result.returncode


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


def _print_step_summary(step_timings, step_failures, log_path):
    """Print the Step Timing Summary with per-step status and elapsed time."""
    if not step_timings:
        return
    both_print("")
    both_print("--- Step Timing Summary ---")
    total = sum(t["elapsed_sec"] for t in step_timings)
    for t in step_timings:
        status = "FAILED" if t["failed"] else "OK"
        both_print(f"  {t['elapsed_sec']:>6.2f}s  [{status}]  {t['step']}")
    both_print(f"  {'':>6}   Total: {total:.2f}s")
    if step_failures:
        both_print(f"\n  Failed steps: {', '.join(step_failures)}")
    both_print(f"\n  Full log: {log_path}")
    both_print("---")




class StepRecord:
    """Mutable record of executed steps, shared between handlers and the epilogue."""

    def __init__(self):
        self.timings = []
        self.failures = []


def _record_step(record, label, start, *, failed):
    """Append one step outcome to the epilogue record."""
    record.timings.append(
        {
            "step": label,
            "elapsed_sec": round(time.time() - start, 2),
            "failed": failed,
        }
    )
    if failed:
        record.failures.append(label)


_target_args = None  # the running mode's args, so heavy steps know which target dir to seed
_seeded = False


def _set_target_args(args):
    global _target_args
    _target_args = args


def _seed_cold_target():
    """Seed a cold target dir once per run, inside the slot so the source is idle."""
    global _seeded
    if _target_args is None or _seeded:
        return
    _seeded = True
    cargo_target_dir, profile_dir = _target_paths(_target_args)
    try:
        target_seed.seed_if_cold(Path.cwd(), cargo_target_dir.resolve(), profile_dir.name, both_print)
    except Exception as err:  # seeding is an optimisation and must never fail the build
        both_print(f"Target seeding skipped: {err}")


def _stamp_target():
    if _target_args is None:
        return
    _, profile_dir = _target_paths(_target_args)
    try:
        target_seed.stamp(profile_dir.resolve(), target_seed.build_signature(Path.cwd()))
    except (OSError, subprocess.SubprocessError):
        pass


@contextlib.contextmanager
def _cargo_slot(cmd, label):
    heavy = build_slot.is_heavy(cmd)
    with build_slot.maybe_hold(cmd, label, both_print):
        if heavy:
            _seed_cold_target()
        yield
        if heavy:
            _stamp_target()


def _timed_run(counter, record, label, cmd, check=True, env=None, timings=False):
    """Run one step, print progress, and record its timing and outcome; the
    recorded time excludes any build-slot wait."""
    counter.next(label)
    _NextestSummary.label = label
    with _cargo_slot(cmd, label):
        start = time.time()
        try:
            if timings:
                rc = run_with_test_timings(cmd, env=env, check=check)
            else:
                rc = run(cmd, check=check, env=env)
        except SystemExit:
            _record_step(record, label, start, failed=True)
            raise
        else:
            _record_step(record, label, start, failed=rc != 0)


class GateStep(NamedTuple):
    """One entry of the expanded gate plan.

    Every plan entry advances the step counter, so the counter total is
    structurally len(plan). The "tests" handler prints the LLM-skip note
    itself (the note is not a counted step, matching the original behavior).
    """

    kind: str  # "cmd" | "copy" | "tests" | "coverage_report" | "note" | "hooks" | "worktrees"
    label: str
    cmd: str = ""
    check: bool = True
    timings: bool = False


def _plan_gate_steps(args) -> list[GateStep]:
    """Expand GATE_ORDER into the concrete gate step list.

    The step counter total is derived from this list (len(plan)), never
    hardcoded, so the count cannot drift from the steps actually run.
    """
    plan: list[GateStep] = []
    for name in GATE_ORDER:
        if name == "HOOKS":
            plan.append(GateStep("hooks", "Installing git hooks..."))
            continue
        if name == "WORKTREES":
            plan.append(GateStep("worktrees", REGISTRY["remove-worktrees"].label))
            continue
        if name == "COPY":
            plan.append(GateStep("copy", "Copying data and assets for deployment..."))
            continue
        if name == "TESTS":
            # The gate runs the test suite as two nextest invocations — the
            # integration tier, then the browser tier — so the browser binary
            # gets its own epilogue verdict and its own fail-fast exit.
            if args.coverage:
                int_cmd = get_coverage_cmd(browser_only=False)
                browser_cmd = get_coverage_cmd(browser_only=True)
                suffix = " with coverage"
            else:
                int_cmd = get_integration_test_cmd(include_llm=args.include_llm)
                browser_cmd = get_browser_test_cmd()
                suffix = ""
            tail = " and timings..." if args.test_timings else "..."
            plan.append(
                GateStep(
                    "tests",
                    f"Running integration tests{suffix}{tail}",
                    int_cmd,
                    check=False,
                    timings=args.test_timings,
                )
            )
            if not args.no_browser:
                plan.append(
                    GateStep(
                        "tests",
                        f"Running browser tests{suffix}{tail}",
                        browser_cmd,
                        check=False,
                        timings=args.test_timings,
                    )
                )
            continue
        if name == "REPORT":
            if args.coverage:
                plan.append(GateStep("coverage_report", "Generating coverage report..."))
            else:
                plan.append(
                    GateStep("note", "Skipping coverage report (use --coverage to enable)")
                )
            continue
        if name == "DUPLICATES":
            plan.append(GateStep("duplicates", REGISTRY["duplicates"].label))
            continue
        spec = REGISTRY[name]
        if name == "fmt" and args.no_fmt:
            continue
        plan.append(GateStep("cmd", spec.label, spec.cmd))
    return plan


def _execute_gate_plan(plan, args, cargo_env, record):
    """Run the expanded gate plan. The counter total is len(plan).

    Both test tiers are kind ``tests``; the LLM-skip NOTE is keyed to the
    first tests step only, so the browser tier does not repeat it.
    """
    counter = StepCounter(len(plan))
    tests_note_printed = False
    for step in plan:
        if step.kind == "cmd":
            _timed_run(
                counter,
                record,
                step.label,
                step.cmd,
                check=step.check,
                env=cargo_env,
                timings=step.timings,
            )
        elif step.kind == "copy":
            counter.next(step.label)
            _copy_deployment_assets(args)
        elif step.kind == "tests":
            _timed_run(
                counter,
                record,
                step.label,
                step.cmd,
                check=step.check,
                env=cargo_env,
                timings=step.timings,
            )
            if not args.coverage and not args.include_llm and not tests_note_printed:
                tests_note_printed = True
                both_print(
                    "    NOTE: 2 LLM tests were skipped. "
                    "Run 'python build.py --llm-only' to execute them."
                )
        elif step.kind == "coverage_report":
            counter.next(step.label)
            _generate_coverage_report(args, cargo_env)
        elif step.kind == "hooks":
            # Best-effort environment setup: a hook problem must not block a
            # build, so a failure warns without recording a step failure.
            counter.next(step.label)
            rc = run(REGISTRY["install-hooks"].cmd, check=False, env=cargo_env)
            if rc != 0:
                both_print(
                    "    Warning: git hook installation failed; "
                    "the build continues without it."
                )
        elif step.kind == "worktrees":
            counter.next(step.label)
            _run_remove_worktrees(cargo_env)
        elif step.kind == "note":
            counter.next(step.label)
        elif step.kind == "duplicates":
            counter.next(step.label)
            _run_duplicates_report(cargo_env)
        else:  # pragma: no cover - guarded by construction
            raise ValueError(f"Unknown gate step kind: {step.kind}")


def _run_duplicates_report(cargo_env):
    """Report duplicate-code clones without ever failing the gate.

    The summary is written to the build log by ``run``'s captured child output.
    A missing jscpd or an unparsable report warns and continues, so this step is
    report-only per decision D3.
    """
    rc = run(REGISTRY["duplicates"].cmd, check=False, env=cargo_env)
    if rc != 0:
        both_print("    Warning: duplicate check did not complete (report-only step).")


def _run_remove_worktrees(cargo_env):
    """Remove safe-to-lose worktrees without ever failing the gate.

    Housekeeping like hook install: a failed removal warns and the build continues.
    """
    rc = run(REGISTRY["remove-worktrees"].cmd, check=False, env=cargo_env)
    if rc != 0:
        both_print("    Warning: worktree removal did not complete; the build continues.")


def _copy_deployment_assets(args):
    """Copy data/ and assets/ into the target dir and prepare the package layout."""
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


def _generate_coverage_report(args, cargo_env):
    """Generate the llvm-cov JSON report and print the parsed summary."""
    cargo_target_dir, _ = _target_paths(args)
    json_path = cargo_target_dir / "llvm-cov" / "coverage.json"
    json_path.parent.mkdir(parents=True, exist_ok=True)
    # Exclude: server infra (integration tests), test_support, bootstrap CLI, LLM backends (mock servers)
    ignore_regex = r"server[\\/](router|server_impl|handlers)\.rs|test_support[\\/].*\.rs|bootstrap[\\/]init_game\.rs|narrative[\\/]llm[\\/](openrouter|ollama|deepseek|backend)\.rs"
    run(
        f'cargo llvm-cov report --json --output-path "{json_path}" --ignore-filename-regex "{ignore_regex}"',
        check=False,
        env=cargo_env,
    )
    if json_path.exists():
        run(
            f'python scripts/parse_coverage.py --json "{json_path}"',
            check=False,
            env=cargo_env,
        )
    else:
        both_print("Warning: Could not generate coverage JSON.")


def _gate_tail(args):
    """Post-test housekeeping: tmp cleanup and SQLite dump. Gate mode only."""
    project_root_tmp = Path(__file__).resolve().parent.parent / "tmp"
    engine_tmp = Path("tmp")
    clean_tmp_dirs([project_root_tmp, engine_tmp], max_age_days=30)

    _, target_dir = _target_paths(args)
    db_path = target_dir / "data" / "chronicler.db"
    if db_path.exists():
        dump_dir = Path("tmp") / "db_dumps"
        dump_sqlite_to_jsonl(db_path, dump_dir)


def _gate_prelude(args) -> dict:
    """Gate-mode prelude: target-dir env + lock warning."""
    cargo_target_dir, _ = _target_paths(args)
    cargo_env = _cargo_env_for(args)
    custom = bool(getattr(args, "target_dir", None))
    if custom:
        log_status(f"Using custom target directory: {cargo_target_dir}")
    _warn_if_target_locked(cargo_target_dir, custom=custom)
    return cargo_env


def run_gate(args, record):
    """Full gate: fmt + validation + clippy + guardrails + tests + packaging."""
    check_rust_version()
    require_nextest()

    cargo_env = _gate_prelude(args)
    _set_target_args(args)
    if args.no_fmt:
        both_print("Skipping formatting (--no-fmt set).")
    if args.no_browser:
        both_print("Skipping the browser tier (--no-browser set).")
    plan = _plan_gate_steps(args)
    _execute_gate_plan(plan, args, cargo_env, record)
    _gate_tail(args)


def run_step(args, record):
    """Single-step mode: run one registry step with a minimal prelude.

    Skips the gate-only prelude (asset copy, SQLite cleanup)
    so quick iteration does not disturb a running dev server.
    """
    check_rust_version()
    spec = REGISTRY[args.command]
    if spec.needs_nextest:
        require_nextest()

    cargo_env = _cargo_env_for(args)
    _set_target_args(args)
    cargo_target_dir, _ = _target_paths(args)
    _warn_if_target_locked(
        cargo_target_dir,
        custom=bool(getattr(args, "target_dir", None)),
    )

    counter = StepCounter(1)
    _timed_run(
        counter,
        record,
        spec.label,
        _step_command(
            spec, getattr(args, "pattern", None), _step_extra_args(spec, args)
        ),
        check=True,
        env=cargo_env,
    )


def _step_extra_args(spec: StepSpec, args) -> list[str] | None:
    """Build the extra pass-through flags for a step from its parsed args."""
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
    """Stop with a clear message when the server's port is already bound."""
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


_pending_exec: list[str] | None = None


def run_server(args, record):
    """Build the dev server and return the argv to exec once main() finishes.

    No exec happens here: main() must first run its finally (summary, banner,
    log close, history line). The port is checked before the build so a busy
    port fails fast without invoking cargo.
    """
    check_rust_version()
    spec = REGISTRY["run"]
    server_flags = _strip_passthrough(getattr(args, "server_flags", []))
    _require_free_port(_parse_bind_target(server_flags))

    cargo_env = _cargo_env_for(args)
    _set_target_args(args)
    cargo_target_dir, _ = _target_paths(args)
    _warn_if_target_locked(
        cargo_target_dir, custom=bool(getattr(args, "target_dir", None))
    )

    counter = StepCounter(1)
    _timed_run(counter, record, spec.label, spec.cmd, check=True, env=cargo_env)

    _, profile_dir = _target_paths(args)
    return [str(profile_dir / "chronicler_engine"), *server_flags]


def _exec_pending(exit_code: int) -> None:
    """Replace this process with the server when a ``run`` step armed it.

    Called only from the ``__main__`` block, after main()'s finally has written
    the epilogue and history line. Re-checks the port because the build took
    time; a port taken meanwhile fails here with the same message.
    """
    global _pending_exec
    argv, _pending_exec = _pending_exec, None
    if exit_code != 0 or not argv:
        return
    _require_free_port(_parse_bind_target(argv[1:]))
    sys.stdout.flush()
    sys.stderr.flush()
    os.execv(argv[0], argv)


def run_cleanup(args, record):
    """Cleanup mode: remove stale port locks and build artifacts."""
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


def run_llm_only(args, record):
    """LLM-only mode: build, then run only the slow LLM tests."""
    check_rust_version()
    require_nextest()

    cargo_env = _gate_prelude(args)

    counter = StepCounter(3)
    counter.next("Building...")
    _set_target_args(args)
    build_cmd = f"cargo build {'--release' if args.release else ''}".strip()
    with _cargo_slot(build_cmd, "Building..."):
        run(build_cmd, env=cargo_env)

    counter.next("Running LLM tests only...")
    both_print("=" * 60)
    both_print("NOTE: LLM tests contact the real OpenRouter API.")
    both_print("      Each test takes 1-3 minutes. Total: ~3-9 minutes.")
    both_print("      Do not interrupt. Set your tool timeout to >= 1200s.")
    both_print("=" * 60)
    # get_test_cmd always returns nextest; the --test llm filter selects the
    # LLM binary inside the llm profile run.
    llm_cmd = get_test_cmd(include_llm=True) + " --test llm"
    with _cargo_slot(llm_cmd, "Running LLM tests only..."):
        run(llm_cmd, check=True, env=cargo_env)

    counter.next("Done")
    both_print("=== Build Complete ===")


_HISTORY_FILE = Path("logs/build_history.txt")
_HISTORY_HEADER = "timestamp | duration_s | exit_code | args | tree | tests"
_HISTORY_MAX_LINES = 1000


def _tree_identity(repo: Path) -> str:
    """Name the tree a run covers: the short HEAD, plus a content digest when dirty.

    The digest hashes ``git diff HEAD`` and every untracked, non-ignored path
    with its blob id, so two dirty trees share a name only when their content
    matches. Returns ``unknown`` when git fails (no git, not a repo, no HEAD).
    """

    def git(*argv: str, stdin: bytes | None = None) -> bytes:
        return subprocess.run(
            ["git", "-C", str(repo), *argv],
            input=stdin,
            capture_output=True,
            check=True,
        ).stdout

    try:
        head = git("rev-parse", "--short=10", "HEAD").decode().strip()
        diff = git("diff", "HEAD", "--binary", "--no-ext-diff", "--no-textconv")
        untracked = sorted(
            p
            for p in git("ls-files", "--others", "--exclude-standard", "-z").split(b"\0")
            if p
        )
        if not diff and not untracked:
            return head
        digest = hashlib.sha256(diff)
        for path in untracked:
            digest.update(b"\0" + path)
        hashable = [p for p in untracked if b"\n" not in p]
        if hashable:
            # --stdin-paths reads one path per line. A file removed by a
            # concurrent agent makes it fail; the path list still names the tree.
            try:
                blobs = git(
                    "hash-object", "--stdin-paths", stdin=b"\n".join(hashable) + b"\n"
                )
            except subprocess.CalledProcessError:
                blobs = b"unhashed"
            digest.update(b"\0" + blobs)
        return f"{head}+{digest.hexdigest()[:8]}"
    except (OSError, subprocess.SubprocessError, UnicodeDecodeError):
        return "unknown"


def _tests_column(sizes: dict[str, _TierSize]) -> str:
    """Render the journal ``tests`` column, e.g. ``architecture=1 integration=1565``.

    Tiers appear in ``_TIERS`` order; ``-`` when no tier recorded a size.
    """
    by_tier = {entry.tier: entry.size for entry in sizes.values()}
    rendered = " ".join(f"{t}={by_tier[t]}" for t in _TIERS if t in by_tier)
    return rendered or "-"


def _last_tier_sizes(path: Path) -> dict[str, _TierRecord]:
    """Return tier -> ``_TierRecord`` from each tier's newest journal record.

    Read under a shared lock. Records older than the ``tree`` and
    ``tests`` columns, and ``-`` values, name no tier and are skipped. A missing
    journal gives an empty result.
    """
    try:
        with open(path, encoding="utf-8") as fh:
            # Shared lock: _append_history truncates and rewrites under LOCK_EX.
            if fcntl is not None:
                fcntl.flock(fh, fcntl.LOCK_SH)
            text = fh.read()
    except FileNotFoundError:
        return {}
    newest: dict[str, _TierRecord] = {}
    for raw in text.splitlines()[1:]:
        parts = raw.split(" | ")
        if len(parts) < 6 or parts[5].strip() == "-":
            continue
        timestamp, tree = parts[0].strip(), parts[4].strip()
        for token in parts[5].split():
            tier, _, value = token.partition("=")
            if tier in _TIERS and value.isdigit():
                newest[tier] = _TierRecord(timestamp, tree, int(value))
    return newest


def _size_suffix(size: int, baseline: _TierRecord | None) -> str:
    """Render a tier's test count against its newest journal record."""
    if baseline is None:
        return f"tests: {size} (no earlier record)"
    where = f"{baseline.timestamp}, tree {baseline.tree}"
    if size == baseline.size:
        return f"tests: {size} (same as {where})"
    return f"tests: {size} ({size - baseline.size:+d} vs {where})"


def _epilogue_baselines() -> dict | None:
    """Load the per-tier baselines for the epilogue; None hides the count change."""
    if not _NextestSummary.sizes:
        return None
    try:
        return _last_tier_sizes(_HISTORY_FILE)
    except Exception:
        return None  # Best-effort: the plain nextest line still prints.


def _append_history(
    path: Path,
    args_str: str,
    duration_sec: float,
    exit_code: int,
    tree: str,
    tests: str,
) -> None:
    """Append one run record to the build-history journal, then trim.

    One pipe-delimited line per run, columns per ``_HISTORY_HEADER``, newest
    last so ``tail`` shows recent runs. An older header in line 1 is replaced
    in place; older records keep their shorter column set. The trim keeps the header plus the
    newest records, rewritten in place while still holding the lock — flock
    keeps concurrent agents (shared ``logs/``) from interleaving appends or
    racing the rewrite. Callers treat any failure as ignorable: the journal
    must never break a build.
    """
    args_str = args_str.replace("|", "/").replace("\n", " ").strip()
    line = (
        f"{time.strftime('%Y-%m-%dT%H:%M:%S')} | {duration_sec:.1f} | "
        f"{exit_code} | {args_str} | {tree} | {tests}\n"
    )
    with open(path, "a+", encoding="utf-8") as fh:
        if fcntl is not None:
            fcntl.flock(fh, fcntl.LOCK_EX)
        try:
            fh.seek(0)
            lines = fh.readlines()
            if lines and lines[0].startswith("timestamp |"):
                lines[0] = _HISTORY_HEADER + "\n"
            else:
                lines.insert(0, _HISTORY_HEADER + "\n")
            lines.append(line)
            if len(lines) > _HISTORY_MAX_LINES:
                lines = lines[:1] + lines[-(_HISTORY_MAX_LINES - 1) :]
            # "a+" writes land at EOF (O_APPEND), but truncate empties the
            # file first, so the rewrite starts back at the top.
            fh.seek(0)
            fh.truncate()
            fh.writelines(lines)
            fh.flush()
        finally:
            if fcntl is not None:
                fcntl.flock(fh, fcntl.LOCK_UN)


def main():
    global _pending_exec
    args = parse_args()

    # Journal inputs, captured before any chdir so the record reflects
    # exactly what the caller invoked.
    _run_start = time.time()
    history_args = " ".join(sys.argv[1:]).strip() or "(full gate)"

    os.chdir(os.path.dirname(os.path.abspath(__file__)) or os.getcwd())
    # Taken at the start: an edit made during the run is not covered.
    tree = _tree_identity(Path.cwd())

    log_dir = Path("logs")
    log_dir.mkdir(exist_ok=True)
    clean_old_logs(log_dir, max_age_days=3)
    log_path = log_dir / f"build_{time.strftime('%Y%m%d_%H%M%S')}.log"
    log_fh = open(log_path, "w", encoding="utf-8")
    _set_log_fh(log_fh)
    _stamp_session_id()

    record = StepRecord()

    exit_code = 0
    browser_skipped = False
    try:
        both_print("=" * 60)
        both_print("=== Chronicler Engine Build ===")
        both_print(f"Full build log: {log_path}")
        both_print("=" * 60)

        if args.command == "run":
            # run_server returns the argv to exec; the exec happens in the
            # __main__ block, after this function's finally has written the
            # epilogue and history line.
            _pending_exec = run_server(args, record)
        elif args.command:
            run_step(args, record)
        elif args.cleanup:
            run_cleanup(args, record)
        elif args.llm_only:
            run_llm_only(args, record)
        else:
            browser_skipped = bool(args.no_browser)
            run_gate(args, record)
    except SystemExit as e:
        # Step failure (check=True) re-raises SystemExit. Capture the code so
        # the finally can print the summary, then propagate.
        exit_code = int(e.code) if e.code is not None else 1
        _pending_exec = None  # a failed build must never exec the server
        raise
    finally:
        # Print the epilogue BEFORE closing the log so the summary + banner are
        # written into the log file, not only to stdout. Closing last preserves
        # the "agent can read a targeted slice immediately" rationale — the
        # close is delayed only by in-process writes, not by any I/O wait.
        _print_step_summary(record.timings, record.failures, log_path)

        # The pass/fail counts, one line per test tier, right before the banner:
        # an agent tailing stdout gets the verdict without grepping the full log.
        if _NextestSummary.lines:
            both_print(_NextestSummary.line_text(_epilogue_baselines()))
        for label, status, name in _NextestLeaks.lines:
            both_print(f"{status.lower()} test: {name}  ({label})")
        if browser_skipped:
            both_print('skipped: browser tier (run "python build.py browser")')
        if _CompileErrors.lines:
            shown = _CompileErrors.lines[: _CompileErrors.LIMIT]
            both_print(
                f"compile errors (first {len(shown)} of {len(_CompileErrors.lines)}):"
            )
            for line in shown:
                both_print(f"  {line}")

        both_print("=" * 60)
        both_print("=== Build Complete ===")
        both_print(f"Full build log: {log_path}")
        both_print("=" * 60)

        try:
            log_fh.flush()
            log_fh.close()
        except Exception:
            pass
        _LogState.fh = None

        # check=False steps (e.g. the test suite) record failures in the record
        # without raising SystemExit. Propagate them into the process exit code
        # so callers that trust the exit code (agents, pre-commit, CI) — and
        # the history journal below — see the true outcome. Done inside the
        # finally (not after the try) because the finally runs on every exit
        # path, so the journal can never disagree with the process exit code.
        if exit_code == 0 and record.failures:
            exit_code = 1

        try:
            _append_history(
                _HISTORY_FILE,
                history_args,
                time.time() - _run_start,
                exit_code,
                tree,
                _tests_column(_NextestSummary.sizes),
            )
        except Exception:
            pass  # Best-effort bookkeeping; never fail a run over it.

    return exit_code


if __name__ == "__main__":
    _exit_code = main()
    _exec_pending(_exit_code)
    sys.exit(_exit_code)
