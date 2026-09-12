"""Full build, validate, and test for Chronicler Engine.

Uses cargo-nextest for parallel test execution.

Two invocation modes:

- Full gate (default): ``python build.py`` runs fmt, validation, clippy,
  guardrails, the full test suite, and packaging.
- Step mode: ``python build.py <step>`` runs one registry step (e.g.
  ``clippy``, ``fmt``, ``nextest <pattern>``) with a minimal prelude — no
  port-3000 kill, no asset copy, no SQLite cleanup. ``--target-dir`` and
  ``--strict`` are accepted on either side of the step; all other top-level
  flags are gate-only and rejected next to a step.

Stdout carries the agent-facing decision signal + tailable progress (banner,
step labels, ``$ cmd`` echoes, failure signals, Step Timing Summary, closing
banner with log path). Full output is written to ``logs/build_*.log``; each
log's first line is a session stamp (see ``_stamp_session_id``). Every
completed run also appends one pipe-delimited summary line (timestamp,
duration, exit code, args) to the append-only journal
``logs/build_history.txt`` — see ``_append_history``.
"""

import argparse
import io
import json
import os
import re
import shlex
import shutil
import signal
import sqlite3
import subprocess
import sys
import tempfile
import time
from collections import defaultdict
from pathlib import Path
from typing import NamedTuple

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


class _NextestSummary:
    """Last nextest summary line captured this run, re-printed at the epilogue.

    Module state (like _LogState) because run() sys.exits on a checked
    failure before its caller could inspect the captured output; main()'s
    finally must still be able to print the pass/fail counts at the tail.
    """

    line = None


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


def get_test_cmd(include_llm=False):
    """Return the test command using nextest."""
    cmd = _NEXTEST_RUN
    if include_llm:
        cmd += " --profile llm --run-ignored all"
    return cmd


# nextest per-test result line, e.g.:
#   "        PASS [  18.645s] ( 4/26) chronicler_engine::browser behaviour::test_x"
# The progress counter is optional: nextest only prints it in some
# --show-progress modes, and the timing report must survive config drift.
_NEXTEST_RESULT_RE = re.compile(
    r"^\s*(?:PASS|FAIL|SKIP)\s*\[([^\]]*)\]\s*(?:\([^)]*\))?\s*(\S.*)$"
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
    A zero "skipped" segment is omitted; a nonzero one is kept so the counts
    still add up to the total run.
    """
    rendered = None
    for line in output.splitlines():
        if not _NEXTEST_SUMMARY_RE.match(line):
            continue
        passed = _summary_count(line, "passed") or "?"
        failed = _summary_count(line, "failed") or "0"
        skipped = _summary_count(line, "skipped")
        rendered = f"nextest: {passed} passed, {failed} failed"
        if skipped and skipped != "0":
            rendered += f", {skipped} skipped"
    return rendered


def _stash_nextest_summary(output: str) -> None:
    """Capture nextest's final summary line for the closing epilogue."""
    line = _nextest_summary_line(output)
    if line is not None:
        _NextestSummary.line = line


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


def run_with_test_timings(cmd, env=None, check=True):
    """Run nextest and print a per-test timing report: slowest tests (top 30)
    and per-binary totals. Returns the exit code; exits on failure when check=True.
    """
    both_print(f"$ {cmd}  (with --test-timings)")
    timed_cmd = f"{cmd} --final-status-level pass"
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

    timings = []
    for line in (result.stdout or "").splitlines() + (result.stderr or "").splitlines():
        m = _NEXTEST_RESULT_RE.match(line)
        if not m:
            continue
        secs = _nextest_duration_to_secs(m.group(1))
        name = m.group(2).strip()
        # SKIP lines carry no duration measurement.
        if secs == 0.0 and "SKIP" in line:
            continue
        timings.append((secs, name))
    _stash_nextest_summary(
        "\n".join(
            (result.stdout or "").splitlines() + (result.stderr or "").splitlines()
        )
    )

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
    both_print("---")

    if result.returncode != 0:
        both_print(f"FAILED with code {result.returncode}")
        if check:
            sys.exit(result.returncode)
    return result.returncode


def get_coverage_cmd():
    """Return the coverage test command using nextest."""
    return "cargo llvm-cov nextest --no-report --no-fail-fast"


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
            f"{_NEXTEST_RUN} --tests",
            needs_nextest=True,
            help="Run the integration test suite (~1-2 minutes).",
        ),
        StepSpec(
            "nextest",
            "Running nextest pattern...",
            _NEXTEST_RUN,
            needs_nextest=True,
            pattern_arg=True,
            help="Run cargo nextest with a test-name pattern.",
        ),
    ]
}

# Full-gate step order. "COPY" expands to the deployment-asset copy step,
# "TESTS" to the composite test step (coverage/timings variants), and
# "REPORT" to the coverage report (or the skip note when coverage is off).
#
# The order is load-bearing: the architecture/guardrail binaries run before
# the ~2-minute full suite and fail the build immediately (check=True) —
# otherwise a guardrail failure only surfaces after the full suite has run.
GATE_ORDER = [
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
    "cleanup",
    "diagnostic_benchmark",
    "test_timings",
)


def parse_args(argv=None):
    """Parse CLI arguments into a namespace. Pure: no filesystem or subprocess
    side effects.

    Step subcommands share the top-level ``--target-dir`` and ``--strict``
    flags; the shared copies use ``SUPPRESS`` defaults so a value given before
    the subcommand is not clobbered by the subparser.
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
        "--strict",
        action="store_true",
        help="Enable strict mode: warnings are errors, debug assertions enabled",
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
        help="Skip cargo fmt (useful for secondary agents to avoid source-file races)",
    )
    parser.add_argument(
        "--cleanup",
        action="store_true",
        dest="cleanup",
        help="Kill lingering chronicler processes and clean build artifacts",
    )
    parser.add_argument(
        "--diagnostic-benchmark",
        action="store_true",
        dest="diagnostic_benchmark",
        help="Run the diagnostic signal quality benchmark and generate a report",
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
        step_parser.add_argument(
            "--strict",
            action="store_true",
            default=argparse.SUPPRESS,
            help=argparse.SUPPRESS,
        )
        if spec.pattern_arg:
            step_parser.add_argument(
                "pattern", help="Test-name pattern passed to cargo nextest"
            )

    args = parser.parse_args(argv)
    _reject_gate_flags_with_step(parser, args)
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


def _cargo_env_for(args) -> dict:
    """Cargo environment shared by gate and step runs."""
    env = {"NEXTEST_STATUS_LEVEL": "fail"}
    if getattr(args, "target_dir", None):
        env["CARGO_TARGET_DIR"] = str(Path(args.target_dir).resolve())
    return env


def _apply_strict(args):
    """Enable strict mode: warnings treated as errors via RUSTFLAGS."""
    if getattr(args, "strict", False):
        os.environ["RUSTFLAGS"] = "-D warnings"
        both_print("Strict mode enabled: warnings treated as errors.")


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


def _step_command(spec: StepSpec, pattern: str | None = None) -> str:
    """Assemble the shell command for a step spec (quotes the nextest pattern)."""
    if spec.pattern_arg:
        return f"{spec.cmd} {shlex.quote(pattern)}"
    return spec.cmd


def kill_port(port: int):
    """Kill any process using the specified port."""
    try:
        result = subprocess.run(
            f"netstat -ano | Select-String ':{port}'",
            shell=True,
            capture_output=True,
            text=True,
        )
        if result.stdout:
            for line in result.stdout.strip().split("\n"):
                if "LISTENING" in line:
                    parts = line.split()
                    if len(parts) >= 5:
                        pid = int(parts[-1])
                        log_status(f"Killing process {pid} on port {port}...")
                        try:
                            os.kill(pid, signal.SIGTERM)
                        except (ProcessLookupError, PermissionError):
                            subprocess.run(f"taskkill /F /PID {pid}", shell=True)
    except Exception as e:
        log_status(f"Note: Could not check port {port}: {e}")


def kill_by_name(name: str):
    """Kill any process with the given name substring."""
    try:
        result = subprocess.run(
            f"tasklist | findstr -i {name}",
            shell=True,
            capture_output=True,
            text=True,
        )
        if result.stdout:
            for line in result.stdout.splitlines():
                # Format: image name PID session# mem usage
                parts = line.split()
                if len(parts) >= 2:
                    pid = parts[1]
                    if pid.isdigit():
                        log_status(f"Killing process {parts[0]} (PID {pid})...")
                        try:
                            subprocess.run(
                                f"taskkill /F /PID {pid}",
                                shell=True,
                                capture_output=True,
                            )
                        except Exception as e:
                            log_status(f"Failed to kill PID {pid}: {e}")
    except Exception as e:
        log_status(f"Note: Could not search for processes: {e}")


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
        _stash_nextest_summary(out or "")
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
        _stash_nextest_summary(
            (result.stdout or "") + "\n" + (result.stderr or "")
        )
        if check and result.returncode != 0:
            both_print(f"FAILED with code {result.returncode}")
            sys.exit(result.returncode)
        return result.returncode


def is_target_locked(target_dir: Path) -> bool:
    """Check if cargo holds a lock on the target directory via .cargo-lock."""
    # Cargo creates .cargo-lock inside the profile subdirectory
    # and holds an OS lock on it.
    for profile in ["debug", "release"]:
        lock_file = target_dir / profile / ".cargo-lock"
        if not lock_file.exists():
            continue
        try:
            fd = os.open(str(lock_file), os.O_RDWR)
            try:
                if sys.platform == "win32":
                    import msvcrt

                    msvcrt.locking(fd, msvcrt.LK_NBLCK, 1)
                    msvcrt.locking(fd, msvcrt.LK_UNLCK, 1)
                else:
                    import fcntl

                    fcntl.flock(fd, fcntl.LOCK_NB | fcntl.LOCK_EX)
                os.close(fd)
                return False  # We got the lock, so cargo doesn't hold it
            except (OSError, BlockingIOError, IOError):
                os.close(fd)
                return True  # Lock is held by another process (cargo)
        except OSError:
            continue
    return False  # No lock file found, assume not locked


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


def _timed_run(counter, record, label, cmd, check=True, env=None, timings=False):
    """Run one step, print progress, and record its timing and outcome."""
    counter.next(label)
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

    kind: str  # "cmd" | "copy" | "tests" | "coverage_report" | "note"
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
        if name == "COPY":
            plan.append(GateStep("copy", "Copying data and assets for deployment..."))
            continue
        if name == "TESTS":
            if args.coverage:
                label = (
                    "Running all tests with coverage and timings..."
                    if args.test_timings
                    else "Running all tests with coverage..."
                )
                plan.append(
                    GateStep(
                        "tests",
                        label,
                        get_coverage_cmd(),
                        check=False,
                        timings=args.test_timings,
                    )
                )
            else:
                label = (
                    "Running all tests with timings..."
                    if args.test_timings
                    else "Running all tests..."
                )
                plan.append(
                    GateStep(
                        "tests",
                        label,
                        get_test_cmd(include_llm=args.include_llm),
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
        spec = REGISTRY[name]
        if name == "fmt" and args.no_fmt:
            continue
        plan.append(GateStep("cmd", spec.label, spec.cmd))
    return plan


def _execute_gate_plan(plan, args, cargo_env, record):
    """Run the expanded gate plan. The counter total is len(plan)."""
    counter = StepCounter(len(plan))
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
            if not args.coverage and not args.include_llm:
                both_print(
                    "    NOTE: 2 LLM tests were skipped. "
                    "Run 'python build.py --llm-only' to execute them."
                )
        elif step.kind == "coverage_report":
            counter.next(step.label)
            _generate_coverage_report(args, cargo_env)
        elif step.kind == "note":
            counter.next(step.label)
        else:  # pragma: no cover - guarded by construction
            raise ValueError(f"Unknown gate step kind: {step.kind}")


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
    """Gate-mode prelude: strict env, port-3000 kill, target-dir env + lock warning."""
    _apply_strict(args)

    # Always kill manual runs on the default port first — this may release
    # the target directory lock if a manual `cargo run` was holding it.
    log_status("Checking for processes on port 3000...")
    kill_port(3000)

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
    if args.no_fmt:
        both_print("Skipping formatting (--no-fmt set).")
    plan = _plan_gate_steps(args)
    _execute_gate_plan(plan, args, cargo_env, record)
    _gate_tail(args)


def run_step(args, record):
    """Single-step mode: run one registry step with a minimal prelude.

    Skips the gate-only prelude (port-3000 kill, asset copy, SQLite cleanup)
    so quick iteration does not disturb a running dev server.
    """
    check_rust_version()
    spec = REGISTRY[args.command]
    if spec.needs_nextest:
        require_nextest()

    _apply_strict(args)
    cargo_env = _cargo_env_for(args)
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
        _step_command(spec, getattr(args, "pattern", None)),
        check=True,
        env=cargo_env,
    )


def run_cleanup(args, record):
    """Cleanup mode: kill lingering processes and remove build artifacts."""
    both_print("=== Cleanup Mode ===")
    log_status("Killing lingering chronicler processes...")
    kill_by_name("chronicler")

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


def run_diagnostic(args, record):
    """Diagnostic benchmark mode."""
    both_print("=== Diagnostic Benchmark Mode ===")
    benchmark_script = Path(__file__).parent / "scripts" / "diagnostic_benchmark.py"
    if benchmark_script.exists():
        run(f'python "{benchmark_script}"')
    else:
        both_print(f"ERROR: Benchmark script not found: {benchmark_script}")
        sys.exit(1)
    both_print("=== Diagnostic Benchmark Complete ===")


def run_llm_only(args, record):
    """LLM-only mode: build, then run only the slow LLM tests."""
    check_rust_version()
    require_nextest()

    cargo_env = _gate_prelude(args)

    counter = StepCounter(3)
    counter.next("Building...")
    run(
        f"cargo build {'--release' if args.release else ''}".strip(),
        env=cargo_env,
    )

    counter.next("Running LLM tests only...")
    both_print("=" * 60)
    both_print("NOTE: LLM tests contact the real OpenRouter API.")
    both_print("      Each test takes 1-3 minutes. Total: ~3-9 minutes.")
    both_print("      Do not interrupt. Set your tool timeout to >= 600s.")
    both_print("=" * 60)
    # get_test_cmd always returns nextest; the --test llm filter selects the
    # LLM binary inside the llm profile run.
    llm_cmd = get_test_cmd(include_llm=True) + " --test llm"
    run(llm_cmd, check=True, env=cargo_env)

    counter.next("Done")
    both_print("=== Build Complete ===")


_HISTORY_FILE = Path("logs/build_history.txt")
_HISTORY_HEADER = "timestamp | duration_s | exit_code | args"
_HISTORY_MAX_LINES = 1000


def _append_history(path: Path, args_str: str, duration_sec: float, exit_code: int) -> None:
    """Append one run record to the build-history journal, then trim.

    One pipe-delimited line per run, columns per ``_HISTORY_HEADER``, newest
    last so ``tail`` shows recent runs. The trim keeps the header plus the
    newest records, rewritten in place while still holding the lock — flock
    keeps concurrent agents (shared ``logs/``) from interleaving appends or
    racing the rewrite. Callers treat any failure as ignorable: the journal
    must never break a build.
    """
    args_str = args_str.replace("|", "/").replace("\n", " ").strip()
    line = (
        f"{time.strftime('%Y-%m-%dT%H:%M:%S')} | {duration_sec:.1f} | "
        f"{exit_code} | {args_str}\n"
    )
    with open(path, "a+", encoding="utf-8") as fh:
        if fcntl is not None:
            fcntl.flock(fh, fcntl.LOCK_EX)
        try:
            fh.seek(0)
            lines = fh.readlines()
            if not lines or lines[0].rstrip("\n") != _HISTORY_HEADER:
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
    args = parse_args()

    # Journal inputs, captured before any chdir so the record reflects
    # exactly what the caller invoked.
    _run_start = time.time()
    history_args = " ".join(sys.argv[1:]).strip() or "(full gate)"

    os.chdir(os.path.dirname(os.path.abspath(__file__)) or os.getcwd())

    log_dir = Path("logs")
    log_dir.mkdir(exist_ok=True)
    clean_old_logs(log_dir, max_age_days=3)
    log_path = log_dir / f"build_{time.strftime('%Y%m%d_%H%M%S')}.log"
    log_fh = open(log_path, "w", encoding="utf-8")
    _set_log_fh(log_fh)
    _stamp_session_id()

    record = StepRecord()

    exit_code = 0
    try:
        both_print("=" * 60)
        both_print("=== Chronicler Engine Build ===")
        both_print(f"Full build log: {log_path}")
        both_print("=" * 60)

        if args.command:
            run_step(args, record)
        elif args.diagnostic_benchmark:
            run_diagnostic(args, record)
        elif args.cleanup:
            run_cleanup(args, record)
        elif args.llm_only:
            run_llm_only(args, record)
        else:
            run_gate(args, record)
    except SystemExit as e:
        # Step failure (check=True) re-raises SystemExit. Capture the code so
        # the finally can print the summary, then propagate.
        exit_code = int(e.code) if e.code is not None else 1
        raise
    finally:
        # Print the epilogue BEFORE closing the log so the summary + banner are
        # written into the log file, not only to stdout. Closing last preserves
        # the "agent can read a targeted slice immediately" rationale — the
        # close is delayed only by in-process writes, not by any I/O wait.
        _print_step_summary(record.timings, record.failures, log_path)

        # The pass/fail counts, one line, right before the banner: an agent
        # tailing stdout gets the test verdict without grepping the full log.
        if _NextestSummary.line:
            both_print(_NextestSummary.line)

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
            )
        except Exception:
            pass  # Best-effort bookkeeping; never fail a run over it.

    return exit_code


if __name__ == "__main__":
    sys.exit(main())
