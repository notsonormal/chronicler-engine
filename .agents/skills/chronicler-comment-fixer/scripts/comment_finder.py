#!/usr/bin/env python3
"""
Comment finder for chronicler-comment-fixer skill.
Finds and filters comment patterns in Rust and Python files.
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
UNCOMMITTED_EXTENSIONS = {".rs", ".py"}


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


def get_all_rust_files() -> list[Path]:
    """Get all .rs files in the workspace."""
    rust_files = []
    for ext in ["src/**/*.rs", "tests/**/*.rs"]:
        rust_files.extend(WORKSPACE_ROOT.glob(ext))
    return sorted(set(rust_files))


def get_files_by_pattern(pattern: str) -> list[Path]:
    """Get files matching a glob pattern."""
    try:
        matches = list(WORKSPACE_ROOT.glob(pattern))
        return [f for f in matches if f.suffix in UNCOMMITTED_EXTENSIONS]
    except Exception:
        return []


def find_comments_in_file(file_path: Path) -> list[CommentMatch]:
    """Find all comment lines in a file."""
    matches = []
    if not file_path.exists():
        return matches
    try:
        with open(file_path, "r", encoding="utf-8", errors="ignore") as f:
            lines = f.readlines()
        for i, line in enumerate(lines, 1):
            stripped = line.strip()
            # Rust comments
            if file_path.suffix == ".rs":
                if stripped.startswith("//") or stripped.startswith("/*") or stripped.startswith("*/"):
                    matches.append(CommentMatch(
                        file_path=str(file_path.relative_to(WORKSPACE_ROOT)),
                        line_number=i,
                        content=stripped
                    ))
            # Python comments
            elif file_path.suffix == ".py":
                if stripped.startswith("#") or stripped.startswith('"""') or stripped.startswith("'''"):
                    matches.append(CommentMatch(
                        file_path=str(file_path.relative_to(WORKSPACE_ROOT)),
                        line_number=i,
                        content=stripped
                    ))
    except Exception as e:
        print(f"Error reading {file_path}: {e}", file=sys.stderr)
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
    parser.add_argument("--all", "-a", action="store_true", help="Check all Rust files")
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
        files = get_all_rust_files()
        args.mode = "all-rust"
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