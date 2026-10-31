"""Generate `docs/diataxis/reference/coding_standards/guardrails.md` from coding-standards sources.

Sources:
  - `src/lib.rs` clippy `#![deny(...)]` lints + `//` rationale comments
  - `arch-lint.toml` `[analyzer]` scan scope + `[[deny-scope-dep]]` rows
  - `tests/infrastructure/guardrails/*.rs` `pub fn check_*` `///` doc comments
  - `tests/infrastructure/guardrails/walker_rows.rs` design-rule row tables

Default mode rewrites the four tables in place between sentinel comments, and
writes the file only when the generated text differs from the text on disk.
`--check` exits non-zero if the committed doc would change or if any source is
missing its required comment.

The row tables use one strict form, so this script can read them. A row is one
struct literal whose type ends in `Row`: an opening `<Kind>Row {` line, one
`field: value,` line per field, a closing `},` line, and the row's reason on the
`//` comment lines directly above the opening line. Every value stays on one
line. A row with no reason, or a row with no `rule:` field, fails the check.
"""

from __future__ import annotations

import argparse
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).parent.parent
DOC_REL = "docs/diataxis/reference/coding_standards/guardrails.md"
LIB_RS = ROOT / "src/lib.rs"
ARCH_LINT_TOML = ROOT / "arch-lint.toml"
GUARDRAILS_DIR = ROOT / "tests/infrastructure/guardrails"

CLIPPY_START = "<!-- AUTO-GUARDRAILS: clippy START -->"
CLIPPY_END = "<!-- AUTO-GUARDRAILS: clippy END -->"
ARCH_LINT_START = "<!-- AUTO-GUARDRAILS: arch-lint START -->"
ARCH_LINT_END = "<!-- AUTO-GUARDRAILS: arch-lint END -->"

WALKER_ROWS_RS = GUARDRAILS_DIR / "walker_rows.rs"

ROW_OPEN_RE = re.compile(r"^(\s*)([A-Z][A-Za-z0-9]*)Row \{\s*$")
ROW_LIKE_RE = re.compile(r"^\s*[A-Z][A-Za-z0-9]*Row\b")
ROW_FIELD_RE = re.compile(r"^\s*([a-z_][a-z0-9_]*):\s*(\S.*?),\s*$")
ROW_CLOSE_RE = re.compile(r"^\s*\},\s*$")

ROW_RULE_RE = re.compile(r"^(\d+)$")
SYN_START = "<!-- AUTO-GUARDRAILS: syn START -->"
SYN_END = "<!-- AUTO-GUARDRAILS: syn END -->"


def humanize(name: str) -> str:
    name = name.removeprefix("check_").removeprefix("guardrails_")
    return name.replace("_", " ")


def parse_clippy_table() -> list[tuple[str, str]]:
    """Return (lint_name, rationale) rows from src/lib.rs #![deny(...)] block."""
    text = LIB_RS.read_text(encoding="utf-8")
    match = re.search(r"#!\[deny\((.*?)\)\]", text, re.DOTALL)
    if not match:
        print("error: could not find clippy #![deny(...)] block in src/lib.rs", file=sys.stderr)
        sys.exit(1)

    block = match.group(1)
    rows: list[tuple[str, str]] = []
    pending_rationale: str | None = None

    for raw_line in block.splitlines():
        line = raw_line.strip()
        if not line:
            continue

        if line.startswith("//"):
            pending_rationale = line.removeprefix("//").strip()
            continue

        m = re.match(r"clippy::([a-z0-9_]+),?\s*$", line)
        if m:
            lint = m.group(1)
            if pending_rationale is None:
                print(
                    f"error: clippy lint `{lint}` in src/lib.rs lacks a preceding `//` rationale",
                    file=sys.stderr,
                )
                sys.exit(1)
            rows.append((lint, pending_rationale))
            pending_rationale = None

    return rows


def format_clippy_table(rows: list[tuple[str, str]]) -> str:
    lines = [
        "| Lint | Rationale |",
        "|------|-----------|",
    ]
    for lint, rationale in rows:
        lines.append(f"| `clippy::{lint}` | {rationale} |")
    return "\n".join(lines)


def parse_arch_lint_table() -> list[tuple[str, str, str]]:
    """Return (from_scope, to_scopes, rationale) rows from arch-lint.toml."""
    with ARCH_LINT_TOML.open("rb") as f:
        data = tomllib.load(f)

    rows: list[tuple[str, str, str]] = []
    for dep in data.get("deny-scope-dep", []):
        from_scope = dep.get("from", "")
        to_scopes = ", ".join(dep.get("to", []))
        rationale = dep.get("message", "")
        rows.append((from_scope, to_scopes, rationale))
    return rows


def format_arch_lint_table(rows: list[tuple[str, str, str]]) -> str:
    lines = [
        "| From Scope | To Scope(s) | Rationale |",
        "|------------|-------------|----------|",
    ]
    for from_scope, to_scopes, rationale in rows:
        lines.append(f"| `{from_scope}` | `{to_scopes}` | {rationale} |")
    return "\n".join(lines)


def parse_analyzer_scope() -> str:
    """Return the arch-lint scan scope line from `[analyzer]` in arch-lint.toml.

    The scope is the fact an agent needs before deciding whether a guardrail
    applies to a given file: which tree is scanned and what is skipped. It is
    emitted from the config, so the reference doc cannot disagree with the
    linter it describes.
    """
    with ARCH_LINT_TOML.open("rb") as f:
        analyzer = tomllib.load(f).get("analyzer")
    if not analyzer:
        print("error: arch-lint.toml has no [analyzer] section", file=sys.stderr)
        sys.exit(1)

    root = analyzer.get("root", "")
    excludes = analyzer.get("exclude", [])
    if not root or not excludes:
        print(
            "error: arch-lint.toml [analyzer] must set both `root` and `exclude`",
            file=sys.stderr,
        )
        sys.exit(1)

    excluded = ", ".join(f"`{pattern}`" for pattern in excludes)
    return f'Scan scope: `root = "{root}"`, excluding {excluded}.'


def format_arch_lint_block(scope: str, rows: list[tuple[str, str, str]]) -> str:
    return f"{scope}\n\n{format_arch_lint_table(rows)}"


def parse_syn_table() -> list[tuple[str, str, str, int]]:
    """Return (rule_name, description, file, line) rows from guardrail check functions.

    Groups consecutive `///` lines into one doc block; the first non-empty doc
    line is the description. Emits exactly one row per `pub fn check_*` that
    follows a doc block. Errors if a `pub fn check_*` has no preceding doc.

    The name pattern is `\\w+`, not `[a-z_]+`: a digit in a rule name (e.g.
    `check_tier3_rule`) must not make the rule invisible to the generated doc.
    """
    rows: list[tuple[str, str, str, int]] = []

    for rs_file in sorted(GUARDRAILS_DIR.glob("*.rs")):
        if rs_file.name == "mod.rs":
            continue
        lines = rs_file.read_text(encoding="utf-8").splitlines()
        rel = rs_file.relative_to(ROOT)

        doc_block: list[str] = []  # accumulated /// lines for the current block
        last_doc_first: str | None = None  # first non-empty doc line of current block

        for i, line in enumerate(lines, start=1):
            stripped = line.strip()
            if stripped.startswith("///"):
                text = stripped.removeprefix("///").strip()
                if not doc_block:
                    last_doc_first = text or None
                doc_block.append(text)
                continue

            m = re.match(r"pub fn (check_\w+)\(", stripped)
            if m:
                fn_name = m.group(1)
                if last_doc_first is None:
                    print(
                        f"error: `{fn_name}` in {rel}:{i} lacks a `///` doc comment",
                        file=sys.stderr,
                    )
                    sys.exit(1)
                rows.append((fn_name, last_doc_first, str(rel), i))
                doc_block = []
                last_doc_first = None
                continue

            if stripped:
                doc_block = []
                last_doc_first = None

    return rows


def format_syn_table(rows: list[tuple[str, str, str, int]]) -> str:
    lines = [
        "| Rule | Description | Source |",
        "|------|-------------|--------|",
    ]
    for fn_name, desc, rel, line in rows:
        rule = humanize(fn_name)
        lines.append(f"| {rule} | {desc} | `{rel}:{line}` |")
    return "\n".join(lines)


def humanize_kind(row_type: str) -> str:
    """Render a row type's name as the walker kind it configures."""
    kind = row_type.removesuffix("Row")
    spaced = re.sub(r"(?<!^)(?=[A-Z])", " ", kind)
    return spaced.lower()


def parse_walker_rows() -> list[tuple[int, str, str, int]]:
    """Return (rule, kind, reason, line) rows from the design-rule row tables.

    The form is the strict one in this module's docstring. A row whose comment
    block is missing is an error: a rule with no reason is a rule nobody can
    re-point or retire confidently.
    """
    rows: list[tuple[int, str, str, int]] = []
    lines = WALKER_ROWS_RS.read_text(encoding="utf-8").splitlines()

    comment_block: list[str] = []
    index = 0
    while index < len(lines):
        stripped = lines[index].strip()
        if stripped.startswith("//"):
            comment_block.append(stripped.lstrip("/").strip())
            index += 1
            continue

        opening = ROW_OPEN_RE.match(lines[index])
        if not opening:
            if ROW_LIKE_RE.match(lines[index]):
                print(
                    f"error: the row at {WALKER_ROWS_RS}:{index + 1} is not in the"
                    " row form — one `<Kind>Row {` line, one `field: value,` line"
                    " per field, and a closing `},` line",
                    file=sys.stderr,
                )
                sys.exit(1)
            if stripped:
                comment_block = []
            index += 1
            continue

        kind = humanize_kind(opening.group(2))
        opening_line = index + 1
        rule: int | None = None
        index += 1
        while True:
            if index >= len(lines):
                print(
                    f"error: the {kind} row at {WALKER_ROWS_RS}:{opening_line}"
                    " has no closing `},` line",
                    file=sys.stderr,
                )
                sys.exit(1)
            if ROW_CLOSE_RE.match(lines[index]):
                break
            field = ROW_FIELD_RE.match(lines[index])
            if not field:
                print(
                    f"error: {WALKER_ROWS_RS}:{index + 1} is not a `field: value,`"
                    " line of the row it sits in",
                    file=sys.stderr,
                )
                sys.exit(1)
            if field.group(1) == "rule":
                rule_match = ROW_RULE_RE.match(field.group(2).strip())
                if not rule_match:
                    print(
                        f"error: row at {WALKER_ROWS_RS}:{opening_line} has a"
                        " non-numeric `rule:` field",
                        file=sys.stderr,
                    )
                    sys.exit(1)
                rule = int(rule_match.group(1))
            index += 1
        index += 1

        reason = " ".join(part for part in comment_block if part)
        if not reason:
            print(
                f"error: the {kind} row at {WALKER_ROWS_RS}:{opening_line} has no"
                " reason — put the rule's reason on the comment lines directly"
                " above it",
                file=sys.stderr,
            )
            sys.exit(1)
        if rule is None:
            print(
                f"error: the {kind} row at {WALKER_ROWS_RS}:{opening_line} has no"
                " `rule:` field",
                file=sys.stderr,
            )
            sys.exit(1)

        rows.append((rule, kind, reason, opening_line))
        comment_block = []

    return rows


def format_walker_rows_table(rows: list[tuple[int, str, str, int]]) -> str:
    lines = [
        "| Rule | Kind | Reason | Source |",
        "|------|------|--------|--------|",
    ]
    if not rows:
        lines.append(
            "| — | — | No row exists yet. | — |"
        )
        return "\n".join(lines)

    rel = WALKER_ROWS_RS.relative_to(ROOT)
    for rule, kind, reason, line in rows:
        lines.append(f"| {rule} | {kind} | {reason} | `{rel}:{line}` |")
    return "\n".join(lines)


def generate_doc() -> str:
    clippy_rows = parse_clippy_table()
    arch_scope = parse_analyzer_scope()
    arch_rows = parse_arch_lint_table()
    syn_rows = parse_syn_table()
    walker_rows = parse_walker_rows()

    doc_path = ROOT / DOC_REL
    content = doc_path.read_text(encoding="utf-8")

    def replace_between(start_marker: str, end_marker: str, new_body: str) -> str:
        pattern = re.compile(
            re.escape(start_marker) + r".*?" + re.escape(end_marker),
            re.DOTALL,
        )
        if not pattern.search(content):
            print(
                f"error: could not find sentinel pair {start_marker} ... {end_marker} in {DOC_REL}",
                file=sys.stderr,
            )
            sys.exit(1)
        replacement = f"{start_marker}\n{new_body}\n{end_marker}"
        return pattern.sub(replacement, content, count=1)

    content = replace_between(CLIPPY_START, CLIPPY_END, format_clippy_table(clippy_rows))
    content = replace_between(
        ARCH_LINT_START, ARCH_LINT_END, format_arch_lint_block(arch_scope, arch_rows)
    )
    content = replace_between(
        SYN_START,
        SYN_END,
        f"{format_walker_rows_table(walker_rows)}\n\n{format_syn_table(syn_rows)}",
    )

    return content


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate or verify guardrails.md")
    parser.add_argument(
        "--check",
        action="store_true",
        help="Exit non-zero if the committed doc would change or if required comments are missing.",
    )
    args = parser.parse_args()

    new_content = generate_doc()
    doc_path = ROOT / DOC_REL

    if args.check:
        current = doc_path.read_text(encoding="utf-8")
        if current != new_content:
            print(f"error: {DOC_REL} is stale; run `python scripts/generate_guardrails_doc.py`", file=sys.stderr)
            return 1
        print(f"  {DOC_REL} is up to date")
        return 0

    if doc_path.read_text(encoding="utf-8") == new_content:
        print(f"  Unchanged {DOC_REL}")
        return 0
    doc_path.write_text(new_content, encoding="utf-8")
    print(f"  Updated {DOC_REL}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
