"""Regression tests for `scripts/validate_docs.py`.

Exercises each rule in isolation by invoking the validator's parser/check
helpers directly with synthetic text. Run from the repo root
directory via ``python -m unittest discover scripts/tests`` or via ``build.py``.

Pinned at 13 markdown-front-matter fixtures + 13 DOC-anchor fixtures. Adding a
new rule to the validator without a fixture here will show up in code review as
a missing test.
"""

from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / "scripts"))

import validate_docs as vd  # noqa: E402


class _FakeReport:
    """Stand-in for FileReport so check_* helpers work in isolation.

    The helpers write ``read_text(report)`` to a temp file under ``tmp_path``
    via the real implementation; we only need a path + violations list.
    """

    def __init__(self, path: Path) -> None:
        self.path = path
        self.violations: list[vd.Violation] = []


def _run_check(
    tmp_path: Path,
    text: str,
) -> list[str]:
    """Parse + check synthetic body; return the rule names found."""
    path = tmp_path / "fixture.md"
    path.write_text(text, encoding="utf-8")
    report = _FakeReport(path)
    vd.check_diataxis_frontmatter(
        report,  # type: ignore[arg-type]
    )
    return sorted({v.rule for v in report.violations})


class TestDiataxisValidator(unittest.TestCase):
    """One test per rule plus two clean-doc baselines."""

    def test_missing_frontmatter(self) -> None:
        rules = _run_check(
            self._make_tmp(),
            "# A heading\n\nJust body, no front-matter.\n",
        )
        self.assertEqual(rules, ["MISSING_FRONTMATTER"])

    def test_empty_frontmatter(self) -> None:
        rules = _run_check(
            self._make_tmp(),
            "---\n---\n# Body\n",
        )
        self.assertEqual(rules, ["EMPTY_FRONTMATTER"])

    def test_unterminated_frontmatter(self) -> None:
        rules = _run_check(
            self._make_tmp(),
            "---\ndiataxis: reference\ntitle: Foo\n# Body without closing fence\n",
        )
        self.assertEqual(rules, ["FRONTMATTER_PARSE_ERROR"])

    def test_yaml_parse_error(self) -> None:
        rules = _run_check(
            self._make_tmp(),
            "---\ndiataxis: [unclosed\n---\n# Body\n",
        )
        self.assertEqual(rules, ["FRONTMATTER_PARSE_ERROR"])

    def test_frontmatter_not_mapping(self) -> None:
        rules = _run_check(
            self._make_tmp(),
            "---\n- one\n- two\n---\n# Body\n",
        )
        self.assertEqual(rules, ["FRONTMATTER_NOT_MAPPING"])

    def test_missing_diataxis_key(self) -> None:
        rules = _run_check(
            self._make_tmp(),
            "---\ntitle: Foo\n---\n# Body\n",
        )
        self.assertEqual(rules, ["FRONTMATTER_MISSING_KEY"])

    def test_missing_title_key(self) -> None:
        rules = _run_check(
            self._make_tmp(),
            "---\ndiataxis: reference\n---\n# Body\n",
        )
        self.assertEqual(rules, ["FRONTMATTER_MISSING_KEY"])

    def test_invalid_mode_vocab(self) -> None:
        rules = _run_check(
            self._make_tmp(),
            "---\ndiataxis: walkthrough\ntitle: Foo\n---\n# Body\n",
        )
        self.assertEqual(rules, ["FRONTMATTER_INVALID_MODE"])

    def test_arc52_not_a_list(self) -> None:
        rules = _run_check(
            self._make_tmp(),
            "---\ndiataxis: reference\ntitle: Foo\narc52: §3\n---\n# Body\n",
        )
        self.assertEqual(rules, ["FRONTMATTER_INVALID_ARC52"])

    def test_arc52_bad_entries(self) -> None:
        rules = _run_check(
            self._make_tmp(),
            "---\ndiataxis: reference\ntitle: Foo\narc52: [§3, §6, §11]\n---\n# Body\n",
        )
        self.assertEqual(rules, ["FRONTMATTER_INVALID_ARC52"])

    def test_reference_clean(self) -> None:
        rules = _run_check(
            self._make_tmp(),
            "---\ndiataxis: reference\ntitle: Foo\n---\n# Body\n\n"
            "Tables exist with these columns. The schema is enumerated below.\n",
        )
        self.assertEqual(rules, [])

    def test_all_clean(self) -> None:
        rules = _run_check(
            self._make_tmp(),
            "---\ndiataxis: reference\ntitle: Foo\n---\n# Body\n\n"
            "Table A: columns are id, key, name.\n",
        )
        self.assertEqual(rules, [])

    @staticmethod
    def _make_tmp() -> Path:
        """Return a fresh tmp directory rooted under the system temp dir."""
        import tempfile

        return Path(tempfile.mkdtemp(prefix="diataxis-fix-"))


class _LineCitationFixture:
    """Builds a synthetic docs/diataxis tree for LINE_CITATION checks."""

    def __init__(self) -> None:
        self.docs_root = Path(tempfile.mkdtemp(prefix="line-cit-")) / "docs" / "diataxis"
        self.docs_root.mkdir(parents=True)

    def write(self, rel: str, text: str) -> Path:
        path = self.docs_root / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
        return path

    def scan(self, path: Path) -> list[vd.Violation]:
        report = vd.FileReport(path)
        vd.check_line_citations(report, self.docs_root)
        return report.violations


class TestLineCitations(unittest.TestCase):
    """LINE_CITATION: flags `path:line`, exempts fenced blocks and generated docs."""

    def test_flags_path_line_citation(self) -> None:
        fix = _LineCitationFixture()
        path = fix.write("reference/foo.md", "See `src/bar.rs:42` for the details.\n")
        rules = [v.rule for v in fix.scan(path)]
        self.assertEqual(rules, ["LINE_CITATION"])

    def test_flags_line_range_and_list(self) -> None:
        fix = _LineCitationFixture()
        path = fix.write(
            "reference/foo.md",
            "A `src/a.rs:12-14`, B `src/b.rs:12,15`.\n",
        )
        self.assertEqual(len(fix.scan(path)), 2)

    def test_clean_symbol_citation(self) -> None:
        fix = _LineCitationFixture()
        path = fix.write(
            "reference/foo.md",
            "See `Storage::get_settings` and `resolve_api_key()`.\n",
        )
        self.assertEqual(fix.scan(path), [])

    def test_fenced_block_exempt(self) -> None:
        fix = _LineCitationFixture()
        path = fix.write(
            "reference/foo.md",
            "Real output:\n\n```\nthread 'x' panicked at src/bar.rs:42\n```\n",
        )
        self.assertEqual(fix.scan(path), [])

    def test_generated_doc_exempt(self) -> None:
        fix = _LineCitationFixture()
        path = fix.write(
            "reference/coding_standards/guardrails.md",
            "| rule | `tests/infrastructure/guardrails/enums.rs:69` |\n",
        )
        self.assertEqual(fix.scan(path), [])


class _SourcePathFixture:
    """Builds a synthetic engine root for BROKEN_SOURCE_PATH checks."""

    def __init__(self) -> None:
        self.engine = Path(tempfile.mkdtemp(prefix="src-path-")) / "engine"
        self.docs_root = self.engine / "docs" / "diataxis"
        self.docs_root.mkdir(parents=True)
        (self.engine / "src").mkdir(parents=True)

    def write_doc(self, rel: str, text: str) -> Path:
        path = self.docs_root / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
        return path

    def scan(self, path: Path) -> list[vd.Violation]:
        report = vd.FileReport(path)
        vd.check_source_paths(report, self.engine)
        return report.violations


class TestSourcePaths(unittest.TestCase):
    """BROKEN_SOURCE_PATH: flags missing repo paths, exempts patterns and fences."""

    def test_flags_missing_src_path(self) -> None:
        fix = _SourcePathFixture()
        path = fix.write_doc(
            "explanation/foo.md", "See `src/gone.rs` for details.\n"
        )
        rules = [v.rule for v in fix.scan(path)]
        self.assertEqual(rules, ["BROKEN_SOURCE_PATH"])

    def test_flags_missing_tests_path(self) -> None:
        fix = _SourcePathFixture()
        path = fix.write_doc("how-to/foo.md", "Run `tests/gone.rs`.\n")
        self.assertEqual(len(fix.scan(path)), 1)

    def test_existing_path_passes(self) -> None:
        fix = _SourcePathFixture()
        (fix.engine / "src" / "here.rs").write_text("", encoding="utf-8")
        path = fix.write_doc("explanation/foo.md", "See `src/here.rs`.\n")
        self.assertEqual(fix.scan(path), [])

    def test_root_doc_flags_missing_path(self) -> None:
        fix = _SourcePathFixture()
        path = fix.engine / "tests" / "AGENTS.md"
        path.parent.mkdir(parents=True)
        path.write_text("Mirror: `src/gone_tests.rs`.\n", encoding="utf-8")
        report = vd.scan_root_doc(path, fix.engine)
        self.assertEqual([v.rule for v in report.violations], ["BROKEN_SOURCE_PATH"])

    def test_root_docs_name_the_hand_written_agent_docs(self) -> None:
        self.assertIn("AGENTS.md", vd.ROOT_DOCS)
        self.assertIn("tests/AGENTS.md", vd.ROOT_DOCS)

    def test_glob_and_line_citation_spans_skipped(self) -> None:
        fix = _SourcePathFixture()
        path = fix.write_doc(
            "explanation/foo.md",
            "`src/domain/**` and `src/gone.rs:42` are not plain paths.\n",
        )
        self.assertEqual(fix.scan(path), [])

    def test_fenced_block_exempt(self) -> None:
        fix = _SourcePathFixture()
        path = fix.write_doc(
            "explanation/foo.md",
            "Tree:\n\n```\nsrc/gone.rs\n```\n",
        )
        self.assertEqual(fix.scan(path), [])

    def test_data_path_out_of_scope(self) -> None:
        fix = _SourcePathFixture()
        path = fix.write_doc(
            "explanation/foo.md", "Runtime file `data/chronicler.db`.\n"
        )
        self.assertEqual(fix.scan(path), [])

    def test_generated_doc_exempt(self) -> None:
        fix = _SourcePathFixture()
        path = fix.write_doc(
            "reference/coding_standards/guardrails.md",
            "See `src/gone.rs`.\n",
        )
        self.assertEqual(fix.scan(path), [])


class _AnchorFixture:
    """Builds a synthetic engine-root tree under a tmp dir for anchor checks."""

    def __init__(self) -> None:
        self.root = Path(tempfile.mkdtemp(prefix="anchor-fix-"))
        self.engine = self.root
        (self.engine / "src").mkdir(parents=True)
        (self.engine / "tests").mkdir(parents=True)
        (self.engine / "docs" / "diataxis" / "reference").mkdir(parents=True)

    def write_storage_md(self) -> Path:
        path = self.engine / "docs" / "diataxis" / "reference" / "storage.md"
        path.write_text("---\ndiataxis: reference\ntitle: Storage\n---\n# Storage\n", encoding="utf-8")
        return path

    def write_src_file(self, rel: str, text: str) -> Path:
        path = self.engine / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
        return path

    def write_test_support_file(self, name: str, text: str) -> Path:
        return self.write_src_file(f"src/test_support/{name}", text)

    def scan(self, path: Path) -> list[vd.Violation]:
        report = vd.FileReport(path)
        vd.check_doc_anchors(report, path, self.engine)
        vd.check_test_support_rules(report, path, self.engine)
        return report.violations


class TestDocAnchors(unittest.TestCase):
    """Fixtures A-L exercising the DOC anchor rules."""

    def test_fixture_a_valid_full_path_anchor(self) -> None:
        fix = _AnchorFixture()
        fix.write_storage_md()
        path = fix.write_src_file(
            "src/foo.rs",
            "//! [DOC: docs/diataxis/reference/storage.md]\n//! Summary.\n",
        )
        self.assertEqual(fix.scan(path), [])

    def test_fixture_b_bogus_target_missing(self) -> None:
        fix = _AnchorFixture()
        path = fix.write_src_file(
            "src/foo.rs",
            "//! [DOC: docs/diataxis/reference/missing.md]\n//! Summary.\n",
        )
        rules = {v.rule for v in fix.scan(path)}
        self.assertEqual(rules, {"BROKEN_DOC_ANCHOR"})

    def test_fixture_d_non_doc_comment_line_still_detected(self) -> None:
        fix = _AnchorFixture()
        path = fix.write_src_file(
            "src/foo.rs",
            "// [DOC: docs/diataxis/reference/storage.md]\n",
        )
        rules = {v.rule for v in fix.scan(path)}
        self.assertEqual(rules, {"BROKEN_DOC_ANCHOR"})

    def test_fixture_e_plain_path_form_parsed_normally(self) -> None:
        fix = _AnchorFixture()
        path = fix.write_src_file(
            "src/foo.rs",
            "//! [DOC: docs/system/storage.md]\n//! Summary.\n",
        )
        rules = {v.rule for v in fix.scan(path)}
        self.assertEqual(rules, {"BROKEN_DOC_ANCHOR"})

    def test_fixture_f_test_support_anchor_forbidden(self) -> None:
        fix = _AnchorFixture()
        path = fix.write_test_support_file(
            "x.rs",
            "//! [DOC: docs/diataxis/reference/storage.md]\n//! Summary.\n",
        )
        rules = {v.rule for v in fix.scan(path)}
        self.assertEqual(rules, {"TEST_SUPPORT_ANCHOR_FORBIDDEN"})

    def test_fixture_g_test_support_clean_summary(self) -> None:
        fix = _AnchorFixture()
        path = fix.write_test_support_file("x.rs", "//! Test helper summary.\n")
        self.assertEqual(fix.scan(path), [])

    def test_fixture_h_test_support_empty_summary_required(self) -> None:
        fix = _AnchorFixture()
        path = fix.write_test_support_file("x.rs", "//!\n")
        rules = {v.rule for v in fix.scan(path)}
        self.assertEqual(rules, {"TEST_SUPPORT_SUMMARY_REQUIRED"})

    def test_fixture_i_tests_file_anchor_forbidden(self) -> None:
        fix = _AnchorFixture()
        path = fix.engine / "tests" / "x.rs"
        path.write_text(
            "//! [DOC: docs/diataxis/reference/storage.md]\n",
            encoding="utf-8",
        )
        rules = {v.rule for v in fix.scan(path)}
        self.assertEqual(rules, {"TEST_FILES_ANCHOR_FORBIDDEN"})

    def test_fixture_j_no_anchors_flag_absent(self) -> None:
        # Fixture J: per plan, there is no `--no-anchors` opt-out flag. The
        # argparse schema does not declare it, so parse_args rejects it.
        with self.assertRaises(SystemExit):
            vd.parse_args(["--no-anchors"])

    def test_fixture_k_violation_message_has_lineno_and_path(self) -> None:
        fix = _AnchorFixture()
        path = fix.write_src_file(
            "src/foo.rs",
            "//! [DOC: docs/system/storage.md]\n//! Summary.\n",
        )
        violations = fix.scan(path)
        self.assertEqual(len(violations), 1)
        msg = violations[0].message
        self.assertIn("Line 1:", msg)
        self.assertIn("src/foo.rs", msg)

    def test_fixture_l_live_anchor_scan_smoke(self) -> None:
        # Fixture L: collect_anchor_files on live engine-root returns files and
        # scanning them produces ZERO anchor-related violations post-migration
        # (proves the full cutover succeeded end-to-end).
        engine_root = REPO_ROOT
        files = vd.collect_anchor_files(engine_root)
        self.assertGreater(len(files), 100)
        anchor_violations: list[vd.Violation] = []
        for f in files:
            report = vd.scan_anchor_file(f, engine_root)
            anchor_violations.extend(
                v for v in report.violations
                if v.rule in {
                    "BROKEN_DOC_ANCHOR",
                    "TEST_SUPPORT_ANCHOR_FORBIDDEN",
                    "TEST_FILES_ANCHOR_FORBIDDEN",
                    "TEST_SUPPORT_SUMMARY_REQUIRED",
                }
            )
        self.assertEqual(
            anchor_violations,
            [],
            f"Expected zero anchor violations on live tree, got {len(anchor_violations)}",
        )

    def test_fixture_m_explanation_target_rejected(self) -> None:
        # Fixture M: anchor pointing at diataxis/explanation/ is rejected —
        # only `docs/diataxis/reference/` is allowed.
        fix = _AnchorFixture()
        explanation_md = fix.engine / "docs" / "diataxis" / "explanation" / "prompt_system_design.md"
        explanation_md.parent.mkdir(parents=True, exist_ok=True)
        explanation_md.write_text("---\ndiataxis: explanation\ntitle: P\n---\n# P\n", encoding="utf-8")
        path = fix.write_src_file(
            "src/foo.rs",
            "//! [DOC: docs/diataxis/explanation/prompt_system_design.md]\n//! Summary.\n",
        )
        violations = fix.scan(path)
        self.assertEqual(len(violations), 1)
        self.assertEqual(violations[0].rule, "BROKEN_DOC_ANCHOR")
        self.assertIn("must resolve under docs/diataxis/reference/", violations[0].message)

    def test_fixture_n_path_traversal_rejected(self) -> None:
        # Fixture N: `reference/../../explanation/foo.md` escapes via `..` and
        # is rejected even though the string starts with the canonical prefix.
        fix = _AnchorFixture()
        explanation_md = fix.engine / "docs" / "diataxis" / "explanation" / "foo.md"
        explanation_md.parent.mkdir(parents=True, exist_ok=True)
        explanation_md.write_text("---\ndiataxis: explanation\ntitle: F\n---\n# F\n", encoding="utf-8")
        path = fix.write_src_file(
            "src/foo.rs",
            "//! [DOC: docs/diataxis/reference/../../explanation/foo.md]\n//! Summary.\n",
        )
        rules = {v.rule for v in fix.scan(path)}
        self.assertEqual(rules, {"BROKEN_DOC_ANCHOR"})

    def test_fixture_o_tests_file_clean_no_anchor(self) -> None:
        # Fixture O: negative fixture for TEST_FILES_ANCHOR_FORBIDDEN — a
        # tests/*.rs file with a plain `//! <summary>` line is clean.
        fix = _AnchorFixture()
        path = fix.engine / "tests" / "clean.rs"
        path.write_text("//! A clean test summary.\n", encoding="utf-8")
        self.assertEqual(fix.scan(path), [])

    def test_fixture_p_format_string_literal_not_detected(self) -> None:
        # Fixture P: a Rust format string like `format!("... [DOC: ...] ...")`
        # must not be mistaken for a real anchor — the strict regex anchors to
        # line-start with an optional comment prefix only.
        fix = _AnchorFixture()
        path = fix.write_src_file(
            "src/foo.rs",
            "            format!(\"Module `{path}` lacks a module-level DOC anchor. Add `//! [DOC: ...]` at the top of the file.\")\n",
        )
        self.assertEqual(fix.scan(path), [])

    def test_fixture_q_suffix_form_in_test_support_still_forbidden(self) -> None:
        # Fixture Q: legacy `— section \"...\"` suffix form in test_support is
        # still caught by the forbidden rule (uses the permissive DOC_ANCHOR_LINE
        # regex, not the strict path-only regex).
        fix = _AnchorFixture()
        path = fix.write_test_support_file(
            "x.rs",
            "//! [DOC: docs/diataxis/reference/storage.md \u2014 section \"Foo\"]\n//! Summary.\n",
        )
        rules = {v.rule for v in fix.scan(path)}
        self.assertEqual(rules, {"TEST_SUPPORT_ANCHOR_FORBIDDEN"})


class TestRouteCheck(unittest.TestCase):
    """UNKNOWN_ROUTE: route strings in docs and skills must be in the generated table."""

    TABLE = (
        "| Method | Path | Handler |\n|---|---|---|\n"
        "| GET | `/` | `index_handler` |\n"
        "| GET | `/status/busy` | `busy_status_handler` |\n"
        "| POST | `/games/:id/switch` | `switch_game_handler` |\n"
        "| POST | `/message/:id/swipe/:index` | `switch_swipe_handler` |\n"
        "| GET | `/fragment/header` | `header_fragment` |\n"
    )

    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self._tmp.cleanup)
        self.root = Path(self._tmp.name)
        table = self.root / vd.ROUTES_DOC
        table.parent.mkdir(parents=True)
        table.write_text(self.TABLE, encoding="utf-8")

    def messages(self, text: str, rel: str = "docs/diataxis/how-to/x.md") -> list[str]:
        path = self.root / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
        (report,) = vd.scan_routes([path], self.root)
        return [v.message for v in report.violations]

    def test_prefixes_come_from_the_table(self) -> None:
        routes = vd.load_route_table(self.root)
        self.assertEqual(vd.route_prefixes(routes), {"status", "games", "message", "fragment"})

    def test_listed_routes_pass_including_params_and_hosts(self) -> None:
        text = (
            "Poll `/status/busy`, then `curl http://localhost:3001/fragment/header`.\n"
            "```sh\ncurl -X POST localhost:$PORT/games/7/switch\n```\n"
            "Use `/message/{id}/swipe/2` or `/games/:id/switch`.\n"
        )
        self.assertEqual(self.messages(text), [])

    def test_unknown_route_is_an_error_with_line_and_route(self) -> None:
        text = "ok `/status/busy`\nstale `/status/generating` here\n"
        (message,) = self.messages(text)
        self.assertIn("Line 2", message)
        self.assertIn("`/status/generating`", message)

    def test_wrong_segment_count_is_unknown(self) -> None:
        self.assertEqual(len(self.messages("`/games/7/switch/now` and `/games/7`")), 2)

    def test_source_paths_and_file_paths_are_not_routes(self) -> None:
        text = (
            "`application/games/active.rs`, `data/settings.json`, "
            "`<target-dir>/status/x`, `http/fragment/y`.\n"
        )
        self.assertEqual(self.messages(text), [])

    def test_family_mentions_are_not_routes(self) -> None:
        self.assertEqual(self.messages("`/fragment/\u2026`, `/fragment/*`, `/fragment/<name>`, `/fragment/`"), [])

    def test_trailing_punctuation_is_not_part_of_the_route(self) -> None:
        self.assertEqual(self.messages("See /status/busy. Also (/fragment/header)."), [])

    def test_collect_route_files_exempts_plans_old_docs_and_the_table(self) -> None:
        for rel in (
            "docs/plans/p.md",
            "docs/old-docs/o.md",
            "docs/specs/s.md",
            ".agents/skills/demo/SKILL.md",
            ".agents/skills/demo/other.md",
        ):
            path = self.root / rel
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("x\n", encoding="utf-8")
        found = {p.relative_to(self.root).as_posix() for p in vd.collect_route_files(self.root)}
        self.assertEqual(found, {"docs/specs/s.md", ".agents/skills/demo/SKILL.md"})

    def test_missing_route_table_is_an_error(self) -> None:
        (self.root / vd.ROUTES_DOC).unlink()
        (report,) = vd.scan_routes([], self.root)
        self.assertEqual([v.rule for v in report.violations], ["UNKNOWN_ROUTE"])

    def test_live_table_lists_a_param_route_and_rejects_a_removed_one(self) -> None:
        routes = vd.load_route_table(REPO_ROOT)
        self.assertTrue(vd.route_is_listed("/message/3/swipe/1", routes))
        self.assertTrue(vd.route_is_listed("/status/busy", routes))
        self.assertFalse(vd.route_is_listed("/status/generating", routes))



class TestPathOption(unittest.TestCase):
    def test_path_accepts_a_diataxis_doc(self) -> None:
        doc = REPO_ROOT / "docs" / "diataxis" / "reference" / "game_flow.md"
        self.assertIn(vd.main(["--path", str(doc), "--list"]), (0, 1))

    def test_path_rejects_a_doc_outside_diataxis(self) -> None:
        self.assertEqual(vd.main(["--path", str(REPO_ROOT / "AGENTS.md")]), 2)


if __name__ == "__main__":
    unittest.main()
