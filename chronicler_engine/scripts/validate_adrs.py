"""Validate ADR files against the standard in docs/adr/README.md.

Usage:
    python scripts/validate_adrs.py
    python scripts/validate_adrs.py --path docs/adr/adr-NNN-foo.md

Exits non-zero if any violations. The standard is enforced uniformly on all ADRs.

Canonical template: docs/adr/adr-000-template.md (skipped by this validator).
Standards doc: docs/adr/README.md.
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import NamedTuple

NON_ADR_FILES = {"README.md", "adr-000-template.md"}

# Status and Date are required as field-style (`**Status:**` / `**Date:**`) at the
# top of the file, NOT as body sections. See STATUS_PATTERNS and DATE_VALUE.
REQUIRED_SECTIONS: dict[str, set[str]] = {
    "Context": {"Problem Statement"},
    "Decision": set(),
    "Consequences": set(),
}

FORBIDDEN_SECTIONS = [
    "Files Changed",
    "Modified Modules",
    "Architecture Impact",
    "Verification",
    "Compliance",
]

# Subsection name -> match predicate. Matched lowercased against `###` headers
# under ## Consequences.
REQUIRED_SUBSECTIONS: dict[str, tuple[str, callable]] = {
    "positive": ("### Positive", lambda s: s == "positive"),
    "negative": ("### Negative", lambda s: s == "negative"),
    "trade-offs": ("### Trade-offs", lambda s: s.startswith("trade-off")),
}

# Matches "## v2 Update", "## v3 Update: Foo", etc.
INLINE_VERSION_HEADER = re.compile(r"^##\s+v\d+\s+Update", re.IGNORECASE)

PLAN_REFERENCE = re.compile(r"docs/plans/[A-Za-z0-9_./-]+\.md")

SECTION_HEADER = re.compile(r"^##\s+(.+?)\s*$")

# Matches field-style "**Status:** ..." or "**Date:** ..." (colon INSIDE the asterisks).
# Captures: group 1 = field name (e.g. "Status", "Date", "Drivers"), group 2 = value.
FIELD_STYLE = re.compile(r"^\*\*([^:*]+):\*\*\s*(.+)$")

# Matches "# ADR-NNN: Title"
TITLE_LINE = re.compile(r"^#\s+ADR-(\d{3}):\s+.+$")

# Matches YYYY-MM-DD
DATE_VALUE = re.compile(r"\b(\d{4})-(\d{2})-(\d{2})\b")

# Valid status values (first matching pattern wins).
STATUS_PATTERNS: list[tuple[str, re.Pattern[str]]] = [
    ("Accepted", re.compile(r"^\s*Accepted\b", re.IGNORECASE)),
    ("Superseded by ADR-XXX", re.compile(r"^\s*Superseded\s+by\s+ADR-(\d{3})\b", re.IGNORECASE)),
    ("Deprecated", re.compile(r"^\s*Deprecated\b", re.IGNORECASE)),
]

# File name pattern: adr-NNN-kebab-case.md (exactly 3 digits per README).
FILENAME_PATTERN = re.compile(r"^adr-(\d{3})-[a-z0-9-]+\.md$")


class Violation(NamedTuple):
    rule: str
    message: str


@dataclass
class ParsedAdr:
    """Structured parse of one ADR file."""

    title: str | None = None
    top_fields: dict[str, str] = field(default_factory=dict)
    sections: dict[str, str] = field(default_factory=dict)
    subsections: dict[str, list[str]] = field(default_factory=dict)
    forbidden: list[str] = field(default_factory=list)
    inline_version_headers: list[tuple[int, str]] = field(default_factory=list)

    @property
    def status(self) -> str | None:
        value = self.top_fields.get("status")
        if value:
            return value
        if "Status" in self.sections:
            return self.sections["Status"].split("\n")[0].strip()
        return None


@dataclass
class AdrReport:
    path: Path
    number: int | None
    violations: list[Violation] = field(default_factory=list)


def extract_adr_number(path: Path) -> int | None:
    m = FILENAME_PATTERN.match(path.name)
    return int(m.group(1)) if m else None


def parse_adr(path: Path) -> ParsedAdr:
    """Parse an ADR file into a ParsedAdr.

    - `title`: first `# ` line
    - `top_fields`: field-style `**Field:** value` lines scanned before any `##`
      header (covers Status, Date, Drivers, etc.)
    - `sections`: `##`-level headers -> body text (first occurrence only)
    - `subsections`: `##` header -> list of `###` headers under it
    - `forbidden`: section headers matching FORBIDDEN_SECTIONS
    - `inline_version_headers`: (line_number, header_text) for `## vN Update`
    """
    text = path.read_text(encoding="utf-8")
    lines = text.splitlines()
    parsed = ParsedAdr()

    current_section: str | None = None
    current_body: list[str] = []

    def flush() -> None:
        nonlocal current_section, current_body
        if current_section is not None:
            parsed.sections.setdefault(current_section, "\n".join(current_body).strip())
        current_section = None
        current_body = []

    for i, line in enumerate(lines, start=1):
        if parsed.title is None and line.startswith("# ") and not line.startswith("## "):
            parsed.title = line[2:].strip()

        m = FIELD_STYLE.match(line)
        if m:
            name = m.group(1).strip()
            value = m.group(2).strip()
            # Only capture top-block fields (before first ## section).
            if current_section is None:
                parsed.top_fields.setdefault(name.lower(), value)
            # Keep inside body too so field-only "## Status" section style
            # still falls through to sections dict for downstream checks.
            if current_section is not None:
                current_body.append(line)
            continue

        if line.startswith("### ") and current_section is not None:
            sub_header = line[4:].strip()
            parsed.subsections.setdefault(current_section, []).append(sub_header)
            current_body.append(line)
            continue

        m = SECTION_HEADER.match(line)
        if m:
            flush()
            header = m.group(1).strip()
            current_section = header

            for forbidden in FORBIDDEN_SECTIONS:
                if header.lower() == forbidden.lower():
                    parsed.forbidden.append(header)

            if INLINE_VERSION_HEADER.match(line):
                parsed.inline_version_headers.append((i, line.strip()))
            continue

        if current_section is not None:
            current_body.append(line)

    flush()
    return parsed


def validate_adr(path: Path) -> AdrReport:
    number = extract_adr_number(path)
    report = AdrReport(path=path, number=number)

    if number is None:
        # Only flag files that look like misnamed ADRs (adr-*.md)
        if path.name.startswith("adr-") and path.suffix == ".md":
            report.violations.append(
                Violation(
                    "filename",
                    f"Filename must match adr-NNN-kebab-case.md, got {path.name}",
                )
            )
        return report

    parsed = parse_adr(path)

    # Title line
    if parsed.title is None:
        report.violations.append(Violation("title", "Missing top-level title (# ADR-NNN: Title)"))
    elif not TITLE_LINE.match(f"# {parsed.title}"):
        report.violations.append(
            Violation(
                "title",
                f"Title must match '# ADR-NNN: Title', got: {parsed.title!r}",
            )
        )

    # Status
    status_value = parsed.status
    if status_value is None:
        report.violations.append(Violation("status", "Missing **Status:** field at top of file"))
    else:
        matched_pattern = next(
            (pat for _, pat in STATUS_PATTERNS if pat.search(status_value)),
            None,
        )
        if matched_pattern is None:
            report.violations.append(
                Violation(
                    "status",
                    f"Status must be 'Accepted', 'Superseded by ADR-XXX', or "
                    f"'Deprecated'. Got: {status_value!r}",
                )
            )
        else:
            # Status is superseded: confirm the referenced ADR file exists.
            superseded_pat = STATUS_PATTERNS[1][1]
            m = superseded_pat.search(status_value)
            if m:
                sup_number = m.group(1)
                # Glob the directory for any adr-NNN-* match.
                if not any(
                    p.name.startswith(f"adr-{sup_number}-") for p in path.parent.glob("*.md")
                ):
                    report.violations.append(
                        Violation(
                            "lifecycle",
                            f"Superseding ADR-{sup_number} referenced in Status "
                            f"does not exist in {path.parent.name}/",
                        )
                    )

    # Date — required as field-style `**Date:** YYYY-MM-DD` at top of file.
    date_value = parsed.top_fields.get("date")
    if date_value is None:
        report.violations.append(
            Violation("date", "Missing **Date:** YYYY-MM-DD field at top of file")
        )
    elif not DATE_VALUE.search(date_value):
        report.violations.append(
            Violation(
                "date",
                f"**Date:** value must be YYYY-MM-DD. Got: {date_value!r}",
            )
        )

    # Required sections (with aliases)
    for name, aliases in REQUIRED_SECTIONS.items():
        found = name in parsed.sections or bool(aliases & parsed.sections.keys())
        if not found:
            accepted = ", ".join([name, *aliases]) if aliases else name
            report.violations.append(
                Violation(
                    f"section:{name}",
                    f"Missing required section (any of: {accepted})",
                )
            )

    # Consequences subsections: ### Positive, ### Negative, ### Trade-offs all required.
    if "Consequences" in parsed.sections:
        cons_subs = [s.lower() for s in parsed.subsections.get("Consequences", [])]
        for key, (label, pred) in REQUIRED_SUBSECTIONS.items():
            if not any(pred(s) for s in cons_subs):
                report.violations.append(
                    Violation(
                        f"subsection:{key}",
                        f"## Consequences must have {label} subsection",
                    )
                )

    # Forbidden sections
    for header in parsed.forbidden:
        report.violations.append(
            Violation("forbidden-section", f"Forbidden section present: ## {header}")
        )

    # Inline version drift
    for line_num, header in parsed.inline_version_headers:
        report.violations.append(
            Violation(
                "inline-version",
                f"Inline version header at line {line_num}: {header!r} — "
                "write a new superseding ADR instead",
            )
        )

    # Plan references — ADRs must not reference docs/plans/*.md. Plans archive +
    # delete; plans reference ADRs, not the reverse.
    text = path.read_text(encoding="utf-8")
    for match in PLAN_REFERENCE.finditer(text):
        lineno = text.count("\n", 0, match.start()) + 1
        report.violations.append(
            Violation(
                "plan-reference",
                f"Line {lineno}: ADR references {match.group(0)!r} — "
                "plans archive + delete; plans reference ADRs, not reverse",
            )
        )

    return report


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate ADR files against the standard.")
    parser.add_argument(
        "--path",
        type=Path,
        help="Validate a single ADR file (for testing).",
    )
    args = parser.parse_args()

    if args.path:
        if not args.path.exists():
            print(f"Error: file not found: {args.path}")
            return 2
        args.path = args.path.resolve()
        reports = [validate_adr(args.path)]
    else:
        adr_dir = Path(__file__).parent.parent / "docs" / "adr"
        if not adr_dir.exists():
            print(f"Error: ADR directory not found: {adr_dir}")
            return 2
        reports = []
        for md_file in sorted(adr_dir.glob("*.md")):
            if md_file.name in NON_ADR_FILES:
                continue
            reports.append(validate_adr(md_file))

    total = 0
    engine_root = Path(__file__).parent.parent
    for report in reports:
        if not report.violations:
            continue
        try:
            rel_path = report.path.relative_to(engine_root)
        except ValueError:
            rel_path = report.path
        label = f"ADR-{report.number:03d}" if report.number is not None else str(rel_path)
        for v in report.violations:
            print(f"{label} [{v.rule}]: {v.message}")
            total += 1

    print(f"\n{'=' * 60}")
    print(f"Scanned {len(reports)} ADR(s)")
    print(f"Summary: {total} violation(s)")

    if total > 0:
        print("\nFAIL — see violations above.")
        return 1

    print("\nPASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
