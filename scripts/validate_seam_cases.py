"""Validate the seam contracts: every test in a declaring file carries one `CASE` tag.

A **seam contract** is the test file of a black box that sits behind a port
boundary: the storage adapters, the LLM providers and transport, the Harper
text checker, each Agent, and the Narrator. Its clauses are declared as
`CASE`s because no route reaches them, so `validate_feature_spec.py` (which
pairs `docs/specs/` scenarios with tests) cannot cover them. The rule that
places a test is `tests/STRATEGY.md`, "The one rule".

The form is strict, and the tag carries the description. The tag sits on the
comment line directly above the test:

    // CASE <id>: <what the case pins>
    // CASE <id> [<label>]: <what the case pins>

`<id>` matches `[a-z0-9][a-z0-9-]*` and identifies one case in the whole
repository; the optional `[<label>]` is a short kind such as `[failure]` and
matches `[a-z][a-z0-9-]*`. Attributes, doc comments and blank lines may sit
between the tag and the test attribute, up to `TAG_LOOKAHEAD` lines down. The
anchor is a test attribute (`#[test]`, `#[tokio::test]`, `#[rstest]`) or a
`proptest! {` block. A file that carries a tag is a *declaring file*.

Scanned trees: every `src/**/*_tests.rs` file, and every file under
`tests/storage/` (the driven-adapter tier holds the storage contract).

The checker fails on:

* a test in a declaring file with no tag;
* a tag with no test attribute after it;
* two tags on one test;
* a duplicate case id;
* a malformed tag: a missing or bad id, a bad label, or no description;
* the old form: a `//! CASE` line in a module doc, or a bare `// CASE: <id>`.

`--list` prints the id and the description of every case, contract by contract.

Exit codes:
    0  every test in a declaring file carries its tag
    1  a violation above
    2  a scanned tree is missing

Run from anywhere:
    python scripts/validate_seam_cases.py [--list]
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path
from typing import NamedTuple

ENGINE_ROOT = Path(__file__).parent.parent
SRC_DIR = ENGINE_ROOT / "src"
STORAGE_DIR = ENGINE_ROOT / "tests" / "storage"

CASE_ID_PATTERN = r"[a-z0-9][a-z0-9-]*"
LABEL_PATTERN = r"[a-z][a-z0-9-]*"

CASE_TAG_RE = re.compile(
    rf"^\s*//\s*CASE\s+({CASE_ID_PATTERN})"
    rf"(?:\s*\[({LABEL_PATTERN})\])?\s*:\s*(\S.*?)\s*$"
)
LOOSE_TAG_RE = re.compile(r"^\s*//\s*CASE\b(.*)$")
OLD_TAG_RE = re.compile(r"^\s*//\s*CASE\s*:")
OLD_DECL_RE = re.compile(r"^\s*//!\s*CASE\b")
TEST_ATTR_RE = re.compile(r"^\s*#\[(?:tokio::)?test\b|^\s*#\[rstest\]|^\s*proptest!\s*\{")

TAG_LOOKAHEAD = 5

OLD_DECL_PROBLEM = (
    "the old module-doc form `//! CASE`; put the description on the tag above "
    "the test: `// CASE <id>: <description>`"
)
OLD_TAG_PROBLEM = (
    "the old bare form `// CASE: <id>`; write `// CASE <id>: <description>`"
)


class CaseTag(NamedTuple):
    """A `// CASE <id>: <description>` tag; ``anchor`` is 0 with no test below."""

    line: int
    case_id: str
    description: str
    anchor: int


class TagFault(NamedTuple):
    line: int
    problem: str


def anchor_line(lines: list[str], tag_line: int) -> int:
    """The 1-based line of the test attribute under ``tag_line``, else 0."""
    for probe in range(tag_line, min(tag_line + TAG_LOOKAHEAD, len(lines))):
        if TEST_ATTR_RE.match(lines[probe]):
            return probe + 1
    return 0


def describe_fault(rest: str) -> str:
    """The reason a `// CASE …` line is not the strict form."""
    rest = rest.lstrip()
    identifier = re.match(CASE_ID_PATTERN, rest)
    if not identifier:
        return "the case id is missing or bad; the id is `[a-z0-9][a-z0-9-]*`"
    rest = rest[identifier.end() :].lstrip()
    if rest.startswith("["):
        closing = rest.find("]")
        if closing < 0:
            return "the label has no closing bracket"
        label = rest[1:closing]
        if not re.fullmatch(LABEL_PATTERN, label):
            return (
                f"the label `[{label}]` is bad; the label is "
                "`[a-z][a-z0-9-]*`, such as `[failure]`"
            )
        rest = rest[closing + 1 :].lstrip()
    if not rest.startswith(":"):
        return "the tag has no `:` after the id"
    if not rest[1:].strip():
        return "the tag has no description; write `// CASE <id>: <description>`"
    return "the tag does not match `// CASE <id> [<label>]: <description>`"


def parse_tags(text: str) -> tuple[list[CaseTag], list[TagFault]]:
    """Return the file's well-formed tags and its tag faults."""
    lines = text.splitlines()
    tags: list[CaseTag] = []
    faults: list[TagFault] = []
    for index, line in enumerate(lines, start=1):
        if OLD_DECL_RE.match(line):
            faults.append(TagFault(index, OLD_DECL_PROBLEM))
            continue
        if OLD_TAG_RE.match(line):
            faults.append(TagFault(index, OLD_TAG_PROBLEM))
            continue
        match = CASE_TAG_RE.match(line)
        if match:
            tags.append(
                CaseTag(index, match.group(1), match.group(3), anchor_line(lines, index))
            )
            continue
        loose = LOOSE_TAG_RE.match(line)
        if loose:
            faults.append(TagFault(index, f"malformed tag: {describe_fault(loose.group(1))}"))
    return tags, faults


def parse_test_lines(text: str) -> list[int]:
    """1-based lines carrying a test attribute or a `proptest!` block."""
    return [
        index
        for index, line in enumerate(text.splitlines(), start=1)
        if TEST_ATTR_RE.match(line)
    ]


def declaring_files(entries: dict[Path, str]) -> dict[Path, list[CaseTag]]:
    """Files carrying at least one tag, keyed by path, in path order."""
    declared: dict[Path, list[CaseTag]] = {}
    for path, text in entries.items():
        tags, _ = parse_tags(text)
        if tags:
            declared[path] = tags
    return declared


def validate(entries: dict[Path, str]) -> list[str]:
    errors: list[str] = []
    declared = declaring_files(entries)

    for path, text in entries.items():
        _, faults = parse_tags(text)
        for fault in faults:
            errors.append(f"{path}:{fault.line}: {fault.problem}")

    seen: dict[str, Path] = {}
    for path, tags in declared.items():
        for tag in tags:
            first = seen.get(tag.case_id)
            if first is not None:
                errors.append(
                    f"{path}:{tag.line}: duplicate case id `{tag.case_id}` "
                    f"(already tagged in {first})"
                )
                continue
            seen[tag.case_id] = path

    for path, tags in declared.items():
        anchored: dict[int, list[int]] = {}
        for tag in tags:
            if tag.anchor == 0:
                errors.append(
                    f"{path}:{tag.line}: tag `{tag.case_id}` has no test "
                    f"attribute within {TAG_LOOKAHEAD} lines"
                )
            else:
                anchored.setdefault(tag.anchor, []).append(tag.line)
        for anchor, tag_lines in sorted(anchored.items()):
            if len(tag_lines) > 1:
                errors.append(
                    f"{path}:{anchor}: the test at this line carries "
                    f"{len(tag_lines)} tags (lines "
                    f"{', '.join(str(line) for line in tag_lines)}): one test "
                    f"pins one case"
                )
        for line in parse_test_lines(entries[path]):
            if line not in anchored:
                errors.append(
                    f"{path}:{line}: test in a declaring file has no "
                    f"`// CASE <id>:` tag"
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


def print_cases(declared: dict[Path, list[CaseTag]]) -> None:
    """Print the id and the description of every case, contract by contract."""
    for path, tags in sorted(declared.items(), key=lambda item: str(item[0])):
        print(f"  {path}: {len(tags)} case(s)")
        for tag in tags:
            print(f"    {tag.case_id}: {tag.description}")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Validate the seam contracts: the `CASE` tag of every test."
    )
    parser.add_argument(
        "--list",
        action="store_true",
        help="print the id and the description of every case",
    )
    args = parser.parse_args(argv)

    for tree in (SRC_DIR, STORAGE_DIR):
        if not tree.is_dir():
            print(f"error: scanned tree {tree} is missing", file=sys.stderr)
            return 2

    entries = collect_entries()
    errors = validate(entries)
    if errors:
        print("SEAM-CASE VIOLATIONS:")
        for message in errors:
            print(f"  {message}")
        print(
            "\nA seam contract tags each test with the case it covers, on the "
            "comment line above the test. See tests/STRATEGY.md, "
            '"The one rule".'
        )
        return 1

    declared = declaring_files(entries)
    cases = sum(len(tags) for tags in declared.values())
    print(f"Seam cases OK: {len(declared)} contract(s), {cases} case(s).")
    if args.list:
        print_cases(declared)
    else:
        for path, tags in sorted(declared.items(), key=lambda item: str(item[0])):
            print(f"  {path}: {len(tags)} case(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
