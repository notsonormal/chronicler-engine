#!/usr/bin/env python3
"""Full build, validate, and test for Chronicler Engine.

Uses cargo-nextest for parallel test execution.
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
    sys.stdout = io.TextIOWrapper(
        sys.stdout.buffer, encoding="utf-8", errors="replace"
    )
    sys.stderr = io.TextIOWrapper(
        sys.stderr.buffer, encoding="utf-8", errors="replace"
    )


class TeeLogger:
    """Write to both stdout and a log file simultaneously."""

    def __init__(self, log_path: Path, original_stdout):
        self.log_file = open(log_path, "w", encoding="utf-8")
        self.original_stdout = original_stdout
        self.log_path = log_path

    def write(self, message):
        self.original_stdout.write(message)
        self.log_file.write(message)
        self.log_file.flush()

    def flush(self):
        self.original_stdout.flush()
        self.log_file.flush()

    def close(self):
        self.log_file.close()


class StepCounter:
    """Simple step counter for build progress output."""

    def __init__(self, total: int):
        self.current = 0
        self.total = total

    def next(self, label: str):
        self.current += 1
        print(f"[{self.current}/{self.total}] {label}")


def check_rust_version():
    """Ensure rustc >= 1.85."""
    result = subprocess.run(
        ["rustc", "--version"],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print("ERROR: Could not determine Rust version.")
        sys.exit(1)
    match = re.search(r"rustc (\d+)\.(\d+)", result.stdout)
    if not match:
        print("ERROR: Could not parse Rust version.")
        sys.exit(1)
    major, minor = int(match.group(1)), int(match.group(2))
    if major < 1 or (major == 1 and minor < 85):
        print(f"ERROR: Rust {major}.{minor} found, but >= 1.85 is required.")
        sys.exit(1)
    print(f"Rust version: {major}.{minor} (OK)")


def require_nextest():
    """Ensure cargo-nextest is installed; exit with error if not."""
    result = subprocess.run(
        ["cargo", "nextest", "--version"],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print("ERROR: The nextest library is required. Install it with: cargo install cargo-nextest --locked")
        sys.exit(1)


def get_test_cmd(include_llm=False):
    """Return the test command using nextest."""
    cmd = "cargo nextest run --retries 2 -j 4"
    if include_llm:
        cmd += " --run-ignored all"
    return cmd


def get_coverage_cmd():
    """Return the coverage test command using nextest."""
    return "cargo llvm-cov nextest --no-report --retries 2 -j 4"


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
                        print(f"Killing process {pid} on port {port}...")
                        try:
                            os.kill(pid, signal.SIGTERM)
                        except (ProcessLookupError, PermissionError):
                            # Try force kill on Windows
                            subprocess.run(f"taskkill /F /PID {pid}", shell=True)
    except Exception as e:
        print(f"Note: Could not check port {port}: {e}")


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
                        print(f"Killing process {parts[0]} (PID {pid})...")
                        try:
                            subprocess.run(
                                f"taskkill /F /PID {pid}",
                                shell=True,
                                capture_output=True,
                            )
                        except Exception as e:
                            print(f"Failed to kill PID {pid}: {e}")
    except Exception as e:
        print(f"Note: Could not search for processes: {e}")


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
        print(f"  Removed stale SQLite DBs: {', '.join(removed)}")


def clean_old_logs(log_dir: Path, max_age_days: int = 3):
    """Remove log files older than max_age_days from the log directory."""
    if not log_dir.exists():
        return
    now = time.time()
    max_age_sec = max_age_days * 86400
    removed = []
    for f in log_dir.iterdir():
        if f.is_file() and f.name.startswith("build_") and f.suffix == ".log" and (now - f.stat().st_mtime) > max_age_sec:
            f.unlink()
            removed.append(f.name)
    if removed:
        print(f"  Removed old build logs (> {max_age_days} days): {', '.join(removed)}")


def clean_old_dumps(dump_dir: Path, max_age_days: int = 3):
    """Remove dump files older than max_age_days from the dump directory."""
    if not dump_dir.exists():
        return
    now = time.time()
    max_age_sec = max_age_days * 86400
    removed = []
    for f in dump_dir.iterdir():
        if f.is_file() and (now - f.stat().st_mtime) > max_age_sec:
            f.unlink()
            removed.append(f.name)
    if removed:
        print(f"  Removed old dumps (> {max_age_days} days): {', '.join(removed)}")


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

        # Get all user tables (exclude sqlite_internal tables)
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
            print(f"  Dumped tables to {output_dir}/: {', '.join(dumped)}")
        else:
            print(f"  No data to dump in {db_path}")
    except Exception as e:
        print(f"  Warning: Could not dump SQLite DB: {e}")


def run(cmd, cwd=None, check=True, show_output=True, env=None):
    """Run a command and handle output.

    Captures both stdout and stderr to avoid PowerShell ErrorRecord wrapping,
    then prints them to stdout so the user still sees everything.
    """
    print(f"$ {cmd}")
    merged_env = os.environ.copy()
    if env:
        merged_env.update(env)

    if show_output:
        # Stream output in real-time to avoid looking "stuck" on long commands
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
        for line in process.stdout:
            # Filter out noisy cargo-llvm-cov info messages
            if line.strip().startswith("info: cargo-llvm-cov"):
                continue
            print(line, end="")
        process.wait()
        if check and process.returncode != 0:
            print(f"FAILED with code {process.returncode}")
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
            print(result.stdout)
        if result.stderr:
            print(result.stderr)
        if check and result.returncode != 0:
            print(f"FAILED with code {result.returncode}")
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
            "Run only the slow LLM tests "
            "(skips formatting, clippy, guardrails, and other tests)"
        ),
    )
    parser.add_argument(
        "--validate-data",
        action="store_true",
        help="Validate JSON data files against schemas",
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

    # Setup tee logging: stdout goes to both terminal and log file
    log_dir = Path("logs")
    log_dir.mkdir(exist_ok=True)
    clean_old_logs(log_dir, max_age_days=3)
    log_path = log_dir / f"build_{time.strftime('%Y%m%d_%H%M%S')}.log"
    tee = TeeLogger(log_path, sys.stdout)
    sys.stdout = tee

    print("=== Chronicler Engine Build ===")
    print(f"Build log: {log_path}")

    # Resolve target directories
    cargo_target_dir = Path(args.target_dir) if args.target_dir else Path("target")
    build_profile = "release" if args.release else "debug"
    target_dir = cargo_target_dir / build_profile

    if args.diagnostic_benchmark:
        print("=== Diagnostic Benchmark Mode ===")
        benchmark_script = Path(__file__).parent / "scripts" / "diagnostic_benchmark.py"
        if benchmark_script.exists():
            run(f'python "{benchmark_script}"')
        else:
            print(f"ERROR: Benchmark script not found: {benchmark_script}")
            sys.exit(1)
        print("=== Diagnostic Benchmark Complete ===")
        return 0

    if args.cleanup:
        print("=== Cleanup Mode ===")
        print("Killing lingering chronicler processes...")
        kill_by_name("chronicler")

        # Clean up stale port lock files from crashed test runs
        lock_dir = Path(tempfile.gettempdir()) / "chronicler_test_ports"
        if lock_dir.exists():
            print(f"Cleaning stale port locks from {lock_dir}...")
            shutil.rmtree(lock_dir)

        # Clean build artifacts for the resolved target directory
        if cargo_target_dir.exists():
            print(f"Removing build directory: {cargo_target_dir}")
            shutil.rmtree(cargo_target_dir)
        else:
            print(f"Build directory does not exist: {cargo_target_dir}")

        print("=== Cleanup Complete ===")
        return 0

    check_rust_version()
    require_nextest()

    if args.strict:
        os.environ["RUSTFLAGS"] = "-D warnings"
        print("Strict mode enabled: warnings treated as errors.")

    # Always kill manual runs on the default port first — this may release
    # the target directory lock if a manual `cargo run` was holding it.
    print("Checking for processes on port 3000...")
    kill_port(3000)

    # Base environment for cargo commands
    cargo_env = {"NEXTEST_STATUS_LEVEL": "fail"}
    if args.target_dir:
        cargo_env["CARGO_TARGET_DIR"] = str(cargo_target_dir.resolve())
        print(f"Using custom target directory: {cargo_target_dir}")
        if is_target_locked(cargo_target_dir):
            print(
                f"WARNING: Target directory {cargo_target_dir} appears to be "
                "locked by another cargo process."
            )
            print("         Another agent may be building in this directory.")
    else:
        # Default target directory
        if is_target_locked(cargo_target_dir):
            print(
                "WARNING: Default target directory (target/) appears to be "
                "locked by another cargo process."
            )
            print(
                "         Use --target-dir to build in a unique folder "
                "and avoid conflicts:"
            )
            print("         python build.py --target-dir target/<unique-name>")

    if args.llm_only:
        steps = StepCounter(3)
        steps.next("Building...")
        run(
            f"cargo build {'--release' if args.release else ''}".strip(),
            env=cargo_env,
        )

        steps.next("Running LLM tests only...")
        print("=" * 60)
        print("NOTE: LLM tests contact the real OpenRouter API.")
        print("      Each test takes 1-3 minutes. Total: ~3-9 minutes.")
        print("      Do not interrupt. Set your tool timeout to >= 600s.")
        print("=" * 60)
        llm_cmd = get_test_cmd(include_llm=True)
        if "nextest" in llm_cmd:
            llm_cmd += " --test flow_llm_tests"
        else:
            llm_cmd += " flow_llm_tests -- --ignored"
        run(llm_cmd, check=False, env=cargo_env)

        steps.next("Done")
        print("=== Build Complete ===")
        return 0

    # Compute total steps for the main build path
    total_steps = 8  # clippy, arch, guardrails, test-structure, build, copy assets, tests, report/skip
    if not args.no_fmt:
        total_steps += 1
    if args.validate_data:
        total_steps += 1
    steps = StepCounter(total_steps)

    step_timings = []
    step_failures = []

    def timed_step(label, cmd, check=True, env=None):
        steps.next(label)
        start = time.time()
        try:
            run(cmd, check=check, env=env)
            elapsed = time.time() - start
            step_timings.append({"step": label, "elapsed_sec": round(elapsed, 2), "failed": False})
        except SystemExit as e:
            elapsed = time.time() - start
            step_timings.append({"step": label, "elapsed_sec": round(elapsed, 2), "failed": True})
            step_failures.append(label)
            raise

    if args.validate_data:
        timed_step("Validating JSON data...", "python scripts/validate_data.py")
        print("Data validation successful.")

    if not args.no_fmt:
        timed_step("Formatting...", "cargo fmt", env=cargo_env)
    else:
        print("Skipping formatting (--no-fmt set).")

    timed_step("Running clippy...", "cargo clippy --all-targets --all-features -- -D warnings", env=cargo_env)

    timed_step("Running architecture guardrail tests...", "cargo nextest run --test architecture", env=cargo_env)

    timed_step("Running custom guardrails tests...", "cargo nextest run --test guardrails", env=cargo_env)

    timed_step("Running test structure guardrail...", "python scripts/check_test_structure.py", env=cargo_env)

    timed_step(f"Building ({build_profile})...", f"cargo build {'--release' if args.release else ''}".strip(), env=cargo_env)

    steps.next("Copying data and assets for deployment...")
    target_dir.mkdir(parents=True, exist_ok=True)

    # Copy data folder (worlds, images, etc.)
    if Path("data").exists():
        dest_data = target_dir / "data"
        if dest_data.exists():
            shutil.rmtree(dest_data)
        shutil.copytree("data", dest_data)
        print(f"  Copied data/ -> {dest_data}")

    # Copy assets folder (HTML, CSS, etc.)
    if Path("assets").exists():
        dest_assets = target_dir / "assets"
        if dest_assets.exists():
            shutil.rmtree(dest_assets)
        shutil.copytree("assets", dest_assets)
        print(f"  Copied assets/ -> {dest_assets}")

    # Create logs directory
    (target_dir / "logs").mkdir(exist_ok=True)
    print("  Created logs/")

    print(f"  Package ready in {target_dir}/")
    print(f"  Deployment: copy {target_dir}/ folder to your target machine")

    # Clean stale SQLite databases before tests/application run.
    # DB lives inside the target folder so each build profile has its own instance.
    target_data_dir = target_dir / "data"
    clean_sqlite_dbs(target_data_dir)

    if args.coverage:
        timed_step("Running all tests with coverage...", get_coverage_cmd(), check=False, env=cargo_env)

        steps.next("Generating coverage report...")
        json_path = cargo_target_dir / "llvm-cov" / "coverage.json"
        json_path.parent.mkdir(parents=True, exist_ok=True)
        run(
            f'cargo llvm-cov report --json --output-path "{json_path}"',
            check=False,
            env=cargo_env,
        )
        if json_path.exists():
            run(
                f'python scripts/parse_coverage.py --json "{json_path}"',
                check=False,
            )
        else:
            print("Warning: Could not generate coverage JSON.")
    else:
        timed_step("Running all tests...", get_test_cmd(include_llm=args.include_llm), check=False, env=cargo_env)
        if not args.include_llm:
            print(
                "    NOTE: 3 LLM tests were skipped. "
                "Run 'python build.py --llm-only' to execute them."
            )
        steps.next("Skipping coverage report (use --coverage to enable)")

    print("=== Build Complete ===")

    # Dump SQLite database contents to JSONL for easy inspection
    db_path = target_dir / "data" / "chronicler.db"
    if db_path.exists():
        dump_dir = Path("tmp") / "db_dumps"
        dump_dir.mkdir(parents=True, exist_ok=True)
        clean_old_dumps(dump_dir, max_age_days=3)
        dump_sqlite_to_jsonl(db_path, dump_dir)

    # Print timing summary
    if step_timings:
        print("\n--- Step Timing Summary ---")
        total = sum(t["elapsed_sec"] for t in step_timings)
        for t in step_timings:
            status = "FAILED" if t["failed"] else "OK"
            print(f"  {t['elapsed_sec']:>6.2f}s  [{status}]  {t['step']}")
        print(f"  {'':>6}   Total: {total:.2f}s")
        if step_failures:
            print(f"\n  Failed steps: {', '.join(step_failures)}")
        print("---")

    return 0


if __name__ == "__main__":
    sys.exit(main())
