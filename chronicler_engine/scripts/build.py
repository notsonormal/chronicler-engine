#!/usr/bin/env python3
"""
Build script for Chronicler Engine.
Runs: build, clippy, tests, and code coverage.
"""

import subprocess
import sys
import os


def run_command(cmd, description):
    """Run a command and print the result."""
    print(f"\n{'=' * 60}")
    print(f"{description}")
    print(f"{'=' * 60}")
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    print(result.stdout)
    if result.stderr:
        print(result.stderr)
    if result.returncode != 0:
        print(f"FAILED: {description}")
        return False
    print(f"PASSED: {description}")
    return True


def kill_existing_process():
    """Search for and kill any existing Chronicler Engine process."""
    # Try to find and kill the server process on common ports
    ports = [3000]

    for port in ports:
        # Windows: use netstat to find process using the port
        result = subprocess.run(
            f"netstat -ano | findstr :{port}",
            shell=True,
            capture_output=True,
            text=True,
        )
        if result.stdout:
            # Extract PID from the netstat output
            # Format: TCP    0.0.0.0:port    0.0.0.0:0    LISTENING    PID
            for line in result.stdout.splitlines():
                parts = line.split()
                if len(parts) >= 5 and parts[-1].isdigit():
                    pid = parts[-1]
                    print(f"Found process on port {port} with PID {pid}, killing...")
                    try:
                        # Windows: kill by PID
                        subprocess.run(f"taskkill /F /PID {pid}", shell=True, capture_output=True)
                    except Exception as e:
                        print(f"Failed to kill process {pid}: {e}")

    # Also try to find and kill any chronicler_engine.exe process
    result = subprocess.run(
        "tasklist | findstr chronicler",
        shell=True,
        capture_output=True,
        text=True,
    )
    if result.stdout:
        for line in result.stdout.splitlines():
            parts = line.split()
            if parts:
                pid = parts[1]
                print(f"Found chronicler process with PID {pid}, killing...")
                try:
                    subprocess.run(f"taskkill /F /PID {pid}", shell=True, capture_output=True)
                except Exception as e:
                    print(f"Failed to kill process {pid}: {e}")


def main():
    os.chdir(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

    # Kill any existing Chronicler Engine process
    kill_existing_process()

    success = True

    # 1. Build
    success = run_command("cargo build", "Building application") and success

    # 2. Clippy
    success = run_command("cargo clippy -- -D warnings", "Running clippy linter") and success

    # 3. Tests
    success = run_command("cargo test", "Running all tests") and success

    # 4. Code Coverage
    # First, check if coverage tool is available
    result = subprocess.run(
        "cargo install cargo-llvm-cov 2>&1 || echo 'already installed'",
        shell=True,
        capture_output=True,
        text=True,
    )
    run_command("cargo llvm-cov --html", "Generating code coverage report")

    print(f"\n{'=' * 60}")
    print("BUILD PROCESS COMPLETE")
    print(f"{'=' * 60}")

    if success:
        print("All steps PASSED")
        return 0
    else:
        print("Some steps FAILED")
        return 1


if __name__ == "__main__":
    sys.exit(main())
