"""Validate markdown docs + DOC anchors under chronicler_engine/.

Role-aware deterministic checks. Each .md file is classified into one of three
roles which determine which rules apply:

  STANDARD  — canonical spec docs (reference/explanation/how-to/tutorials
              under docs/diataxis/). All rules enforced.
  TRANSIENT — historical or forward-looking docs (CHANGELOG, plans). Exempt
              from all checks; they may reference anything historically.
  EXCLUDED  — auto-generated indexes, archives, process notes
              (AGENTS.md, _PILOT_NOTES.md). Exempt from all checks.

Rules (only enforced on STANDARD docs):

  BROKEN_MARKDOWN_LINK     — [text](relative/path.md) where target file missing.
  STANDARD_PLAN_LINK       — link into docs/plans/ or old-docs/archived-plans/.
                              Standards must be self-contained; plans are transient
                              and cannot be leaned on as canonical reference.
  STANDARD_DOC_BODY_REFERENCE — doc-internal reference in body prose (before the
                              file's `## Document References` section). Catches any
                              `.md` mention before that section, in any of:
                                (a) `[text](relative/path.md)` markdown-link form
                                (b) backtick form `` `path.md` `` where the content
                                    is a path-like token ending in `.md`
                                (c) plain §Section text on a line whose earlier text
                                    contains a `.md` path
                              All forms must appear only in the `## Document
                              References` section at the bottom of the file.

Diátaxis-front-matter rules (only enforced on STANDARD docs under
docs/diataxis/):

  MISSING_FRONTMATTER          — no YAML front-matter (`---` block) at top of file.
  EMPTY_FRONTMATTER            — front-matter block present but parses to empty.
  FRONTMATTER_PARSE_ERROR      — YAML does not parse.
  FRONTMATTER_NOT_MAPPING      — YAML parses but the root is not a mapping.
  FRONTMATTER_MISSING_KEY      — required key (`diataxis:` or `title:`) absent.
  FRONTMATTER_INVALID_MODE     — `diataxis:` value not in the vocabulary.
  FRONTMATTER_INVALID_ARC52    — `arc52:` present but not a list of valid sections.

Mirrors the validator structure (Violation NamedTuple, per-file report,
summary line at bottom). Intentionally decoupled from the
`chronicler-docs-hygiene` skill — skill handles LLM semantic analysis, this
script handles deterministic checks. CI / pre-commit can run this in <1s.

DOC anchor rules (applied to src/**/*.rs, tests/**/*.rs, *.toml):

  BROKEN_DOC_ANCHOR              — `[DOC: <path>.md]` target file missing,
                                  OR path not under
                                  `chronicler_engine/docs/diataxis/reference/`
                                  (two message variants of one rule).
  TEST_SUPPORT_ANCHOR_FORBIDDEN  — any `src/test_support/*.rs` carrying a
                                  `[DOC: ...]` line. Test helpers are
                                  organised by fixture weight.
  TEST_FILES_ANCHOR_FORBIDDEN    — any `tests/**/*.rs` carrying a
                                  `[DOC: ...]` line.
  TEST_SUPPORT_SUMMARY_REQUIRED  — `src/test_support/*.rs` line 1 is missing
                                  or empty `//! <summary>`.

The validator does NOT support the `— section "..."` suffix form. Anchors are
path-only. Anchors are scanned unconditionally — there is no opt-out flag.

Usage:
    python scripts/validate_docs.py
    python scripts/validate_docs.py --strict
    python scripts/validate_docs.py --list
    python scripts/validate_docs.py --path reference/data_layer.md
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

# Heading regex for finding the `## Document References` section boundary.
# Matches `## Document References` at start of line (any leading whitespace).
DOC_REFERENCES_HEADING = re.compile(r"^\s*##\s+Document\s+References\s*$")

# Backtick span: `…`. Captures the content between backticks. Body-prose backtick
# refs only fire when the captured content is path-like and ends in `.md`
# (see _is_md_path_in_backticks below). Naive (`[^`\n]+`) — multi-line backtick
# spans in body prose are vanishingly rare.
BACKTICK_SPAN = re.compile(r"`([^`\n]+)`")

# Plain §Section token. Fires only when the same body-prose line contains a
# `.md` path earlier (see check_standard_body_references). Captures the §Token
# plus optional multi-word section names that include dashes, quotes, and
# underscores; stops at punctuation, periods, or end of line.
SECTION_TOKEN = re.compile(r"§[\w\"]+(?:\s+[\w\-\"]+)*")

# Plain path-shaped .md token on a body-prose line. Used only to find the
# document a § token refers to (the preceding path on the same line). Resolves
# via the same inside-docs_root check as MARKDOWN_LINK.
LINE_MD_TOKEN = re.compile(r"[A-Za-z0-9_./-]+\.md")

# Fenced code block delimiter (``` or ~~~), optionally with language tag.
FENCE_DELIMITER = re.compile(r"^\s*(```|~~~)")

# Front-matter delimiter: a line that is exactly `---` (optional whitespace).
FRONTMATTER_DELIMITER = re.compile(r"^---\s*$")

# DOC anchor presence on a line: `[DOC: ...]` in any form. Used to detect
# forbidden anchors in test_support/tests paths where the strict path-parsing
# regex would miss malformed or suffix-form anchors. Anchored to line start
# (after an optional `//!` / `//` / `#` comment prefix) so mid-line mentions in
# format strings or prose doc-comments inside guardrail code are not mistaken
# for real anchors.
DOC_ANCHOR_LINE = re.compile(r"^\s*(?://!|//|#)?\s*\[DOC:[^]]*\]")

# DOC anchor strict regex: `[DOC: <path>.md]`. Path-only, no section suffix.
# Capture group 1 is the target path, which must end in `.md`. Char class
# excludes the em dash so the legacy `— section "..."` suffix form simply
# will not match. Anchored to line start (after an optional `//!` / `//` / `#`
# comment prefix) so format-string literals mentioning `[DOC: ...]` inside
# test guardrails are not mistaken for real anchors.
DOC_ANCHOR = re.compile(r"^\s*(?://!|//|#)?\s*\[DOC:\s+([a-zA-Z0-9_/.\\-]+\.md)\s*\]")

# Paths whose existence we never check (network, mail, in-page anchors).
URI_SCHEMES = ("http://", "https://", "mailto:")

# ---------------------------------------------------------------------------
# Diátaxis front-matter vocabulary.
# ---------------------------------------------------------------------------

VALID_DIATAXIS_MODES: frozenset[str] = frozenset(
    {"tutorial", "how-to", "reference", "explanation"}
)

VALID_ARC52_SECTIONS: frozenset[str] = frozenset({"§3", "§5", "§7", "§10"})

# ---------------------------------------------------------------------------
# Heuristic patterns for the simple mode-vs-content check (warn-only).
# ---------------------------------------------------------------------------

# ---------------------------------------------------------------------------
# File / directory classification.
# ---------------------------------------------------------------------------

# Files exempt from all checks regardless of tree.
EXCLUDED_FILE_NAMES: set[str] = {
    "AGENTS.md",
    "README.md",
    "_PILOT_NOTES.md",  # process artifact, not a Diátaxis doc
}

# Directories whose entire subtree is exempt (archives, etc.).
EXCLUDED_DIR_NAMES: set[str] = {"old-docs"}

# Transient file paths: exempt because they are historical or forward-looking
# logs, not canonical specs.
TRANSIENT_FILE_PATHS: tuple[str, ...] = (
    "CHANGELOG.md",
    "plans",
)

# Standard dir names. docs/diataxis/ uses reference/explanation/how-to/
# (and tutorials/ once content earns it).
STANDARD_DIR_NAMES: set[str] = {
    "reference",
    "explanation",
    "how-to",
    "tutorials",
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
      2. Any path segment in EXCLUDED_DIR_NAMES → EXCLUDED
      3. First segment is 'plans' OR path is 'CHANGELOG.md' → TRANSIENT
      4. First segment in STANDARD_DIR_NAMES → STANDARD
      5. Otherwise → EXCLUDED (unrecognized top-level doc — be permissive)
    """
    rel = relative_to(path, docs_root)
    if rel is None:
        return "EXCLUDED"

    parts = rel.parts
    if not parts:
        return "EXCLUDED"

    if parts[-1] in EXCLUDED_FILE_NAMES:
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
    """True if `rel_to_engine` lives under engine_root/docs/diataxis/."""
    return (
        len(rel_to_engine.parts) >= 2
        and rel_to_engine.parts[0] == "docs"
        and rel_to_engine.parts[1] == "diataxis"
    )


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


def _is_md_path_in_backticks(token: str) -> bool:
    """True if a backtick-wrapped token looks like a `.md` doc-path, not code.

    Accepts:
      - paths with explicit prefix: `./foo.md`, `../foo/bar.md`, `/abs/foo.md`
      - bare filenames ending in `.md` whose stem has no dot/slash (so
        `startup.md` and `2026-q3-notes.md` match, but `AppState.foo.md`
        and `example.com/foo.md` do not).
    """
    if not token.endswith(".md") or len(token) < 4:
        return False
    if token.startswith(URI_SCHEMES):
        return False
    if token.startswith(("./", "../", "/")):
        return True
    stem = token[:-3]
    if not stem:
        return False
    return all(c.isalnum() or c in "_-" for c in stem)


def check_standard_body_references(report: FileReport, docs_root: Path) -> None:
    """Flag doc-internal references in body prose.

    Body prose = lines before the file's `## Document References` section
    (or the entire file if no such section exists). Fenced code blocks
    inside body prose are exempt.

    Fires on any of the following in body prose, when the resolved target is
    inside the docs root:

      (a) Markdown link form: `[text](path.md)`.
      (b) Backtick form: `` `path.md` `` (backtick content is path-shaped;
          see _is_md_path_in_backticks).
      (c) Plain §Section text where the same line earlier in the text
          contains a `.md` path token (resolves via that preceding path).

    Forms (a) and (b) are primary: if either fires on a line, form (c) is
    skipped for that line to avoid noise (the existing markdown-link
    dedup is preserved). Form (c) can fire multiple times on a single line when
    several different § tokens appear with the same preceding path.
    """
    try:
        rel_to_docs = report.path.resolve().relative_to(docs_root.resolve())
    except ValueError:
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

    def _line_has_flag() -> bool:
        return any(
            v.rule == "STANDARD_DOC_BODY_REFERENCE"
            and v.message.startswith(f"Line {lineno}:")
            for v in report.violations
        )

    in_fence = False
    for lineno, line in enumerate(lines, start=1):
        if lineno >= body_end_lineno:
            break
        if FENCE_DELIMITER.match(line):
            in_fence = not in_fence
            continue
        if in_fence:
            continue

        # Stage 1: markdown-link form `[text](path.md)`.
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

        if _line_has_flag():
            # Existing dedup: a markdown link on this line already flags the
            # same conceptual issue; skip the secondary forms.
            continue

        # Stage 2: backtick form `` `path.md` ``.
        for match in BACKTICK_SPAN.finditer(line):
            inner = match.group(1).strip()
            if not _is_md_path_in_backticks(inner):
                continue
            resolved = (report.path.parent / inner).resolve()
            try:
                resolved.relative_to(docs_root_resolved)
            except ValueError:
                continue
            report.violations.append(
                Violation(
                    ERROR,
                    "STANDARD_DOC_BODY_REFERENCE",
                    f"Line {lineno}: doc-internal backtick ref "
                    f"`{inner}` appears in body prose. Move to "
                    f"`## Document References` section at the bottom of the "
                    f"file.",
                )
            )

        if _line_has_flag():
            # Backtick ref already flags this line; skip § form.
            continue

        # Stage 3: plain §Section text. Only fires when the same line has a
        # `.md` path token earlier; the path is what the § resolves to.
        line_md_tokens = list(LINE_MD_TOKEN.finditer(line))
        if line_md_tokens:
            for sect_match in SECTION_TOKEN.finditer(line):
                preceding_token = None
                for cand in line_md_tokens:
                    if cand.start() < sect_match.start():
                        preceding_token = cand
                if preceding_token is None:
                    continue
                path = preceding_token.group(0)
                resolved = (report.path.parent / path).resolve()
                try:
                    resolved.relative_to(docs_root_resolved)
                except ValueError:
                    continue
                # Trim trailing sentence punctuation from the § token for the
                # message; the regex is greedy and would otherwise report
                # `§Messages.` or `§Rel` etc.
                section = sect_match.group(0).rstrip(".,;:!?")
                report.violations.append(
                    Violation(
                        ERROR,
                        "STANDARD_DOC_BODY_REFERENCE",
                        f"Line {lineno}: body-prose `{section}` cross-ref "
                        f"preceded by `{path}` on the same line. Move to "
                        f"`## Document References` section at the bottom of "
                        f"the file.",
                    )
                )


# ---------------------------------------------------------------------------
# Diátaxis front-matter checks.
# ---------------------------------------------------------------------------


def check_diataxis_frontmatter(report: FileReport) -> None:
    """Enforce YAML front-matter conventions on docs/diataxis/ STANDARD docs.

    Emits MISSING_FRONTMATTER, EMPTY_FRONTMATTER, FRONTMATTER_PARSE_ERROR,
    FRONTMATTER_NOT_MAPPING, FRONTMATTER_MISSING_KEY, FRONTMATTER_INVALID_MODE,
    and FRONTMATTER_INVALID_ARC52 violations as appropriate. See module
    docstring for the rule set.
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
                "Every docs/diataxis/ doc must declare `diataxis:` and `title:`.",
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

        # FRONTMATTER_ARC52_OUT_OF_PLACE used to live here: `arc52:` on a
        # non-architecture doc was warned. Removed because the architecture
        # doc moved to `explanation/architecture.md` (flat, no subfolder).


# ---------------------------------------------------------------------------
# DOC anchor checks.
# ---------------------------------------------------------------------------


def check_doc_anchors(report: FileReport, path: Path, engine_root: Path) -> None:
    """Emit BROKEN_DOC_ANCHOR for src/ (non-test_support) and top-level *.toml.

    Two violation variants under one rule name:
      * Path-form variant: target not under
        `chronicler_engine/docs/diataxis/reference/`.
      * Target-missing variant: target file does not exist on disk.

    `src/test_support/**` and `tests/**` paths are handled by
    `check_test_support_rules`; here we skip them to avoid double-counting.
    """
    rel = relative_to(path, engine_root)
    if rel is None:
        return
    parts = rel.parts
    if parts[:1] == ("tests",):
        return
    if "test_support" in parts:
        return

    text = read_text(report)
    if text is None:
        return

    repo_root = engine_root.parent
    reference_root = (repo_root / "chronicler_engine" / "docs" / "diataxis" / "reference").resolve()
    for lineno, line in enumerate(text.splitlines(), start=1):
        for match in DOC_ANCHOR.finditer(line):
            target = match.group(1)
            resolved = (repo_root / target).resolve()
            try:
                resolved.relative_to(reference_root)
            except ValueError:
                report.violations.append(
                    Violation(
                        ERROR,
                        "BROKEN_DOC_ANCHOR",
                        f"Line {lineno}: DOC anchor must resolve under chronicler_engine/docs/diataxis/reference/: `{target}` ({rel})",
                    )
                )
                continue
            if not resolved.is_file():
                report.violations.append(
                    Violation(
                        ERROR,
                        "BROKEN_DOC_ANCHOR",
                        f"Line {lineno}: DOC anchor target file does not exist: `{target}` ({rel})",
                    )
                )


def check_test_support_rules(report: FileReport, path: Path, engine_root: Path) -> None:
    """Emit TEST_SUPPORT_ANCHOR_FORBIDDEN / TEST_FILES_ANCHOR_FORBIDDEN /
    TEST_SUPPORT_SUMMARY_REQUIRED for src/test_support/*.rs and tests/**/*.rs.
    """
    rel = relative_to(path, engine_root)
    if rel is None:
        return
    parts = rel.parts

    is_test_support = parts[:1] == ("src",) and "test_support" in parts
    is_test_file = parts[:1] == ("tests",)
    if not (is_test_support or is_test_file):
        return

    # Mirror the Rust guardrail's MODULE_DOC_EXEMPTIONS: lib.rs/main.rs/mod.rs
    # are module-declaration files, not source files that need a summary.
    is_module_decl = path.name in {"lib.rs", "main.rs", "mod.rs"}

    text = read_text(report)
    if text is None:
        return

    for lineno, line in enumerate(text.splitlines(), start=1):
        if DOC_ANCHOR_LINE.search(line):
            rule = (
                "TEST_SUPPORT_ANCHOR_FORBIDDEN"
                if is_test_support
                else "TEST_FILES_ANCHOR_FORBIDDEN"
            )
            report.violations.append(
                Violation(
                    ERROR,
                    rule,
                    f"Line {lineno}: `{rule}`: this file must not carry a `[DOC: ...]` line ({rel})",
                )
            )

    if is_test_support and not is_module_decl:
        lines = text.splitlines()
        if not lines or not lines[0].strip().startswith("//!"):
            report.violations.append(
                Violation(
                    ERROR,
                    "TEST_SUPPORT_SUMMARY_REQUIRED",
                    f"Line 1: `src/test_support/*.rs` must start with a `//! <summary>` line ({rel})",
                )
            )
        else:
            after = lines[0].strip()[3:].strip()
            if not after:
                report.violations.append(
                    Violation(
                        ERROR,
                        "TEST_SUPPORT_SUMMARY_REQUIRED",
                        f"Line 1: `src/test_support/*.rs` line 1 is an empty `//!`; must be a `//! <summary>` line ({rel})",
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
    scans files under `docs/diataxis/`.
    """
    report = FileReport(path)
    rel_to_engine = relative_to(path, engine_root)
    if rel_to_engine is None or not rel_to_engine.parts:
        return report
    if not is_diataxis_tree_path(rel_to_engine):
        return report  # not under docs/diataxis/; nothing to check

    docs_root = engine_root / "docs" / "diataxis"

    role = classify_file(path, docs_root)
    if role != "STANDARD":
        return report

    check_markdown_links(report, docs_root)
    check_standard_plan_links(report, docs_root)
    check_standard_body_references(report, docs_root)

    check_diataxis_frontmatter(report)

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
            "Validate markdown docs under chronicler_engine/docs/diataxis/."
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


def scan_anchor_file(path: Path, engine_root: Path) -> FileReport:
    """Doc-anchor scan for a single .rs/.toml file."""
    report = FileReport(path)
    check_doc_anchors(report, path, engine_root)
    check_test_support_rules(report, path, engine_root)
    return report


def collect_anchor_files(engine_root: Path) -> list[Path]:
    """Collect .rs files under src/ and tests/, plus top-level *.toml files."""
    files: set[Path] = set()
    for pat in ("src/**/*.rs", "tests/**/*.rs"):
        files.update(p.resolve() for p in engine_root.glob(pat) if p.is_file())
    for p in engine_root.glob("*.toml"):
        if p.is_file():
            files.add(p.resolve())
    return sorted(files)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)

    engine_root = Path(__file__).parent.parent
    diataxis_root = engine_root / "docs" / "diataxis"

    if not diataxis_root.exists():
        print(f"Error: docs/diataxis/ not found: {diataxis_root}", file=sys.stderr)
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
        if rel is None or not rel.parts[:2] == ["docs", "diataxis"]:
            print(
                f"Error: --path file must be under docs/diataxis/: {args.path}",
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

    md_errors, md_warnings = error_count, warning_count

    anchor_files = collect_anchor_files(engine_root)
    if anchor_files:
        anchor_reports = [scan_anchor_file(f, engine_root) for f in anchor_files]
        anchor_render, anchor_err, anchor_warn, _ = render_reports(
            anchor_reports, engine_root
        )
        print(anchor_render)
        md_errors += anchor_err
        md_warnings += anchor_warn

    if args.list_only:
        return 0

    if md_errors > 0:
        print("\nFAIL — see violations above.")
        return 1
    if args.strict and md_warnings > 0:
        print("\nFAIL (--strict) — warnings present.")
        return 1

    print("\nPASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
