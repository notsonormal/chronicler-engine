"""Move a one-off issue to another status folder, and rewrite the links and cited paths to it.

A one-off issue is `work/issues/<status>/NN-<slug>.md`, and its folder is its
status (`docs/agents/issue-tracker.md`). Its number is unique across all
status folders, so the number alone finds it. The links to the issue change,
and so do its own links to the issues it leaves behind in its old folder.

The incoming links are rewritten in every `.md` under `work/` and `docs/`, in
`AGENTS.md` and in every file under `.agents/skills/`. Backticked
`work/issues/…` paths in the same files are rewritten too.

Exit codes:
    0  the issue moved (or already sat in that folder)
    1  the number matched no issue, or more than one

Run from anywhere:
    python scripts/move_issue.py <NN> <pending|ready|in-progress|closed>
"""

from __future__ import annotations

import argparse
import os
import re
import sys
from pathlib import Path

from work_records import STATUS_FOLDERS, citing_files, rewrite_backticked, rewrite_links_in_text

ENGINE_ROOT = Path(__file__).resolve().parent.parent


class IssueNotFound(Exception):
    """The number matched zero issues, or several."""


def find_issue(root: Path, number: str) -> Path:
    """The one issue file whose name starts with `NN-`, in any status folder."""
    matches = sorted((root / "work" / "issues").glob(f"*/{number}-*.md"))
    if len(matches) != 1:
        found = ", ".join(path.relative_to(root).as_posix() for path in matches) or "none"
        raise IssueNotFound(f"issue {number}: expected one file, found {found}")
    return matches[0]


def files_that_cite(root: Path) -> list[Path]:
    """The citing files of the validator, and every `.md` under `work/`."""
    return sorted({*citing_files(root), *(root / "work").rglob("*.md")})


def issue_number(text: str) -> str:
    """The argparse type of the number: two or more digits."""
    if not re.fullmatch(r"\d{2,}", text):
        raise argparse.ArgumentTypeError(f"{text!r} is not an issue number: use two or more digits, such as 07")
    return text


def move_issue(root: Path, number: str, status: str) -> Path:
    old = find_issue(root, number)
    new = root / "work" / "issues" / status / old.name
    if new == old:
        return new
    moves = {old: new}
    for file in files_that_cite(root):
        try:
            text = file.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        rewritten = rewrite_backticked(text, moves, root)
        if file.suffix == ".md":
            rewritten = rewrite_links_in_text(file, rewritten, moves)
        if rewritten != text:
            file.write_text(rewritten, encoding="utf-8")
    new.parent.mkdir(parents=True, exist_ok=True)
    os.replace(old, new)
    return new


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("number", type=issue_number, help="the issue number, such as 07")
    parser.add_argument("status", type=lambda text: text.rstrip("/"), choices=STATUS_FOLDERS)
    parser.add_argument("--root", type=Path, default=ENGINE_ROOT, help="repo root (default: this checkout)")
    args = parser.parse_args(argv)
    root = args.root.resolve()
    try:
        new = move_issue(root, args.number, args.status)
    except IssueNotFound as error:
        print(error, file=sys.stderr)
        return 1
    print(new.relative_to(root).as_posix())
    return 0


if __name__ == "__main__":
    sys.exit(main())
