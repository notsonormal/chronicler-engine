"""Validate the seam contracts: every declared `CASE` has exactly one tagged test.

A **seam contract** is the test file of a black box that sits behind a port
boundary: the storage adapters, the LLM providers and transport, the Harper
text checker, and each Agent. Its clauses are declared as
`CASE`s because no route reaches them, so `validate_feature_spec.py` (which
pairs `docs/specs/` scenarios with tests) cannot cover them. The rule that
places a test is `tests/STRATEGY.md`, "The one rule".

The form is strict, and it lives in the box's test file:

    //! CASE <id>: <what the case pins>
    //! CASE <id> [<label>]: <what the case pins>

`<id>` matches `[a-z0-9][a-z0-9-]*` and identifies one case in the whole
repository; the optional `[<label>]` is a short kind such as `[failure]`.

    // CASE: <id>
    #[test]
    fn test_<behaviour>() { /* ... */ }

The tag sits on the comment line directly above the test attribute (blank
lines and other `//` comment lines may sit in between, up to `TAG_LOOKAHEAD`),
and the anchor is a test attribute (`#[test]`, `#[tokio::test]`, `#[rstest]`)
or a `proptest! {` block.

Scanned trees: every `src/**/*_tests.rs` file, and every file under
`tests/storage/` (the driven-adapter tier holds the storage contract).

The checker fails on:

* a declared case with no tagged test;
* a declared case with more than one tagged test;
* a tag with no declared case in its own file;
* a test in a declaring file with no tag;
* a duplicate id;
* a tag with no test attribute after it.

Exit codes:
    0  every declared case has its tagged test
    1  a violation above
    2  a scanned tree is missing

Run from anywhere:
    python scripts/validate_seam_cases.py
"""

from __future__ import annotations

import re
import sys
from pathlib import Path
from typing import NamedTuple

ENGINE_ROOT = Path(__file__).parent.parent
SRC_DIR = ENGINE_ROOT / "src"
STORAGE_DIR = ENGINE_ROOT / "tests" / "storage"

CASE_DECL_RE = re.compile(
    r"^//!\s*CASE\s+([a-z0-9][a-z0-9-]*)\s*(?:\[[^\]]*\])?\s*:\s*\S"
)
CASE_TAG_RE = re.compile(r"^\s*//\s*CASE:\s*([a-z0-9][a-z0-9-]*)\s*$")
TEST_ATTR_RE = re.compile(r"^\s*#\[(?:tokio::)?test\b|^\s*#\[rstest\]|^\s*proptest!\s*\{")

TAG_LOOKAHEAD = 5


class CaseDeclaration(NamedTuple):
    line: int
    case_id: str


class CaseTag(NamedTuple):
    """A `// CASE: <id>` tag; ``anchor`` is 0 when no test line follows."""

    line: int
    case_id: str
    anchor: int


def parse_declarations(text: str) -> list[CaseDeclaration]:
    declarations: list[CaseDeclaration] = []
    for index, line in enumerate(text.splitlines(), start=1):
        match = CASE_DECL_RE.match(line)
        if match:
            declarations.append(CaseDeclaration(index, match.group(1)))
    return declarations


def parse_tags(text: str) -> list[CaseTag]:
    lines = text.splitlines()
    tags: list[CaseTag] = []
    for index, line in enumerate(lines, start=1):
        match = CASE_TAG_RE.match(line)
        if not match:
            continue
        anchor = 0
        for probe in range(index, min(index + TAG_LOOKAHEAD, len(lines))):
            if TEST_ATTR_RE.match(lines[probe]):
                anchor = probe + 1
                break
        tags.append(CaseTag(index, match.group(1), anchor))
    return tags


def parse_test_lines(text: str) -> list[int]:
    """1-based lines carrying a test attribute or a `proptest!` block."""
    return [
        index
        for index, line in enumerate(text.splitlines(), start=1)
        if TEST_ATTR_RE.match(line)
    ]


def declaring_files(entries: dict[Path, str]) -> dict[Path, list[CaseDeclaration]]:
    declared: dict[Path, list[CaseDeclaration]] = {}
    for path, text in entries.items():
        declarations = parse_declarations(text)
        if declarations:
            declared[path] = declarations
    return declared


def validate(entries: dict[Path, str]) -> list[str]:
    """Return one message per violation of the seam-contract rule, sorted."""
    errors: list[str] = []
    declared = declaring_files(entries)

    seen: dict[str, Path] = {}
    for path, declarations in declared.items():
        for declaration in declarations:
            first = seen.get(declaration.case_id)
            if first is not None:
                errors.append(
                    f"{path}:{declaration.line}: duplicate case id "
                    f"`{declaration.case_id}` (already declared in {first})"
                )
                continue
            seen[declaration.case_id] = path

    for path, text in entries.items():
        declarations = declared.get(path)
        tags = parse_tags(text)
        declared_ids = {d.case_id for d in declarations} if declarations else set()

        for tag in tags:
            if tag.anchor == 0:
                errors.append(
                    f"{path}:{tag.line}: `// CASE: {tag.case_id}` has no test "
                    f"attribute within {TAG_LOOKAHEAD} lines"
                )
            if tag.case_id not in declared_ids:
                where = "" if declarations else " (the file declares no case)"
                errors.append(
                    f"{path}:{tag.line}: tag `{tag.case_id}` has no declared case"
                    f" in this file{where}"
                )

        if declarations is None:
            continue

        tagged: dict[str, list[int]] = {}
        for tag in tags:
            if tag.anchor:
                tagged.setdefault(tag.case_id, []).append(tag.line)
        for declaration in declarations:
            lines = tagged.get(declaration.case_id, [])
            if not lines:
                errors.append(
                    f"{path}:{declaration.line}: declared case "
                    f"`{declaration.case_id}` has no tagged test"
                )
            elif len(lines) > 1:
                errors.append(
                    f"{path}:{declaration.line}: declared case "
                    f"`{declaration.case_id}` has more than one tagged test "
                    f"(lines {', '.join(str(line) for line in lines)}): one test "
                    f"pins one case"
                )

        anchored = {tag.anchor for tag in tags if tag.anchor}
        for line in parse_test_lines(text):
            if line not in anchored:
                errors.append(
                    f"{path}:{line}: test in a declaring file has no "
                    f"`// CASE:` tag"
                )

    return sorted(errors)


def collect_entries() -> dict[Path, str]:
    """Paths in the result are relative to the repo root."""
    entries: dict[Path, str] = {}
    for path in sorted(SRC_DIR.rglob("*_tests.rs")):
        entries[path.relative_to(ENGINE_ROOT)] = path.read_text(encoding="utf-8")
    for path in sorted(STORAGE_DIR.rglob("*.rs")):
        entries[path.relative_to(ENGINE_ROOT)] = path.read_text(encoding="utf-8")
    return entries


def main() -> int:
    for tree in (SRC_DIR, STORAGE_DIR):
        if not tree.is_dir():
            print(f"error: scanned tree {tree} is missing", file=sys.stderr)
            return 2

    errors = validate(collect_entries())
    if errors:
        print("SEAM-CASE VIOLATIONS:")
        for message in errors:
            print(f"  {message}")
        print(
            "\nA seam contract declares its cases in the box's test file and tags "
            "each test with the case it covers. See tests/STRATEGY.md, "
            '"The one rule".'
        )
        return 1

    print("Seam cases OK.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
