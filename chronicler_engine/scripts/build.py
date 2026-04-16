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


def main():
    os.chdir(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

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
