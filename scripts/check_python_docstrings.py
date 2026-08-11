"""Scan Python files in scripts/ and scripts/issue_tracker/ for missing module docstrings."""

from __future__ import annotations

import sys
from pathlib import Path


def check_python_file(filepath: Path) -> list[str]:
    violations = []

    try:
        content = filepath.read_text(encoding="utf-8")
        lines = content.splitlines()

        first_nonblank_idx = None
        for i, line in enumerate(lines):
            if line.strip():
                first_nonblank_idx = i
                break

        if first_nonblank_idx is None:
            violations.append(f"{filepath}: File is empty")
            return violations

        first_line = lines[first_nonblank_idx]

        if first_line.startswith("#!/usr/bin/env python3"):
            violations.append(
                f"{filepath}:{first_nonblank_idx + 1}: error: "
                f"Shebang found. Remove '#!/usr/bin/env python3' - "
                f"scripts are invoked via 'python script.py'"
)

        content_start_idx = first_nonblank_idx
        if first_line.startswith("#!/"):
            content_start_idx = first_nonblank_idx + 1
            while content_start_idx < len(lines) and not lines[content_start_idx].strip():
                content_start_idx += 1

        if content_start_idx >= len(lines):
            violations.append(
                f"{filepath}: warning: No module docstring found. "
                f'Add """Summary""" as first non-blank line after shebang removal'
            )
            return violations

        first_content = lines[content_start_idx].strip()

        if not (first_content.startswith('\"\"\"') or first_content.startswith("'''")):
            violations.append(
                f"{filepath}:{content_start_idx + 1}: warning: "
                f"Missing module docstring. "
                f'Add """One-line summary.""" as first non-blank line'
            )

    except Exception as e:
        violations.append(f"{filepath}: error: {e}")

    return violations


def scan_python_files(directories: list[Path]) -> tuple[int, int]:
    errors = 0
    warnings = 0

    for directory in directories:
        if not directory.exists():
            print(f"Skipping non-existent directory: {directory}")
            continue

        for filepath in directory.rglob("*.py"):
            violations = check_python_file(filepath)

            for violation in violations:
                print(violation)
                if ": error:" in violation:
                    errors += 1
                elif ": warning:" in violation:
                    warnings += 1

    return errors, warnings


def main() -> int:
    root = Path(__file__).parent.parent

    directories = [root / "scripts"]

    print(f"Scanning Python files in: {', '.join(str(d) for d in directories)}\n")

    errors, warnings = scan_python_files(directories)

    print(f"\n{'=' * 60}")
    print(f"Summary: {errors} error(s), {warnings} warning(s)")

    return 1 if errors > 0 else 0


if __name__ == "__main__":
    sys.exit(main())
