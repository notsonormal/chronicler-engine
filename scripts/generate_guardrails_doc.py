"""Generate `docs/diataxis/reference/coding_standards/guardrails.md` from coding-standards sources.

Sources:
  - `src/lib.rs` clippy `#![deny(...)]` lints + `//` rationale comments
  - `arch-lint.toml` `[[deny-scope-dep]]` rows
  - `tests/infrastructure/guardrails/*.rs` `pub fn check_*` `///` doc comments

Default mode rewrites the three tables in place between sentinel comments.
`--check` exits non-zero if the committed doc would change or if any source is
missing its required comment.
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
SYN_START = "<!-- AUTO-GUARDRAILS: syn START -->"
SYN_END = "<!-- AUTO-GUARDRAILS: syn END -->"


def humanize(name: str) -> str:
    """Convert `check_import_ordering` or `guardrails_import_ordering` to a readable label."""
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

        m = re.match(r"clippy::([a-z_]+),?\s*$", line)
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


def parse_syn_table() -> list[tuple[str, str, str, int]]:
    """Return (rule_name, description, file, line) rows from guardrail check functions.

    Groups consecutive `///` lines into one doc block; the first non-empty doc
    line is the description. Emits exactly one row per `pub fn check_*` that
    follows a doc block. Errors if a `pub fn check_*` has no preceding doc.
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

            m = re.match(r"pub fn (check_[a-z_]+)\(", stripped)
            if m:
                fn_name = m.group(1)
                if last_doc_first is None:
                    print(
                        f"error: `{fn_name}` in {rel}:{i} lacks a `///` doc comment",
                        file=sys.stderr,
                    )
                    sys.exit(1)
                rows.append((fn_name, last_doc_first, str(rel), i))
                # Reset for the next function; a new doc block must start fresh.
                doc_block = []
                last_doc_first = None
                continue

            # Non-doc, non-fn line ends any in-progress doc block without a match.
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


def generate_doc() -> str:
    clippy_rows = parse_clippy_table()
    arch_rows = parse_arch_lint_table()
    syn_rows = parse_syn_table()

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
    content = replace_between(ARCH_LINT_START, ARCH_LINT_END, format_arch_lint_table(arch_rows))
    content = replace_between(SYN_START, SYN_END, format_syn_table(syn_rows))

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

    doc_path.write_text(new_content, encoding="utf-8")
    print(f"  Updated {DOC_REL}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
