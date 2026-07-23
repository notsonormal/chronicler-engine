"""Regression tests for `scripts/find_free_fn_smells.py`."""

from __future__ import annotations

import contextlib
import io
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

import find_free_fn_smells as scanner  # noqa: E402


class TestSuppressionList(unittest.TestCase):
    def test_suppression_requires_known_path_and_name(self) -> None:
        known_path = Path("src/bootstrap/load.rs")
        self.assertTrue(scanner.is_suppressed(known_path, "seed_game_data"))
        self.assertFalse(scanner.is_suppressed(Path("src/other/load.rs"), "seed_game_data"))
        self.assertFalse(scanner.is_suppressed(known_path, "unknown_function"))

    def test_unsuppressed_smell_makes_scanner_fail(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "unexpected.rs").write_text(
                "pub fn misplaced(value: &DomainType) {}\n",
                encoding="utf-8",
            )
            with contextlib.redirect_stdout(io.StringIO()):
                result = scanner.main(["find_free_fn_smells.py", str(root)])
        self.assertEqual(result, 1)


if __name__ == "__main__":
    unittest.main()
