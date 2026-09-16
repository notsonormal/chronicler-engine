#!/usr/bin/env python3
"""
Comment finder for chronicler-comment-fixer skill.
Finds and filters comment patterns in Rust, Python, HTML and CSS files.
"""

import argparse
import re
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

# Fix Windows stdout encoding for Unicode support
import sys as _sys
if _sys.platform == "win32":
    import io
    _sys.stdout = io.TextIOWrapper(_sys.stdout.buffer, encoding="utf-8", errors="replace")
    _sys.stderr = io.TextIOWrapper(_sys.stderr.buffer, encoding="utf-8", errors="replace")


@dataclass
class CommentMatch:
    """Represents a single comment match."""
    file_path: str
    line_number: int
    content: str

    def format_rich(self) -> str:
        """Format match as path:line - comment."""
        return f"{self.file_path}:{self.line_number} - {self.content}"


SCRIPT_DIR = Path(__file__).parent.resolve()
WORKSPACE_ROOT = SCRIPT_DIR.parent.parent.parent.parent  # repo root
UNCOMMITTED_EXTENSIONS = {".rs", ".py", ".html", ".css"}


def get_uncommitted_files() -> list[Path]:
    """Get list of uncommitted/new files with relevant extensions."""
    try:
        # Staged + unstaged tracked changes
        result = subprocess.run(
            ["git", "diff", "--name-only", "--diff-filter=ACM", "HEAD"],
            capture_output=True,
            text=True,
            cwd=WORKSPACE_ROOT
        )
        if result.returncode != 0:
            return []
        files = []
        for f in result.stdout.strip().split("\n"):
            if f and Path(f).suffix in UNCOMMITTED_EXTENSIONS:
                files.append(WORKSPACE_ROOT / f)
        # Untracked files (not yet staged)
        result2 = subprocess.run(
            ["git", "ls-files", "--others", "--exclude-standard"],
            capture_output=True,
            text=True,
            cwd=WORKSPACE_ROOT
        )
        if result2.returncode == 0:
            for f in result2.stdout.strip().split("\n"):
                if f and Path(f).suffix in UNCOMMITTED_EXTENSIONS:
                    files.append(WORKSPACE_ROOT / f)
        return files
    except Exception:
        return []
def get_branch_files(base_branch: Optional[str] = None) -> list[Path]:
    """Get list of files changed in current branch vs base branch."""
    try:
        branch = base_branch if base_branch else "main"
        result = subprocess.run(
            ["git", "diff", "--name-only", "--diff-filter=ACM", branch],
            capture_output=True,
            text=True,
            cwd=WORKSPACE_ROOT
        )
        if result.returncode != 0:
            return []
        files = []
        for f in result.stdout.strip().split("\n"):
            if f and Path(f).suffix in UNCOMMITTED_EXTENSIONS:
                files.append(WORKSPACE_ROOT / f)
        return files
    except Exception:
        return []


def get_all_source_files() -> list[Path]:
    """Get all Rust, HTML and CSS files in the workspace."""
    source_files = []
    for pattern in ["src/**/*.rs", "tests/**/*.rs", "assets/*.html", "assets/*.css"]:
        source_files.extend(WORKSPACE_ROOT.glob(pattern))
    return sorted(set(source_files))


def get_files_by_pattern(pattern: str) -> list[Path]:
    """Get files matching a glob pattern."""
    try:
        matches = list(WORKSPACE_ROOT.glob(pattern))
        return [f for f in matches if f.suffix in UNCOMMITTED_EXTENSIONS]
    except Exception:
        return []


def _block_comment_lines(lines: list[str]) -> list[tuple[int, str]]:
    """Return every non-blank line inside a `/* ... */` block.

    Continuation lines count as comments. A rule violation often sits on the
    middle line of a multi-line block, so prefix matching alone would miss it.
    """
    found = []
    inside = False
    for i, line in enumerate(lines, 1):
        stripped = line.strip()
        if inside:
            if stripped:
                found.append((i, stripped))
            if "*/" in stripped:
                inside = False
            continue
        if "/*" in stripped:
            if stripped:
                found.append((i, stripped))
            inside = "*/" not in stripped.split("/*", 1)[1]
    return found


def _html_comment_lines(lines: list[str]) -> list[tuple[int, str]]:
    """Return HTML comments, plus JS and CSS comments inside their blocks."""
    found = []
    in_html_comment = False
    region = None
    for i, line in enumerate(lines, 1):
        stripped = line.strip()
        if in_html_comment:
            if stripped:
                found.append((i, stripped))
            if "-->" in stripped:
                in_html_comment = False
            continue
        if region == "script":
            if stripped.startswith(("//", "/*", "*")):
                found.append((i, stripped))
            if "</script>" in stripped:
                region = None
            continue
        if region == "style":
            if stripped.startswith(("/*", "*")):
                found.append((i, stripped))
            if "</style>" in stripped:
                region = None
            continue
        if "<!--" in stripped:
            found.append((i, stripped))
            in_html_comment = "-->" not in stripped.split("<!--", 1)[1]
            continue
        if "<script" in stripped:
            region = "script"
            continue
        if "<style" in stripped:
            region = "style"
    return found


def _comment_lines(suffix: str, lines: list[str]) -> list[tuple[int, str]]:
    """Return `(line_number, text)` for every comment line, by file type."""
    if suffix == ".rs":
        return [
            (i, stripped)
            for i, line in enumerate(lines, 1)
            if (stripped := line.strip()).startswith(("//", "/*", "*/"))
        ]
    if suffix == ".py":
        return [
            (i, stripped)
            for i, line in enumerate(lines, 1)
            if (stripped := line.strip()).startswith(("#", '"""', "'''"))
        ]
    if suffix == ".css":
        return _block_comment_lines(lines)
    if suffix == ".html":
        return _html_comment_lines(lines)
    return []


def find_comments_in_file(file_path: Path) -> list[CommentMatch]:
    """Find all comment lines in a file."""
    matches = []
    if not file_path.exists():
        return matches
    try:
        with open(file_path, "r", encoding="utf-8", errors="ignore") as f:
            lines = f.readlines()
        for line_number, content in _comment_lines(file_path.suffix, lines):
            matches.append(CommentMatch(
                file_path=str(file_path.relative_to(WORKSPACE_ROOT)),
                line_number=line_number,
                content=content
            ))
    except Exception as e:
        print(f"Error reading {file_path}: {e}", file=_sys.stderr)
    return matches


def print_results(mode: str, files: list[Path], all_matches: list[tuple[Path, list[CommentMatch]]]) -> None:
    """Print formatted results."""
    total_comments = sum(len(m) for _, m in all_matches)
    print(f"# {mode}")
    print(f"# Files: {len(files)}, Comments: {total_comments}\n")
    for file_path, matches in all_matches:
        for match in matches:
            print(match.format_rich())


def main():
    parser = argparse.ArgumentParser(
        description="Find AI slop, verbose docs, and convention violations in comments"
    )
    parser.add_argument("--files", "-f", nargs="*", help="Specific files to check")
    parser.add_argument("--pattern", "-p", help="Glob pattern to match files")
    parser.add_argument("--uncommitted", "-u", action="store_true", help="Check uncommitted files")
    parser.add_argument("--all", "-a", action="store_true", help="Check all Rust, HTML and CSS files")
    parser.add_argument("--branch", "-b", nargs="?", const="main", metavar="BASE", help="Check files changed in branch vs BASE (default: main)")
    parser.add_argument("--mode", "-m", default="default", help="Output mode (for testing)")
    args = parser.parse_args()

    # Determine files to check
    if args.files:
        files = [WORKSPACE_ROOT / Path(f) for f in args.files]
    elif args.pattern:
        files = get_files_by_pattern(args.pattern)
    elif args.uncommitted:
        files = get_uncommitted_files()
        args.mode = "uncommitted"
    elif args.all:
        files = get_all_source_files()
        args.mode = "all-source"
    elif args.branch is not None:
        files = get_branch_files(args.branch)
        args.mode = f"branch-{args.branch or 'main'}"
    else:
        files = get_uncommitted_files()
        args.mode = "uncommitted"
    all_matches = []
    for f in files:
        matches = find_comments_in_file(f)
        if matches:
            all_matches.append((f, matches))

    print_results(args.mode, files, all_matches)


if __name__ == "__main__":
    main()