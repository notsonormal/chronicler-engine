"""Remove linked git worktrees, but only ones that are safe to lose, and prune stale target dirs.

Lists every worktree with ``git worktree list --porcelain`` and inspects each one. A tree is
refused while any of these hold:

- it is the main worktree, or it is locked;
- its directory is missing (run ``git worktree prune`` for those);
- it is younger than ``--min-age-hours`` (default 24). Age counts from the newer mtime of
  the tree's ``HEAD`` and ``gitdir`` admin files, i.e. its creation or last checkout. This
  spares a fresh tree an agent is about to edit by path from another directory. The admin
  directory and ``index`` are not used: ``git status`` touches them, so a tree would never age;
- it is dirty (``git status --porcelain`` is not empty);
- it has commits that are not pushed. With an upstream, that means ``@{u}..HEAD`` is not
  empty. Without one, it means ``HEAD`` has commits that no remote-tracking ref reaches;
- a live process has its cwd inside it. This reads ``/proc/<pid>/cwd``. Without ``/proc``
  every tree is refused. Processes whose cwd can't be read (other users) are skipped.

The default is a dry run that prints a verdict per tree. ``--apply`` removes the safe ones
with ``git worktree remove`` (never ``--force``), re-inspecting each tree right before it
goes. Branches are kept; only the checked-out directory is removed. That includes ignored
files, so a tree's own ``target/`` goes with it.

``--prune-targets`` also weighs ``<tree>/target`` of every tree that survives, reporting it on the
line under that tree's verdict: ``prune`` (dry run), ``pruned``, or ``keep`` with a reason. Those
survivors are the trees that keep a multi-GiB target dir around. A target dir is kept while any of
these hold:

- it belongs to the main worktree. That checkout is long-lived, so its target dir is the last one
  worth reclaiming;
- it is a symlink. It may point into another tree, so it is refused rather than followed;
- this process or a live process has its cwd inside the tree (the same process check as above,
  with the holders computed once per tree);
- it is not idle. Idle counts from the newest mtime of the entries directly inside
  ``<tree>/target``, against the same ``--min-age-hours``. This is approximate on purpose: a
  build in progress is caught by the process check, so the age only has to be roughly right.

Tree-level vetoes do not carry over: a target dir ages on its own, so dirty, unpushed, locked and
too-new trees still get their target pruned. Only ``<tree>/target`` is considered or deleted; a
``CARGO_TARGET_DIR`` or ``--target-dir`` outside the tree is out of scope.

``--apply`` extends the closing count with the pruned target dirs and the bytes they freed. A
target that cannot be deleted prints ``failed`` and, like the tree-level failures, only warns. A
tree removed here takes its target with it and prints no target line.

The full gate runs this with ``--apply --prune-targets`` (``python build.py remove-worktrees``
runs it alone). The tree the gate runs in is always refused, and its target is kept as well,
because the build's own cwd is inside it. Exit code 1 means a removal failed; the gate only warns
on it.

Usage: ``python scripts/remove_worktrees.py [--apply] [--min-age-hours N] [--prune-targets]
[PATH ...]``. With paths, only those worktrees are considered, for both removal and pruning.
"""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path

PROC = Path("/proc")
DEFAULT_MIN_AGE_HOURS = 24.0
# Written when a worktree is created or checked out; not touched by `git status`.
_AGE_FILES = ("HEAD", "gitdir")


@dataclass
class Worktree:
    path: Path
    is_main: bool
    locked: bool
    prunable: bool


def git(cwd: Path, *args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(["git", *args], cwd=cwd, capture_output=True, text=True)


def list_worktrees(repo: Path) -> list[Worktree]:
    """Parse ``git worktree list --porcelain``; the first entry is the main worktree."""
    out = git(repo, "worktree", "list", "--porcelain")
    if out.returncode != 0:
        raise RuntimeError(out.stderr.strip() or "git worktree list failed")
    trees: list[Worktree] = []
    for block in out.stdout.strip().split("\n\n"):
        lines = block.splitlines()
        if not lines or not lines[0].startswith("worktree "):
            continue
        keys = {line.split(" ", 1)[0] for line in lines[1:]}
        trees.append(
            Worktree(
                path=Path(lines[0][len("worktree ") :]),
                is_main=not trees,
                locked="locked" in keys,
                prunable="prunable" in keys,
            )
        )
    return trees


def process_cwds(proc: Path | None = None) -> list[Path] | None:
    """Every readable process cwd, or ``None`` when processes can't be inspected at all."""
    proc = proc or PROC
    if not proc.is_dir():
        return None
    cwds: list[Path] = []
    for entry in proc.iterdir():
        if not entry.name.isdigit() or int(entry.name) == os.getpid():
            continue
        try:
            cwds.append(Path(os.readlink(entry / "cwd")))
        except OSError:
            continue  # exited, or owned by another user
    return cwds


def is_inside(child: Path, parent: Path) -> bool:
    try:
        child.resolve().relative_to(parent.resolve())
    except ValueError:
        return False
    return True


def unpushed_reason(path: Path) -> str | None:
    upstream = git(path, "rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}")
    if upstream.returncode == 0:
        count = git(path, "rev-list", "--count", "@{u}..HEAD")
        where = f"upstream {upstream.stdout.strip()}"
    else:
        count = git(path, "rev-list", "--count", "HEAD", "--not", "--remotes")
        where = "any remote"
    if count.returncode != 0:
        return f"cannot count unpushed commits: {count.stderr.strip()}"
    n = int(count.stdout.strip() or "0")
    return f"{n} commit(s) not on {where}" if n else None


def age_hours(path: Path) -> float | None:
    """Hours since the tree was created or last checked out; ``None`` if unreadable."""
    git_dir = git(path, "rev-parse", "--absolute-git-dir")
    if git_dir.returncode != 0:
        return None
    admin = Path(git_dir.stdout.strip())
    try:
        newest = max((admin / name).stat().st_mtime for name in _AGE_FILES)
    except OSError:
        return None
    return (time.time() - newest) / 3600


def refusal(
    tree: Worktree, cwds: list[Path] | None, min_age_hours: float = DEFAULT_MIN_AGE_HOURS
) -> str | None:
    """Why ``tree`` must not be removed, or ``None`` when it is safe."""
    if tree.is_main:
        return "main worktree"
    if tree.locked:
        return "locked"
    if tree.prunable or not tree.path.is_dir():
        return "directory missing; run `git worktree prune`"
    age = age_hours(tree.path)
    if age is None:
        return "cannot read worktree age"
    if age < min_age_hours:
        return f"too new ({age:.1f}h old, minimum {min_age_hours:g}h)"
    status = git(tree.path, "status", "--porcelain")
    if status.returncode != 0:
        return f"cannot read status: {status.stderr.strip()}"
    if status.stdout.strip():
        return "dirty working tree"
    reason = unpushed_reason(tree.path)
    if reason:
        return reason
    if cwds is None:
        return "cannot inspect processes (no /proc)"
    holders = [c for c in cwds if is_inside(c, tree.path)]
    if holders:
        return f"live process cwd inside tree ({holders[0]})"
    return None


def target_idle_hours(target: Path) -> float | None:
    """Hours since the newest mtime of the entries directly inside ``target``.

    ``None`` when the target dir cannot be listed.
    """
    try:
        stamps = [entry.stat(follow_symlinks=False).st_mtime for entry in target.iterdir()]
        # An empty target dir has no entry to time it by; its own mtime is when it emptied.
        newest = max(stamps) if stamps else target.stat().st_mtime
    except OSError:
        return None
    return (time.time() - newest) / 3600


def dir_size(target: Path) -> int:
    """Space the files below ``target`` occupy, for the report only.

    Cargo hard-links uplifted artifacts, so an inode is counted once.
    """
    total = 0
    seen: set[tuple[int, int]] = set()
    for root, _dirs, names in os.walk(target):
        for name in names:
            try:
                stat = (Path(root) / name).lstat()
            except OSError:
                continue  # vanished mid-walk
            if (stat.st_dev, stat.st_ino) in seen:
                continue
            seen.add((stat.st_dev, stat.st_ino))
            total += stat.st_size
    return total


def size_text(size: int) -> str:
    """Human size for the target lines: ``8.3 GiB``, ``512.0 MiB``, ``4.0 KiB``, else bytes."""
    for unit, factor in (("GiB", 1024**3), ("MiB", 1024**2), ("KiB", 1024)):
        if size >= factor:
            return f"{size / factor:.1f} {unit}"
    return f"{size} bytes"


def prune_target(
    tree: Worktree, cwds: list[Path] | None, min_age_hours: float, apply: bool
) -> tuple[bool, int]:
    """Report one tree's ``target`` dir and, when ``apply`` allows, delete it.

    A target dir is kept while it is the main worktree's, is a symlink (it may point into another
    tree), is this checkout's own, has a live process cwd inside the tree, or was touched within
    ``min_age_hours``. Any other target dir is idle and unused, so it is prunable. Returns whether
    it was pruned and how many bytes that freed; it prints nothing when there is no target dir.
    """
    target = tree.path / "target"
    if not target.is_dir():
        return False, 0
    idle = target_idle_hours(target)
    holders = [] if cwds is None else [cwd for cwd in cwds if is_inside(cwd, target.parent)]
    if tree.is_main:
        reason = "main worktree"
    elif target.is_symlink():
        reason = "is a symlink"
    elif is_inside(Path.cwd(), target.parent):
        reason = "current checkout"
    elif cwds is None:
        reason = "cannot inspect processes (no /proc)"
    elif holders:
        reason = f"live process cwd inside tree ({holders[0]})"
    elif idle is None:
        reason = "cannot read target dir"
    elif idle < min_age_hours:
        reason = f"idle {idle:.1f}h (minimum {min_age_hours:g}h)"
    else:
        reason = None
    if reason:
        print(f"keep    {target}: {reason}")
        return False, 0
    size = dir_size(target)
    if not apply:
        print(f"prune   {target} (dry run; {size_text(size)}, idle {idle:.0f}h)")
        return False, 0
    try:
        shutil.rmtree(target)
    except OSError as exc:
        print(f"failed  {target}: cannot delete ({exc})")
        return False, 0
    print(f"pruned  {target} (freed {size_text(size)}, idle {idle:.0f}h)")
    return True, size


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--apply", action="store_true", help="remove safe trees (default: dry run)")
    parser.add_argument(
        "--min-age-hours",
        type=float,
        default=DEFAULT_MIN_AGE_HOURS,
        help=(
            "refuse trees created or checked out more recently, and keep target dirs"
            " touched more recently (default: %(default)g)"
        ),
    )
    parser.add_argument(
        "--prune-targets",
        action="store_true",
        help="also prune the surviving trees' stale target dirs (default: report them only)",
    )
    parser.add_argument("paths", nargs="*", type=Path, help="only consider these worktrees")
    args = parser.parse_args(argv)

    repo = Path.cwd()
    trees = list_worktrees(repo)
    if args.paths:
        wanted = {p.resolve() for p in args.paths}
        trees = [t for t in trees if t.path.resolve() in wanted]
        missing = wanted - {t.path.resolve() for t in trees}
        for path in sorted(missing):
            print(f"skip    {path}: not a worktree of this repo")

    removed = failed = pruned = freed = 0
    for tree in trees:
        cwds = process_cwds()  # one snapshot per tree, as the module docstring promises
        reason = refusal(tree, cwds, args.min_age_hours)
        if reason:
            print(f"refuse  {tree.path}: {reason}")
        elif not args.apply:
            print(f"safe    {tree.path} (dry run; pass --apply to remove)")
            continue  # an --apply run would remove this tree together with its target
        else:
            result = git(repo, "worktree", "remove", str(tree.path))
            if result.returncode != 0:
                print(f"failed  {tree.path}: {result.stderr.strip()}")
                failed += 1
            else:
                print(f"removed {tree.path}")
                removed += 1
                continue  # its target went with it
        if args.prune_targets:
            done, size = prune_target(tree, cwds, args.min_age_hours, args.apply)
            pruned += done
            freed += size
    if args.apply:
        summary = f"{removed} worktree(s) removed"
        if args.prune_targets:
            summary += f", {pruned} target dir(s) pruned (freed {size_text(freed)})"
        print(summary)
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
