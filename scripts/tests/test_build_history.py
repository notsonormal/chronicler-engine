"""Unit tests for scripts/build_history.py: the journal, the size comparison and the tree name.

Run via ``python -m unittest discover scripts/tests`` or via ``build.py``.
"""

from __future__ import annotations

import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))

import build_history  # noqa: E402


class JournalTests(unittest.TestCase):
    """The journal records the tree and per-tier test counts of each run."""

    OLD_HEADER = "timestamp | duration_s | exit_code | args\n"

    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.path = Path(tmp.name) / "build_history.txt"

    def lines(self):
        return self.path.read_text(encoding="utf-8").splitlines()

    def test_record_has_six_columns(self):
        build_history.append(
            self.path, "--no-browser", 51.2, 0, "abc+12345678", "integration=1565"
        )
        header, record = self.lines()
        self.assertEqual(header, build_history.HEADER)
        parts = record.split(" | ")
        self.assertEqual(len(parts), 6)
        self.assertEqual(parts[3:], ["--no-browser", "abc+12345678", "integration=1565"])

    def test_old_header_is_replaced_not_duplicated(self):
        old_record = "2026-10-08T19:46:00 | 1.0 | 0 | clippy\n"
        self.path.write_text(self.OLD_HEADER + old_record, encoding="utf-8")
        build_history.append(self.path, "clippy", 1.0, 0, "unknown", "-")
        lines = self.lines()
        self.assertEqual(lines[0], build_history.HEADER)
        self.assertEqual(lines[1], old_record.rstrip("\n"))
        self.assertEqual(len(lines), 3)
        self.assertTrue(lines[2].endswith(" | clippy | unknown | -"))

    def test_trim_keeps_the_header(self):
        with mock.patch.object(build_history, "MAX_LINES", 3):
            for n in range(5):
                build_history.append(self.path, f"run{n}", 1.0, 0, "abc", "-")
        lines = self.lines()
        self.assertEqual(lines[0], build_history.HEADER)
        self.assertEqual([line.split(" | ")[3] for line in lines[1:]], ["run3", "run4"])

    def test_last_tier_sizes_takes_each_tiers_newest_record(self):
        self.path.write_text(
            build_history.HEADER + "\n"
            "2026-10-08T19:00:00 | 1.0 | 0 | clippy\n"
            "2026-10-08T19:10:00 | 90.0 | 0 | (full gate) | aaa | "
            "architecture=1 guardrails=165 integration=1572 browser=68\n"
            "2026-10-08T19:20:00 | 20.0 | 0 | integration | bbb+1234abcd | integration=1565\n"
            "2026-10-08T19:30:00 | 1.0 | 0 | clippy | bbb | -\n",
            encoding="utf-8",
        )
        newest = build_history.last_tier_sizes(self.path)
        # `architecture` is not a tier, so its key in the gate row is skipped.
        self.assertEqual(
            newest,
            {
                "guardrails": build_history.TierRecord("2026-10-08T19:10:00", "aaa", 165),
                "integration": build_history.TierRecord(
                    "2026-10-08T19:20:00", "bbb+1234abcd", 1565
                ),
                "browser": build_history.TierRecord("2026-10-08T19:10:00", "aaa", 68),
            },
        )
        # A NamedTuple equals a plain tuple, so check the type as well.
        for record in newest.values():
            self.assertIsInstance(record, build_history.TierRecord)

    def test_last_tier_sizes_without_journal_or_tier(self):
        self.assertEqual(build_history.last_tier_sizes(self.path), {})
        self.path.write_text(self.OLD_HEADER + "2026-10-08T19:00:00 | 1.0 | 0 | x\n")
        self.assertEqual(build_history.last_tier_sizes(self.path), {})

    def test_tests_column_follows_tier_order(self):
        sizes = {
            "integration": 1565,
            "guardrails": 163,
        }
        self.assertEqual(build_history.tests_column(sizes), "guardrails=163 integration=1565")
        self.assertEqual(build_history.tests_column({}), "-")


class SizeSuffixTests(unittest.TestCase):
    """The epilogue suffix compares a tier's count with its newest record."""

    BASE = build_history.TierRecord("2026-10-08T19:46:00", "a1b2c3d4e5", 1572)

    def test_decrease(self):
        self.assertEqual(
            build_history.size_suffix(1565, self.BASE),
            "tests: 1565 (-7 vs 2026-10-08T19:46:00, tree a1b2c3d4e5)",
        )

    def test_increase(self):
        self.assertEqual(
            build_history.size_suffix(1575, self.BASE),
            "tests: 1575 (+3 vs 2026-10-08T19:46:00, tree a1b2c3d4e5)",
        )

    def test_same(self):
        self.assertEqual(
            build_history.size_suffix(1572, self.BASE),
            "tests: 1572 (same as 2026-10-08T19:46:00, tree a1b2c3d4e5)",
        )

    def test_no_record(self):
        self.assertEqual(build_history.size_suffix(68, None), "tests: 68 (no earlier record)")


class TreeIdentityTests(unittest.TestCase):
    """The tree name is the short HEAD, plus a content digest when dirty."""

    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.repo = Path(tmp.name)
        self.git("init", "-q")
        (self.repo / "a.txt").write_text("one\n")
        self.git("add", "a.txt")
        self.git(
            "-c", "user.email=t@example.com", "-c", "user.name=t",
            "commit", "-q", "-m", "init",
        )

    def git(self, *argv):
        return subprocess.run(
            ["git", "-C", str(self.repo), *argv], check=True, capture_output=True, text=True
        ).stdout.strip()

    def test_clean_tree_is_the_short_head(self):
        head = self.git("rev-parse", "--short=10", "HEAD")
        self.assertEqual(build_history.tree_identity(self.repo), head)

    def test_tracked_change_adds_a_digest(self):
        head = self.git("rev-parse", "--short=10", "HEAD")
        (self.repo / "a.txt").write_text("two\n")
        self.assertRegex(build_history.tree_identity(self.repo), rf"^{head}\+[0-9a-f]{{8}}$")

    def test_untracked_file_changes_the_digest(self):
        (self.repo / "a.txt").write_text("two\n")
        before = build_history.tree_identity(self.repo)
        (self.repo / "new.txt").write_text("x\n")
        with_new = build_history.tree_identity(self.repo)
        self.assertNotEqual(before, with_new)
        (self.repo / "new.txt").write_text("y\n")
        self.assertNotEqual(with_new, build_history.tree_identity(self.repo))

    def test_failed_blob_hash_keeps_a_dirty_name(self):
        head = self.git("rev-parse", "--short=10", "HEAD")
        (self.repo / "new.txt").write_text("x\n")
        real_run = subprocess.run

        def fail_hash_object(argv, **kwargs):
            if "hash-object" in argv:
                raise subprocess.CalledProcessError(128, argv)
            return real_run(argv, **kwargs)

        with mock.patch.object(build_history.subprocess, "run", side_effect=fail_hash_object):
            self.assertRegex(build_history.tree_identity(self.repo), rf"^{head}\+[0-9a-f]{{8}}$")

    def test_outside_a_repo_is_unknown(self):
        with tempfile.TemporaryDirectory() as plain:
            with mock.patch.dict(os.environ, {"GIT_CEILING_DIRECTORIES": plain}):
                self.assertEqual(build_history.tree_identity(Path(plain)), "unknown")
