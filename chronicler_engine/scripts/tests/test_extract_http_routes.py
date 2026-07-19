"""Regression tests for `scripts/extract_http_routes.py`.

Exercises the parser and emitter in isolation against synthetic router.rs
fixtures. The tests cover:

- Single-line `.route(...)` calls (the common case)
- Multi-line `.route(...)` calls (e.g. `debug::debug_state_handler`)
- `:id` / `:key` / `:index` path-parameter preservation
- Both `get(...)` and `post(...)` verbs
- Handler-module prefix grouping (the seven-area structure)
- Doc emission (front-matter, required H2s, table row counts)
- `use super::...` import normalization (bare handler → qualified)
- `validate_docs.py --strict` pass on the emitted doc

Mirrors the `scripts/tests/test_validate_docs.py` pattern: REPO_ROOT
sys.path insert, `_run_check` style helpers, unittest discoverable.

Run from the ``chronicler_engine/`` directory via
``python -m unittest discover scripts/tests`` or via ``build.py``.
"""

from __future__ import annotations

import re
import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path("/home/moridin84/projects/mrn-general")
sys.path.insert(0, str(REPO_ROOT / "chronicler_engine" / "scripts"))

import extract_http_routes as er  # noqa: E402

ENGINE_ROOT = REPO_ROOT / "chronicler_engine"


# ---------------------------------------------------------------------------
# Fixtures: synthetic router.rs content + golden helpers.
# ---------------------------------------------------------------------------


def _run_extract(source: str) -> list[er.Route]:
    """Run the parser against `source`; return the list of routes."""
    return er.extract_routes(source)


def _render(routes: list[er.Route]) -> str:
    """Render the doc for `routes`."""
    grouped = er.group_routes_by_area(routes)
    return er._render_document(routes, grouped)


def _front_matter() -> str:
    return "---\ndiataxis: reference\ntitle: HTTP Routes\n---\n"


def _single_line_fixture() -> str:
    """A synthetic router with only single-line .route(...) calls."""
    return (
        _front_matter()
        + "\n"
        + ".route(\"/\", get(handlers::index_handler))\n"
        + ".route(\"/fragment/header\", get(fragments::header_fragment))\n"
        + ".route(\"/action\", post(fragments::action_handler))\n"
        + ".route(\"/games\", post(games_fragment::create_game_handler))\n"
        + ".route(\"/debug/state\", get(debug::debug_state_handler))\n"
    )


def _multi_line_fixture() -> str:
    """A synthetic router with several multi-line .route(...) calls."""
    return (
        _front_matter()
        + "\n"
        + ".route(\n"
        + "    \"/fragment/visual-sidebar\",\n"
        + "    get(fragments::visual_sidebar_fragment),\n"
        + ")\n"
        + ".route(\n"
        + "    \"/worlds/:key/delete\",\n"
        + "    post(worlds_fragment::delete_world_handler),\n"
        + ")\n"
        + ".route(\"/debug/state\", get(debug::debug_state_handler))\n"
    )


def _bare_handler_fixture() -> str:
    """A synthetic router that uses bare handler references (no `::`)."""
    return (
        "use super::handlers::index_handler;\n"
        + "use super::fragments;\n"
        + "use super::debug;\n"
        + "\n"
        + ".route(\"/\", get(index_handler))\n"
        + ".route(\"/fragment/header\", get(fragments::header_fragment))\n"
        + ".route(\"/debug/state\", get(debug::debug_state_handler))\n"
    )


# ---------------------------------------------------------------------------
# Parser tests.
# ---------------------------------------------------------------------------


class TestParserSingleLine(unittest.TestCase):
    """Single-line `.route(...)` calls — the common shape."""

    def test_parses_each_route(self) -> None:
        routes = _run_extract(_single_line_fixture())
        self.assertEqual(len(routes), 5)

    def test_extracts_method_get(self) -> None:
        routes = _run_extract(_single_line_fixture())
        self.assertEqual(routes[0].verb, "GET")
        self.assertEqual(routes[1].verb, "GET")

    def test_extracts_method_post(self) -> None:
        routes = _run_extract(_single_line_fixture())
        self.assertEqual(routes[2].verb, "POST")

    def test_extracts_path(self) -> None:
        routes = _run_extract(_single_line_fixture())
        self.assertEqual(routes[0].path, "/")
        self.assertEqual(routes[1].path, "/fragment/header")
        self.assertEqual(routes[2].path, "/action")

    def test_extracts_handler(self) -> None:
        routes = _run_extract(_single_line_fixture())
        self.assertEqual(routes[0].handler, "handlers::index_handler")
        self.assertEqual(routes[4].handler, "debug::debug_state_handler")


class TestParserMultiLine(unittest.TestCase):
    """Multi-line `.route(...)` calls where args span several lines."""

    def test_multi_line_parses_as_single_route(self) -> None:
        routes = _run_extract(_multi_line_fixture())
        # 3 .route(...) calls, each potentially multi-line.
        self.assertEqual(len(routes), 3)

    def test_multi_line_extracts_path(self) -> None:
        routes = _run_extract(_multi_line_fixture())
        self.assertEqual(routes[0].path, "/fragment/visual-sidebar")
        self.assertEqual(routes[1].path, "/worlds/:key/delete")

    def test_multi_line_extracts_handler(self) -> None:
        routes = _run_extract(_multi_line_fixture())
        self.assertEqual(
            routes[0].handler, "fragments::visual_sidebar_fragment"
        )
        self.assertEqual(
            routes[1].handler, "worlds_fragment::delete_world_handler"
        )


class TestParserPathParameters(unittest.TestCase):
    """:id / :key / :index path parameters must be preserved verbatim."""

    def test_id_param_preserved(self) -> None:
        routes = _run_extract(
            ".route(\"/history/:id\", post(fragments::edit_history_handler))\n"
        )
        self.assertEqual(routes[0].path, "/history/:id")

    def test_key_param_preserved(self) -> None:
        routes = _run_extract(
            ".route(\"/worlds/:key\", post(worlds_fragment::update_world_handler))\n"
        )
        self.assertEqual(routes[0].path, "/worlds/:key")

    def test_index_param_preserved(self) -> None:
        routes = _run_extract(
            ".route("
            "\"/message/:id/swipe/:index\", "
            "post(fragments::switch_swipe_handler))\n"
        )
        self.assertEqual(routes[0].path, "/message/:id/swipe/:index")

    def test_multiple_params_in_one_path(self) -> None:
        routes = _run_extract(
            ".route("
            "\"/prompt-presets/:id/activate\", "
            "post(prompt_presets_fragment::activate_preset_handler))\n"
        )
        self.assertEqual(routes[0].path, "/prompt-presets/:id/activate")


class TestParserUseImportNormalization(unittest.TestCase):
    """Bare handler references are normalized via `use super::...` imports."""

    def test_bare_handler_normalized(self) -> None:
        routes = _run_extract(_bare_handler_fixture())
        # index_handler → handlers::index_handler
        self.assertEqual(routes[0].handler, "handlers::index_handler")

    def test_already_qualified_handler_unchanged(self) -> None:
        routes = _run_extract(_bare_handler_fixture())
        # fragments::header_fragment stays qualified (already has ::).
        self.assertEqual(routes[1].handler, "fragments::header_fragment")

    def test_unknown_bare_handler_left_alone(self) -> None:
        # A bare identifier with no matching `use` import is left as-is
        # (it will surface in the "Unknown areas" section).
        routes = _run_extract(
            "use super::fragments;\n"
            + ".route(\"/x\", get(some_unknown_handler))\n"
        )
        self.assertEqual(routes[0].handler, "some_unknown_handler")


# ---------------------------------------------------------------------------
# Grouping tests.
# ---------------------------------------------------------------------------


class TestGrouping(unittest.TestCase):
    """Routes are grouped by handler-module prefix → area."""

    def test_seven_areas_initialized(self) -> None:
        routes = _run_extract(_single_line_fixture())
        grouped = er.group_routes_by_area(routes)
        # All seven area names from AREA_GROUPS appear, even if empty.
        for _prefix, area in er.AREA_GROUPS:
            self.assertIn(area, grouped)

    def test_fragments_grouped_together(self) -> None:
        routes = _run_extract(
            ".route(\"/a\", get(fragments::a))\n"
            + ".route(\"/b\", get(fragments::b))\n"
            + ".route(\"/c\", get(fragments::c))\n"
        )
        grouped = er.group_routes_by_area(routes)
        self.assertEqual(
            len(grouped["Action / Status / History / Lifecycle"]), 3
        )

    def test_unknown_prefix_bucketed(self) -> None:
        routes = _run_extract(
            ".route(\"/x\", get(mystery_module::handler))\n"
        )
        grouped = er.group_routes_by_area(routes)
        self.assertEqual(len(grouped["_unknown"]), 1)


# ---------------------------------------------------------------------------
# Doc emission tests.
# ---------------------------------------------------------------------------


class TestDocEmission(unittest.TestCase):
    """The rendered doc has the structure mandated by the ticket."""

    def setUp(self) -> None:
        self.routes = _run_extract(_single_line_fixture())
        self.doc = _render(self.routes)

    def test_front_matter_diataxis_reference(self) -> None:
        self.assertIn("diataxis: reference", self.doc)

    def test_front_matter_title(self) -> None:
        self.assertIn("title: HTTP Routes", self.doc)

    def test_overview_h2(self) -> None:
        self.assertIn("## Overview", self.doc)

    def test_document_references_h2(self) -> None:
        self.assertIn("## Document References", self.doc)

    def test_seven_area_h2s(self) -> None:
        for _prefix, area in er.AREA_GROUPS:
            self.assertIn(f"## {area}", self.doc)

    def test_table_row_count_matches_routes(self) -> None:
        # Each `.route(...)` call must yield exactly one table row in the
        # rendered doc. Count by counting `| GET |` and `| POST |` rows.
        rows = re.findall(r"^\| (?:GET|POST) \|", self.doc, flags=re.MULTILINE)
        self.assertEqual(len(rows), len(self.routes))

    def test_table_renders_backticked_path_and_handler(self) -> None:
        # Pick the index_handler row specifically.
        self.assertIn(
            "| GET | `/` | `handlers::index_handler` |", self.doc
        )


# ---------------------------------------------------------------------------
# End-to-end test against the real router.rs.
# ---------------------------------------------------------------------------


class TestRealRouter(unittest.TestCase):
    """The script runs end-to-end against the real router.rs file."""

    def test_real_router_yields_52_routes(self) -> None:
        router_path = ENGINE_ROOT / er.ROUTER_REL
        if not router_path.exists():
            self.skipTest(f"router.rs not found: {router_path}")
        source = router_path.read_text(encoding="utf-8")
        routes = er.extract_routes(source)
        # Hard criterion from the ticket: must be 52 routes.
        self.assertEqual(len(routes), 52)

    def test_real_router_route_count_matches_grep(self) -> None:
        # Cross-check the parser's count against `grep -c '\.route('` on
        # the same file. This guards against drift between the parser and
        # the file's structure.
        router_path = ENGINE_ROOT / er.ROUTER_REL
        if not router_path.exists():
            self.skipTest(f"router.rs not found: {router_path}")
        source = router_path.read_text(encoding="utf-8")
        grep_count = source.count(".route(")
        routes = er.extract_routes(source)
        self.assertEqual(len(routes), grep_count)

    def test_real_router_doc_passes_validator(self) -> None:
        """The emitted doc passes validate_docs.py --strict.

        We write the doc to a temp file under a fake engine root and run
        the validator in single-file mode so the warning from the
        pre-existing architecture.md does not contaminate the result.
        The fake engine root includes stub files for the relative links
        the doc body references (`../../src/...`, `../../chronicler_engine/scripts/...`).
        """
        router_path = ENGINE_ROOT / er.ROUTER_REL
        if not router_path.exists():
            self.skipTest(f"router.rs not found: {router_path}")

        # Late import to keep import order clean.
        sys.path.insert(
            0,
            str(REPO_ROOT / "chronicler_engine" / "scripts"),
        )
        import validate_docs as vd

        source = router_path.read_text(encoding="utf-8")
        routes = er.extract_routes(source)
        grouped = er.group_routes_by_area(routes)
        rendered = er._render_document(routes, grouped)

        with tempfile.TemporaryDirectory(
            prefix="http-routes-validate-"
        ) as tmp_str:
            fake_engine_root = Path(tmp_str)
            # Build the minimum directory structure so the doc's relative
            # body links resolve to existing files. The validator checks
            # link existence via the docs_root resolution; for our doc
            # (under docs/diataxis/reference/), the docs_root it computes
            # is <engine_root>/docs/diataxis, and the doc's relative links
            # traverse `../../` back into engine_root territory.
            (
                fake_engine_root / "docs" / "diataxis" / "reference" / "frontend"
            ).mkdir(parents=True)
            (fake_engine_root / "docs" / "AGENTS.md").write_text(
                "# stub\n", encoding="utf-8"
            )
            (fake_engine_root / "src" / "adapters" / "driving" / "http").mkdir(
                parents=True
            )
            (
                fake_engine_root
                / "src"
                / "adapters"
                / "driving"
                / "http"
                / "router.rs"
            ).write_text("// stub\n", encoding="utf-8")
            (fake_engine_root / "chronicler_engine" / "scripts").mkdir(
                parents=True
            )
            (
                fake_engine_root
                / "chronicler_engine"
                / "scripts"
                / "extract_http_routes.py"
            ).write_text("# stub\n", encoding="utf-8")

            doc_path = (
                fake_engine_root
                / "docs"
                / "diataxis"
                / "reference"
                / "http_routes.md"
            )
            doc_path.write_text(rendered, encoding="utf-8")

            report = vd.scan_file(doc_path, fake_engine_root)
            self.assertEqual(
                [v.rule for v in report.violations],
                [],
                f"violations: {report.violations}",
            )

    def test_rendered_doc_table_rows_sum_to_route_count(self) -> None:
        """Sum of per-area table rows in the emitted doc equals 52.

        This is the ticket's completion criterion #3 — the visible marker
        that no route was silently dropped.
        """
        router_path = ENGINE_ROOT / er.ROUTER_REL
        if not router_path.exists():
            self.skipTest(f"router.rs not found: {router_path}")
        source = router_path.read_text(encoding="utf-8")
        routes = er.extract_routes(source)
        grouped = er.group_routes_by_area(routes)
        rendered = er._render_document(routes, grouped)

        rows = re.findall(r"^\| (?:GET|POST) \|", rendered, flags=re.MULTILINE)
        self.assertEqual(len(rows), len(routes))
        self.assertEqual(len(rows), 52)


# ---------------------------------------------------------------------------
# CLI / writer tests.
# ---------------------------------------------------------------------------


class TestWriter(unittest.TestCase):
    """write_document + CLI sanity checks."""

    def test_write_document_round_trip(self) -> None:
        with tempfile.TemporaryDirectory(prefix="http-routes-write-") as tmp_str:
            tmp = Path(tmp_str)
            routes = _run_extract(_single_line_fixture())
            grouped = er.group_routes_by_area(routes)
            rendered = er._render_document(routes, grouped)

            out_path = (
                tmp / "docs" / "diataxis" / "reference" / "frontend" / "http_routes.md"
            )
            written = er.write_document(tmp, rendered)
            self.assertEqual(written, out_path)
            self.assertTrue(out_path.exists())
            self.assertEqual(out_path.read_text(encoding="utf-8"), rendered)

    def test_stdout_flag_does_not_write_file(self) -> None:
        # Smoke test: --stdout returns 0 (no file written). We capture
        # stdout via subprocess-style invocation of main().
        import io

        captured = io.StringIO()
        old_stdout = sys.stdout
        sys.stdout = captured
        try:
            rc = er.main(["--stdout", "--router", str(ENGINE_ROOT / er.ROUTER_REL)])
        finally:
            sys.stdout = old_stdout
        self.assertEqual(rc, 0)
        # Output starts with front-matter.
        self.assertTrue(captured.getvalue().startswith("---"))


if __name__ == "__main__":
    unittest.main()