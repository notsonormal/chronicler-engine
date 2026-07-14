"""Validate markdown docs under chronicler_engine/docs/.

Role-aware deterministic checks. Each .md file is classified into one of three
roles which determine which rules apply:

  STANDARD  — canonical spec docs (architecture/system/reference/diagnostics/
              external_applications/adr-0XX). All rules enforced.
  TRANSIENT — historical or forward-looking docs (CHANGELOG, plans). Exempt
              from all checks; they may reference anything historically.
  EXCLUDED  — auto-generated indexes, ADR standards readme, template, archives.
              Exempt from all checks.

Rules (only enforced on STANDARD docs):

  BROKEN_MARKDOWN_LINK     — [text](relative/path.md) where target file missing.
  BROKEN_ADR_REF           — `ADR-NNN` mention where adr-NNN-*.md file is missing.
  STANDARD_PLAN_LINK       — link into docs/plans/ or old-docs/archived-plans/.
                              Standards must be self-contained; plans are transient
                              and cannot be leaned on as canonical reference.
  STANDARD_DOC_BODY_REFERENCE — doc-internal reference in body prose (before the
                              file's `## Document References` section). Catches:
                                (a) `[text](relative/path.md)` where target is .md
                                    inside docs/
                                (b) `\bADR-NNN\b` mentions
                              Both must appear only in the `## Document References`
                              section at the bottom of the file. Body prose must
                              stand alone as a Specification — no pointers to
                              decision context or further reading inline.

Mirrors `validate_adrs.py` structure (Violation NamedTuple, per-file report,
summary line at bottom). Intentionally decoupled from the
`chronicler-docs-hygiene` skill — skill handles LLM semantic analysis, this
script handles deterministic checks. CI / pre-commit can run this in <1s.

Usage:
    python scripts/validate_docs.py
    python scripts/validate_docs.py --strict
    python scripts/validate_docs.py --list
    python scripts/validate_docs.py --path chronicler_engine/docs/system/llm_processing.md
    python scripts/validate_docs.py --links
    python scripts/validate_docs.py --adr-refs
    python scripts/validate_docs.py --plan-links
    python scripts/validate_docs.py --body-refs
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path
from typing import NamedTuple

# Severities.
ERROR = "error"  # fails build.
WARNING = "warning"  # does not fail unless --strict.

# Markdown link: [text](target). Capture group 1 = target.
# Excludes images (leading !).
MARKDOWN_LINK = re.compile(r"(?<!\!)\[[^\]]*\]\(([^)]+)\)")

# ADR cross-reference: ADR-NNN (one or more digits, no leading zeros required).
# Word boundary on left; not preceded by alphanumerics on right (so ADR-12x fails).
ADR_REF = re.compile(r"\bADR-(\d+)\b")

# Heading regex for finding the `## Document References` section boundary.
# Matches `## Document References` at start of line (any leading whitespace).
DOC_REFERENCES_HEADING = re.compile(r"^\s*##\s+Document\s+References\s*$")

# Fenced code block delimiter (``` or ~~~), optionally with language tag.
FENCE_DELIMITER = re.compile(r"^\s*(```|~~~)")

# Paths whose existence we never check (network, mail, in-page anchors).
URI_SCHEMES = ("http://", "https://", "mailto:")

# Files exempt from all checks (auto-generated indexes, ADR standards readme,
# template file containing placeholders, top-level README).
EXCLUDED_FILE_NAMES: set[str] = {
    "AGENTS.md",
    "README.md",
}

# File patterns exempt from all checks (template, etc.). Matched on stem.
EXCLUDED_STEM_PREFIXES: tuple[str, ...] = ("adr-000-template",)

# Directories whose entire subtree is exempt (archives, transient plan
# archives, et al.). Matched on any path segment under docs_root.
EXCLUDED_DIR_NAMES: set[str] = {"old-docs"}

# Transient file paths: exempt from all checks because they are historical or
# forward-looking logs, not canonical specs.
TRANSIENT_FILE_PATHS: tuple[str, ...] = (
    "CHANGELOG.md",
    "plans",  # whole plans/ directory
)

# Directories whose *.md files are STANDARDS (canonical spec docs).
STANDARD_DIR_NAMES: set[str] = {
    "architecture",
    "system",
    "reference",
    "diagnostics",
    "external_applications",
    "adr",  # ADR cross-file link/ref checks enforced here
}


class Violation(NamedTuple):
    severity: str
    rule: str
    message: str


class FileReport:
    """Per-file scan result."""

    __slots__ = ("path", "violations")

    def __init__(self, path: Path) -> None:
        self.path = path
        self.violations: list[Violation] = []


def relative_to_docs(path: Path, docs_root: Path) -> Path | None:
    """Return path relative to docs_root, or None if not inside docs_root."""
    try:
        return path.resolve().relative_to(docs_root.resolve())
    except ValueError:
        return None


def classify_file(path: Path, docs_root: Path) -> str:
    """Return one of: 'STANDARD', 'TRANSIENT', 'EXCLUDED'.

    Classification rules (first match wins):
      1. File name in EXCLUDED_FILE_NAMES  → EXCLUDED
      2. Stem starts with EXCLUDED_STEM_PREFIXES → EXCLUDED
      3. Any path segment in EXCLUDED_DIR_NAMES → EXCLUDED
      4. First segment is 'plans' OR path is 'CHANGELOG.md' → TRANSIENT
      5. First segment in STANDARD_DIR_NAMES → STANDARD
      6. Otherwise → EXCLUDED (unrecognized top-level doc — be permissive)
    """
    rel = relative_to_docs(path, docs_root)
    if rel is None:
        return "EXCLUDED"

    parts = rel.parts
    if not parts:
        return "EXCLUDED"

    if parts[-1] in EXCLUDED_FILE_NAMES:
        return "EXCLUDED"

    if any(parts[-1].startswith(prefix) for prefix in EXCLUDED_STEM_PREFIXES):
        return "EXCLUDED"

    if any(part in EXCLUDED_DIR_NAMES for part in parts):
        return "EXCLUDED"

    first = parts[0]
    if first == "plans":
        return "TRANSIENT"
    if rel.as_posix() == "CHANGELOG.md":
        return "TRANSIENT"

    if first in STANDARD_DIR_NAMES:
        return "STANDARD"

    return "EXCLUDED"


def collect_markdown_files(targets: list[Path], docs_root: Path) -> list[Path]:
    """Expand targets to a sorted list of .md files under docs_root.

    All files are returned regardless of role; role classification happens at
    scan time. Excluded directories (old-docs/) are pruned here for efficiency.
    """
    files: set[Path] = set()
    for target in targets:
        if target.is_file():
            files.add(target)
            continue
        if target.is_dir():
            for md_file in target.rglob("*.md"):
                rel = relative_to_docs(md_file, docs_root)
                if rel is None:
                    continue
                if any(part in EXCLUDED_DIR_NAMES for part in rel.parts):
                    continue
                files.add(md_file)
            continue
        # Fallback: treat as glob pattern.
        for md_file in docs_root.glob(target.as_posix()):
            if md_file.is_file():
                rel = relative_to_docs(md_file, docs_root)
                if rel is None:
                    continue
                if any(part in EXCLUDED_DIR_NAMES for part in rel.parts):
                    continue
                files.add(md_file)
    return sorted(files)


def read_text(report: FileReport) -> str | None:
    try:
        return report.path.read_text(encoding="utf-8")
    except OSError as exc:
        report.violations.append(
            Violation(ERROR, "READ_ERROR", f"Could not read file: {exc}")
        )
        return None


def check_markdown_links(report: FileReport, docs_root: Path) -> None:
    """Flag [text](relative/path.md) where target file does not exist in docs/."""
    text = read_text(report)
    if text is None:
        return

    for lineno, line in enumerate(text.splitlines(), start=1):
        for match in MARKDOWN_LINK.finditer(line):
            target = match.group(1).strip()
            # Strip optional title (`href "title"`) and fragment (`href#frag`).
            target_path_only = target.split(" ", 1)[0].split("#", 1)[0]
            if not target_path_only:
                continue
            if target_path_only.startswith(URI_SCHEMES):
                continue
            if target_path_only.startswith("#"):
                continue
            # Only check relative .md paths. Skip absolute paths and other exts.
            if not target_path_only.endswith(".md"):
                continue
            # Skip `adr-000-template.md` (placeholder file referenced by docs).
            if target_path_only.endswith("adr-000-template.md"):
                continue
            # Resolve relative to the linking file's directory.
            resolved = (report.path.parent / target_path_only).resolve()
            try:
                resolved.relative_to(docs_root.resolve())
            except ValueError:
                # Target escapes docs/ tree — out of scope for this check.
                # (Plan-link rule catches standards→plans/ outside docs/.)
                continue
            if not resolved.exists():
                report.violations.append(
                    Violation(
                        ERROR,
                        "BROKEN_MARKDOWN_LINK",
                        f"Line {lineno}: target does not exist: {target_path_only}",
                    )
                )


def check_adr_refs(report: FileReport, adr_dir: Path) -> None:
    """Flag ADR-NNN mentions where the corresponding ADR file is missing."""
    text = read_text(report)
    if text is None:
        return

    seen: set[int] = set()
    for lineno, line in enumerate(text.splitlines(), start=1):
        for match in ADR_REF.finditer(line):
            number = int(match.group(1))
            if number in seen:
                continue
            seen.add(number)
            padded = f"{number:03d}"
            matches = list(adr_dir.glob(f"adr-{padded}-*.md"))
            if not matches:
                # Fallback: any case.
                matches = [
                    p
                    for p in adr_dir.iterdir()
                    if p.is_file() and p.stem.lower().startswith(f"adr-{padded.lower()}-")
                ]
            if not matches:
                report.violations.append(
                    Violation(
                        ERROR,
                        "BROKEN_ADR_REF",
                        f"Line {lineno}: ADR-{padded} referenced but "
                        f"adr-{padded}-*.md not found in {adr_dir.name}/",
                    )
                )


def check_standard_plan_links(report: FileReport, docs_root: Path) -> None:
    """Flag standards linking into docs/plans/ or old-docs/archived-plans/.

    Plans are transient; standards must be self-contained.
    """
    text = read_text(report)
    if text is None:
        return

    docs_root_resolved = docs_root.resolve()
    for lineno, line in enumerate(text.splitlines(), start=1):
        for match in MARKDOWN_LINK.finditer(line):
            target = match.group(1).strip()
            target_path_only = target.split(" ", 1)[0].split("#", 1)[0]
            if not target_path_only:
                continue
            if target_path_only.startswith(URI_SCHEMES):
                continue
            if target_path_only.startswith("#"):
                continue
            if not target_path_only.endswith(".md"):
                continue

            # Normalise the target for plan-shape matching.
            normalised = target_path_only.replace("\\", "/").lstrip("./")
            lower = normalised.lower()

            is_plan_link = (
                lower.startswith("plans/")
                or "/plans/".lower() in lower
                or "old-docs/archived-plans/" in lower
            )
            if not is_plan_link:
                continue

            # Resolve to confirm it really lives under docs_root/plans or
            # docs_root/old-docs/archived-plans (not e.g. a sibling docs/plans
            # outside the engine).
            resolved = (report.path.parent / target_path_only).resolve()
            try:
                rel_to_docs = resolved.relative_to(docs_root_resolved)
            except ValueError:
                # Escapes docs/ — still a plan-shape link from a standard.
                # Report by target string as written.
                report.violations.append(
                    Violation(
                        ERROR,
                        "STANDARD_PLAN_LINK",
                        f"Line {lineno}: standards must not link to plans: "
                        f"{target_path_only}",
                    )
                )
                continue

            rel_str = rel_to_docs.as_posix()
            if (
                rel_str.startswith("plans/")
                or rel_str.startswith("old-docs/archived-plans/")
            ):
                report.violations.append(
                    Violation(
                        ERROR,
                        "STANDARD_PLAN_LINK",
                        f"Line {lineno}: standards must not link to plans: "
                        f"{target_path_only}",
                    )
                )


def check_standard_body_references(report: FileReport, docs_root: Path) -> None:
    """Flag doc-internal references in body prose.

    Body prose = lines before the file's `## Document References` section
    (or the entire file if no such section exists). Fenced code blocks
    inside body prose are exempt.

    Flagged in body:
      (a) `[text](relative/path.md)` where target is .md and resolves inside docs/
      (b) `\bADR-NNN\b` mentions

    Both belong only in the `## Document References` section at the bottom.
    Body prose must stand alone as a Specification — no inline pointers to
    decision context or further reading.

    ADRs (docs/adr/*.md) are exempt from this rule — they are decision records
    that link context (status, tradeoffs, related decisions) inline by design.
    """
    # ADR exemption: decision records allow inline cross-references.
    try:
        rel_to_docs = report.path.resolve().relative_to(docs_root.resolve())
    except ValueError:
        return
    if rel_to_docs.parts and rel_to_docs.parts[0] == "adr":
        return

    text = read_text(report)
    if text is None:
        return

    docs_root_resolved = docs_root.resolve()
    lines = text.splitlines()

    # Find the `## Document References` section boundary line index (1-based
    # end of body). Lines from boundary onward are reference section, exempt.
    body_end_lineno = len(lines) + 1  # default: entire file is body
    for idx, line in enumerate(lines, start=1):
        if DOC_REFERENCES_HEADING.match(line):
            body_end_lineno = idx
            break

    # Walk body lines, tracking fenced-code-block state. Fenced blocks contain
    # documentation examples with literal `[...](...)` markup that is content,
    # not a reference.
    in_fence = False
    for lineno, line in enumerate(lines, start=1):
        if lineno >= body_end_lineno:
            break
        if FENCE_DELIMITER.match(line):
            in_fence = not in_fence
            continue
        if in_fence:
            continue

        # (a) Doc-internal markdown link
        for match in MARKDOWN_LINK.finditer(line):
            target = match.group(1).strip()
            target_path_only = target.split(" ", 1)[0].split("#", 1)[0]
            if not target_path_only:
                continue
            if target_path_only.startswith(URI_SCHEMES):
                continue
            if target_path_only.startswith("#"):
                continue
            if not target_path_only.endswith(".md"):
                continue
            if target_path_only.endswith("adr-000-template.md"):
                continue
            # Resolve relative to the linking file's directory. Only flag if
            # target is inside docs/ — anything escaping docs/ is out of scope
            # for this rule (plan-link rule catches docs/plans separately).
            resolved = (report.path.parent / target_path_only).resolve()
            try:
                resolved.relative_to(docs_root_resolved)
            except ValueError:
                continue
            report.violations.append(
                Violation(
                    ERROR,
                    "STANDARD_DOC_BODY_REFERENCE",
                    f"Line {lineno}: doc-internal link to "
                    f"{target_path_only} appears in body prose. Move to "
                    f"`## Document References` section at the bottom of the "
                    f"file.",
                )
            )
            # Don't double-report: an inline ADR-NNN mention like `ADR-014`
            # typically appears in the same line as `[ADR-014](path.md)`.
            # Reporting the link is sufficient; the ADR-NNN mention in the
            # link text is the same reference. Skip ADR scan for this line.

        # (b) ADR-NNN mention — only if no link on this line was reported
        # for the same ADR. If the line already had a flagged link, the ADR
        # mention is part of the same reference — don't double-flag.
        has_flagged_link = any(
            v.rule == "STANDARD_DOC_BODY_REFERENCE" and v.message.startswith(
                f"Line {lineno}:"
            )
            for v in report.violations
        )
        if has_flagged_link:
            continue
        for match in ADR_REF.finditer(line):
            report.violations.append(
                Violation(
                    ERROR,
                    "STANDARD_DOC_BODY_REFERENCE",
                    f"Line {lineno}: ADR-{match.group(1)} mentioned in body "
                    f"prose. Move to `## Document References` section at the "
                    f"bottom of the file.",
                )
            )


def scan_file(
    path: Path, docs_root: Path, adr_dir: Path, modes: set[str]
) -> FileReport:
    report = FileReport(path)
    role = classify_file(path, docs_root)
    if role != "STANDARD":
        return report
    if "links" in modes:
        check_markdown_links(report, docs_root)
    if "adr-refs" in modes:
        check_adr_refs(report, adr_dir)
    if "plan-links" in modes:
        check_standard_plan_links(report, docs_root)
    if "body-refs" in modes:
        check_standard_body_references(report, docs_root)
    return report


def render_reports(
    reports: list[FileReport],
    engine_root: Path,
) -> tuple[str, int, int, int]:
    """Return (rendered_text, error_count, warning_count, file_count_with_violations)."""
    lines: list[str] = []
    error_count = 0
    warning_count = 0
    files_with_violations = 0

    for report in reports:
        if not report.violations:
            continue
        files_with_violations += 1
        try:
            rel_path = report.path.relative_to(engine_root)
        except ValueError:
            rel_path = report.path
        for v in report.violations:
            lines.append(f"{rel_path} [{v.rule}] ({v.severity}): {v.message}")
            if v.severity == ERROR:
                error_count += 1
            elif v.severity == WARNING:
                warning_count += 1

    summary = (
        f"{error_count} error(s), {warning_count} warning(s) across {files_with_violations} file(s)"
    )
    lines.append("")
    lines.append("=" * 60)
    lines.append(f"Scanned {len(reports)} file(s)")
    lines.append(f"Summary: {summary}")
    return "\n".join(lines), error_count, warning_count, files_with_violations


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate markdown docs under chronicler_engine/docs/.",
    )
    parser.add_argument(
        "--path",
        type=Path,
        help="Validate a single file (relative to repo root or absolute).",
    )
    parser.add_argument(
        "--strict",
        action="store_true",
        help="Treat warnings as errors (exit non-zero if any warning present).",
    )
    parser.add_argument(
        "--list",
        action="store_true",
        dest="list_only",
        help="Print violations only, no PASS/FAIL summary. Always exits 0.",
    )
    parser.add_argument(
        "--links",
        action="store_true",
        help="Run only the broken-markdown-link check.",
    )
    parser.add_argument(
        "--adr-refs",
        action="store_true",
        help="Run only the broken-ADR-ref check.",
    )
    parser.add_argument(
        "--plan-links",
        action="store_true",
        help="Run only the standards-must-not-link-to-plans check.",
    )
    parser.add_argument(
        "--body-refs",
        action="store_true",
        help="Run only the doc-internal-reference-in-body check.",
    )
    return parser.parse_args(argv)


def resolve_modes(args: argparse.Namespace) -> set[str]:
    # Translate argparse flag attrs to mode names.
    mode_names: set[str] = set()
    if args.links:
        mode_names.add("links")
    if args.adr_refs:
        mode_names.add("adr-refs")
    if args.plan_links:
        mode_names.add("plan-links")
    if args.body_refs:
        mode_names.add("body-refs")
    if mode_names:
        return mode_names
    return {"links", "adr-refs", "plan-links", "body-refs"}


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)

    engine_root = Path(__file__).parent.parent
    docs_root = engine_root / "docs"
    adr_dir = docs_root / "adr"

    if not docs_root.exists():
        print(f"Error: docs directory not found: {docs_root}", file=sys.stderr)
        return 2

    modes = resolve_modes(args)

    if args.path:
        target = args.path.resolve()
        if not target.exists():
            print(f"Error: file not found: {args.path}", file=sys.stderr)
            return 2
        files = [target]
    else:
        files = collect_markdown_files([docs_root], docs_root)

    if not files:
        print(f"No markdown files found under {docs_root}", file=sys.stderr)
        return 0

    reports = [scan_file(f, docs_root, adr_dir, modes) for f in files]
    rendered, error_count, warning_count, files_with_violations = render_reports(
        reports, engine_root
    )
    print(rendered)

    if args.list_only:
        return 0

    if error_count > 0:
        print("\nFAIL — see violations above.")
        return 1
    if args.strict and warning_count > 0:
        print("\nFAIL (--strict) — warnings present.")
        return 1

    print("\nPASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
