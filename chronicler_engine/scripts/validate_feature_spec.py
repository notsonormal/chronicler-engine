"""Validate that every scenario in a feature spec has a covering integration test and every annotated test references a declared scenario.

Discovers specs in `chronicler_engine/docs/specs/*.md` and walks the
HTTP-tier test directory (`chronicler_engine/tests/http/`) for
`// SCENARIO: X.Y` comments that appear immediately before a `#[test]`
(or `#[tokio::test]`) attribute. Per `tests/STRATEGY.md`, SCENARIO tags
live only in `tests/http/` — not in `src/` (unit tier) or
`tests/integration/` (the dissolved component tier).

Exit codes:
    0  all declared scenarios have at least one covering test and no orphans
    1  gaps (declared scenario with no test) or orphans (annotation with
       no matching declared scenario)
    2  parse error (missing dirs, unreadable files, no specs)

Run from anywhere:
    python chronicler_engine/scripts/validate_feature_spec.py
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ENGINE_ROOT = Path(__file__).parent.parent
SPECS_DIR = ENGINE_ROOT / "docs" / "specs"
# Per tests/STRATEGY.md: SCENARIO tags live in tests/http/ (HTTP E2E) and
# tests/browser/behaviour.rs (browser behaviour tests). Unit tier (src/)
# and driven-adapter tier (tests/integration/storage/) don't carry
# SCENARIO tags. tests/integration/ is the dissolved component tier.
# tests/browser/invariants.rs has a named exemption (no tags, no spec link).
TEST_DIRS = [
    ENGINE_ROOT / "tests" / "http",
    ENGINE_ROOT / "tests" / "browser",
]

# Heading-style scenario declarations: `#### Scenario 1.1: Title`.
# Captures the scenario ID (digits.digits).
SCENARIO_RE = re.compile(r"^#{1,6}\s+Scenario\s+(\d+\.\d+)\b")

# Comment annotations: `// [path/to/spec.md] SCENARIO: 1.1`.
# The bracketed spec path is mandatory — every annotation must link back
# to the spec it covers. Captures the scenario ID (digits.digits).
SCENARIO_COMMENT_RE = re.compile(r"^\s*//\s*\[[^\]]+\]\s*SCENARIO:\s*(\d+\.\d+)\s*$")

# `#[test]` or `#[tokio::test]` (any attr starting with `#[test`).
TEST_ATTR_RE = re.compile(r"^\s*#\[(tokio::)?test\b")

# How many lines ahead of a // SCENARIO: comment we'll look for a #[test]
# attribute before declaring the comment orphan.
COMMENT_LOOKAHEAD = 5


def parse_spec_scenarios(spec_path: Path) -> set[str]:
    """Return the set of scenario IDs declared in a spec file."""
    try:
        text = spec_path.read_text(encoding="utf-8")
    except OSError as exc:
        print(f"Error reading spec {spec_path}: {exc}", file=sys.stderr)
        sys.exit(2)

    scenarios: set[str] = set()
    for line in text.splitlines():
        m = SCENARIO_RE.match(line)
        if m:
            scenarios.add(m.group(1))
    return scenarios


def parse_test_annotations(
    test_path: Path,
) -> list[tuple[int, str]]:
    """Find `// SCENARIO: X.Y` comments paired with a following `#[test]`
    attribute. Returns list of (comment_line_number, scenario_id) for every
    comment that is followed (within COMMENT_LOOKAHEAD lines) by a
    `#[test]` attribute."""
    try:
        text = test_path.read_text(encoding="utf-8")
    except OSError as exc:
        print(f"Error reading test {test_path}: {exc}", file=sys.stderr)
        sys.exit(2)

    lines = text.splitlines()
    annotations: list[tuple[int, str]] = []

    for i, line in enumerate(lines):
        m = SCENARIO_COMMENT_RE.match(line)
        if not m:
            continue
        scenario_id = m.group(1)
        for j in range(i + 1, min(i + 1 + COMMENT_LOOKAHEAD, len(lines))):
            if TEST_ATTR_RE.match(lines[j]):
                annotations.append((i + 1, scenario_id))  # 1-based
                break

    return annotations


def main() -> int:
    if not SPECS_DIR.exists():
        print(f"Error: specs directory not found: {SPECS_DIR}", file=sys.stderr)
        return 2
    for test_dir in TEST_DIRS:
        if not test_dir.exists():
            rel = test_dir.relative_to(ENGINE_ROOT)
            print(f"Error: test directory not found: {rel}", file=sys.stderr)
            return 2

    spec_files = sorted(SPECS_DIR.glob("*.md"))
    if not spec_files:
        print(f"No spec files found in {SPECS_DIR}", file=sys.stderr)
        return 2

    declared: set[str] = set()
    for spec in spec_files:
        declared.update(parse_spec_scenarios(spec))

    covered: dict[str, list[tuple[Path, int]]] = {}
    orphans: list[tuple[Path, int, str]] = []

    test_files = sorted(
        f for test_dir in TEST_DIRS for f in test_dir.rglob("*.rs")
    )
    for test in test_files:
        for lineno, scenario_id in parse_test_annotations(test):
            if scenario_id in declared:
                covered.setdefault(scenario_id, []).append((test, lineno))
            else:
                orphans.append((test, lineno, scenario_id))

    gaps = sorted(declared - set(covered.keys()))
    declared_count = len(declared)
    covered_count = len(covered)
    gap_count = len(gaps)
    orphan_count = len(orphans)

    print(
        f"{declared_count} declared, {covered_count} covered, "
        f"{gap_count} gap(s), {orphan_count} orphan(s)"
    )

    if gaps:
        print("\nGaps (declared scenario has no covering test):")
        for sid in gaps:
            print(f"  {sid}")

    if orphans:
        print("\nOrphans (test annotation references undeclared scenario):")
        for path, lineno, sid in orphans:
            rel = path.relative_to(ENGINE_ROOT)
            print(f"  {rel}:{lineno}  {sid}")

    if gap_count > 0 or orphan_count > 0:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
