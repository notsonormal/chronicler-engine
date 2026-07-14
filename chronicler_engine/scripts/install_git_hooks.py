"""Install git hooks from chronicler_engine/scripts/git-hooks/ to .git/hooks/."""

import shutil
import sys
from pathlib import Path


def find_repo_root() -> Path:
    current = Path(__file__).resolve().parent
    while current != current.parent:
        if (current / ".git").is_dir():
            return current
        current = current.parent
    raise RuntimeError("Could not find git repository root")


def install_hooks() -> int:
    repo_root = find_repo_root()
    hooks_src = repo_root / "chronicler_engine" / "scripts" / "git-hooks"
    hooks_dst = repo_root / ".git" / "hooks"

    if not hooks_src.exists():
        print(f"Error: source hooks directory not found: {hooks_src}")
        return 1

    installed = 0
    for hook_file in hooks_src.iterdir():
        if hook_file.is_file():
            dst = hooks_dst / hook_file.name
            shutil.copy2(hook_file, dst)
            if sys.platform != "win32":
                import stat
                dst.chmod(dst.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
            print(f"Installed {dst.name}")
            installed += 1

    print(f"\nInstalled {installed} hook(s) to {hooks_dst}")
    return 0


if __name__ == "__main__":
    sys.exit(install_hooks())
