"""One-off move of the work records from `.scratch/` and `docs/plans/` to `work/`.

The layout and the placement are the decisions of the work-records map:
tickets 06 to 09, as ticket 11 changed them. The script runs in seven steps:

1. Precheck: every folder of `.scratch/` is named in the tables below, and
   `work/` does not exist yet.
2. Before-scan: every relative link of every `.md` file of the repo, and the
   path it resolves to.
3. The path table: the 17 maps to `work/wayfinder-maps/<map>/`, `_closed/`
   to `work/_closed/`, and the 23 one-off issues to `work/issues/<status>/`
   with new numbers 01 to 23.
4. Rewrite, then move: the links of every `.md` file and the backticked
   paths of the live records and the citing files go through the path table;
   then the files move with `os.replace`, so ignored files move too.
5. Edit the records: the `Status:` line of each map, the old status of each
   one-off issue, the cross-link of issues 19 and 23, the 15 broken links of
   live maps and tickets, and the empty wiki index.
6. After-scan: every link that resolved before still resolves. Then
   `scripts/validate_work_records.py` runs, and the path table is written to
   `work/wayfinder-maps/work-records/assets/path-moves.md`.
7. `--reverse <path-moves.md>` moves the files back and rewrites the links
   again. The removed files and the record edits come back from git.

Run from the repo root:
    python .scratch/work-records/assets/move_to_work.py [--root <dir>] [--dry-run]
    python work/wayfinder-maps/work-records/assets/move_to_work.py --reverse <path-moves.md>
"""

from __future__ import annotations

import argparse
import os
import re
import sys
from collections import Counter
from pathlib import Path


def _find_repo_root(start: Path) -> Path:
    for folder in start.parents:
        if (folder / "scripts" / "work_records.py").is_file():
            return folder
    raise SystemExit("no repo root with scripts/work_records.py above this script")


sys.path.insert(0, str(_find_repo_root(Path(__file__).resolve()) / "scripts"))

from validate_work_records import validate  # noqa: E402
from work_records import (  # noqa: E402
    iter_links,
    map_path,
    resolve_target,
    rewrite_backticked,
    rewrite_links_in_text,
    rewrite_path_tokens,
)

# The path tokens of the old layout. The shared module knows only `work/` paths.
OLD_PATH_TOKEN = re.compile(r"(?<![\w/.-])((?:\.scratch|docs/plans|work)(?:/[^\s`'\")\],;:]*)?)")

OPEN_MAPS = (
    "character-art-from-code",
    "code-concepts",
    "conceptual-integrity",
    "dashboard-design",
    "fable-audit",
    "invariant-audit",
    "long-term-memory-and-state",
    "pi-explain",
    "retro-issue-bucket",
    "state-machines-next",
    "unclebob-quality-tools",
    "weekly-retro-2026-11-04",
    "work-records",
)
CLOSED_MAPS = ("layout-trials", "metacognitive-audit", "prompt-flexibility", "statechart-models")
DISSOLVED_FOLDERS = ("autoplay-follow-ups", "state-machines-work", "story-scroll-back")
CLOSED_FOLDER = "_closed"
# Git history keeps these files.
DROPPED_FILES = (".scratch/autoplay-follow-ups/map.md", ".scratch/state-machines-work/README.md")

SKIPPED_SCAN_FOLDERS = {".git", "target", "tmp", "old-docs", "node_modules", "logs"}
CITING_FILES = (
    ".agents/skills/simple-after-plan/SKILL.md",
    ".agents/skills/verify-gate/SKILL.md",
    ".agents/skills/verify-gate/mise.toml",
    ".agents/skills/chronicler-ui-investigator/ENVIRONMENT.md",
    "docs/diataxis/reference/coding_standards/guardrails.md",
)
PATH_TABLE = "work/wayfinder-maps/work-records/assets/path-moves.md"
WIKI_INDEX = "work/wiki/index.md"
STORY_SCROLL_BACK_ISSUE = "19-story-scroll-back.md"
STORY_LOG_PLAN = "story-log-earlier-messages-investigation-plan.md"

STATUS_LINE = re.compile(r"^Status: [^\n]*\n", re.MULTILINE)
PLAN_STATUS = re.compile(r"^\*\*Status:\*\*", re.MULTILINE)
IDEA_STATUS = re.compile(r"^Status: ", re.MULTILINE)
# A link into the git-ignored `tmp/` breaks in every other checkout, so it becomes text.
TMP_LINK = re.compile(r"\[([^\]]+)\]\((?:\.\./)+(tmp/[^)]*)\)")
TMP_LINK_FILES = (
    "work/wayfinder-maps/metacognitive-audit/map.md",
    "work/wayfinder-maps/metacognitive-audit/issues/04-verify-quotes-mechanically.md",
    "work/wayfinder-maps/metacognitive-audit/issues/08-re-extract-at-pattern-level.md",
    "work/wayfinder-maps/metacognitive-audit/issues/09-re-synthesize-audit-report.md",
    "work/wayfinder-maps/character-art-from-code/issues/04-prototype-the-art-style.md",
)


def _tmp_link_as_text(match: re.Match[str]) -> str:
    label, path = match.group(1), match.group(2)
    return label if label == f"`{path}`" else f"{label} (`{path}`)"


def one_off_issues(root: Path) -> list[tuple[Path, Path]]:
    """The 23 one-off issues, old path to new path, numbered 01 to 23 in the order of ticket 11."""
    scratch = root / ".scratch"
    sources: list[Path] = []
    sources += sorted((scratch / "autoplay-follow-ups" / "issues").glob("*.md"))
    sources += sorted((scratch / "state-machines-work" / "issues").glob("*.md"))
    sources.append(scratch / "story-scroll-back" / "idea.md")
    sources += sorted((root / "docs" / "plans").glob("*.md"))
    pairs: list[tuple[Path, Path]] = []
    for number, source in enumerate(sources, start=1):
        if source.name == "idea.md":
            slug = "story-scroll-back"
        else:
            slug = re.sub(r"^\d+-", "", source.stem)
        status = "closed" if _old_status(source) == "closed" else "pending"
        target = root / "work" / "issues" / status / f"{number:02d}-{slug}.md"
        pairs.append((source, target))
    return pairs


def _old_status(path: Path) -> str:
    match = re.search(r"^Status: (\S+)", path.read_text(encoding="utf-8"), re.MULTILINE)
    return match.group(1) if match else ""


def precheck(root: Path) -> None:
    """Stop on a `.scratch/` folder that the tables do not name, or on an existing `work/`."""
    if (root / "work").exists():
        raise SystemExit("work/ exists already; the move runs once")
    named = set(OPEN_MAPS) | set(CLOSED_MAPS) | set(DISSOLVED_FOLDERS) | {CLOSED_FOLDER}
    present = {path.name for path in (root / ".scratch").iterdir() if path.is_dir()}
    unnamed = sorted(present - named)
    missing = sorted(named - present)
    if unnamed or missing:
        raise SystemExit(f"the folders of .scratch/ do not match the tables: unnamed {unnamed}, missing {missing}")


def build_moves(root: Path) -> dict[Path, Path]:
    scratch = root / ".scratch"
    moves: dict[Path, Path] = {}
    for name in OPEN_MAPS + CLOSED_MAPS:
        moves[scratch / name] = root / "work" / "wayfinder-maps" / name
    moves[scratch / CLOSED_FOLDER] = root / "work" / CLOSED_FOLDER
    for source, target in one_off_issues(root):
        moves[source] = target
    return moves


def markdown_files(root: Path) -> list[Path]:
    found: list[Path] = []
    for folder, subfolders, files in os.walk(root):
        subfolders[:] = [name for name in subfolders if name not in SKIPPED_SCAN_FOLDERS]
        found += [Path(folder) / name for name in files if name.endswith(".md")]
    return sorted(found)


def scan(files: list[Path]) -> dict[Path, Counter[Path]]:
    """For each file, the paths its resolving links name; plus the totals of resolving and broken links."""
    resolved: dict[Path, Counter[Path]] = {}
    for file in files:
        counter: Counter[Path] = Counter()
        for link in iter_links(file.read_text(encoding="utf-8", errors="replace")):
            target = resolve_target(file, link.target)
            if target is None:
                continue
            counter[target if target.exists() else Path("<broken>")] += 1
        resolved[file] = counter
    return resolved


def _broken(counter: Counter[Path]) -> int:
    return counter[Path("<broken>")]


def _resolving(counter: Counter[Path]) -> Counter[Path]:
    return Counter({path: count for path, count in counter.items() if path != Path("<broken>")})


def is_closed_record(path: Path, root: Path) -> bool:
    return (root / ".scratch" / CLOSED_FOLDER) in path.parents


def is_moved_record(path: Path, moves: dict[Path, Path]) -> bool:
    return map_path(path, moves) != path


def rewrite_all(root: Path, moves: dict[Path, Path], files: list[Path]) -> int:
    """Rewrite links in every scanned file, and backticked paths in the live records and citing files.

    `_closed/` gets only the link rewrite: its links into the live maps would
    break, and its text stays history.
    """
    changed = 0
    for file in files:
        text = file.read_text(encoding="utf-8")
        new_text = rewrite_links_in_text(file, text, moves)
        if is_moved_record(file, moves) and not is_closed_record(file, root):
            new_text = rewrite_backticked(new_text, moves, root, OLD_PATH_TOKEN)
        if new_text != text:
            file.write_text(new_text, encoding="utf-8")
            changed += 1
    for rel in CITING_FILES:
        file = root / rel
        text = file.read_text(encoding="utf-8")
        new_text = rewrite_path_tokens(text, moves, root, OLD_PATH_TOKEN)
        if new_text != text:
            file.write_text(new_text, encoding="utf-8")
            changed += 1
    return changed


def move_files(root: Path, moves: dict[Path, Path]) -> None:
    for old, new in sorted(moves.items()):
        new.parent.mkdir(parents=True, exist_ok=True)
        os.replace(old, new)
    for rel in DROPPED_FILES:
        (root / rel).unlink()
    for name in DISSOLVED_FOLDERS:
        _remove_empty_tree(root / ".scratch" / name)
    (root / ".scratch").rmdir()
    (root / "docs" / "plans").rmdir()
    plans_link = root / "plans"
    if plans_link.is_symlink():
        plans_link.unlink()


def _remove_empty_tree(folder: Path) -> None:
    for sub in sorted(folder.rglob("*"), key=lambda path: len(path.parts), reverse=True):
        sub.rmdir()
    folder.rmdir()


def _edit(path: Path, change) -> None:
    text = path.read_text(encoding="utf-8")
    new_text = change(text)
    if new_text == text:
        raise SystemExit(f"edit changed nothing: {path}")
    path.write_text(new_text, encoding="utf-8")


def _add_map_status(text: str, status: str) -> str:
    line = f"Status: {status}\n"
    if re.search(r"^Labels: [^\n]*\n", text, re.MULTILINE):
        return re.sub(r"^(Labels: [^\n]*\n)", lambda match: match.group(1) + line, text, count=1, flags=re.MULTILINE)
    title, rest = text.split("\n", 1)
    return f"{title}\n\n{line}\n" + rest.lstrip("\n")


def edit_records(root: Path, moves: dict[Path, Path]) -> None:
    maps = root / "work" / "wayfinder-maps"
    for name in OPEN_MAPS:
        _edit(maps / name / "map.md", lambda text: _add_map_status(text, "open"))
    for name in CLOSED_MAPS:
        _edit(maps / name / "map.md", lambda text: _add_map_status(text, "closed"))

    for source, target in one_off_issues_from_moves(moves):
        if source.parent.name == "plans":
            _edit(target, lambda text: PLAN_STATUS.sub("**Before the move:**", text, count=1))
        elif source.name == "idea.md":
            _edit(target, lambda text: IDEA_STATUS.sub("**Before the move:** ", text, count=1))
        else:
            _edit(target, lambda text: STATUS_LINE.sub("", text, count=1))

    pending = root / "work" / "issues" / "pending"
    story_log = next(pending.glob(f"*-{STORY_LOG_PLAN}"))
    scroll_back = pending / STORY_SCROLL_BACK_ISSUE
    _add_cross_link(scroll_back, story_log)
    _add_cross_link(story_log, scroll_back)

    for rel in TMP_LINK_FILES:
        _edit(root / rel, lambda text: TMP_LINK.sub(_tmp_link_as_text, text))
    _edit(
        maps / "fable-audit" / "issues" / "02-task-engine-fixes.md",
        lambda text: text.replace("](../invariant-audit/map.md)", "](../../invariant-audit/map.md)"),
    )

    index = root / WIKI_INDEX
    index.parent.mkdir(parents=True, exist_ok=True)
    index.write_text(
        "# Wiki index\n\n"
        "The one wiki of this repo. Each page is listed here with a link and a one-line summary,"
        " grouped by category.\n\n"
        "No pages yet.\n",
        encoding="utf-8",
    )


def one_off_issues_from_moves(moves: dict[Path, Path]) -> list[tuple[Path, Path]]:
    return [(old, new) for old, new in moves.items() if new.parent.parent.name == "issues" and old.suffix == ".md"]


def _add_cross_link(source: Path, target: Path) -> None:
    text = source.read_text(encoding="utf-8")
    if target.name in text:
        return
    title = target.read_text(encoding="utf-8").splitlines()[0].lstrip("# ").strip()
    link = os.path.relpath(target, source.parent)
    see_also = f"\n\n## See also\n\n- [{title}]({link}): the other one-off issue about the older Messages of a long story.\n"
    source.write_text(text.rstrip("\n") + see_also, encoding="utf-8")


def after_scan(
    root: Path, before: dict[Path, Counter[Path]], moves: dict[Path, Path], dropped: set[Path]
) -> tuple[int, int, list[str]]:
    """The resolving and broken links after the move, and each link that resolved before and not after.

    A link that resolved before must resolve after, to the moved place of its
    target. A link into `tmp/` is left out: step 5 turns it into text on purpose.
    """
    lost: list[str] = []
    ignored_tmp = root / "tmp"
    after_resolving = after_broken = 0
    for old_file, counter in before.items():
        if old_file in dropped:
            continue
        new_file = map_path(old_file, moves)
        after = scan([new_file])[new_file]
        after_resolving += sum(_resolving(after).values())
        after_broken += _broken(after)
        expected = Counter(
            {
                map_path(path, moves): count
                for path, count in _resolving(counter).items()
                if ignored_tmp not in path.parents
            }
        )
        missing = expected - _resolving(after)
        for path, count in missing.items():
            lost.append(f"{new_file.relative_to(root)} -> {os.path.relpath(path, root)} (x{count})")
    return after_resolving, after_broken, lost


def write_path_table(root: Path, moves: dict[Path, Path]) -> None:
    rows = [
        f"| `{old.relative_to(root).as_posix()}` | `{new.relative_to(root).as_posix()}` |"
        for old, new in sorted(moves.items())
    ]
    dropped = [f"- `{rel}`" for rel in DROPPED_FILES]
    text = (
        "# Path moves of the work records\n\n"
        "The move to `work/` (ticket 10) moved each path in the left column to the path in the right column."
        " A folder moved whole, with its files. `move_to_work.py --reverse` reads this table.\n\n"
        "| Before | After |\n|---|---|\n"
        + "\n".join(rows)
        + "\n\n## Removed\n\nThe move dropped these files. Git history keeps them.\n\n"
        + "\n".join(dropped)
        + "\n- `docs/plans/` and the `plans` symlink at the repo root\n"
    )
    (root / PATH_TABLE).write_text(text, encoding="utf-8")


def read_path_table(root: Path, table: Path) -> dict[Path, Path]:
    moves: dict[Path, Path] = {}
    for line in table.read_text(encoding="utf-8").splitlines():
        match = re.match(r"^\| `([^`]+)` \| `([^`]+)` \|$", line)
        if match:
            moves[root / match.group(1)] = root / match.group(2)
    return moves


def reverse(root: Path, table: Path) -> int:
    inverted = {new: old for old, new in read_path_table(root, table).items()}
    files = markdown_files(root)
    for file in files:
        text = file.read_text(encoding="utf-8")
        new_text = rewrite_backticked(rewrite_links_in_text(file, text, inverted), inverted, root, OLD_PATH_TOKEN)
        if new_text != text:
            file.write_text(new_text, encoding="utf-8")
    for rel in CITING_FILES:
        file = root / rel
        file.write_text(rewrite_path_tokens(file.read_text(encoding="utf-8"), inverted, root, OLD_PATH_TOKEN), encoding="utf-8")
    for new, old in sorted(inverted.items(), reverse=True):
        old.parent.mkdir(parents=True, exist_ok=True)
        os.replace(new, old)
    print(f"Moved {len(inverted)} paths back. Restore the removed files and the record edits from git.")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--root", type=Path, default=Path.cwd(), help="repo root (default: the working folder)")
    parser.add_argument("--dry-run", action="store_true", help="print the path table and change nothing")
    parser.add_argument("--reverse", type=Path, help="move back through this path-moves.md")
    args = parser.parse_args(argv)
    root = args.root.resolve()
    if args.reverse:
        return reverse(root, args.reverse.resolve())

    precheck(root)
    files = markdown_files(root)
    before = scan(files)
    resolving_before = sum(sum(_resolving(c).values()) for c in before.values())
    broken_before = sum(_broken(c) for c in before.values())
    print(f"Before: {len(files)} files, {resolving_before} links resolve, {broken_before} do not.")
    moves = build_moves(root)
    if args.dry_run:
        for old, new in sorted(moves.items()):
            print(f"{old.relative_to(root)} -> {new.relative_to(root)}")
        return 0

    dropped = {root / rel for rel in DROPPED_FILES}
    changed = rewrite_all(root, moves, files)
    print(f"Rewrote {changed} files.")
    move_files(root, moves)
    edit_records(root, moves)
    after_resolving, after_broken, lost = after_scan(root, before, moves, dropped)
    print(f"After: {after_resolving} links resolve, {after_broken} do not (the same files, at their new paths).")
    print(f"Resolved before and broken after: {len(lost)}.")
    for line in lost:
        print(f"  LOST {line}")
    write_path_table(root, moves)
    violations = validate(root)
    for violation in violations:
        print(f"  {violation.path}:{violation.line}: {violation.rule}: {violation.detail}")
    print(f"validate_work_records: {len(violations)} violation(s).")
    return 1 if lost or violations else 0


if __name__ == "__main__":
    sys.exit(main())
