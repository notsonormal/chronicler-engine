import re
import sys
from pathlib import Path


def check() -> int:
    src = Path("src")
    if not src.exists():
        print("ERROR: src/ directory not found.")
        return 1

    errors = []

    for rs_file in src.rglob("*.rs"):
        content = rs_file.read_text(encoding="utf-8")

        for match in re.finditer(r"#\[cfg\(test\)\]\s*mod\s+\w+\s*\{", content):
            errors.append(f"Inline test block found: {rs_file}")

    if errors:
        print("TEST STRUCTURE VIOLATIONS:")
        for e in errors:
            print(f"  {e}")
        print("\nAll unit tests must live in separate files. See AGENTS.md for the standard.")
        return 1

    print("Test structure OK.")
    return 0


if __name__ == "__main__":
    sys.exit(check())
