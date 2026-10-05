"""Unit tests for scripts/precommit_regenerate.py.

Run via ``python -m unittest discover scripts/tests`` or via ``build.py``.
"""

from __future__ import annotations

import contextlib
import io
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import precommit_regenerate  # noqa: E402

KEYS = ("AUTO-STRUCTURE",)
FILES = (("TARGET.md", KEYS),)
GENERATOR = (("gen.py", "test generation"),)

GEN_SCRIPT = """\
import re
from pathlib import Path

text = Path("TARGET.md").read_text()
text = re.sub(
    r"<!-- AUTO-STRUCTURE START -->.*?<!-- AUTO-STRUCTURE END -->",
    "<!-- AUTO-STRUCTURE START -->\\nGEN\\n<!-- AUTO-STRUCTURE END -->",
    text,
    flags=re.DOTALL,
)
Path("TARGET.md").write_text(text)
"""

FAIL_SCRIPT = """\
from pathlib import Path

Path("TARGET.md").write_text("PARTIAL\\n")
raise SystemExit(1)
"""


def target(region: str = "OLD", before: str = "prose before", after: str = "prose after") -> str:
    """Build a target file with hand-written prose around a generated block."""
    return (
        "# Target\n"
        f"{before}\n"
        "<!-- AUTO-STRUCTURE START -->\n"
        f"{region}\n"
        "<!-- AUTO-STRUCTURE END -->\n"
        f"{after}\n"
    )


class SplitProseTests(unittest.TestCase):
    def test_removes_only_the_generated_block(self):
        self.assertEqual(
            precommit_regenerate.split_prose(target(), KEYS),
            "# Target\nprose before\nprose after\n",
        )

    def test_keeps_prose_around_multiple_pairs_in_order(self):
        text = (
            "p\n<!-- A START -->\na\n<!-- A END -->\nm\n"
            "<!-- B START -->\nb\n<!-- B END -->\nq\n"
        )
        self.assertEqual(precommit_regenerate.split_prose(text, ("A", "B")), "p\nm\nq\n")

    def test_missing_start_marker_is_rejected(self):
        text = "p\n<!-- AUTO-STRUCTURE END -->\nq\n"
        self.assertIsNone(precommit_regenerate.split_prose(text, KEYS))

    def test_deleted_pair_is_rejected(self):
        self.assertIsNone(precommit_regenerate.split_prose("p\nq\n", KEYS))

    def test_out_of_order_markers_are_rejected(self):
        text = "<!-- AUTO-STRUCTURE END -->\n<!-- AUTO-STRUCTURE START -->\n"
        self.assertIsNone(precommit_regenerate.split_prose(text, KEYS))

    def test_extra_marker_pair_is_rejected(self):
        text = (
            "<!-- AUTO-STRUCTURE START -->\na\n<!-- AUTO-STRUCTURE END -->\n"
            "<!-- AUTO-INDEX START -->\nb\n<!-- AUTO-INDEX END -->\n"
        )
        self.assertIsNone(precommit_regenerate.split_prose(text, KEYS))


class RunTests(unittest.TestCase):
    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.root = Path(tmp.name).resolve()
        self._git("init", "-q", "-b", "main", ".")
        for key, value in (
            ("user.name", "t"),
            ("user.email", "t@t"),
            ("commit.gpgsign", "false"),
        ):
            self._git("config", key, value)
        (self.root / "gen.py").write_text(GEN_SCRIPT)
        self.write("TARGET.md", target())
        self._git("add", "--", "TARGET.md")
        self._git("commit", "-q", "-m", "init")

    def _git(self, *args: str) -> str:
        return subprocess.run(
            ["git", "-C", str(self.root), *args],
            check=True,
            capture_output=True,
            text=True,
        ).stdout

    def write(self, rel: str, text: str) -> None:
        (self.root / rel).write_text(text)

    def read(self, rel: str) -> str:
        return (self.root / rel).read_text()

    def staged(self, rel: str) -> str:
        return self._git("show", f":{rel}")

    def run_hook(self, generators=GENERATOR) -> tuple[int, str]:
        out = io.StringIO()
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(out):
            status = precommit_regenerate.run(
                self.root, files=FILES, generators=generators
            )
        return status, out.getvalue()

    def test_clean_worktree_is_regenerated_and_staged(self):
        status, out = self.run_hook()
        self.assertEqual(status, 0)
        self.assertIn("GEN", self.read("TARGET.md"))
        self.assertIn("GEN", self.staged("TARGET.md"))
        self.assertNotIn("regenerated", out)

    def test_stale_generated_block_is_regenerated_not_aborted(self):
        # Leftover output from a prior run is the false abort this hook must fix.
        self.write("TARGET.md", target(region="OLD OWN OUTPUT"))
        status, out = self.run_hook()
        self.assertEqual(status, 0)
        self.assertIn("GEN", self.staged("TARGET.md"))
        self.assertIn("regenerated TARGET.md", out)

    def test_foreign_prose_aborts_and_leaves_worktree_untouched(self):
        edited = target(before="prose before EDITED")
        self.write("TARGET.md", edited)
        status, out = self.run_hook()
        self.assertEqual(status, 1)
        self.assertEqual(self.read("TARGET.md"), edited)
        self.assertIn("OLD", self.staged("TARGET.md"))
        self.assertIn("outside its generated block", out)

    def test_in_block_edit_is_regenerated(self):
        self.write("TARGET.md", target(region="HAND EDITED"))
        status, _ = self.run_hook()
        self.assertEqual(status, 0)
        self.assertNotIn("HAND EDITED", self.read("TARGET.md"))
        self.assertIn("GEN", self.staged("TARGET.md"))

    def test_deleted_marker_pair_aborts_untouched(self):
        broken = "# Target\nprose before\nprose after\n"
        self.write("TARGET.md", broken)
        status, out = self.run_hook()
        self.assertEqual(status, 1)
        self.assertEqual(self.read("TARGET.md"), broken)
        self.assertIn("markers", out)

    def test_generator_failure_restores_worktree(self):
        (self.root / "fail.py").write_text(FAIL_SCRIPT)
        self.write("TARGET.md", target(region="GEN"))
        original = self.read("TARGET.md")
        status, out = self.run_hook(generators=(("fail.py", "test generation"),))
        self.assertEqual(status, 1)
        self.assertEqual(self.read("TARGET.md"), original)
        self.assertIn("restored the generated files", out)


if __name__ == "__main__":
    unittest.main()
