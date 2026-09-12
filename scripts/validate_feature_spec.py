"""Validate that every scenario in a feature spec has a covering integration test and every annotated test references a declared scenario.

It also enforces the SCENARIO-tag rules of `tests/STRATEGY.md`
("SCENARIO tags"): every test under `tests/http/` and `tests/browser/`
carries a `// [spec] SCENARIO: X.Y` tag, unless an exemption constant
below spares it, and tags match their observation surface: `browser_*.md`
specs are tagged only from `tests/browser/`, and non-`browser_*` specs are
never tagged from `tests/browser/`. `tests/http/requires_migration/` is the
legacy quarantine: untagged by design, count-pinned by
`REQUIRES_MIGRATION_TEST_COUNT` (the count may only go down).

Exit codes:
    0  all declared scenarios covered, tag rule satisfied
    1  gaps (declared scenario with no test), orphans (annotation with no
       matching declared scenario), untagged tests, surface mismatches, or
       a quarantine count above the pin
    2  parse error (missing dirs, unreadable files, no specs)

Run from anywhere:
    python scripts/validate_feature_spec.py
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ENGINE_ROOT = Path(__file__).parent.parent
SPECS_DIR = ENGINE_ROOT / "docs" / "specs"
# Per tests/STRATEGY.md, SCENARIO tags live in these two directories. The
# unit tier (src/) and the driven-adapter tier (tests/storage/) carry none.
TEST_DIRS = [
    ENGINE_ROOT / "tests" / "http",
    ENGINE_ROOT / "tests" / "browser",
]

# Heading-style scenario declarations: `#### Scenario 1.1: Title`.
# Captures the scenario ID (digits.digits).
SCENARIO_RE = re.compile(r"^#{1,6}\s+Scenario\s+(\d+\.\d+)\b")

SCENARIO_COMMENT_RE = re.compile(
    r"^\s*//\s*\[([^\]]+)\]\s*SCENARIO:\s*(\d+\.\d+)\s*$"
)

# `#[test]` or `#[tokio::test]` (any attr starting with `#[test`).
TEST_ATTR_RE = re.compile(r"^\s*#\[(tokio::)?test\b")

# How many lines ahead of a // SCENARIO: comment we'll look for a #[test]
# attribute before declaring the comment orphan.
COMMENT_LOOKAHEAD = 5

# ---------------------------------------------------------------------------
# Mandatory SCENARIO-tag rule (contract: tests/STRATEGY.md "SCENARIO tags").
#
# Every #[test] / #[tokio::test] under tests/http/ and tests/browser/ must
# carry a `// [spec] SCENARIO: N.N` tag. Each exemption below is declared
# with its reason. The surface-consistency rule (same STRATEGY.md section)
# is checked in find_surface_violations: `browser_*.md` specs are tagged
# only from tests/browser/, and non-`browser_*` specs never from
# tests/browser/.

# Directories exempt from the tag rule (matched by path prefix).
TAG_EXEMPT_DIRS = {
    Path("tests/http/requires_migration"): (
        "legacy quarantine — untagged e2e tests pending spec migration; "
        "count-pinned via REQUIRES_MIGRATION_TEST_COUNT"
    ),
}

# Whole files exempt from the tag rule.
TAG_EXEMPT_FILES = {
    Path("tests/browser/invariants.rs"): (
        "named exemption in tests/STRATEGY.md — test code is the definition"
    ),
}

# Individual tests exempt by (file, fn name).
TAG_EXEMPT_TESTS = {
    (
        Path("tests/browser/dashboard.rs"),
        "test_engine_output_teed_to_file",
    ): (
        "infrastructure health check (engine stdout tee), not a spec scenario"
    ),
}

# Pins the requires_migration quarantine: the count may only go down.
# Migration cleanups lower it deliberately; a new test in the folder fails
# the gate.
REQUIRES_MIGRATION_TEST_COUNT = 86


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
) -> list[tuple[int, int, str, str]]:
    """Find `// [spec] SCENARIO: X.Y` comments paired with a following
    `#[test]` attribute. Returns list of (comment_line_number,
    attribute_line_number, spec_path, scenario_id) for every comment that
    is followed (within COMMENT_LOOKAHEAD lines) by a `#[test]`
    attribute."""
    try:
        text = test_path.read_text(encoding="utf-8")
    except OSError as exc:
        print(f"Error reading test {test_path}: {exc}", file=sys.stderr)
        sys.exit(2)

    lines = text.splitlines()
    annotations: list[tuple[int, int, str, str]] = []

    for i, line in enumerate(lines):
        m = SCENARIO_COMMENT_RE.match(line)
        if not m:
            continue
        spec_path, scenario_id = m.group(1), m.group(2)
        for j in range(i + 1, min(i + 1 + COMMENT_LOOKAHEAD, len(lines))):
            if TEST_ATTR_RE.match(lines[j]):
                annotations.append((i + 1, j + 1, spec_path, scenario_id))
                break

    return annotations


FN_NAME_RE = re.compile(r"^\s*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)")


def find_test_fn_name(lines: list[str], attr_index: int) -> str:
    """Best-effort fn name for the test attribute at 0-based `attr_index`:
    the first `fn <name>` line within five lines after the attribute."""
    for line in lines[attr_index + 1 : attr_index + 6]:
        m = FN_NAME_RE.match(line)
        if m:
            return m.group(1)
    return "<unnamed>"


def is_tag_exempt(rel: Path) -> bool:
    """True when the SCENARIO-tag rule does not apply to `rel` (a path
    relative to ENGINE_ROOT), per the declared exemptions."""
    if rel in TAG_EXEMPT_FILES:
        return True
    return any(rel.is_relative_to(d) for d in TAG_EXEMPT_DIRS)


def find_untagged_tests(
    test_files: list[Path],
) -> list[tuple[Path, int, str]]:
    """Return (path, attr_line, fn name) for every test attribute in a
    non-exempt file that carries no SCENARIO tag and is not individually
    exempt."""
    violations: list[tuple[Path, int, str]] = []
    for path in test_files:
        rel = path.relative_to(ENGINE_ROOT)
        if is_tag_exempt(rel):
            continue
        try:
            lines = path.read_text(encoding="utf-8").splitlines()
        except OSError as exc:
            print(f"Error reading test {path}: {exc}", file=sys.stderr)
            sys.exit(2)
        tagged = {attr for _, attr, _, _ in parse_test_annotations(path)}
        for i, line in enumerate(lines):
            if not TEST_ATTR_RE.match(line):
                continue
            if (i + 1) in tagged:
                continue
            fn_name = find_test_fn_name(lines, i)
            if (rel, fn_name) in TAG_EXEMPT_TESTS:
                continue
            violations.append((path, i + 1, fn_name))
    return violations


def find_surface_violations(
    test_files: list[Path],
) -> list[tuple[Path, int, str, str]]:
    """Return (path, line, spec_path, reason) for every SCENARIO tag whose
    observation surface does not match its test directory: `browser_*.md`
    specs must be tagged only from `tests/browser/`, and non-`browser_*`
    specs never from `tests/browser/` (contract: tests/STRATEGY.md
    "SCENARIO tags", per-surface rule)."""
    violations: list[tuple[Path, int, str, str]] = []
    browser_dir = Path("tests/browser")
    for path in test_files:
        rel = path.relative_to(ENGINE_ROOT)
        in_browser = rel.is_relative_to(browser_dir)
        for _cmt, _attr, spec_path, _sid in parse_test_annotations(path):
            is_browser_spec = Path(spec_path).name.startswith("browser_")
            if is_browser_spec and not in_browser:
                violations.append(
                    (
                        path,
                        _cmt,
                        spec_path,
                        "browser_* spec tagged outside tests/browser/",
                    )
                )
            elif not is_browser_spec and in_browser:
                violations.append(
                    (
                        path,
                        _cmt,
                        spec_path,
                        "non-browser_* spec tagged from tests/browser/",
                    )
                )
    return violations


def count_quarantine_tests() -> int:
    """Count test attributes in the requires_migration quarantine."""
    total = 0
    for path in sorted(
        (ENGINE_ROOT / "tests" / "http" / "requires_migration").rglob("*.rs")
    ):
        try:
            text = path.read_text(encoding="utf-8")
        except OSError as exc:
            print(f"Error reading test {path}: {exc}", file=sys.stderr)
            sys.exit(2)
        total += sum(1 for line in text.splitlines() if TEST_ATTR_RE.match(line))
    return total


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

    # Coverage is keyed by (spec_path, scenario_id) so the same ID declared
    # in two specs is tracked as two distinct scenarios. The duplicate-ID
    # check is no longer needed: colliding IDs in different specs are simply
    # two keys that must each be covered.
    declared: set[tuple[str, str]] = set()
    for spec in spec_files:
        spec_rel = str(spec.relative_to(ENGINE_ROOT))
        for sid in parse_spec_scenarios(spec):
            declared.add((spec_rel, sid))

    covered: dict[tuple[str, str], list[tuple[Path, int]]] = {}
    orphans: list[tuple[Path, int, str, str]] = []

    test_files = sorted(
        f for test_dir in TEST_DIRS for f in test_dir.rglob("*.rs")
    )
    for test in test_files:
        for _cmt, _attr, spec_path, scenario_id in parse_test_annotations(test):
            pair = (spec_path, scenario_id)
            if pair in declared:
                covered.setdefault(pair, []).append((test, _cmt))
            else:
                orphans.append((test, _cmt, spec_path, scenario_id))

    gaps = sorted(declared - set(covered.keys()))
    declared_count = len(declared)
    covered_count = len(covered)
    gap_count = len(gaps)
    orphan_count = len(orphans)
    untagged = find_untagged_tests(test_files)
    surface = find_surface_violations(test_files)
    quarantine = count_quarantine_tests()
    ratchet_exceeded = quarantine > REQUIRES_MIGRATION_TEST_COUNT

    print(
        f"{declared_count} declared, {covered_count} covered, "
        f"{gap_count} gap(s), {orphan_count} orphan(s), "
        f"{len(untagged)} untagged, {len(surface)} surface mismatch(es), "
        f"quarantine {quarantine}/{REQUIRES_MIGRATION_TEST_COUNT}"
    )

    if gaps:
        print("\nGaps (declared scenario has no covering test):")
        for spec_rel, sid in gaps:
            print(f"  {spec_rel}  {sid}")

    if orphans:
        print("\nOrphans (test annotation references undeclared scenario):")
        for path, lineno, spec_path, sid in orphans:
            rel = path.relative_to(ENGINE_ROOT)
            print(f"  {rel}:{lineno}  [{spec_path}] {sid}")

    if untagged:
        print(
            "\nUntagged (tag-rule test carries no SCENARIO tag; see "
            "TAG_EXEMPT_* in this script):"
        )
        for path, lineno, fn_name in untagged:
            rel = path.relative_to(ENGINE_ROOT)
            print(f"  {rel}:{lineno}  {fn_name}")

    if surface:
        print(
            "\nSurface mismatches (tag's spec surface does not match its "
            "test directory; see STRATEGY.md \"SCENARIO tags\"):")
        for path, lineno, spec_path, reason in surface:
            rel = path.relative_to(ENGINE_ROOT)
            print(f"  {rel}:{lineno}  [{spec_path}]  {reason}")

    if ratchet_exceeded:
        print(
            f"\nQuarantine count exceeded: {quarantine} untagged tests in "
            f"tests/http/requires_migration/ > {REQUIRES_MIGRATION_TEST_COUNT} "
            "pinned. New tests must carry SCENARIO tags; migration cleanups "
            "lower the constant in this script."
        )

    if (
        gap_count > 0
        or orphan_count > 0
        or untagged
        or surface
        or ratchet_exceeded
    ):
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
