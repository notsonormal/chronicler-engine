"""Validate the links of the work records under `work/`, and the `work/` paths that docs and skills cite.

The rules of the records are in `docs/agents/issue-tracker.md`. Two checks,
both errors:

  BROKEN_RECORD_LINK  — a relative link in a live record does not resolve.
                        The live records are each `map.md`, the other `.md`
                        beside it, the tickets of a map, the one-off issues,
                        the wiki and the plans. Assets and `work/_closed/`
                        are history and are not checked. Links inside fenced
                        code and HTML comments are not links, so the
                        `(link)` placeholder of the wayfinder template
                        passes.
  STALE_WORK_PATH     — a backticked `work/…` path in `AGENTS.md`, `docs/`
                        or `.agents/skills/` does not exist. A path is cut
                        back to its part before a `<placeholder>`, a glob
                        or `…`, so `work/issues/<status>/` checks
                        `work/issues`.

An issue that changes status moves with `scripts/move_issue.py`, which
rewrites the links to it. A move by hand fails the first check.

Exit codes:
    0  every link and every cited path resolves
    1  a violation above

Run from anywhere:
    python scripts/validate_work_records.py
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path
from typing import NamedTuple

from work_records import BACKTICK_SPAN, STATUS_FOLDERS, citing_files, iter_links, link_resolves, lines_outside_fences

ENGINE_ROOT = Path(__file__).resolve().parent.parent
WORK_DIR_NAME = "work"

# A part that stands for many paths: a placeholder, a glob, an ellipsis, or the `NN` of an issue number.
PATTERN_PART = re.compile(r"[<>*{}\[\]]|\.\.\.|…|^NN")
WORK_PATH_TOKEN = re.compile(r"(?<![\w/.-])(work(?:/[^\s`'\"),;:]*)?)")


class RecordViolation(NamedTuple):
    rule: str
    path: Path
    line: int
    detail: str


def live_record_files(root: Path) -> list[Path]:
    """The live records of `work/`: maps, the files beside them, tickets, one-off issues, the wiki and the plans."""
    work = root / WORK_DIR_NAME
    files: set[Path] = set()
    files.update(work.glob("wayfinder-maps/*/*.md"))
    files.update(work.glob("wayfinder-maps/*/issues/*.md"))
    for folder in STATUS_FOLDERS:
        files.update(work.glob(f"issues/{folder}/*.md"))
    files.update(work.glob("wiki/**/*.md"))
    files.update(work.glob("plans/**/*.md"))
    return sorted(path for path in files if path.is_file())


def check_record_links(file: Path, root: Path) -> list[RecordViolation]:
    text = file.read_text(encoding="utf-8")
    return [
        RecordViolation("BROKEN_RECORD_LINK", file.relative_to(root), link.line, link.target)
        for link in iter_links(text)
        if not link_resolves(file, link.target)
    ]


def checked_prefix(token: str) -> str:
    """The part of a cited path before its first placeholder, glob or ellipsis part, with no trailing `/`."""
    kept: list[str] = []
    for part in token.rstrip("/").split("/"):
        if not part or PATTERN_PART.search(part):
            break
        kept.append(part)
    return "/".join(kept)


def check_cited_work_paths(file: Path, root: Path) -> list[RecordViolation]:
    try:
        text = file.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        return []
    violations: list[RecordViolation] = []
    for line in lines_outside_fences(text):
        for span in BACKTICK_SPAN.finditer(line.text):
            for token in WORK_PATH_TOKEN.finditer(span.group(1)):
                prefix = checked_prefix(token.group(1))
                if not (root / prefix).exists():
                    violations.append(
                        RecordViolation("STALE_WORK_PATH", file.relative_to(root), line.number, token.group(1))
                    )
    return violations


def validate(root: Path, records: list[Path] | None = None) -> list[RecordViolation]:
    violations: list[RecordViolation] = []
    for file in live_record_files(root) if records is None else records:
        violations.extend(check_record_links(file, root))
    for file in citing_files(root):
        violations.extend(check_cited_work_paths(file, root))
    return violations


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--root", type=Path, default=ENGINE_ROOT, help="repo root (default: this checkout)")
    args = parser.parse_args(argv)
    root = args.root.resolve()
    records = live_record_files(root)
    violations = validate(root, records)
    for violation in violations:
        print(f"{violation.path}:{violation.line}: {violation.rule}: {violation.detail}")
    if violations:
        print(f"\n{len(violations)} work-record violation(s).")
        return 1
    print(f"Work records OK ({len(records)} live records checked).")
    return 0


if __name__ == "__main__":
    sys.exit(main())
