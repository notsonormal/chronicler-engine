"""Shared helpers for the work-record tests: a file writer and a test case with a temp repo root."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path


def write(root: Path, rel: str, text: str = "") -> Path:
    path = root / rel
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")
    return path


class TempRootTestCase(unittest.TestCase):
    """A test case whose `self.root` is an empty temp folder, removed after each test."""

    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.root = Path(self._tmp.name)

    def tearDown(self) -> None:
        self._tmp.cleanup()
