"""Install git hooks from scripts/git-hooks/ to .git/hooks/.

Safe to run repeatedly and invoked as ``python build.py install-hooks``
(which the full gate runs first): an installed hook identical to its source
is left untouched, a missing one is installed, and a differing one is backed
up to ``<name>.bak-<timestamp>`` before being replaced.
"""

import shutil
import subprocess
import sys
import time
from pathlib import Path


def find_repo_root() -> Path:
    """Resolve the worktree root via git so linked worktrees resolve too."""
    out = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        cwd=Path(__file__).resolve().parent,
        capture_output=True,
        text=True,
        check=True,
    )
    return Path(out.stdout.strip())


def find_hooks_dst(repo_root: Path) -> Path:
    """Resolve the shared hooks dir (per-repository, common across worktrees)."""
    out = subprocess.run(
        ["git", "rev-parse", "--git-common-dir"],
        cwd=repo_root,
        capture_output=True,
        text=True,
        check=True,
    )
    git_common = Path(out.stdout.strip())
    if not git_common.is_absolute():
        git_common = (repo_root / git_common).resolve()
    return git_common / "hooks"


def install_hooks() -> int:
    try:
        repo_root = find_repo_root()
        hooks_dst = find_hooks_dst(repo_root)
    except subprocess.CalledProcessError as e:
        print(f"Error: git could not resolve the repository: {e.stderr.strip()}", file=sys.stderr)
        return 1

    hooks_src = repo_root / "scripts" / "git-hooks"
    if not hooks_src.is_dir():
        print(f"Error: source hooks directory not found: {hooks_src}", file=sys.stderr)
        return 1

    hooks_dst.mkdir(parents=True, exist_ok=True)

    installed = 0
    for hook_file in sorted(hooks_src.iterdir()):
        if not hook_file.is_file():
            continue
        dst = hooks_dst / hook_file.name
        if dst.exists():
            if dst.read_bytes() == hook_file.read_bytes():
                print(f"Up to date: {dst.name}")
                continue
            backup = dst.with_name(f"{dst.name}.bak-{time.strftime('%Y%m%d-%H%M%S')}")
            shutil.copy2(dst, backup)
            print(f"Backed up modified {dst.name} -> {backup.name}")
        shutil.copy2(hook_file, dst)
        if sys.platform != "win32":
            import stat

            dst.chmod(dst.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
        print(f"Installed {dst.name}")
        installed += 1

    print(f"\n{installed} hook(s) installed to {hooks_dst}")
    return 0


if __name__ == "__main__":
    sys.exit(install_hooks())
