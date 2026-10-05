"""Unit tests for scripts/remove_worktrees.py."""

from __future__ import annotations

import contextlib
import io
import os
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import remove_worktrees  # noqa: E402


def run(cwd: Path, *args: str) -> str:
    return subprocess.run(
        list(args), cwd=cwd, check=True, capture_output=True, text=True
    ).stdout


class RemoveTests(unittest.TestCase):
    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.root = Path(tmp.name).resolve()
        self.remote = self.root / "remote.git"
        self.repo = self.root / "repo"
        run(self.root, "git", "init", "-q", "--bare", str(self.remote))
        run(self.root, "git", "init", "-q", "-b", "main", str(self.repo))
        for key, value in (("user.name", "t"), ("user.email", "t@t"), ("commit.gpgsign", "false")):
            run(self.repo, "git", "config", key, value)
        (self.repo / "f.txt").write_text("a\n")
        run(self.repo, "git", "add", "f.txt")
        run(self.repo, "git", "commit", "-q", "-m", "init")
        run(self.repo, "git", "remote", "add", "origin", str(self.remote))
        run(self.repo, "git", "push", "-q", "-u", "origin", "main")

    def add_tree(self, name: str, age_hours: float = 48) -> Path:
        """Create a worktree and backdate its admin files so it is past the age rule."""
        path = self.root / name
        run(self.repo, "git", "worktree", "add", "-q", "-b", name, str(path), "main")
        self.set_age(path, age_hours)
        return path

    def set_age(self, path: Path, age_hours: float) -> None:
        admin = Path(run(path, "git", "rev-parse", "--absolute-git-dir").strip())
        stamp = time.time() - age_hours * 3600
        for name in ("HEAD", "gitdir"):
            os.utime(admin / name, (stamp, stamp))

    def remove(self, *args: str) -> str:
        out = io.StringIO()
        cwd = os.getcwd()
        os.chdir(self.repo)
        try:
            with contextlib.redirect_stdout(out):
                remove_worktrees.main(list(args))
        finally:
            os.chdir(cwd)
        return out.getvalue()

    def test_dry_run_removes_nothing(self):
        tree = self.add_tree("clean")
        out = self.remove()
        self.assertIn(f"safe    {tree}", out)
        self.assertTrue(tree.is_dir())

    def test_apply_removes_clean_pushed_idle_tree_and_keeps_branch(self):
        tree = self.add_tree("clean")
        out = self.remove("--apply")
        self.assertIn(f"removed {tree}", out)
        self.assertFalse(tree.exists())
        self.assertIn("clean", run(self.repo, "git", "branch", "--list", "clean"))

    def test_ignored_files_do_not_block_removal_and_go_with_the_tree(self):
        (self.repo / ".gitignore").write_text("target/\n")
        run(self.repo, "git", "add", ".gitignore")
        run(self.repo, "git", "commit", "-qm", "ignore target")
        run(self.repo, "git", "push", "-q")
        tree = self.add_tree("built")
        (tree / "target").mkdir()
        (tree / "target" / "artifact").write_text("x")
        self.assertIn(f"removed {tree}", self.remove("--apply"))
        self.assertFalse(tree.exists())

    def test_fresh_tree_is_refused(self):
        tree = self.add_tree("fresh", age_hours=1)
        out = self.remove("--apply")
        self.assertIn(f"refuse  {tree}: too new (1.0h old, minimum 24h)", out)
        self.assertTrue(tree.is_dir())

    def test_min_age_hours_is_configurable(self):
        tree = self.add_tree("fresh", age_hours=1)
        self.assertIn(f"removed {tree}", self.remove("--apply", "--min-age-hours", "0.5"))

    def test_git_status_does_not_reset_age(self):
        tree = self.add_tree("aged")
        (tree / "f.txt").touch()  # stat change makes `git status` refresh the index
        run(tree, "git", "status", "--porcelain")
        self.assertIsNotNone(remove_worktrees.age_hours(tree))
        self.assertGreater(remove_worktrees.age_hours(tree), 47)

    def test_checkout_resets_age(self):
        tree = self.add_tree("switched")
        run(tree, "git", "checkout", "-q", "-b", "other")
        self.assertIn("too new", self.remove("--apply"))
        self.assertTrue(tree.is_dir())

    def test_main_worktree_is_refused(self):
        out = self.remove("--apply")
        self.assertIn(f"refuse  {self.repo}: main worktree", out)

    def test_dirty_tree_is_refused(self):
        tree = self.add_tree("dirty")
        (tree / "f.txt").write_text("changed\n")
        out = self.remove("--apply")
        self.assertIn(f"refuse  {tree}: dirty working tree", out)
        self.assertTrue(tree.is_dir())

    def test_untracked_file_counts_as_dirty(self):
        tree = self.add_tree("untracked")
        (tree / "new.txt").write_text("x\n")
        self.assertIn("dirty working tree", self.remove("--apply"))
        self.assertTrue(tree.is_dir())

    def test_commit_without_upstream_and_not_on_any_remote_is_refused(self):
        tree = self.add_tree("local")
        (tree / "f.txt").write_text("b\n")
        run(tree, "git", "commit", "-qam", "local work")
        out = self.remove("--apply")
        self.assertIn(f"refuse  {tree}: 1 commit(s) not on any remote", out)
        self.assertTrue(tree.is_dir())

    def test_commit_ahead_of_upstream_is_refused(self):
        tree = self.add_tree("ahead")
        run(tree, "git", "push", "-q", "-u", "origin", "ahead")
        (tree / "f.txt").write_text("b\n")
        run(tree, "git", "commit", "-qam", "unpushed")
        out = self.remove("--apply")
        self.assertIn(f"refuse  {tree}: 1 commit(s) not on upstream origin/ahead", out)
        self.assertTrue(tree.is_dir())

    def test_pushed_commit_is_safe(self):
        tree = self.add_tree("pushed")
        (tree / "f.txt").write_text("b\n")
        run(tree, "git", "commit", "-qam", "pushed")
        run(tree, "git", "push", "-q", "-u", "origin", "pushed")
        self.assertIn(f"removed {tree}", self.remove("--apply"))

    @unittest.skipUnless(Path("/proc/self/cwd").exists(), "needs /proc")
    def test_tree_with_live_process_cwd_is_refused(self):
        tree = self.add_tree("busy")
        (tree / "sub").mkdir()
        proc = subprocess.Popen(["sleep", "30"], cwd=tree / "sub")
        self.addCleanup(proc.wait)
        self.addCleanup(proc.kill)
        out = self.remove("--apply")
        self.assertIn(f"refuse  {tree}: live process cwd inside tree", out)
        self.assertTrue(tree.is_dir())

    def test_without_proc_every_tree_is_refused(self):
        tree = self.add_tree("blind")
        with mock.patch.object(remove_worktrees, "PROC", self.root / "no-proc"):
            out = self.remove("--apply")
        self.assertIn(f"refuse  {tree}: cannot inspect processes (no /proc)", out)
        self.assertTrue(tree.is_dir())

    def test_locked_tree_is_refused(self):
        tree = self.add_tree("locked")
        run(self.repo, "git", "worktree", "lock", str(tree))
        self.assertIn(f"refuse  {tree}: locked", self.remove("--apply"))
        self.assertTrue(tree.is_dir())

    def test_paths_limit_which_trees_are_considered(self):
        keep = self.add_tree("keep")
        drop = self.add_tree("drop")
        out = self.remove("--apply", str(drop))
        self.assertIn(f"removed {drop}", out)
        self.assertNotIn(str(keep), out)
        self.assertTrue(keep.is_dir())

    def test_process_cwd_inside_detection(self):
        proc = self.root / "proc"
        (proc / "123").mkdir(parents=True)
        os.symlink(self.root / "x" / "y", proc / "123" / "cwd")
        (proc / "self").mkdir()
        self.assertEqual(remove_worktrees.process_cwds(proc), [self.root / "x" / "y"])
        self.assertTrue(remove_worktrees.is_inside(self.root / "x" / "y", self.root / "x"))
        self.assertFalse(remove_worktrees.is_inside(self.root / "xy", self.root / "x"))


if __name__ == "__main__":
    unittest.main()
