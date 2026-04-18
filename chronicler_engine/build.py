#!/usr/bin/env python3
"""Full build, validate, and test for Chronicler Engine."""

import subprocess
import sys
import os
import signal


def kill_port(port: int):
    """Kill any process using the specified port."""
    try:
        # Find process using the port
        result = subprocess.run(
            f"netstat -ano | findstr :{port}",
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


def run(cmd, cwd=None, check=True):
    """Run a command and handle output."""
    print(f"$ {cmd}")
    result = subprocess.run(cmd, shell=True, cwd=cwd or os.getcwd(), capture_output=True, text=True)
    if result.stdout:
        print(result.stdout)
    if result.stderr and "warning" not in result.stderr.lower():
        print(result.stderr, file=sys.stderr)
    if check and result.returncode != 0:
        print(f"FAILED with code {result.returncode}")
        sys.exit(result.returncode)
    return result.returncode


def main():
    print("=== Chronicler Engine Build ===")
    os.chdir(os.path.dirname(os.path.abspath(__file__)) or os.getcwd())

    # Kill any process on ports used by tests
    print("Checking for processes on test ports...")
    for port in [3000, 3001, 3002]:
        kill_port(port)

    # Also kill any lingering server processes by name
    print("Checking for lingering server processes...")
    kill_by_name("chronicler")

    print("[1/5] Checking formatting...")
    run("cargo fmt --check", check=False)

    print("[2/5] Running clippy...")
    run("cargo clippy -- -D warnings")

    print("[3/5] Building...")
    run("cargo build")

    print("[4/5] Running unit tests...")
    run("cargo test --lib")

    print("[5/5] Running integration tests...")
    # Run tests sequentially due to port conflicts between test binaries
    test_suites = [
        "component_tests",  # In-process tests (fast)
        "e2e_tests",        # Browser tests
        "flow_mock_tests",  # Mock LLM tests
    ]
    if os.environ.get("OPENROUTER_API_KEY"):
        test_suites.append("flow_llm_tests")

    for test_name in test_suites:
        run(f"cargo test --test {test_name}")

    print("[6/6] Running coverage check (full test suite)...")
    run("cargo llvm-cov test --text", check=False)
    # Note: server/* handlers and main.rs CLI will show 0% - they need different test approaches

    print("=== Build Complete ===")
    return 0


if __name__ == "__main__":
    sys.exit(main())
