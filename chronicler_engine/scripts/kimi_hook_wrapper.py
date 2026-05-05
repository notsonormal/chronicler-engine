#!/usr/bin/env python3
"""Project-aware Kimi Code CLI hook wrapper for generate_docs_index.py.

Reads hook context JSON from stdin, checks if the session is inside the
chronicler_engine project, and conditionally runs the docs index generator.

Usage in ~/.kimi/config.toml:

    [[hooks]]
    event = "SessionStart"
    command = "python /absolute/path/to/chronicler_engine/scripts/kimi_hook_wrapper.py"
    timeout = 10
"""

import json
import subprocess
import sys
from pathlib import Path

# The directory that must appear in the session cwd for the hook to fire.
PROJECT_MARKER = "chronicler_engine"


def get_project_root() -> Path:
    """Return the absolute path to the chronicler_engine directory."""
    return Path(__file__).resolve().parent.parent


def should_run(cwd: str) -> bool:
    """Check if the session cwd is inside this project."""
    return PROJECT_MARKER in Path(cwd).resolve().parts


def run_generator() -> int:
    """Run the docs index generator."""
    script = get_project_root() / "scripts" / "generate_docs_index.py"
    result = subprocess.run([sys.executable, str(script)], capture_output=True, text=True)
    if result.stdout:
        print(result.stdout, end="")
    if result.stderr:
        print(result.stderr, end="", file=sys.stderr)
    return result.returncode


def main() -> int:
    try:
        data = json.load(sys.stdin)
    except json.JSONDecodeError:
        # Fail-open: if we can't parse stdin, do nothing.
        return 0

    cwd = data.get("cwd", "")
    if not cwd:
        return 0

    if should_run(cwd):
        return run_generator()

    return 0


if __name__ == "__main__":
    sys.exit(main())
