"""Parse coverage report from cargo-llvm-cov JSON output.

Usage:
    python scripts/parse_coverage.py
    # Or with custom path:
    python scripts/parse_coverage.py --json target/llvm-cov/coverage.json
"""

import argparse
import json
import os
import sys


def parse_args():
    parser = argparse.ArgumentParser(description="Parse cargo-llvm-cov coverage report")
    parser.add_argument(
        "--json",
        type=str,
        default=None,
        help="Path to JSON coverage file (default: target/llvm-cov/coverage.json)",
    )
    parser.add_argument(
        "--threshold",
        type=int,
        default=80,
        help="Coverage threshold for warnings (default: 80)",
    )
    parser.add_argument(
        "--show-all",
        action="store_true",
        help="Show all files, not just those below threshold",
    )
    parser.add_argument(
        "--ignore-regex",
        type=str,
        default=r"main\.rs|server[/\\]mod\.rs|server[/\\]fragments\.rs|openrouter_client\.rs",
        help="Regex pattern for files to exclude from coverage check",
    )
    return parser.parse_args()


def find_json_file(default_path: str) -> str | None:
    """Find the JSON coverage file."""
    if os.path.exists(default_path):
        return default_path

    alt_paths = [
        "coverage.json",
        "target/llvm-cov/coverage.json",
        "target/llvm-cov/codecov.json",
    ]

    for path in alt_paths:
        if os.path.exists(path):
            return path

    return None


def parse_coverage_report(
    json_path: str, threshold: int, show_all: bool, ignore_regex: str
) -> dict:
    """Parse the coverage JSON and return summary data."""
    import re

    with open(json_path) as f:
        data = json.load(f)

    totals = data["data"][0]["totals"]["lines"]

    result = {
        "total_covered": totals["covered"],
        "total_count": totals["count"],
        "total_percent": totals["percent"],
        "files": [],
    }

    files = data["data"][0]["files"]
    ignore_pattern = re.compile(ignore_regex) if ignore_regex else None

    for f in files:
        filename = f["filename"]
        if ignore_pattern and ignore_pattern.search(filename):
            continue

        short_name = filename
        for sep in ["chronicler_engine/", "chronicler_engine\\"]:
            if sep in short_name:
                short_name = short_name.split(sep)[-1]
                break

        summary = f.get("summary", {})
        line_info = summary.get("lines", {})
        covered = line_info.get("covered", 0)
        total = line_info.get("count", 0)

        if total > 0:
            pct = (covered / total) * 100
            result["files"].append(
                {
                    "name": short_name,
                    "full_path": filename,
                    "covered": covered,
                    "total": total,
                    "percent": pct,
                    "below_threshold": pct < threshold,
                }
            )

    result["files"].sort(key=lambda x: x["percent"])

    return result


def print_coverage_report(report: dict, threshold: int, show_all: bool):
    """Print a coverage report."""
    total = report["total_percent"]
    covered = report["total_covered"]
    count = report["total_count"]

    print("=" * 60)
    print("COVERAGE REPORT")
    print("=" * 60)
    print(f"Total Lines: {covered}/{count} = {total:.1f}%")

    if total >= threshold:
        print(f"[OK] Coverage meets threshold ({threshold}%)")
    else:
        print(f"[LOW] Coverage below threshold ({threshold}%)")

    print("=" * 60)

    files_to_show = (
        report["files"] if show_all else [f for f in report["files"] if f["below_threshold"]]
    )

    if not files_to_show:
        print("\nAll files meet coverage threshold!")
        return

    print(f"\nFiles {'below' if not show_all else 'at/below'} {threshold}% coverage:")
    print("-" * 60)

    for f in files_to_show:
        status = "LOW" if f["below_threshold"] else "OK "
        print(f"[{status:3}] {f['name']:45} {f['covered']:4}/{f['total']:4} = {f['percent']:5.1f}%")


def main():
    args = parse_args()

    json_path = args.json
    if json_path is None:
        json_path = find_json_file("target/llvm-cov/coverage.json")

    if json_path is None or not os.path.exists(json_path):
        print("Error: Could not find coverage JSON file.", file=sys.stderr)
        print(
            "Run: cargo llvm-cov nextest --json --output-path target/llvm-cov/coverage.json",
            file=sys.stderr,
        )
        sys.exit(1)

    report = parse_coverage_report(json_path, args.threshold, args.show_all, args.ignore_regex)
    print_coverage_report(report, args.threshold, args.show_all)

    if report["total_percent"] < args.threshold:
        sys.exit(1)


if __name__ == "__main__":
    main()
