"""Regression tests for `scripts/generate_structure_index.py`.

Run via ``python -m unittest discover scripts/tests`` or via ``build.py``.
Covers the depth-4 rendering regression (agent `utils/` folders were silently
dropped by the old three-level walk), the top-level `mod.rs` hiding quirk,
and the module-summary extraction contract.
"""

from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / "scripts"))

import generate_structure_index as gsi  # noqa: E402


DOC = "//! [DOC: docs/diataxis/reference/narrative/agent_system.md]"


def _anchored(summary: str) -> str:
    return f"{DOC}\n//! {summary}\n"


def _plain(summary: str) -> str:
    return f"//! {summary}\n"


def _build_fixture(root: Path) -> None:
    """Mirror the repo shape: root file, depth-4 utils, and skip-cases."""
    (root / "error.rs").write_text(_plain("Error types and result aliases"))
    # No line-1 `//!` header — must be skipped.
    (root / "lib.rs").write_text("pub fn nothing() {}\n")
    # Anchor style without a line-2 summary — must be skipped.
    (root / "orphan.rs").write_text(f"{DOC}\npub fn orphan() {{}}\n")

    app = root / "application"
    (app / "agents" / "quantifier" / "utils").mkdir(parents=True)
    (app / "mod.rs").write_text(_plain("Application orchestrators"))
    (app / "agents" / "mod.rs").write_text(_anchored("Agent registry and trait definitions"))
    (app / "agents" / "quantifier" / "mod.rs").write_text(_anchored("Quantifier agent system"))
    (app / "agents" / "quantifier" / "agent.rs").write_text(_anchored("Quantifier agent implementation."))
    (app / "agents" / "quantifier" / "utils" / "mod.rs").write_text(_anchored("Quantifier utility modules."))
    (app / "agents" / "quantifier" / "utils" / "orchestration.rs").write_text(
        _anchored("Quantifier orchestration — LLM call + result processing + entry point.")
    )


class BuildBulletStructureTests(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self._tmp.cleanup)
        self.root = Path(self._tmp.name)
        _build_fixture(self.root)
        self.lines = gsi.build_bullet_structure(self.root).splitlines()

    def _bullet_line(self, needle: str) -> str | None:
        return next((l for l in self.lines if needle in l), None)

    def test_depth_four_utils_folder_renders(self) -> None:
        """The old three-level walk silently dropped depth-4 directories."""
        self.assertIsNotNone(self._bullet_line("**utils/**"))
        self.assertIsNotNone(self._bullet_line("`orchestration.rs`"))

    def test_depth_bullet_indentation(self) -> None:
        self.assertIn("  - **application/**", self.lines)
        self.assertIn("    - **agents/**", self.lines)
        self.assertIn("      - **quantifier/**", self.lines)
        self.assertIn("        - **utils/**", self.lines)

    def test_top_level_mod_rs_hidden_deeper_rendered(self) -> None:
        mod_lines = [l for l in self.lines if "mod.rs" in l]
        self.assertEqual(len(mod_lines), 3)
        # Deeper aggregator modules render; only the top-level one is hidden.
        self.assertTrue(any("`mod.rs` — Agent registry" in l for l in mod_lines))
        self.assertTrue(any("`mod.rs` — Quantifier agent system" in l for l in mod_lines))
        self.assertTrue(any("`mod.rs` — Quantifier utility modules." in l for l in mod_lines))

    def test_root_files_render_before_top_dirs(self) -> None:
        error_idx = self.lines.index("  - `error.rs` — Error types and result aliases")
        app_idx = self.lines.index("  - **application/**")
        self.assertLess(error_idx, app_idx)

    def test_files_without_module_summary_skipped(self) -> None:
        self.assertFalse(any("lib.rs" in l for l in self.lines))
        self.assertFalse(any("orphan.rs" in l for l in self.lines))


class ExtractModuleInfoTests(unittest.TestCase):
    def test_anchorless_summary_accepted(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "plain.rs"
            path.write_text(_plain("A summary"))
            self.assertEqual(gsi.extract_module_info(path), ("plain.rs", "A summary"))

    def test_anchor_without_line2_summary_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "orphan.rs"
            path.write_text(f"{DOC}\npub fn orphan() {{}}\n")
            self.assertIsNone(gsi.extract_module_info(path))


if __name__ == "__main__":
    unittest.main()
