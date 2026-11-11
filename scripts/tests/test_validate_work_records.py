"""Regression tests for `scripts/validate_work_records.py`.

Each test builds a small repo tree in a temp folder and runs the validator on
it. Run from the repo root via ``python -m unittest discover scripts/tests``
or via ``build.py``.
"""

from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / "scripts"))

import validate_work_records  # noqa: E402
from work_records_support import TempRootTestCase, write  # noqa: E402


class RecordLinkTests(TempRootTestCase):
    def setUp(self) -> None:
        super().setUp()
        write(self.root, "work/wayfinder-maps/alpha/issues/01-first.md", "# First\n")

    def rules(self) -> list[tuple[str, str]]:
        return [(v.rule, v.detail) for v in validate_work_records.validate(self.root)]

    def test_a_link_that_resolves_passes(self) -> None:
        write(self.root, "work/wayfinder-maps/alpha/map.md", "[first](issues/01-first.md#answer)\n")
        self.assertEqual(self.rules(), [])

    def test_a_broken_link_in_a_ticket_fails(self) -> None:
        write(self.root, "work/wayfinder-maps/alpha/issues/02-second.md", "[gone](03-gone.md)\n")
        self.assertEqual(self.rules(), [("BROKEN_RECORD_LINK", "03-gone.md")])

    def test_a_broken_link_in_a_one_off_issue_fails(self) -> None:
        write(self.root, "work/issues/pending/04-loose.md", "[gone](../ready/09-moved.md)\n")
        self.assertEqual(self.rules(), [("BROKEN_RECORD_LINK", "../ready/09-moved.md")])

    def test_links_in_fenced_code_and_html_comments_are_not_links(self) -> None:
        write(
            self.root,
            "work/wayfinder-maps/alpha/map.md",
            "<!-- [ticket](link) -->\n<!--\n[x](nowhere.md)\n-->\n```\n[y](nowhere.md)\n```\n",
        )
        self.assertEqual(self.rules(), [])

    def test_assets_and_closed_maps_are_not_checked(self) -> None:
        write(self.root, "work/wayfinder-maps/alpha/assets/notes.md", "[gone](nowhere.md)\n")
        write(self.root, "work/_closed/old/map.md", "[gone](nowhere.md)\n")
        write(self.root, "work/issues/assets/notes.md", "[gone](nowhere.md)\n")
        self.assertEqual(self.rules(), [])

    def test_urls_and_anchors_are_not_checked(self) -> None:
        write(self.root, "work/wiki/index.md", "[web](https://example.com) [here](#top)\n")
        self.assertEqual(self.rules(), [])


class CitedWorkPathTests(TempRootTestCase):
    def setUp(self) -> None:
        super().setUp()
        write(self.root, "work/issues/pending/01-one.md")

    def rules(self) -> list[tuple[str, str]]:
        return [(v.rule, v.detail) for v in validate_work_records.validate(self.root)]

    def test_a_stale_backticked_path_in_a_doc_fails(self) -> None:
        write(self.root, "docs/agents/tracker.md", "See `work/issues/ready/01-one.md`.\n")
        self.assertEqual(self.rules(), [("STALE_WORK_PATH", "work/issues/ready/01-one.md")])

    def test_a_stale_path_in_a_skill_file_fails(self) -> None:
        write(self.root, ".agents/skills/x/mise.toml", "# see `work/wayfinder-maps/gone/`\n")
        self.assertEqual(self.rules(), [("STALE_WORK_PATH", "work/wayfinder-maps/gone/")])

    def test_placeholder_and_glob_paths_are_cut_to_their_prefix(self) -> None:
        write(
            self.root,
            "AGENTS.md",
            "`work/issues/<status>/NN-<slug>.md` `work/issues/*/NN-*` `work/issues/pending/` `work/`\n",
        )
        self.assertEqual(self.rules(), [])

    def test_a_placeholder_under_a_missing_folder_fails(self) -> None:
        write(self.root, "AGENTS.md", "`work/wikis/<page>.md`\n")
        self.assertEqual(self.rules(), [("STALE_WORK_PATH", "work/wikis/<page>.md")])

    def test_unbackticked_and_fenced_paths_are_not_checked(self) -> None:
        write(self.root, "docs/a.md", "work/gone/x.md\n```\n`work/gone/y.md`\n```\n")
        self.assertEqual(self.rules(), [])

    def test_a_fence_inside_an_html_comment_does_not_hide_later_lines(self) -> None:
        write(self.root, "docs/a.md", "<!--\n```\n-->\n`work/gone/z.md`\n")
        self.assertEqual(self.rules(), [("STALE_WORK_PATH", "work/gone/z.md")])


class MainTests(unittest.TestCase):
    def test_exit_code_follows_the_violations(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(root, "work/wiki/index.md", "[ok](index.md)\n")
            self.assertEqual(validate_work_records.main(["--root", str(root)]), 0)
            write(root, "work/wiki/page.md", "[gone](nowhere.md)\n")
            self.assertEqual(validate_work_records.main(["--root", str(root)]), 1)


if __name__ == "__main__":
    unittest.main()
