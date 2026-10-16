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
        # The real repo ignores target/, so a worktree that has one stays clean.
        info = self.repo / ".git" / "info"
        info.mkdir(exist_ok=True)
        (info / "exclude").write_text("target/\n")

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

    def add_target(self, tree: Path, age_hours: float = 48) -> Path:
        """Create ``tree/target`` with a backdated ``debug/artifact`` past the idle rule."""
        target = tree / "target"
        (target / "debug").mkdir(parents=True)
        (target / "debug" / "artifact").write_bytes(b"x")
        self.set_target_age(target, age_hours)
        return target

    def set_target_age(self, target: Path, age_hours: float) -> None:
        """Backdate every entry under ``target``, deepest first, then the target dir itself."""
        stamp = time.time() - age_hours * 3600
        for dirpath, dirnames, filenames in os.walk(target, topdown=False):
            for name in dirnames + filenames:
                os.utime(Path(dirpath) / name, (stamp, stamp))
        os.utime(target, (stamp, stamp))

    def apparent_size(self, root: Path) -> int:
        """Sum of the files' ``st_size`` under ``root``, mirroring the script's freed report."""
        return sum(p.lstat().st_size for p in root.rglob("*") if p.is_file())

    def keep_tree(self, tree: Path) -> None:
        """Dirty ``tree`` so that ``--apply`` keeps it and its target gets weighed."""
        (tree / "dirty.txt").write_text("x\n")

    def remove(self, *args: str, cwd: Path | None = None) -> str:
        """Run ``main`` from ``cwd`` (the main worktree unless given) and capture its output."""
        out = io.StringIO()
        previous = os.getcwd()
        os.chdir(cwd or self.repo)
        try:
            with contextlib.redirect_stdout(out):
                remove_worktrees.main(list(args))
        finally:
            os.chdir(previous)
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

    def test_size_text_scales_with_one_decimal(self):
        self.assertEqual(remove_worktrees.size_text(8912057139), "8.3 GiB")
        self.assertEqual(remove_worktrees.size_text(1024**3), "1.0 GiB")
        self.assertEqual(remove_worktrees.size_text(2 * 1024**2), "2.0 MiB")
        self.assertEqual(remove_worktrees.size_text(1024**2), "1.0 MiB")
        self.assertEqual(remove_worktrees.size_text(1025), "1.0 KiB")
        self.assertEqual(remove_worktrees.size_text(1024), "1.0 KiB")
        self.assertEqual(remove_worktrees.size_text(1023), "1023 bytes")
        self.assertEqual(remove_worktrees.size_text(0), "0 bytes")

    def test_prune_targets_dry_run_reports_and_deletes_nothing(self):
        tree = self.add_tree("stale")
        target = self.add_target(tree)
        self.keep_tree(tree)
        expected = self.apparent_size(target)
        out = self.remove("--prune-targets")
        self.assertIn(f"refuse  {tree}: dirty working tree", out)
        self.assertIn(f"prune   {target} (dry run; {expected} bytes, idle 48h)", out)
        self.assertTrue(target.is_dir())

    def test_apply_prunes_target_of_a_tree_that_survives(self):
        tree = self.add_tree("dirty-built")
        target = self.add_target(tree)
        self.keep_tree(tree)
        out = self.remove("--apply", "--prune-targets")
        self.assertIn(f"refuse  {tree}: dirty working tree", out)
        self.assertIn(f"pruned  {target} (freed", out)
        self.assertFalse(target.exists())
        self.assertTrue(tree.is_dir())

    def test_fresh_target_is_kept_as_not_idle(self):
        tree = self.add_tree("fresh-target")
        target = self.add_target(tree, age_hours=1)
        self.keep_tree(tree)
        out = self.remove("--prune-targets")
        self.assertIn(f"keep    {target}: idle 1.0h (minimum 24h)", out)
        self.assertTrue(target.is_dir())

    def test_min_age_hours_makes_a_fresh_target_prunable(self):
        tree = self.add_tree("young-target")
        target = self.add_target(tree, age_hours=1)
        self.keep_tree(tree)
        out = self.remove("--prune-targets", "--min-age-hours", "0.5")
        self.assertIn(f"prune   {target} (dry run;", out)

    def test_main_worktree_target_is_kept(self):
        target = self.repo / "target"
        (target / "debug").mkdir(parents=True)
        (target / "debug" / "artifact").write_bytes(b"x")
        self.set_target_age(target, 48)
        out = self.remove("--apply", "--prune-targets")
        self.assertIn(f"keep    {target}: main worktree", out)
        self.assertTrue(target.is_dir())

    def test_gate_in_a_linked_tree_keeps_its_own_and_the_main_target(self):
        main_target = self.repo / "target"
        (main_target / "debug").mkdir(parents=True)
        (main_target / "debug" / "artifact").write_bytes(b"x")
        self.set_target_age(main_target, 48)
        tree = self.add_tree("own-tree")
        own_target = self.add_target(tree)
        self.keep_tree(tree)
        other = self.add_tree("other-built")
        other_target = self.add_target(other)
        self.keep_tree(other)
        out = self.remove("--apply", "--prune-targets", cwd=tree)
        self.assertIn(f"keep    {main_target}: main worktree", out)
        self.assertIn(f"keep    {own_target}: current checkout", out)
        self.assertIn(f"pruned  {other_target}", out)
        self.assertTrue(main_target.is_dir())
        self.assertTrue(own_target.is_dir())
        self.assertFalse(other_target.exists())

    @unittest.skipUnless(Path("/proc/self/cwd").exists(), "needs /proc")
    def test_live_process_in_the_tree_keeps_the_target(self):
        tree = self.add_tree("busy-target")
        target = self.add_target(tree)
        proc = subprocess.Popen(["sleep", "30"], cwd=tree)
        self.addCleanup(proc.wait)
        self.addCleanup(proc.kill)
        out = self.remove("--apply", "--prune-targets")
        self.assertIn(f"keep    {target}: live process cwd inside tree (", out)
        self.assertTrue(target.is_dir())

    def test_symlinked_target_is_kept_and_left_alone(self):
        tree = self.add_tree("linked")
        self.keep_tree(tree)
        elsewhere = self.root / "elsewhere"
        (elsewhere / "debug").mkdir(parents=True)
        (elsewhere / "debug" / "artifact").write_bytes(b"x")
        self.set_target_age(elsewhere, 48)
        (tree / "target").symlink_to(elsewhere)
        out = self.remove("--apply", "--prune-targets")
        self.assertIn(f"keep    {tree / 'target'}: is a symlink", out)
        self.assertTrue((elsewhere / "debug" / "artifact").is_file())

    def test_without_prune_targets_nothing_is_reported_or_deleted(self):
        tree = self.add_tree("built-dirty")
        target = self.add_target(tree)
        self.keep_tree(tree)
        out = self.remove("--apply")
        self.assertIn(f"refuse  {tree}: dirty working tree", out)
        self.assertNotIn("prune", out)
        self.assertNotIn(str(target), out)
        self.assertTrue(target.is_dir())

    def test_tree_without_a_target_dir_gets_no_target_line(self):
        tree = self.add_tree("bare")
        self.keep_tree(tree)
        out = self.remove("--prune-targets")
        self.assertIn(f"refuse  {tree}: dirty working tree", out)
        self.assertNotIn(str(tree / "target"), out)

    def test_target_of_a_removed_tree_is_not_reported(self):
        tree = self.add_tree("gone")
        target = self.add_target(tree)
        out = self.remove("--apply", "--prune-targets")
        self.assertIn(f"removed {tree}", out)
        self.assertNotIn(str(target), out)
        self.assertFalse(tree.exists())

    def test_without_proc_targets_are_kept(self):
        tree = self.add_tree("blind-target")
        target = self.add_target(tree)
        with mock.patch.object(remove_worktrees, "PROC", self.root / "no-proc"):
            out = self.remove("--apply", "--prune-targets")
        self.assertIn(f"keep    {target}: cannot inspect processes (no /proc)", out)
        self.assertTrue(target.is_dir())

    def test_paths_limit_which_targets_are_considered(self):
        keep = self.add_tree("keep-built")
        keep_target = self.add_target(keep)
        self.keep_tree(keep)
        drop = self.add_tree("drop-built")
        drop_target = self.add_target(drop)
        self.keep_tree(drop)
        out = self.remove("--prune-targets", str(drop))
        self.assertIn(f"prune   {drop_target} (dry run;", out)
        self.assertNotIn(str(keep_target), out)

    def test_summary_reports_the_freed_size(self):
        tree = self.add_tree("sized")
        target = self.add_target(tree)
        (target / "debug" / "artifact").write_bytes(b"x" * 4096)
        self.set_target_age(target, 48)
        self.keep_tree(tree)
        out = self.remove("--apply", "--prune-targets")
        self.assertIn(", 1 target dir(s) pruned (freed 4.0 KiB)", out)

    def test_empty_target_dir_is_timed_from_its_own_mtime(self):
        tree = self.add_tree("empty-target")
        self.keep_tree(tree)
        target = tree / "target"
        target.mkdir()
        stamp = time.time() - 48 * 3600
        os.utime(target, (stamp, stamp))
        out = self.remove("--prune-targets")
        self.assertIn(f"prune   {target} (dry run; 0 bytes, idle 48h)", out)

    @unittest.skipIf(os.geteuid() == 0, "root ignores directory permissions")
    def test_unreadable_target_dir_is_kept(self):
        tree = self.add_tree("unreadable-target")
        target = self.add_target(tree)
        self.keep_tree(tree)
        os.chmod(target, 0)
        self.addCleanup(os.chmod, target, 0o755)
        out = self.remove("--prune-targets")
        self.assertIn(f"keep    {target}: cannot read target dir", out)

    def test_undeletable_target_warns_without_failing(self):
        tree = self.add_tree("undeletable-target")
        target = self.add_target(tree)
        self.keep_tree(tree)
        with mock.patch.object(remove_worktrees.shutil, "rmtree", side_effect=OSError("boom")):
            out = self.remove("--apply", "--prune-targets")
        self.assertIn(f"failed  {target}: cannot delete (boom)", out)
        self.assertIn("0 target dir(s) pruned", out)
        self.assertTrue(target.is_dir())

    def test_failed_removal_still_weighs_the_target(self):
        tree = self.add_tree("remove-fails")
        target = self.add_target(tree)
        real_git = remove_worktrees.git

        def git_that_fails_to_remove(cwd: Path, *args: str):
            """Fail only ``git worktree remove``, so the tree survives its removal attempt."""
            if args[:2] == ("worktree", "remove"):
                return subprocess.CompletedProcess([], 1, "", "remove failed")
            return real_git(cwd, *args)

        with mock.patch.object(remove_worktrees, "git", git_that_fails_to_remove):
            out = self.remove("--apply", "--prune-targets")
        self.assertIn(f"failed  {tree}: remove failed", out)
        self.assertIn(f"pruned  {target}", out)
        self.assertTrue(tree.is_dir())


if __name__ == "__main__":
    unittest.main()
