"""Chronicler Engine healthcheck dispatcher.

Runs code-quality checks (duplicates detection, and future checks)
and prints LLM-consumable summaries.

Usage:
    python scripts/healthcheck.py                 # list available checks
    python scripts/healthcheck.py duplicates      # run duplicate-code check
    python scripts/healthcheck.py duplicates --top-pairs 25
    python scripts/healthcheck.py all            # run all checks

Exit codes:
    0  all checks ran
    1  a check failed (jscpd missing, parse error, etc.)
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
from collections import defaultdict
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path

WORKSPACE_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_REPORT = WORKSPACE_ROOT / "report" / "jscpd-report.json"


@dataclass
class CheckResult:
    name: str
    ok: bool
    message: str
    output_path: Path | None = None


CHECKS: dict[str, Callable[[argparse.Namespace], CheckResult]] = {}


def register(name: str):
    def deco(fn):
        CHECKS[name] = fn
        return fn

    return deco


def is_executable(path: str) -> bool:
    """Check if a file is executable on the current platform."""
    if sys.platform == "win32":
        return path.lower().endswith((".exe", ".cmd", ".bat"))
    try:
        return os.access(path, os.X_OK)
    except OSError:
        return False


def find_jscpd() -> tuple[str, list[str]] | None:
    """Discover jscpd binary or npx. Returns (binary, extra_args) or None."""
    found = shutil.which("jscpd")
    if found and is_executable(found):
        return (found, [])
    candidate = Path.home() / ".pi-lens" / "tools" / "node_modules" / ".bin" / "jscpd"
    if candidate.exists() and is_executable(str(candidate)):
        return (str(candidate), [])
    # Fallback: npx will fetch and run jscpd on demand
    npx = shutil.which("npx")
    if npx:
        return (npx, ["jscpd"])
    return None


def run_jscpd(report_path: Path, min_lines: int = 3, min_tokens: int = 30) -> tuple[bool, str]:
    """Run jscpd on chronicler_engine src/tests. Returns (ok, message)."""
    result = find_jscpd()
    if not result:
        return False, (
            "jscpd not found. Install with `npm install -g jscpd`, or "
            "ensure pi-lens tools are installed at ~/.pi-lens/tools/"
        )
    binary, extra_args = result

    report_path.parent.mkdir(parents=True, exist_ok=True)
    cmd = [
        binary,
        *extra_args,
        "--min-lines",
        str(min_lines),
        "--min-tokens",
        str(min_tokens),
        "--format",
        "rust",
        "--ignore",
        "target,report,tmp,docs/external_applications",
        "--output",
        str(report_path.parent),
        "--reporters",
        "json",
        str(WORKSPACE_ROOT),
    ]
    try:
        # npx.CMD on Windows requires shell=True to execute as a script
        use_shell = binary.lower().endswith(".cmd")
        result = subprocess.run(
            cmd,
            cwd=WORKSPACE_ROOT,
            capture_output=True,
            text=True,
            timeout=120,
            shell=use_shell,
        )
    except subprocess.TimeoutExpired:
        return False, "jscpd timed out after 120s"
    if result.returncode != 0:
        return False, f"jscpd failed: {result.stderr.strip()[:500]}"
    return True, result.stdout.strip()


def file_pair_key(a: str, b: str) -> tuple[str, str]:
    a_norm = a.replace("\\", "/")
    b_norm = b.replace("\\", "/")
    return (a_norm, b_norm) if a_norm <= b_norm else (b_norm, a_norm)


def dedupe_pair_overlaps(entries: list[dict]) -> list[dict]:
    """Drop overlapping sliding-window clones within a single file pair."""
    seen_ranges: dict[str, list[tuple[int, int]]] = defaultdict(list)
    unique: list[dict] = []
    for e in entries:
        fa = e["firstFile"]
        fb = e["secondFile"]
        a_overlaps = any(
            not (fa["end"] <= s or fa["start"] >= end) for s, end in seen_ranges[fa["name"]]
        )
        b_overlaps = any(
            not (fb["end"] <= s or fb["start"] >= end) for s, end in seen_ranges[fb["name"]]
        )
        if a_overlaps or b_overlaps:
            continue
        seen_ranges[fa["name"]].append((fa["start"], fa["end"]))
        seen_ranges[fb["name"]].append((fb["start"], fb["end"]))
        unique.append(e)
    return unique


@register("duplicates")
def check_duplicates(args: argparse.Namespace) -> CheckResult:
    """Run jscpd and summarize the top duplicate regions for LLM review."""
    report_path = Path(args.report) if args.report else DEFAULT_REPORT

    if not args.skip_run:
        ok, msg = run_jscpd(report_path, min_lines=args.jscpd_min_lines)
        if not ok:
            return CheckResult("duplicates", False, msg)
        if args.verbose:
            print(msg, file=sys.stderr)

    if not report_path.exists():
        return CheckResult(
            "duplicates",
            False,
            f"Report not found at {report_path}. Run without --skip-run.",
        )

    try:
        report = json.loads(report_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as e:
        return CheckResult(
            "duplicates",
            False,
            f"Failed to parse jscpd JSON report: {e}",
        )
    text = summarize_report(
        report,
        cross_file_only=not args.include_self,
        min_lines=args.min_lines,
        max_snippet=args.max_snippet,
        top_pairs=args.top_pairs,
        top_clusters=args.top_clusters,
    )

    out_path = None
    if args.out:
        out_path = Path(args.out)
        out_path.write_text(text, encoding="utf-8")
        print(f"Wrote {len(text)} bytes to {out_path}", file=sys.stderr)
    else:
        print(text)

    dup_count = len(report.get("duplicates", []))
    return CheckResult(
        "duplicates",
        True,
        f"{dup_count} raw clones analyzed",
        output_path=out_path,
    )


def summarize_report(
    report: dict,
    *,
    cross_file_only: bool = True,
    min_lines: int = 5,
    max_snippet: int = 200,
    top_pairs: int = 25,
    top_clusters: int = 3,
) -> str:
    """Convert jscpd report JSON to LLM-consumable prioritized text."""
    dups = [d for d in report["duplicates"] if d["lines"] >= min_lines]
    if cross_file_only:
        dups = [d for d in dups if d["firstFile"]["name"] != d["secondFile"]["name"]]

    by_pair: dict[tuple[str, str], list[dict]] = defaultdict(list)
    for d in dups:
        key = file_pair_key(d["firstFile"]["name"], d["secondFile"]["name"])
        by_pair[key].append(d)

    pair_stats: list[tuple[tuple[str, str], int, list[dict]]] = []
    for key, entries in by_pair.items():
        total_lines = sum(e["lines"] for e in entries)
        unique_entries = dedupe_pair_overlaps(entries)
        pair_stats.append((key, total_lines, unique_entries))

    pair_stats.sort(key=lambda x: -x[1])

    out: list[str] = []
    out.append("# Duplicate Summary")
    out.append("")
    out.append(
        f"- Total clone pairs: {len(dups)} "
        f"(filtered: min-lines={min_lines}, cross-file-only={cross_file_only})"
    )
    out.append(f"- Unique file pairs: {len(by_pair)}")
    out.append(f"- Showing top {min(top_pairs, len(pair_stats))} pairs by dup lines")
    out.append("")

    for pair_key, total_lines, entries in pair_stats[:top_pairs]:
        a, b = pair_key
        out.append(f"## {a}  <->  {b}")
        out.append("")
        out.append(f"Clones: {len(entries)} | Dup lines: {total_lines}")
        out.append("")
        entries_sorted = sorted(entries, key=lambda e: -e["lines"])[:top_clusters]
        for i, e in enumerate(entries_sorted, 1):
            fa = e["firstFile"]
            fb = e["secondFile"]
            out.append(f"### Clone {i} ({e['lines']}L)")
            out.append(f"- {fa['name'].replace(chr(92), '/')}:{fa['start']}-{fa['end']}")
            out.append(f"- {fb['name'].replace(chr(92), '/')}:{fb['start']}-{fb['end']}")
            snippet = e["fragment"].strip()
            if len(snippet) > max_snippet:
                snippet = snippet[:max_snippet] + "  ... [truncated]"
            out.append("")
            out.append("```rust")
            out.append(snippet)
            out.append("```")
            out.append("")
        out.append("")

    return "\n".join(out)


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description=__doc__)
    sub = p.add_subparsers(dest="check", metavar="CHECK")

    dup = sub.add_parser("duplicates", help="Run duplicate-code detection via jscpd")
    dup.add_argument("--report", type=str, default=None, help="Reuse existing jscpd JSON report")
    dup.add_argument(
        "--skip-run",
        action="store_true",
        help="Skip running jscpd, use existing report",
    )
    dup.add_argument("--min-lines", type=int, default=5, help="Min clone lines (summarize filter)")
    dup.add_argument("--jscpd-min-lines", type=int, default=3, help="Min lines passed to jscpd")
    dup.add_argument("--top-pairs", type=int, default=25)
    dup.add_argument("--top-clusters", type=int, default=3)
    dup.add_argument("--max-snippet", type=int, default=200)
    dup.add_argument("--include-self", action="store_true", help="Include same-file dups")
    dup.add_argument(
        "--out",
        type=str,
        default=None,
        help="Write summary to file instead of stdout",
    )
    dup.add_argument("--verbose", action="store_true")

    all_p = sub.add_parser("all", help="Run all available checks")
    all_p.add_argument("--verbose", action="store_true")
    return p


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()

    if args.check is None:
        print("Available checks:")
        for name in CHECKS:
            print(f"  {name}")
        print("\nUsage: python scripts/healthcheck.py <check> [options]")
        return 0

    if args.check == "all":
        results: list[CheckResult] = []
        for name, fn in CHECKS.items():
            print(f"== Running: {name} ==", file=sys.stderr)
            sub_args = argparse.Namespace(verbose=args.verbose)
            if name == "duplicates":
                for k, v in vars(build_parser().parse_args(["duplicates"])).items():
                    setattr(sub_args, k, v)
            results.append(fn(sub_args))
        print("\n== Summary ==", file=sys.stderr)
        for r in results:
            status = "OK" if r.ok else "FAIL"
            print(f"  [{status}] {r.name}: {r.message}", file=sys.stderr)
        return 0 if all(r.ok for r in results) else 1

    fn = CHECKS.get(args.check)
    if not fn:
        print(f"Unknown check: {args.check}", file=sys.stderr)
        print(f"Available: {', '.join(CHECKS.keys())}", file=sys.stderr)
        return 1

    result = fn(args)
    if not result.ok:
        print(f"[FAIL] {result.name}: {result.message}", file=sys.stderr)
        return 1
    if args.verbose:
        print(f"[OK] {result.name}: {result.message}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
