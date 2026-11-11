"""Regression tests for `scripts/move_issue.py` and the link rewrite it shares (`scripts/work_records.py`).

Each test builds a small repo tree in a temp folder. Run from the repo root
via ``python -m unittest discover scripts/tests`` or via ``build.py``.
"""

from __future__ import annotations

import contextlib
import io
import re
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / "scripts"))

import move_issue  # noqa: E402
import validate_work_records  # noqa: E402
import work_records  # noqa: E402
from work_records_support import TempRootTestCase, write  # noqa: E402


class MoveIssueTests(TempRootTestCase):
    def setUp(self) -> None:
        super().setUp()
        write(self.root, "work/issues/pending/07-fix-it.md", "[sibling](08-other.md)\n")
        write(self.root, "work/issues/pending/08-other.md", "[fix](07-fix-it.md#answer)\n")
        write(
            self.root,
            "work/wayfinder-maps/alpha/map.md",
            "[fix](../../issues/pending/07-fix-it.md) and `work/issues/pending/07-fix-it.md`\n",
        )
        write(self.root, "docs/agents/tracker.md", "Cited: `work/issues/pending/07-fix-it.md`.\n")

    def read(self, rel: str) -> str:
        return (self.root / rel).read_text(encoding="utf-8")

    def test_the_issue_moves_and_every_citation_follows(self) -> None:
        new = move_issue.move_issue(self.root, "07", "ready")
        self.assertEqual(new, self.root / "work/issues/ready/07-fix-it.md")
        self.assertFalse((self.root / "work/issues/pending/07-fix-it.md").exists())
        self.assertEqual(self.read("work/issues/pending/08-other.md"), "[fix](../ready/07-fix-it.md#answer)\n")
        self.assertEqual(
            self.read("work/wayfinder-maps/alpha/map.md"),
            "[fix](../../issues/ready/07-fix-it.md) and `work/issues/ready/07-fix-it.md`\n",
        )
        self.assertEqual(self.read("docs/agents/tracker.md"), "Cited: `work/issues/ready/07-fix-it.md`.\n")
        self.assertEqual(validate_work_records.validate(self.root), [])

    def test_the_moved_issue_keeps_its_own_links(self) -> None:
        move_issue.move_issue(self.root, "07", "closed")
        self.assertEqual(self.read("work/issues/closed/07-fix-it.md"), "[sibling](../pending/08-other.md)\n")

    def test_an_unknown_number_fails(self) -> None:
        with self.assertRaises(move_issue.IssueNotFound):
            move_issue.move_issue(self.root, "99", "ready")

    def test_a_number_on_two_files_fails(self) -> None:
        write(self.root, "work/issues/closed/07-twin.md")
        with self.assertRaises(move_issue.IssueNotFound):
            move_issue.move_issue(self.root, "07", "ready")

    def test_main_reports_an_unknown_number(self) -> None:
        self.assertEqual(move_issue.main(["99", "ready", "--root", str(self.root)]), 1)

    def test_a_number_needs_two_or_more_digits(self) -> None:
        for bad in ("7", "x7", "07a", "-07", ""):
            with self.subTest(number=bad), contextlib.redirect_stderr(io.StringIO()) as err:
                with self.assertRaises(SystemExit) as raised:
                    move_issue.main([bad, "ready", "--root", str(self.root)])
                self.assertEqual(raised.exception.code, 2)
                self.assertIn("two or more digits", err.getvalue())
        self.assertTrue((self.root / "work/issues/pending/07-fix-it.md").exists())

    def test_a_folder_with_a_trailing_slash_is_accepted(self) -> None:
        write(self.root, "work/issues/pending/07-fix-it.md")
        self.assertEqual(move_issue.main(["07", "ready/", "--root", str(self.root)]), 0)
        self.assertTrue((self.root / "work/issues/ready/07-fix-it.md").exists())

    def test_a_number_of_three_digits_is_accepted(self) -> None:
        write(self.root, "work/issues/pending/107-big.md")
        self.assertEqual(move_issue.main(["107", "closed", "--root", str(self.root)]), 0)


class RewriteLinksTests(TempRootTestCase):
    def rewrite(self, source: Path, moves: dict[Path, Path]) -> str:
        return work_records.rewrite_links_in_text(source, source.read_text(encoding="utf-8"), moves)

    def test_a_moved_folder_moves_its_files_and_links_follow_both_ends(self) -> None:
        source = write(self.root, "old/a/map.md", "[b](../b/map.md#top) [web](https://x.y) [gone](nope.md)\n")
        write(self.root, "old/b/map.md")
        moves = {self.root / "old/a": self.root / "new/deep/a"}
        self.assertEqual(
            self.rewrite(source, moves),
            "[b](../../../old/b/map.md#top) [web](https://x.y) [gone](nope.md)\n",
        )

    def test_a_link_that_still_points_right_stays_byte_for_byte(self) -> None:
        source = write(self.root, "old/a/map.md", "[asset](./assets/x.md)\n")
        write(self.root, "old/a/assets/x.md")
        moves = {self.root / "old/a": self.root / "new/a"}
        self.assertEqual(self.rewrite(source, moves), "[asset](./assets/x.md)\n")

    def test_image_and_reference_links_are_rewritten(self) -> None:
        source = write(self.root, "doc.md", "![pic](old/p.png)\n[ref]: old/p.png\n")
        write(self.root, "old/p.png")
        self.assertEqual(
            self.rewrite(source, {self.root / "old": self.root / "new"}),
            "![pic](new/p.png)\n[ref]: new/p.png\n",
        )

    def test_backticked_paths_follow_the_longest_moved_prefix(self) -> None:
        moves = {
            self.root / "work/alpha": self.root / "work/wayfinder-maps/alpha",
            self.root / "work/alpha/issues/01-x.md": self.root / "work/issues/pending/01-x.md",
        }
        text = "`work/alpha/map.md` `work/alpha/issues/01-x.md` `work/<map>/` `work/`\n"
        self.assertEqual(
            work_records.rewrite_backticked(text, moves, self.root),
            "`work/wayfinder-maps/alpha/map.md` `work/issues/pending/01-x.md` `work/<map>/` `work/`\n",
        )

    def test_a_caller_can_pass_its_own_token_pattern(self) -> None:
        old = re.compile(r"(?<![\w/.-])(\.scratch(?:/[^\s`]*)?)")
        moves = {self.root / ".scratch/alpha": self.root / "work/alpha"}
        text = "`.scratch/alpha/m.md`\n"
        self.assertEqual(work_records.rewrite_backticked(text, moves, self.root, old), "`work/alpha/m.md`\n")
        self.assertEqual(work_records.rewrite_backticked(text, moves, self.root), text)

    def test_backticked_paths_in_fenced_code_stay(self) -> None:
        moves = {self.root / "work/alpha": self.root / "work/beta"}
        text = "```\n`work/alpha/x.md`\n```\n`work/alpha/x.md`\n"
        self.assertEqual(
            work_records.rewrite_backticked(text, moves, self.root),
            "```\n`work/alpha/x.md`\n```\n`work/beta/x.md`\n",
        )


if __name__ == "__main__":
    unittest.main()
