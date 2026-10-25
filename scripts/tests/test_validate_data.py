"""Regression tests for `scripts/validate_data.py`.

Run via ``python -m unittest discover scripts/tests`` or via ``build.py``.
"""

from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / "scripts"))

import validate_data as vd  # noqa: E402


class ScenarioCharacterTests(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self._tmp.cleanup)
        self.chars_dir = Path(self._tmp.name)
        for name in ("carla", "bartender"):
            (self.chars_dir / f"{name}.json").write_text("{}", encoding="utf-8")

    def test_reads_the_key_characters(self) -> None:
        scenario = {"id": "s1", "characters": ["carla"]}
        self.assertEqual(vd.scenario_character_ids(scenario), ["carla"])
        self.assertEqual(vd.missing_scenario_characters([scenario], self.chars_dir), [])

    def test_reads_the_older_key_npcs(self) -> None:
        scenario = {"id": "s1", "npcs": ["bartender", "ghost"]}
        self.assertEqual(vd.scenario_character_ids(scenario), ["bartender", "ghost"])
        self.assertEqual(
            vd.missing_scenario_characters([scenario], self.chars_dir),
            [("s1", "ghost", self.chars_dir / "ghost.json")],
        )

    def test_an_unknown_character_id_is_reported(self) -> None:
        scenario = {"id": "s1", "characters": ["carla", "nobody"]}
        self.assertEqual(
            vd.missing_scenario_characters([scenario], self.chars_dir),
            [("s1", "nobody", self.chars_dir / "nobody.json")],
        )

    def test_a_scenario_with_neither_key_names_no_character(self) -> None:
        self.assertEqual(vd.scenario_character_ids({"id": "s1"}), [])
        self.assertEqual(vd.missing_scenario_characters([{"id": "s1"}], self.chars_dir), [])


if __name__ == "__main__":
    unittest.main()
