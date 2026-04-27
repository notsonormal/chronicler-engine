#!/usr/bin/env python3
"""Full build, validate, and test for Chronicler Engine.

Uses cargo-nextest for parallel test execution.
"""

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


def run(cmd, cwd=None, check=True, show_output=True):
    """Run a command and handle output."""
    print(f"$ {cmd}")
    # Use live output for nextest (real-time progress), capture for others
    if show_output:
        result = subprocess.run(cmd, shell=True, cwd=cwd or os.getcwd())
        if check and result.returncode != 0:
            print(f"FAILED with code {result.returncode}")
            sys.exit(result.returncode)
        return result.returncode
    else:
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

    # Clean up stale port lock files from crashed test runs
    import tempfile
    lock_dir = Path(tempfile.gettempdir()) / "chronicler_test_ports"
    if lock_dir.exists():
        print(f"Cleaning stale port locks from {lock_dir}...")
        shutil.rmtree(lock_dir)

    print("[1/5] Formatting...")
    run("cargo fmt")

    print("[2/5] Running clippy...")
    run("cargo clippy -- -D warnings")

    print("[3/5] Building...")
    run("cargo build")

    print("[4/6] Copying data and assets for deployment...")
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

    print("[5/6] Running all tests with coverage...")
    # Run tests via nextest with coverage collection (single pass)
    # Do NOT exclude anything - run all tests including main.rs
    run("cargo llvm-cov nextest --no-report --retries 2 -j 4", check=False)

    print("[6/6] Generating coverage report...")
    # Exclude from coverage math:
    # - main.rs: CLI entry point, hard to unit test
    # - server/mod.rs: async server runtime, only runs when server starts
    # - server/fragments.rs: async handlers, hard to unit test
    # - narrative/openrouter_client.rs: HTTP client, requires external API
    result = subprocess.run(
        "cargo llvm-cov report --summary-only --ignore-filename-regex 'main.rs|server/mod.rs|server/fragments.rs|openrouter_client.rs'",
        shell=True,
        cwd=os.getcwd(),
        capture_output=True,
        text=True,
    )
    if result.stdout:
        print(result.stdout)
    if result.returncode != 0:
        print(f"Coverage check exited with code {result.returncode}")

    print("=== Build Complete ===")
    return 0


if __name__ == "__main__":
    sys.exit(main())
