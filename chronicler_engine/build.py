#!/usr/bin/env python3
"""Full build, validate, and test for Chronicler Engine.

Uses cargo-nextest for parallel test execution.
"""

import argparse
import subprocess
import sys
import os
import signal
from pathlib import Path
import shutil

# Force UTF-8 for stdout/stderr on Windows to handle cargo's Unicode output
if sys.platform == "win32":
    import io
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding="utf-8", errors="replace")


def kill_port(port: int):
    """Kill any process using the specified port."""
    try:
        # Find process using the port
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
                            subprocess.run(f"taskkill /F /PID {pid}", shell=True, capture_output=True)
                        except Exception as e:
                            print(f"Failed to kill PID {pid}: {e}")
    except Exception as e:
        print(f"Note: Could not search for processes: {e}")


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
        help="Run only the slow LLM tests (skips formatting, clippy, guardrails, and other tests)",
    )
    args = parser.parse_args()

    print("=== Chronicler Engine Build ===")
    os.chdir(os.path.dirname(os.path.abspath(__file__)) or os.getcwd())

    # Kill any process on ports used by tests
    print("Checking for processes on test ports...")
    for port in [3000, 3001, 3002]:
        kill_port(port)

    # Also kill any lingering server processes by name
    print("Checking for lingering server processes...")
    kill_by_name("chronicler")

    # Clean up stale port lock files from crashed test runs
    import tempfile
    lock_dir = Path(tempfile.gettempdir()) / "chronicler_test_ports"
    if lock_dir.exists():
        print(f"Cleaning stale port locks from {lock_dir}...")
        shutil.rmtree(lock_dir)

    print("[1/8] Formatting...")
    run("cargo fmt")

    print("[2/8] Running clippy...")
    run("cargo clippy --all-targets --all-features -- -D warnings")

    print("[3/8] Running architecture guardrail tests...")
    run("cargo test --test architecture")

    print("[4/8] Running custom guardrails tests...")
    run("cargo test --test guardrails")

    build_profile = "release" if args.release else "debug"
    build_flag = "--release" if args.release else ""
    target_dir = Path(f"target/{build_profile}")

    print(f"[5/8] Building ({build_profile})...")
    run(f"cargo build {build_flag}".strip())

    print("[6/8] Copying data and assets for deployment...")
    target_dir.mkdir(exist_ok=True)

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

    test_env = {"NEXTEST_STATUS_LEVEL": "fail"}

    if args.llm_only:
        print("[1/3] Building...")
        run(f"cargo build {build_flag}".strip())

        print("[2/3] Running LLM tests only...")
        print("=" * 60)
        print("NOTE: LLM tests contact the real OpenRouter API.")
        print("      Each test takes 1-3 minutes. Total: ~3-9 minutes.")
        print("      Do not interrupt. Set your tool timeout to >= 600s.")
        print("=" * 60)
        run(
            "cargo nextest run --retries 2 -j 4 --run-ignored all --test flow_llm_tests",
            check=False,
            env=test_env,
        )

        print("[3/3] Done")
        print("=== Build Complete ===")
        return 0

    if args.coverage:
        print("[7/8] Running all tests with coverage...")
        run(
            "cargo llvm-cov nextest --no-report --retries 2 -j 4",
            check=False,
            env=test_env,
        )

        print("[8/8] Generating coverage report...")
        json_path = Path("target/llvm-cov/coverage.json")
        json_path.parent.mkdir(parents=True, exist_ok=True)
        run(
            f'cargo llvm-cov report --json --output-path "{json_path}"',
            check=False,
        )
        if json_path.exists():
            run(
                f'python scripts/parse_coverage.py --json "{json_path}"',
                check=False,
            )
        else:
            print("Warning: Could not generate coverage JSON.")
    else:
        print("[7/8] Running all tests...")
        nextest_cmd = "cargo nextest run --retries 2 -j 4"
        if args.include_llm:
            nextest_cmd += " --run-ignored all"
            print("=" * 60)
            print("NOTE: Including LLM tests. These contact the real OpenRouter API.")
            print("      Each LLM test takes 1-3 minutes. Total suite: ~3-9 minutes longer.")
            print("      Do not interrupt. Set your tool timeout to >= 600s.")
            print("=" * 60)
        run(nextest_cmd, check=False, env=test_env)
        if not args.include_llm:
            print(
                "    NOTE: 3 LLM tests were skipped. "
                "Run 'python build.py --llm-only' to execute them."
            )
        print("[8/8] Skipping coverage report (use --coverage to enable)")

    print("=== Build Complete ===")
    return 0


if __name__ == "__main__":
    sys.exit(main())
