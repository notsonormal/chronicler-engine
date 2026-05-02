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
        "--verbose",
        action="store_true",
        help="Show full per-test output and coverage table",
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
    run("cargo clippy -- -D warnings")

    print("[3/8] Running architecture guardrail tests...")
    run("cargo test --test architecture")

    print("[4/8] Running custom guardrails tests...")
    run("cargo test --test guardrails")

    print("[5/8] Building...")
    run("cargo build")

    print("[6/8] Copying data and assets for deployment...")
    release_dir = Path("target/release")
    release_dir.mkdir(exist_ok=True)

    # Copy data folder (worlds, images, etc.)
    if Path("data").exists():
        dest_data = release_dir / "data"
        if dest_data.exists():
            shutil.rmtree(dest_data)
        shutil.copytree("data", dest_data)
        print(f"  Copied data/ -> {dest_data}")


    # Copy assets folder (HTML, CSS, etc.)
    if Path("assets").exists():
        dest_assets = release_dir / "assets"
        if dest_assets.exists():
            shutil.rmtree(dest_assets)
        shutil.copytree("assets", dest_assets)
        print(f"  Copied assets/ -> {dest_assets}")

    # Create logs directory
    (release_dir / "logs").mkdir(exist_ok=True)
    print("  Created logs/")

    print(f"  Release package ready in {release_dir}/")
    print("  Deployment: copy target/release/ folder to your target machine")

    print("[7/8] Running all tests with coverage...")
    # Suppress per-test PASS lines unless --verbose is set
    test_env = {}
    if not args.verbose:
        test_env["NEXTEST_STATUS_LEVEL"] = "fail"

    # Run tests via nextest with coverage collection (single pass)
    # Do NOT exclude anything - run all tests including main.rs
    run(
        "cargo llvm-cov nextest --no-report --retries 2 -j 4",
        check=False,
        env=test_env,
    )

    print("[8/8] Generating coverage report...")
    if args.verbose:
        # Full table for verbose mode
        result = subprocess.run(
            'cargo llvm-cov report --summary-only --ignore-filename-regex "main.rs|server/mod.rs|server/fragments.rs|openrouter_client.rs"',
            shell=True,
            cwd=os.getcwd(),
            capture_output=True,
            text=True,
        )
        if result.stdout:
            print(result.stdout)
        if result.returncode != 0:
            print(f"Coverage check exited with code {result.returncode}")
    else:
        # Concise summary via JSON + parse_coverage.py
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

    print("=== Build Complete ===")
    return 0


if __name__ == "__main__":
    sys.exit(main())
