"""Full build, validate, and test for Chronicler Engine.

Uses cargo-nextest for parallel test execution.

Stdout carries the agent-facing decision signal + tailable progress (banner,
step labels, ``$ cmd`` echoes, failure signals, Step Timing Summary, closing
banner with log path). Full output is written to ``logs/build_*.log``.
"""

import argparse
import io
import json
import os
import re
import shutil
import signal
import sqlite3
import subprocess
import sys
import tempfile
import time
from pathlib import Path

# Force UTF-8 for stdout/stderr on Windows to handle cargo's Unicode output
if sys.platform == "win32":
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding="utf-8", errors="replace")


class _LogState:
    """Module-level log handle for the current build run."""

    fh = None


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
    cmd = "cargo nextest run --no-fail-fast --retries 2 -j 4 --features testing"
    if include_llm:
        cmd += " --run-ignored all"
    return cmd


def get_coverage_cmd():
    """Return the coverage test command using nextest."""
    return "cargo llvm-cov nextest --no-report --no-fail-fast --retries 2 -j 4 --features testing"


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


def main():
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
    args = parser.parse_args()

    os.chdir(os.path.dirname(os.path.abspath(__file__)) or os.getcwd())

    log_dir = Path("logs")
    log_dir.mkdir(exist_ok=True)
    clean_old_logs(log_dir, max_age_days=3)
    log_path = log_dir / f"build_{time.strftime('%Y%m%d_%H%M%S')}.log"
    log_fh = open(log_path, "w", encoding="utf-8")
    _set_log_fh(log_fh)

    # Defined before the try block so the finally can always reach them.
    step_timings = []
    step_failures = []

    def timed_step(label, cmd, check=True, env=None):
        steps.next(label)
        start = time.time()
        try:
            rc = run(cmd, check=check, env=env)
            elapsed = time.time() - start
            failed = rc != 0
            step_timings.append(
                {
                    "step": label,
                    "elapsed_sec": round(elapsed, 2),
                    "failed": failed,
                }
            )
            if failed:
                step_failures.append(label)
        except SystemExit:
            elapsed = time.time() - start
            step_timings.append(
                {
                    "step": label,
                    "elapsed_sec": round(elapsed, 2),
                    "failed": True,
                }
            )
            step_failures.append(label)
            raise

    exit_code = 0
    try:
        both_print("=" * 60)
        both_print("=== Chronicler Engine Build ===")
        both_print(f"Full build log: {log_path}")
        both_print("=" * 60)

        cargo_target_dir = Path(args.target_dir) if args.target_dir else Path("target")
        build_profile = "release" if args.release else "debug"
        target_dir = cargo_target_dir / build_profile

        if args.diagnostic_benchmark:
            both_print("=== Diagnostic Benchmark Mode ===")
            benchmark_script = Path(__file__).parent / "scripts" / "diagnostic_benchmark.py"
            if benchmark_script.exists():
                run(f'python "{benchmark_script}"')
            else:
                both_print(f"ERROR: Benchmark script not found: {benchmark_script}")
                exit_code = 1
                return
            both_print("=== Diagnostic Benchmark Complete ===")
            return

        if args.cleanup:
            both_print("=== Cleanup Mode ===")
            log_status("Killing lingering chronicler processes...")
            kill_by_name("chronicler")

            lock_dir = Path(tempfile.gettempdir()) / "chronicler_test_ports"
            if lock_dir.exists():
                log_status(f"Cleaning stale port locks from {lock_dir}...")
                shutil.rmtree(lock_dir)

            if cargo_target_dir.exists():
                log_status(f"Removing build directory: {cargo_target_dir}")
                shutil.rmtree(cargo_target_dir)
            else:
                log_status(f"Build directory does not exist: {cargo_target_dir}")

            both_print("=== Cleanup Complete ===")
            return

        check_rust_version()
        require_nextest()

        if args.strict:
            os.environ["RUSTFLAGS"] = "-D warnings"
            both_print("Strict mode enabled: warnings treated as errors.")

        # Always kill manual runs on the default port first — this may release
        # the target directory lock if a manual `cargo run` was holding it.
        log_status("Checking for processes on port 3000...")
        kill_port(3000)

        cargo_env = {"NEXTEST_STATUS_LEVEL": "fail"}
        if args.target_dir:
            cargo_env["CARGO_TARGET_DIR"] = str(cargo_target_dir.resolve())
            log_status(f"Using custom target directory: {cargo_target_dir}")
            if is_target_locked(cargo_target_dir):
                both_print(
                    f"WARNING: Target directory {cargo_target_dir} appears to be "
                    "locked by another cargo process."
                )
                both_print("         Another agent may be building in this directory.")
        else:
            if is_target_locked(cargo_target_dir):
                both_print(
                    "WARNING: Default target directory (target/) appears to be "
                    "locked by another cargo process."
                )
                both_print("         Use --target-dir to build in a unique folder and avoid conflicts:")
                both_print("         python build.py --target-dir target/<unique-name>")

        if args.llm_only:
            steps = StepCounter(3)
            steps.next("Building...")
            run(
                f"cargo build {'--release' if args.release else ''} --features testing".strip(),
                env=cargo_env,
            )

            steps.next("Running LLM tests only...")
            both_print("=" * 60)
            both_print("NOTE: LLM tests contact the real OpenRouter API.")
            both_print("      Each test takes 1-3 minutes. Total: ~3-9 minutes.")
            both_print("      Do not interrupt. Set your tool timeout to >= 600s.")
            both_print("=" * 60)
            llm_cmd = get_test_cmd(include_llm=True)
            if "nextest" in llm_cmd:
                llm_cmd += " --test flow_llm_tests"
            else:
                llm_cmd += " flow_llm_tests -- --ignored"
            run(llm_cmd, check=False, env=cargo_env)

            steps.next("Done")
            both_print("=== Build Complete ===")
            return

        total_steps = 11  # Non-format validation, packaging, tests, and report steps.
        if not args.no_fmt:
            total_steps += 1
        steps = StepCounter(total_steps)

        if not args.no_fmt:
            timed_step("Formatting...", "cargo fmt", env=cargo_env)
        else:
            both_print("Skipping formatting (--no-fmt set).")

        timed_step(
            "Validating JSON data...",
            "python scripts/validate_data.py",
            env=cargo_env,
        )

        timed_step(
            "Running clippy...",
            "cargo clippy --all-targets --all-features -- -D warnings",
            env=cargo_env,
        )

        timed_step(
            "Running test structure guardrail...",
            "python scripts/check_test_structure.py",
            env=cargo_env,
        )

        timed_step(
            "Running Python docstring guardrail...",
            "python scripts/check_python_docstrings.py",
            env=cargo_env,
        )

        timed_step(
            "Running Python tests...",
            "python -m unittest discover scripts/tests -v",
            env=cargo_env,
        )

        timed_step(
            "Checking http_routes.md freshness...",
            "python scripts/extract_http_routes.py --check",
            env=cargo_env,
        )

        timed_step(
            "Validating markdown docs...",
            "python scripts/validate_docs.py",
            env=cargo_env,
        )

        steps.next("Copying data and assets for deployment...")
        target_dir.mkdir(parents=True, exist_ok=True)

        if Path("data").exists():
            dest_data = target_dir / "data"
            if dest_data.exists():
                shutil.rmtree(dest_data)
            shutil.copytree("data", dest_data)
            log_status(f"  Copied data/ -> {dest_data}")

        if Path("assets").exists():
            dest_assets = target_dir / "assets"
            if dest_assets.exists():
                shutil.rmtree(dest_assets)
            shutil.copytree("assets", dest_assets)
            log_status(f"  Copied assets/ -> {dest_assets}")

        (target_dir / "logs").mkdir(exist_ok=True)
        log_status("  Created logs/")

        log_status(f"  Package ready in {target_dir}/")
        log_status(f"  Deployment: copy {target_dir}/ folder to your target machine")

        # DB lives inside the target folder so each build profile has its own instance.
        target_data_dir = target_dir / "data"
        clean_sqlite_dbs(target_data_dir)

        if args.coverage:
            timed_step(
                "Running all tests with coverage...", get_coverage_cmd(), check=False, env=cargo_env
            )

            steps.next("Generating coverage report...")
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
                )
            else:
                both_print("Warning: Could not generate coverage JSON.")
        else:
            timed_step(
                "Running all tests...",
                get_test_cmd(include_llm=args.include_llm),
                check=False,
                env=cargo_env,
            )
            if not args.include_llm:
                both_print(
                    "    NOTE: 2 LLM tests were skipped. "
                    "Run 'python build.py --llm-only' to execute them."
                )
            steps.next("Skipping coverage report (use --coverage to enable)")

        both_print("=== Build Complete ===")

        project_root_tmp = Path(__file__).resolve().parent.parent / "tmp"
        engine_tmp = Path("tmp")
        clean_tmp_dirs([project_root_tmp, engine_tmp], max_age_days=30)

        db_path = target_dir / "data" / "chronicler.db"
        if db_path.exists():
            dump_dir = Path("tmp") / "db_dumps"
            dump_sqlite_to_jsonl(db_path, dump_dir)
    except SystemExit as e:
        # Step failure (check=True) re-raises SystemExit. Capture the code so
        # the finally can print the summary, then propagate.
        exit_code = int(e.code) if e.code is not None else 1
        raise
    finally:
        # Flush + close the log so an agent can read a targeted slice immediately.
        try:
            log_fh.flush()
            log_fh.close()
        except Exception:
            pass
        _LogState.fh = None

        _print_step_summary(step_timings, step_failures, log_path)

        both_print("=" * 60)
        both_print("=== Build Complete ===")
        both_print(f"Full build log: {log_path}")
        both_print("=" * 60)

    return exit_code


if __name__ == "__main__":
    sys.exit(main())
