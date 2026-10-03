"""Tests for `scripts/prepare_review_bundle.py`, run against a throwaway fixture repository.

Run via ``python -m unittest discover scripts/tests`` or via ``build.py``.
"""

from __future__ import annotations

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / "scripts"))

import prepare_review_bundle as prb  # noqa: E402

SCRIPT = REPO_ROOT / "scripts" / "prepare_review_bundle.py"


def _git(repo: Path, *args: str) -> str:
    result = subprocess.run(
        ["git", *args], cwd=repo, capture_output=True, text=True, check=True
    )
    return result.stdout


def _run(repo: Path, *args: str) -> subprocess.CompletedProcess:
    return subprocess.run(
        [sys.executable, str(SCRIPT), *args], cwd=repo, capture_output=True, text=True
    )


class ClassifyAreaTests(unittest.TestCase):
    def test_path_prefixes_map_to_their_area(self):
        cases = {
            "src/main.rs": "src",
            "tests/foo.rs": "tests",
            "docs/plans/x.md": "docs",
            "assets/icon.png": "docs",
            "scripts/build_slot.py": "tooling",
            "build.py": "tooling",
            ".agents/skills/x/SKILL.md": "tooling",
            "Cargo.toml": "tooling",
        }
        for path, expected in cases.items():
            self.assertEqual(prb.classify_area(path), expected, path)

    def test_leading_dot_slash_is_ignored(self):
        self.assertEqual(prb.classify_area("./src/lib.rs"), "src")


class BundleCliTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.repo = Path(self.tmp.name) / "fixture"
        self.repo.mkdir()
        _git(self.repo, "init", "-q")
        _git(self.repo, "config", "user.email", "test@example.com")
        _git(self.repo, "config", "user.name", "Test")
        (self.repo / ".gitignore").write_text("tmp/\n")
        (self.repo / "README.md").write_text("base\n")
        (self.repo / "src").mkdir()
        (self.repo / "src" / "a.py").write_text("original\n")
        _git(self.repo, "add", ".")
        _git(self.repo, "commit", "-qm", "base")

    def _status(self) -> str:
        return _git(self.repo, "status", "--porcelain")

    def _sha(self, *args: str) -> str:
        return _git(self.repo, "rev-parse", *args).strip()

    def test_uncommitted_is_the_default_mode(self):
        result = _run(self.repo)
        self.assertEqual(result.returncode, 0, result.stderr)
        out = Path(result.stdout.strip())
        self.assertEqual(out, self.repo / "tmp" / "review" / f"{self._sha('--short', 'HEAD')}-uncommitted")
        self.assertTrue(out.is_dir())

    def test_ref_mode_selects_three_dot_diff_and_writes_commits(self):
        base = self._sha("HEAD")
        (self.repo / "src" / "a.py").write_text("changed after base\n")
        _git(self.repo, "add", ".")
        _git(self.repo, "commit", "-qm", "second commit")

        result = _run(self.repo, "--ref", base)
        self.assertEqual(result.returncode, 0, result.stderr)
        out = Path(result.stdout.strip())
        self.assertEqual(out.name, f"{self._sha('--short', 'HEAD')}-ref")
        self.assertIn("second commit", (out / "commits.txt").read_text())
        self.assertIn("changed after base", (out / "src.patch").read_text())

    def test_untracked_contents_reach_their_area_patch(self):
        untracked = {
            "src/new.py": "brand new source contents\n",
            "tests/new_test.py": "brand new test contents\n",
            "docs/note.md": "brand new doc contents\n",
            "scripts/tool.py": "brand new tool contents\n",
        }
        for path, content in untracked.items():
            target = self.repo / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(content)

        result = _run(self.repo)
        self.assertEqual(result.returncode, 0, result.stderr)
        out = Path(result.stdout.strip())

        self.assertIn("src/new.py", (out / "untracked.txt").read_text())
        self.assertIn("brand new source contents", (out / "src.patch").read_text())
        self.assertIn("brand new test contents", (out / "tests.patch").read_text())
        self.assertIn("brand new doc contents", (out / "docs.patch").read_text())
        self.assertIn("brand new tool contents", (out / "tooling.patch").read_text())
        self.assertNotIn("brand new source contents", (out / "docs.patch").read_text())

    def test_deleted_files_are_listed(self):
        (self.repo / "src" / "a.py").unlink()
        result = _run(self.repo)
        self.assertEqual(result.returncode, 0, result.stderr)
        out = Path(result.stdout.strip())
        self.assertIn("src/a.py", (out / "deleted.txt").read_text())

    def test_run_does_not_change_the_index_or_working_tree_status(self):
        (self.repo / "src" / "a.py").write_text("edited\n")
        (self.repo / "src" / "untracked.py").write_text("untracked\n")
        before = self._status()
        result = _run(self.repo)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self._status(), before)

    def test_owned_run_directory_is_cleared(self):
        out = self.repo / "tmp" / "review" / "manual"
        out.mkdir(parents=True)
        (out / prb.MARKER_NAME).write_text(prb.MARKER_TEXT)
        (out / "stale.patch").write_text("from an earlier run\n")
        result = _run(self.repo, "--out", str(out))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertFalse((out / "stale.patch").exists())

    def test_second_run_at_the_same_head_and_mode_is_refused(self):
        first = _run(self.repo)
        self.assertEqual(first.returncode, 0, first.stderr)
        out = Path(first.stdout.strip())
        sentinel = out / "SENTINEL.txt"
        sentinel.write_text("a reviewer is reading this\n")
        second = _run(self.repo)
        self.assertNotEqual(second.returncode, 0)
        self.assertIn(str(out), second.stderr)
        self.assertIn("did not create", second.stderr)
        self.assertEqual(sentinel.read_text(), "a reviewer is reading this\n")
        self.assertTrue((out / "README.md").is_file())

    def test_unowned_run_directory_is_refused_and_left_alone(self):
        out = self.repo / "tmp" / "review" / "manual"
        out.mkdir(parents=True)
        sentinel = out / "another-run.patch"
        sentinel.write_text("from a run this script did not create\n")
        result = _run(self.repo, "--out", str(out))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(str(out), result.stderr)
        self.assertIn("did not create", result.stderr)
        self.assertEqual(sentinel.read_text(), "from a run this script did not create\n")

    def test_out_outside_a_gitignored_path_is_refused(self):
        result = _run(self.repo, "--out", "review-bundle")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("gitignored", result.stderr)
        self.assertFalse((self.repo / "review-bundle").exists())

    def test_bare_gitignored_directory_is_refused_and_survives(self):
        scratch = self.repo / "tmp" / "scratch"
        scratch.mkdir(parents=True)
        keep = scratch / "keep.txt"
        keep.write_text("another agent's scratch\n")
        result = _run(self.repo, "--out", str(scratch))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(str(scratch), result.stderr)
        self.assertIn("did not create", result.stderr)
        self.assertEqual(keep.read_text(), "another agent's scratch\n")

    def test_out_outside_the_repository_is_refused(self):
        result = _run(self.repo, "--out", str(Path(self.tmp.name) / "outside"))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("gitignored", result.stderr)

    def test_unresolvable_ref_fails(self):
        result = _run(self.repo, "--ref", "no-such-ref")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("does not resolve", result.stderr)


if __name__ == "__main__":
    unittest.main()
