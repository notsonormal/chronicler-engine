"""Vale prose linter wrapper for Chronicler Engine docs.

Thin wrapper around `vale` (https://vale.sh) that:
  - Loads the chronicler_engine/.vale.ini config
  - Restricts scope to STANDARD docs (mirrors scripts/validate_docs.py)
  - Skips TRANSIENT (plans/, CHANGELOG.md) and EXCLUDED paths
  - Supports single-file, multi-file, --all, --json, and --fix modes

Vale itself is responsible for rule evaluation. This wrapper owns:
  - Scope (which files Vale sees)
  - CLI ergonomics (sensible defaults, multiple invocations)
  - Exit code (0 = clean, 1 = violations found, 2 = vale missing/errored)

Prerequisites:
  - `vale` on PATH (brew install vale)

Usage:
    python scripts/vale_lint.py                          # all STANDARD docs
    python scripts/vale_lint.py --all                    # explicit --all
    python scripts/vale_lint.py docs/system/action_pipeline.md
    python scripts/vale_lint.py docs/system/action_pipeline.md docs/system/llm_processing.md
    python scripts/vale_lint.py docs/system/            # whole directory
    python scripts/vale_lint.py --json                   # JSON output
    python scripts/vale_lint.py --json --quiet           # CI-friendly
    python scripts/vale_lint.py --fix docs/system/foo.md # auto-fix (RISKY on spec docs)

Exit codes:
    0 — no violations
    1 — violations found
    2 — vale not installed or vale errored
    3 — invalid arguments

Note: Vale reads config from chronicler_engine/.vale.ini via --config.
Styles live in chronicler_engine/styles/Chronicler/.
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from pathlib import Path

# --- Scope (mirrors validate_docs.py) ----------------------------------------

ENGINE_ROOT = Path(__file__).resolve().parent.parent
DOCS_ROOT = ENGINE_ROOT / "docs"
VALE_INI = ENGINE_ROOT / ".vale.ini"

# Transient — historical/forward-looking, no prose rules apply.
TRANSIENT_DIR_NAMES: set[str] = {"plans", "old-docs"}

# Excluded file names — auto-generated indexes, AGENTS meta.
# AGENTS.mdAUTO-INDEX block is auto-regenerated; prose preamble is owned by
# the per-edit gate, not Vale.
EXCLUDED_FILE_NAMES: set[str] = {
    "AGENTS.md",
    "README.md",
    "CHANGELOG.md",
}


# --- Scope filtering ---------------------------------------------------------


def is_in_scope(md_file: Path) -> bool:
    """Return True if md_file is a STANDARD doc (in scope for Vale).

    Mirrors validate_docs.py classify_file() — drops TRANSIENT (plans/,
    old-docs/, CHANGELOG.md) and EXCLUDED (auto-indexes, templates, AGENTS.md).
    """
    try:
        rel = md_file.resolve().relative_to(DOCS_ROOT.resolve())
    except ValueError:
        # Outside docs/ — Vale was invoked on an arbitrary file. Allow it.
        return True

    parts = rel.parts
    if not parts:
        return False

    if any(part in TRANSIENT_DIR_NAMES for part in parts):
        return False

    if parts[-1] in EXCLUDED_FILE_NAMES:
        return False

    return True


def collect_targets(paths: list[Path]) -> list[Path]:
    """Expand user-supplied paths to in-scope .md files.

    - File path  → kept if in scope
    - Directory  → rglob('*.md'), filtered by is_in_scope
    - --all path → DOCS_ROOT rglob, filtered
    """
    files: set[Path] = set()
    for target in paths:
        if target.is_file():
            if target.suffix == ".md" and is_in_scope(target):
                files.add(target.resolve())
            continue
        if target.is_dir():
            for md_file in target.rglob("*.md"):
                if is_in_scope(md_file):
                    files.add(md_file.resolve())
            continue
        print(f"WARNING: path not found: {target}", file=sys.stderr)
    return sorted(files)


# --- Vale invocation ---------------------------------------------------------


def find_vale() -> str | None:
    return shutil.which("vale")


def run_vale(
    files: list[Path],
    json_output: bool,
    auto_fix: bool,
    quiet: bool,
) -> tuple[int, str, str]:
    """Invoke vale with the engine config. Returns (rc, stdout, stderr)."""
    if not files:
        if quiet:
            return 0, "", ""
        return 0, "No in-scope .md files to lint.\n", ""

    cmd = [
        "vale",
        f"--config={VALE_INI}",
    ]
    if json_output:
        cmd.append("--output=JSON")
    if auto_fix:
        cmd.append("--mode=auto")
    cmd.extend(str(f) for f in files)

    proc = subprocess.run(
        cmd,
        cwd=ENGINE_ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    return proc.returncode, proc.stdout, proc.stderr


def print_text_report(stdout: str, stderr: str, rc: int, quiet: bool) -> None:
    if stderr.strip():
        print(stderr, file=sys.stderr)
    if quiet:
        # In quiet mode, Vale's compact output is the only thing to print.
        sys.stdout.write(stdout)
        return
    sys.stdout.write(stdout)


def print_summary(total_files: int, json_out: str | None) -> None:
    """Parse Vale JSON to print a human summary."""
    if not json_out:
        return
    try:
        data = json.loads(json_out)
    except json.JSONDecodeError:
        return
    total_alerts = 0
    for alerts in data.values():
        if isinstance(alerts, list):
            total_alerts += len(alerts)
    print(
        f"\n{total_alerts} alert(s) across {len(data)} file(s); {total_files} file(s) scanned.",
        file=sys.stderr,
    )


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Vale prose linter wrapper for Chronicler Engine docs.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )
    parser.add_argument(
        "paths",
        nargs="*",
        type=Path,
        help=(
            "Specific .md files or dirs to lint. "
            "If omitted, lints all in-scope STANDARD docs (mirror of "
            "validate_docs.py)."
        ),
    )
    parser.add_argument(
        "--all",
        action="store_true",
        help="Lint all in-scope STANDARD docs (default when no paths given).",
    )
    parser.add_argument(
        "--json",
        dest="json_output",
        action="store_true",
        help="Output Vale results as JSON (CI-friendly).",
    )
    parser.add_argument(
        "--quiet",
        action="store_true",
        help="Minimal output — Vale's compact text only, no headers.",
    )
    parser.add_argument(
        "--fix",
        action="store_true",
        help=(
            "Run Vale in auto-fix mode (--mode=auto). RISKY on spec docs — "
            "phrasing carries contract weight. Use only after manual review."
        ),
    )
    args = parser.parse_args()

    vale_bin = find_vale()
    if not vale_bin:
        print(
            "ERROR: vale not found on PATH.\n"
            "Install: brew install vale\n"
            "Docs:   https://vale.sh/docs",
            file=sys.stderr,
        )
        return 2

    # Validate config exists.
    if not VALE_INI.exists():
        print(f"ERROR: Vale config missing: {VALE_INI}", file=sys.stderr)
        return 2

    # Determine target paths.
    if args.paths:
        targets = collect_targets(args.paths)
        # Normalize: if user passed a path that resolved to nothing, fail loud.
        if not targets and not any(p.exists() for p in args.paths):
            print(
                f"ERROR: no .md files found in: {[str(p) for p in args.paths]}",
                file=sys.stderr,
            )
            return 3
    else:
        # Default: all STANDARD docs.
        targets = collect_targets([DOCS_ROOT])

    if not args.quiet:
        print(f"Vale: {vale_bin}", file=sys.stderr)
        print(f"Config: {VALE_INI.relative_to(ENGINE_ROOT)}", file=sys.stderr)
        print(f"Files: {len(targets)} in scope", file=sys.stderr)
        print("-" * 60, file=sys.stderr)

    rc, stdout, stderr = run_vale(
        files=targets,
        json_output=args.json_output,
        auto_fix=args.fix,
        quiet=args.quiet,
    )

    print_text_report(stdout, stderr, rc, args.quiet)

    if args.json_output:
        print_summary(len(targets), stdout)

    # Vale rc: 0 = clean, 1 = violations, 2+ = error
    if rc >= 2:
        return 2
    return 1 if rc == 1 else 0


if __name__ == "__main__":
    sys.exit(main())
