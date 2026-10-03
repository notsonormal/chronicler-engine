"""Assemble a review bundle (stat, per-area patches, untracked/deleted lists) under tmp/.

Read-only: the script never stages files and never writes to ``.git``. Untracked
file contents reach their area patch through ``git diff --no-index``; ``git add
-N`` is deliberately not used, because it would rewrite the index. It also
refuses to clear a directory it did not create, so a run can never delete another
run's bundle or unrelated scratch under ``tmp/``.
"""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
from pathlib import Path

# Patch files and the path prefixes that feed them, in bundle order. The last
# area is the catch-all: files that match no prefix (build.py, .agents/, ...).
AREAS: tuple[tuple[str, str, tuple[str, ...]], ...] = (
    ("src", "src.patch", ("src/",)),
    ("tests", "tests.patch", ("tests/",)),
    ("docs", "docs.patch", ("docs/", "assets/")),
    ("tooling", "tooling.patch", ()),
)
DEFAULT_OUT_ROOT = "tmp/review"
MARKER_NAME = ".review-bundle-marker"
MARKER_TEXT = "review bundle marker written by scripts/prepare_review_bundle.py\n"


class BundleError(Exception):
    """A fatal problem with the repository state or the requested output path."""


def classify_area(path: str) -> str:
    """Return the area name owning a repo-relative path; every path lands in one area."""
    posix = path.replace("\\", "/")
    if posix.startswith("./"):
        posix = posix[2:]
    for area, _filename, prefixes in AREAS:
        if prefixes and posix.startswith(prefixes):
            return area
    return "tooling"


def _git(args: list[str], cwd: Path, check: bool = True) -> subprocess.CompletedProcess:
    """Run a read-only git command in ``cwd`` and capture its output."""
    result = subprocess.run(["git", *args], cwd=cwd, capture_output=True, text=True)
    if check and result.returncode != 0:
        detail = result.stderr.strip() or f"git {' '.join(args)} failed"
        raise BundleError(detail)
    return result


def _nul_split(text: str) -> list[str]:
    return [item for item in text.split("\0") if item]


def _repo_root() -> Path:
    result = _git(["rev-parse", "--show-toplevel"], Path.cwd(), check=False)
    if result.returncode != 0:
        raise BundleError("not inside a git work tree")
    return Path(result.stdout.strip())


def _resolve_ref(ref: str, repo_root: Path) -> None:
    result = _git(["rev-parse", "--verify", "--quiet", f"{ref}^{{commit}}"], repo_root, check=False)
    if result.returncode != 0:
        raise BundleError(f"fixed point {ref!r} does not resolve to a commit")


def _is_gitignored(path: Path, repo_root: Path) -> bool:
    """True when ``path`` sits inside the repo and is covered by a gitignore rule."""
    try:
        rel = path.resolve().relative_to(repo_root.resolve())
    except ValueError:
        return False
    result = _git(["check-ignore", "-q", "--", str(rel)], repo_root, check=False)
    return result.returncode == 0


def _unowned_dir_message(out_dir: Path) -> str:
    return (
        f"refusing to clear a directory this script did not create: {out_dir} "
        "(pass a different --out, or remove the directory yourself)"
    )


def _prepare_run_dir(out_dir: Path) -> None:
    """Create the run directory, clearing it only when a previous run created it.

    The marker is an in-progress claim written here and dropped by
    ``build_bundle`` on success. A marker left behind means an earlier run died
    before finishing, so its directory can be reclaimed; a completed bundle has
    no marker and a later run at the same path refuses rather than clearing it.
    """
    if out_dir.is_symlink():
        raise BundleError(_unowned_dir_message(out_dir))
    if out_dir.exists():
        if not out_dir.is_dir() or not (out_dir / MARKER_NAME).is_file():
            raise BundleError(_unowned_dir_message(out_dir))
        shutil.rmtree(out_dir)
    out_dir.mkdir(parents=True)
    (out_dir / MARKER_NAME).write_text(MARKER_TEXT, encoding="utf-8")


def _base_arg(mode: str, ref: str | None) -> str:
    return f"{ref}...HEAD" if mode == "ref" else "HEAD"


def _patch_text(base: str, files: list[str], repo_root: Path) -> str:
    if not files:
        return ""
    return _git(["diff", base, "--", *files], repo_root).stdout


def _untracked_text(path: str, repo_root: Path) -> str:
    # git diff --no-index exits 1 when the inputs differ, which is the point.
    result = _git(
        ["diff", "--no-index", "--", os.devnull, path], repo_root, check=False
    )
    if result.returncode not in (0, 1):
        detail = result.stderr.strip() or f"could not diff untracked file {path}"
        raise BundleError(detail)
    return result.stdout


def _stat_text(
    base: str,
    repo_root: Path,
    area_counts: list[tuple[str, int, int]],
    untracked: list[str],
    deleted: list[str],
) -> str:
    lines = [
        f"base: {base}",
        f"repo: {repo_root}",
        "",
        "== diff --stat ==",
        _git(["diff", "--stat", base], repo_root).stdout.rstrip("\n"),
        "",
        "== areas ==",
    ]
    for area, tracked_count, untracked_count in area_counts:
        lines.append(f"{area}: {tracked_count} tracked, {untracked_count} untracked")
    lines += [
        "",
        f"untracked files: {len(untracked)}",
        f"deleted files: {len(deleted)}",
        "",
    ]
    return "\n".join(lines)


def _readme_text(
    mode: str,
    base: str,
    ref: str | None,
    head_sha: str,
    out_dir: Path,
    has_commits: bool,
) -> str:
    mode_line = (
        f"working tree against {base}" if mode == "uncommitted" else f"three-dot diff against {ref}"
    )
    lines = [
        "# Review bundle",
        "",
        f"- mode: {mode} ({mode_line})",
        f"- fixed point: {base}",
        f"- HEAD: {head_sha}",
        f"- directory: {out_dir}",
        "",
        "Page the files smallest-first; `stat.txt` is the index.",
        "",
        "| file | contents |",
        "| --- | --- |",
        "| `stat.txt` | diffstat and per-area file counts |",
    ]
    area_note = "tracked + untracked contents" if mode == "uncommitted" else "tracked changes"
    for area, filename, _prefixes in AREAS:
        lines.append(f"| `{filename}` | {area} patches ({area_note}) |")
    if has_commits:
        lines.append("| `commits.txt` | `git log <fixed point>..HEAD --oneline` |")
    lines += [
        "| `untracked.txt` | untracked file names |",
        "| `deleted.txt` | deleted file names |",
        "",
        "## How to read",
        "",
        "```sh",
        f"less {out_dir}/stat.txt",
        f"less {out_dir}/src.patch        # largest areas first",
        f"grep -n '^diff --git' {out_dir}/*.patch",
        "```",
        "",
    ]
    return "\n".join(lines)


def build_bundle(mode: str, ref: str | None, out_dir: Path, repo_root: Path) -> Path:
    """Write the bundle for ``mode`` into ``out_dir`` and return that directory."""
    if not _is_gitignored(out_dir, repo_root):
        raise BundleError(
            f"refusing to write outside a gitignored path: {out_dir} "
            "(use a path under tmp/)"
        )
    if mode == "ref":
        if ref is None:
            raise BundleError("--ref needs a fixed point")
        _resolve_ref(ref, repo_root)

    base = _base_arg(mode, ref)
    tracked = _nul_split(_git(["diff", base, "--name-only", "-z"], repo_root).stdout)
    deleted = _nul_split(
        _git(["diff", base, "--diff-filter=D", "--name-only", "-z"], repo_root).stdout
    )
    untracked = (
        _nul_split(_git(["ls-files", "--others", "--exclude-standard", "-z"], repo_root).stdout)
        if mode == "uncommitted"
        else []
    )

    _prepare_run_dir(out_dir)

    area_counts: list[tuple[str, int, int]] = []
    for area, filename, _prefixes in AREAS:
        area_tracked = [path for path in tracked if classify_area(path) == area]
        area_untracked = [path for path in untracked if classify_area(path) == area]
        chunks = [_patch_text(base, area_tracked, repo_root)]
        chunks += [_untracked_text(path, repo_root) for path in area_untracked]
        (out_dir / filename).write_text("".join(chunks), encoding="utf-8")
        area_counts.append((area, len(area_tracked), len(area_untracked)))

    (out_dir / "untracked.txt").write_text(
        "".join(f"{path}\n" for path in untracked), encoding="utf-8"
    )
    (out_dir / "deleted.txt").write_text(
        "".join(f"{path}\n" for path in deleted), encoding="utf-8"
    )
    head_sha = _git(["rev-parse", "HEAD"], repo_root).stdout.strip()
    (out_dir / "stat.txt").write_text(
        _stat_text(base, repo_root, area_counts, untracked, deleted), encoding="utf-8"
    )
    has_commits = mode == "ref"
    if has_commits:
        (out_dir / "commits.txt").write_text(
            _git(["log", f"{ref}..HEAD", "--oneline"], repo_root).stdout, encoding="utf-8"
        )
    (out_dir / "README.md").write_text(
        _readme_text(mode, base, ref, head_sha, out_dir, has_commits),
        encoding="utf-8",
    )
    # Drop the in-progress claim: a completed bundle carries no marker, so a
    # later run at the same path refuses instead of clearing it.
    (out_dir / MARKER_NAME).unlink()
    return out_dir


def parse_args(argv: list[str] | None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Assemble a review bundle under tmp/review/.")
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument(
        "--uncommitted", action="store_true", help="diff the working tree against HEAD (default)"
    )
    mode.add_argument("--ref", metavar="REF", help="three-dot diff against REF")
    parser.add_argument("--out", metavar="DIR", help="run directory (default tmp/review/<sha>-<mode>)")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    try:
        repo_root = _repo_root()
        mode = "ref" if args.ref else "uncommitted"
        if args.out:
            out_dir = Path(args.out)
            if not out_dir.is_absolute():
                out_dir = repo_root / out_dir
        else:
            short_sha = _git(["rev-parse", "--short", "HEAD"], repo_root).stdout.strip()
            out_dir = repo_root / DEFAULT_OUT_ROOT / f"{short_sha}-{mode}"
        out_dir = build_bundle(mode, args.ref, out_dir, repo_root)
    except BundleError as error:
        print(f"error: {error}", file=sys.stderr)
        return 2
    print(out_dir)
    return 0


if __name__ == "__main__":
    sys.exit(main())
