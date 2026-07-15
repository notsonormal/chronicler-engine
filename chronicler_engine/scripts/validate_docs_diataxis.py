"""Validate markdown docs under chronicler_engine/docs/ and docs-diataxis/.

Role-aware deterministic checks. Each .md file is classified into one of three
roles which determine which rules apply:

  STANDARD  — canonical spec docs (architecture/system/reference/diagnostics/
              external_applications/adr-0XX under docs/; reference/explanation
              under docs-diataxis/). All rules enforced.
  TRANSIENT — historical or forward-looking docs (CHANGELOG, plans). Exempt
              from all checks; they may reference anything historically.
  EXCLUDED  — auto-generated indexes, ADR standards readme, template, archives,
              process notes (AGENTS.md, _PILOT_NOTES.md). Exempt from all checks.

Rules (only enforced on STANDARD docs):

  BROKEN_MARKDOWN_LINK     — [text](relative/path.md) where target file missing.
  BROKEN_ADR_REF           — `ADR-NNN` mention where adr-NNN-*.md file is missing.
  STANDARD_PLAN_LINK       — link into docs/plans/ or old-docs/archived-plans/.
                              Standards must be self-contained; plans are transient
                              and cannot be leaned on as canonical reference.
  STANDARD_DOC_BODY_REFERENCE — doc-internal reference in body prose (before the
                              file's `## Document References` section). Catches:
                                (a) `[text](relative/path.md)` where target is .md
                                    inside the same docs root
                                (b) `\bADR-NNN\b` mentions
                              Both must appear only in the `## Document References`
                              section at the bottom of the file.

Diátaxis-front-matter rules (only enforced on STANDARD docs under
docs-diataxis/; the legacy docs/ tree predates the convention and is exempt):

  MISSING_FRONTMATTER          — no YAML front-matter (`---` block) at top of file.
  EMPTY_FRONTMATTER            — front-matter block present but parses to empty.
  FRONTMATTER_PARSE_ERROR      — YAML does not parse.
  FRONTMATTER_NOT_MAPPING      — YAML parses but the root is not a mapping.
  FRONTMATTER_MISSING_KEY      — required key (`diataxis:` or `title:`) absent.
  FRONTMATTER_INVALID_MODE     — `diataxis:` value not in the vocabulary.
  FRONTMATTER_INVALID_ARC52    — `arc52:` present but not a list of valid sections.
  FRONTMATTER_ARC52_OUT_OF_PLACE — `arc52:` present on a non-architecture doc
                                   (warning only; convention is arc52 only on
                                   architecture-shaped docs).
  MODE_CONTENT_MISMATCH        — declared mode and a simple body heuristic
                                 disagree (warning only; the chronicler-docs-hygiene
                                 skill does the deeper semantic check).

Mirrors `validate_adrs.py` structure (Violation NamedTuple, per-file report,
summary line at bottom). Intentionally decoupled from the
`chronicler-docs-hygiene` skill — skill handles LLM semantic analysis, this
script handles deterministic checks. CI / pre-commit can run this in <1s.

Usage:
    python scripts/validate_docs_diataxis.py
    python scripts/validate_docs_diataxis.py --strict
    python scripts/validate_docs_diataxis.py --list
    python scripts/validate_docs_diataxis.py --path reference/data_layer.md
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path
from typing import NamedTuple

import yaml

# ---------------------------------------------------------------------------
# Severities.
# ---------------------------------------------------------------------------

ERROR = "error"  # fails build.
WARNING = "warning"  # does not fail unless --strict.

# ---------------------------------------------------------------------------
# Shared markdown regexes (same shape as validate_docs.py).
# ---------------------------------------------------------------------------

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

# Front-matter delimiter: a line that is exactly `---` (optional whitespace).
FRONTMATTER_DELIMITER = re.compile(r"^---\s*$")

# Paths whose existence we never check (network, mail, in-page anchors).
URI_SCHEMES = ("http://", "https://", "mailto:")

# ---------------------------------------------------------------------------
# Diátaxis front-matter vocabulary.
# ---------------------------------------------------------------------------

VALID_DIATAXIS_MODES: frozenset[str] = frozenset(
    {"tutorial", "how-to", "reference", "explanation"}
)

VALID_ARC52_SECTIONS: frozenset[str] = frozenset({"§3", "§5", "§7", "§10"})

# First-segment path names that mark an `arc52`-shaped doc (where `arc52:`
# front-matter is appropriate). Anything else gets a warning.
ARC52_FIRST_SEGMENTS: frozenset[str] = frozenset({"architecture"})

# ---------------------------------------------------------------------------
# Heuristic patterns for the simple mode-vs-content check (warn-only).
# ---------------------------------------------------------------------------

# Reader-directed procedural markers: "Let's...", "First, ...", "You should...",
# "we'll", "you'll", "Step N:". These are intentionally simple — the
# chronicler-docs-hygiene skill (ticket 05) handles the deeper semantic version.
#
# Notably NOT included: a top-level numbered-step pattern like `^\s*1\.\s+[A-Z]`.
# Reference docs frequently describe engine behavior in numbered lists
# (e.g. "1. Set state.X = Y; 2. Persist via save_state()") that look procedural
# but are factual description, not reader instruction. The heuristic cannot
# reliably distinguish the two without prose-level context; the hygiene skill
# (ticket 05) does the deep semantic check.
PROCEDURAL_MARKERS: tuple[re.Pattern[str], ...] = (
    re.compile(r"\bLet's\b", re.IGNORECASE),
    re.compile(r"\bwe'll\b", re.IGNORECASE),
    re.compile(r"\byou'll\b", re.IGNORECASE),
    re.compile(r"\bFirst,\b"),
    re.compile(r"\bNext,\b"),
    re.compile(r"\bThen,\b"),
    re.compile(r"\bYou should\b", re.IGNORECASE),
    re.compile(r"\bRun the following\b", re.IGNORECASE),
    re.compile(r"^\s*Step\s+\d+:", re.IGNORECASE | re.MULTILINE),
)

# ---------------------------------------------------------------------------
# File / directory classification.
# ---------------------------------------------------------------------------

# Files exempt from all checks regardless of tree.
EXCLUDED_FILE_NAMES: set[str] = {
    "AGENTS.md",
    "README.md",
    "_PILOT_NOTES.md",  # process artifact, not a Diátaxis doc
}

# File patterns exempt from all checks (template, etc.). Matched on stem.
EXCLUDED_STEM_PREFIXES: tuple[str, ...] = ("adr-000-template",)

# Directories whose entire subtree is exempt (archives, etc.).
EXCLUDED_DIR_NAMES: set[str] = {"old-docs"}

# Transient file paths: exempt because they are historical or forward-looking
# logs, not canonical specs.
TRANSIENT_FILE_PATHS: tuple[str, ...] = (
    "CHANGELOG.md",
    "plans",
)

# Standard dir names. The legacy docs/ tree uses architecture/system/reference/
# diagnostics/external_applications/adr; the docs-diataxis/ tree uses reference/
# explanation/ (with explanation/architecture/ as a nested subfolder). Both
# shapes resolve to STANDARD via the first segment.
STANDARD_DIR_NAMES: set[str] = {
    "architecture",
    "system",
    "reference",
    "diagnostics",
    "external_applications",
    "adr",
    "explanation",  # docs-diataxis only
}


# ---------------------------------------------------------------------------
# Result types.
# ---------------------------------------------------------------------------


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


class FrontmatterResult(NamedTuple):
    """Result of front-matter extraction.

    `present`     True if the file has a `---` block at the top.
    `empty`       True if the block exists but parses to nothing.
    `parsed`      The parsed mapping (None on error or absence).
    `error`       Human-readable error message if extraction failed.
    `error_kind`  One of None / "yaml_parse" / "not_mapping" / "unterminated".
                  Lets the caller map errors to specific violation rule
                  names (FRONTMATTER_PARSE_ERROR vs FRONTMATTER_NOT_MAPPING).
    `body_offset` Line index in the original file where the body starts
                  (0 if no front-matter).
    """

    present: bool
    empty: bool
    parsed: dict | None
    error: str | None
    error_kind: str | None
    body_offset: int


# ---------------------------------------------------------------------------
# Helpers.
# ---------------------------------------------------------------------------


def relative_to(path: Path, root: Path) -> Path | None:
    """Return path relative to root, or None if not inside root."""
    try:
        return path.resolve().relative_to(root.resolve())
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
    rel = relative_to(path, docs_root)
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


def is_diataxis_tree_path(rel_to_engine: Path) -> bool:
    """True if `rel_to_engine` lives under engine_root/docs-diataxis/."""
    return bool(rel_to_engine.parts) and rel_to_engine.parts[0] == "docs-diataxis"


def is_architecture_shaped(rel_to_docs: Path) -> bool:
    """True if the doc lives under an `architecture/` directory.

    Only checked for docs-diataxis/ docs. Used to validate `arc52:` placement.
    """
    return "architecture" in rel_to_docs.parts


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
                rel = relative_to(md_file, docs_root)
                if rel is None:
                    continue
                if any(part in EXCLUDED_DIR_NAMES for part in rel.parts):
                    continue
                files.add(md_file)
            continue
        # Fallback: treat as glob pattern.
        for md_file in docs_root.glob(target.as_posix()):
            if md_file.is_file():
                rel = relative_to(md_file, docs_root)
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


# ---------------------------------------------------------------------------
# Front-matter extraction.
# ---------------------------------------------------------------------------


def parse_frontmatter(text: str) -> FrontmatterResult:
    """Parse YAML front-matter from the top of `text`.

    Front-matter shape:
        ---
        key: value
        ---
        <body>

    The opening `---` must be on line 0 (line 1 in 1-indexed). Anything else
    means "no front-matter" (caller decides what that implies).
    """
    lines = text.splitlines()
    if not lines or not FRONTMATTER_DELIMITER.match(lines[0]):
        return FrontmatterResult(
            present=False,
            empty=False,
            parsed=None,
            error=None,
            error_kind=None,
            body_offset=0,
        )

    end_idx: int | None = None
    for idx in range(1, len(lines)):
        if FRONTMATTER_DELIMITER.match(lines[idx]):
            end_idx = idx
            break
    if end_idx is None:
        return FrontmatterResult(
            present=True,
            empty=False,
            parsed=None,
            error="front-matter opened with `---` but never closed",
            error_kind="unterminated",
            body_offset=len(lines),
        )

    fm_text = "\n".join(lines[1:end_idx])
    body_offset = end_idx + 1
    if not fm_text.strip():
        return FrontmatterResult(
            present=True,
            empty=True,
            parsed={},
            error=None,
            error_kind=None,
            body_offset=body_offset,
        )

    try:
        parsed = yaml.safe_load(fm_text)
    except yaml.YAMLError as exc:
        return FrontmatterResult(
            present=True,
            empty=False,
            parsed=None,
            error=f"YAML parse error: {exc}",
            error_kind="yaml_parse",
            body_offset=body_offset,
        )

    if parsed is None:
        return FrontmatterResult(
            present=True,
            empty=True,
            parsed={},
            error=None,
            error_kind=None,
            body_offset=body_offset,
        )

    if not isinstance(parsed, dict):
        return FrontmatterResult(
            present=True,
            empty=False,
            parsed=None,
            error=f"front-matter must be a YAML mapping, got {type(parsed).__name__}",
            error_kind="not_mapping",
            body_offset=body_offset,
        )

    return FrontmatterResult(
        present=True,
        empty=False,
        parsed=parsed,
        error=None,
        error_kind=None,
        body_offset=body_offset,
    )


def body_text(text: str, fm: FrontmatterResult) -> str:
    """Return the markdown body (lines after the front-matter)."""
    lines = text.splitlines()
    return "\n".join(lines[fm.body_offset:])


# ---------------------------------------------------------------------------
# Existing checks (link / ADR / plan / body references).
# ---------------------------------------------------------------------------


def check_markdown_links(report: FileReport, docs_root: Path) -> None:
    """Flag [text](relative/path.md) where target file does not exist."""
    text = read_text(report)
    if text is None:
        return

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
            if target_path_only.endswith("adr-000-template.md"):
                continue
            resolved = (report.path.parent / target_path_only).resolve()
            try:
                resolved.relative_to(docs_root.resolve())
            except ValueError:
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
    """Flag ADR-NNN mentions where the corresponding ADR file is missing.

    `adr_dir` may be None if the engine has no docs/adr/ directory; in that
    case the check is a no-op (matches the existing validate_docs.py behavior
    of silently skipping when the ADR dir is absent).
    """
    if adr_dir is None or not adr_dir.exists():
        return

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
    """Flag standards linking into docs/plans/ or old-docs/archived-plans/."""
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

            normalised = target_path_only.replace("\\", "/").lstrip("./")
            lower = normalised.lower()

            is_plan_link = (
                lower.startswith("plans/")
                or "/plans/".lower() in lower
                or "old-docs/archived-plans/" in lower
            )
            if not is_plan_link:
                continue

            resolved = (report.path.parent / target_path_only).resolve()
            try:
                rel_to_docs = resolved.relative_to(docs_root_resolved)
            except ValueError:
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

    ADRs (docs/adr/*.md) are exempt from this rule — they are decision records
    that link context inline by design.
    """
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

    body_end_lineno = len(lines) + 1
    for idx, line in enumerate(lines, start=1):
        if DOC_REFERENCES_HEADING.match(line):
            body_end_lineno = idx
            break

    in_fence = False
    for lineno, line in enumerate(lines, start=1):
        if lineno >= body_end_lineno:
            break
        if FENCE_DELIMITER.match(line):
            in_fence = not in_fence
            continue
        if in_fence:
            continue

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


# ---------------------------------------------------------------------------
# Diátaxis front-matter checks.
# ---------------------------------------------------------------------------


def check_diataxis_frontmatter(
    report: FileReport,
    *,
    is_architecture_shaped: bool = True,
) -> None:
    """Enforce YAML front-matter conventions on docs-diataxis/ STANDARD docs.

    `is_architecture_shaped`: True if the file lives under an `architecture/`
    subfolder. `arc52:` is appropriate there; the validator warns with
    FRONTMATTER_ARC52_OUT_OF_PLACE when it's declared on a non-architecture
    doc. The default is True so the function is safe to call in isolation
    (e.g. from unit tests) without false-positive warnings.

    Emits MISSING_FRONTMATTER, EMPTY_FRONTMATTER, FRONTMATTER_PARSE_ERROR,
    FRONTMATTER_NOT_MAPPING, FRONTMATTER_MISSING_KEY, FRONTMATTER_INVALID_MODE,
    FRONTMATTER_INVALID_ARC52, and FRONTMATTER_ARC52_OUT_OF_PLACE violations
    as appropriate. See module docstring for the rule set.
    """
    text = read_text(report)
    if text is None:
        return

    fm = parse_frontmatter(text)

    if not fm.present:
        report.violations.append(
            Violation(
                ERROR,
                "MISSING_FRONTMATTER",
                "Line 1: file has no YAML front-matter block "
                "(expected `---` on line 1, content, then closing `---`). "
                "Every docs-diataxis/ doc must declare `diataxis:` and `title:`.",
            )
        )
        return
    if fm.error is not None:
        # Map structured error_kind to the specific rule. unterminated and
        # yaml_parse both fall under FRONTMATTER_PARSE_ERROR (the front-matter
        # block exists but is malformed); not_mapping is its own rule because
        # the YAML is well-formed but not a key/value mapping.
        if fm.error_kind == "not_mapping":
            rule = "FRONTMATTER_NOT_MAPPING"
        else:
            rule = "FRONTMATTER_PARSE_ERROR"
        report.violations.append(
            Violation(ERROR, rule, f"Line 1: {fm.error}")
        )
        return
    if fm.empty or not fm.parsed:
        report.violations.append(
            Violation(
                ERROR,
                "EMPTY_FRONTMATTER",
                "Line 1: front-matter block is empty; "
                "must declare at least `diataxis:` and `title:`.",
            )
        )
        return

    parsed = fm.parsed

    # Required keys.
    for required in ("diataxis", "title"):
        if required not in parsed:
            report.violations.append(
                Violation(
                    ERROR,
                    "FRONTMATTER_MISSING_KEY",
                    f"Line 1: front-matter is missing required key `{required}:`",
                )
            )

    # Mode vocabulary check (only if `diataxis:` is present and scalar-ish).
    if "diataxis" in parsed:
        mode_value = parsed["diataxis"]
        if not isinstance(mode_value, str):
            report.violations.append(
                Violation(
                    ERROR,
                    "FRONTMATTER_INVALID_MODE",
                    f"Line 1: `diataxis:` must be a string, got "
                    f"{type(mode_value).__name__}",
                )
            )
        elif mode_value not in VALID_DIATAXIS_MODES:
            valid = ", ".join(sorted(VALID_DIATAXIS_MODES))
            report.violations.append(
                Violation(
                    ERROR,
                    "FRONTMATTER_INVALID_MODE",
                    f"Line 1: `diataxis: {mode_value}` is not a valid mode. "
                    f"Valid values: {valid}.",
                )
            )

    # arc52 check (only if present).
    if "arc52" in parsed:
        arc52_value = parsed["arc52"]
        # Must be a list of strings drawn from VALID_ARC52_SECTIONS.
        if not isinstance(arc52_value, list):
            report.violations.append(
                Violation(
                    ERROR,
                    "FRONTMATTER_INVALID_ARC52",
                    f"Line 1: `arc52:` must be a YAML list, got "
                    f"{type(arc52_value).__name__}",
                )
            )
        else:
            bad: list[str] = []
            for entry in arc52_value:
                if not isinstance(entry, str) or entry not in VALID_ARC52_SECTIONS:
                    bad.append(repr(entry))
            if bad:
                valid = ", ".join(sorted(VALID_ARC52_SECTIONS))
                report.violations.append(
                    Violation(
                        ERROR,
                        "FRONTMATTER_INVALID_ARC52",
                        f"Line 1: `arc52:` contains invalid entries "
                        f"{bad}. Valid sections: {valid}.",
                    )
                )

        # arc52 is only appropriate on architecture-shaped docs. Warn if not.
        # Caller (scan_file) supplies the right value via is_architecture_shaped;
        # this function does not derive it from the path because unit tests
        # invoke it with synthetic paths that don't carry that signal.
        if not is_architecture_shaped:
            report.violations.append(
                Violation(
                    WARNING,
                    "FRONTMATTER_ARC52_OUT_OF_PLACE",
                    "Line 1: `arc52:` is declared but the file does not "
                    "live under an `architecture/` subfolder. arc52 is "
                    "reserved for architecture-shaped docs.",
                )
            )


def count_procedural_markers(body: str) -> int:
    """Count how many reader-directed procedural markers appear in `body`.

    Used by the simple mode-vs-content heuristic. Returns the number of
    distinct patterns that matched at least once (a count of patterns, not a
    count of matches) — so a doc with ten "First," sentences still counts as
    one match. The chronicler-docs-hygiene skill does the deeper check.
    """
    return sum(1 for pattern in PROCEDURAL_MARKERS if pattern.search(body))


def check_mode_content_heuristic(report: FileReport, fm: FrontmatterResult) -> None:
    """Best-effort heuristic: does the declared mode match simple body signals?

    Warn-only. Three cases per ticket 04 scope item 5:
      - Declared `reference` but body has reader-directed procedural steps.
      - Declared `tutorial` but body has no procedural steps.
      - Declared `how-to` but body has no procedural steps.

    `explanation` is intentionally not checked by this heuristic: the inverse
    signal ("why" language) is too noisy to detect mechanically, and ticket 05
    covers the deep semantic check.
    """
    if not fm.present or fm.parsed is None or "diataxis" not in fm.parsed:
        return
    mode = fm.parsed["diataxis"]
    if not isinstance(mode, str) or mode not in VALID_DIATAXIS_MODES:
        return

    text = read_text(report)
    if text is None:
        return
    body = body_text(text, fm)
    if not body.strip():
        return

    marker_hits = count_procedural_markers(body)

    if mode == "reference" and marker_hits > 0:
        report.violations.append(
            Violation(
                WARNING,
                "MODE_CONTENT_MISMATCH",
                f"Declared `diataxis: reference` but body contains "
                f"{marker_hits} reader-directed procedural marker(s) "
                f"(e.g. \"Let's\", \"First,\", \"You should\"). Reference "
                f"docs should be neutral and information-oriented; "
                f"procedural steps belong in tutorial or how-to.",
            )
        )
    elif mode == "tutorial" and marker_hits == 0:
        report.violations.append(
            Violation(
                WARNING,
                "MODE_CONTENT_MISMATCH",
                "Declared `diataxis: tutorial` but body has no reader-directed "
                "procedural markers. Tutorials are learning-oriented and "
                "should walk the reader through steps.",
            )
        )
    elif mode == "how-to" and marker_hits == 0:
        report.violations.append(
            Violation(
                WARNING,
                "MODE_CONTENT_MISMATCH",
                "Declared `diataxis: how-to` but body has no reader-directed "
                "procedural markers. How-to guides are goal-oriented and "
                "should give the reader steps to follow.",
            )
        )


# ---------------------------------------------------------------------------
# Scanning and rendering.
# ---------------------------------------------------------------------------


def scan_file(
    path: Path,
    engine_root: Path,
) -> FileReport:
    """Scan a single .md file with all checks.

    `engine_root` is the chronicler_engine/ directory; the file's parent path
    is used to determine which docs root it lives under. This script only
    scans files under `docs-diataxis/`; legacy `docs/` is scanned by
    `validate_docs.py`.
    """
    report = FileReport(path)
    rel_to_engine = relative_to(path, engine_root)
    if rel_to_engine is None or not rel_to_engine.parts:
        return report
    if rel_to_engine.parts[0] != "docs-diataxis":
        return report  # not under docs-diataxis; nothing to check

    docs_root = engine_root / "docs-diataxis"
    # Cross-tree ADR lookup: docs-diataxis files reference ADRs that live in
    # the legacy docs/adr/ directory.
    adr_dir = engine_root / "docs" / "adr"

    role = classify_file(path, docs_root)
    if role != "STANDARD":
        return report

    check_markdown_links(report, docs_root)
    check_adr_refs(report, adr_dir)
    check_standard_plan_links(report, docs_root)
    check_standard_body_references(report, docs_root)

    rel_to_docs = relative_to(path, docs_root)
    is_arch = (
        is_architecture_shaped(rel_to_docs) if rel_to_docs is not None else False
    )
    check_diataxis_frontmatter(report, is_architecture_shaped=is_arch)
    text = read_text(report)
    if text is not None:
        fm = parse_frontmatter(text)
        check_mode_content_heuristic(report, fm)

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
        f"{error_count} error(s), {warning_count} warning(s) across "
        f"{files_with_violations} file(s)"
    )
    lines.append("")
    lines.append("=" * 60)
    lines.append(f"Scanned {len(reports)} file(s)")
    lines.append(f"Summary: {summary}")
    return "\n".join(lines), error_count, warning_count, files_with_violations


# ---------------------------------------------------------------------------
# CLI.
# ---------------------------------------------------------------------------


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Validate markdown docs under chronicler_engine/docs-diataxis/."
        ),
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
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)

    engine_root = Path(__file__).parent.parent
    diataxis_root = engine_root / "docs-diataxis"

    if not diataxis_root.exists():
        print(f"Error: docs-diataxis/ not found: {diataxis_root}", file=sys.stderr)
        return 2

    if args.path:
        target = args.path.resolve()
        if not target.exists():
            print(f"Error: file not found: {args.path}", file=sys.stderr)
            return 2
        try:
            rel = target.relative_to(engine_root)
        except ValueError:
            rel = None
        if rel is None or not rel.parts or rel.parts[0] != "docs-diataxis":
            print(
                f"Error: --path file must be under docs-diataxis/: {args.path}",
                file=sys.stderr,
            )
            return 2
        files = [target]
    else:
        files = collect_markdown_files([diataxis_root], diataxis_root)
        if not files:
            print(f"No markdown files found under {diataxis_root}", file=sys.stderr)
            return 0

    reports = [scan_file(f, engine_root) for f in files]
    rendered, error_count, warning_count, _ = render_reports(reports, engine_root)
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
