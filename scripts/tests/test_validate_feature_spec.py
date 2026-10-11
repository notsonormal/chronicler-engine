"""Regression tests for `scripts/validate_feature_spec.py`.

Exercises the SCENARIO-tag scanner and the untagged rule with synthetic Rust
text: a tag anchors on a test attribute, and under `tests/browser/stub/` it
anchors on the `check_*` function that a module's `run_*` runner drives. Run
from the repo root via ``python -m unittest discover scripts/tests`` or via
``build.py``.
"""

from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / "scripts"))

import validate_feature_spec as vfs  # noqa: E402

STUB_REL = Path("tests/browser/stub/story_log.rs")
HTTP_REL = Path("tests/http/story_log.rs")

CHECK_TEXT = """// [docs/specs/browser_story_log.md] SCENARIO: 30.11
async fn check_text_selection_survives_the_poll(page: playwright_rs::Page, _stub: StubServer) {
    let _ = page;
}

#[tokio::test]
async fn run_story_log_checks() {
    let mut runner = StubRunner::launch().await;
    runner.run(
        StubActionOutcome::Pending,
        check_text_selection_survives_the_poll,
    )
    .await;
    runner.finish().await;
}
"""


def scan_text(text: str, name: str) -> tuple[Path, vfs.TestFileScan]:
    """Write `text` to a temp file named `name` and scan it."""
    directory = Path(tempfile.mkdtemp(prefix="feature_spec_scan_"))
    path = directory / name
    path.write_text(text, encoding="utf-8")
    return path, vfs.scan_test_file(path)


def untagged(text: str, rel: Path) -> list[tuple[Path, int, str]]:
    path, scan = scan_text(text, rel.name)
    return vfs.find_untagged_tests([(path, rel, scan)])


class CheckTagAnchorTests(unittest.TestCase):
    """A stub check carries the scenario tag on the function itself."""

    def test_tag_anchors_on_the_check_function(self) -> None:
        _path, scan = scan_text(CHECK_TEXT, STUB_REL.name)
        self.assertEqual(
            scan.annotations,
            [(1, 2, "docs/specs/browser_story_log.md", "30.11")],
            "the tag must anchor on the check function's line",
        )
        self.assertEqual(
            scan.checks,
            [(2, "check_text_selection_survives_the_poll")],
            "every check function is a tag target",
        )

    def test_tagged_check_is_not_untagged(self) -> None:
        self.assertEqual(untagged(CHECK_TEXT, STUB_REL), [])

    def test_untagged_check_is_reported(self) -> None:
        text = CHECK_TEXT.replace(
            "// [docs/specs/browser_story_log.md] SCENARIO: 30.11\n", ""
        )
        violations = untagged(text, STUB_REL)
        self.assertEqual(
            [(line, name) for _path, line, name in violations],
            [(1, "check_text_selection_survives_the_poll")],
            "a check without a tag must fail the tag rule",
        )


class RunnerExemptionTests(unittest.TestCase):
    """The `run_*` runner observes no scenario of its own."""

    def test_untagged_runner_is_exempt(self) -> None:
        text = CHECK_TEXT.replace(
            "// [docs/specs/browser_story_log.md] SCENARIO: 30.11\n", ""
        )
        names = [name for _path, _line, name in untagged(text, STUB_REL)]
        self.assertNotIn("run_story_log_checks", names)

    def test_untagged_non_runner_test_is_still_reported(self) -> None:
        text = """#[tokio::test]
async fn test_something_unrelated() {
    let _ = 1;
}
"""
        violations = untagged(text, STUB_REL)
        self.assertEqual(
            [name for _path, _line, name in violations],
            ["test_something_unrelated"],
            "the runner exemption is by name, not by directory",
        )

    def test_check_outside_the_stub_tier_is_not_a_tag_target(self) -> None:
        text = """async fn check_something(page: playwright_rs::Page) {
    let _ = page;
}
"""
        self.assertEqual(
            untagged(text, HTTP_REL),
            [],
            "only the stub tier's checks are tag targets",
        )


def unwired(text: str, rel: Path) -> list[tuple[Path, int, str]]:
    path, scan = scan_text(text, rel.name)
    return vfs.find_unwired_checks([(path, rel, scan)])


class CheckWiringTests(unittest.TestCase):
    """A tagged check counts as coverage only if a runner drives it."""

    def test_check_a_runner_drives_is_wired(self) -> None:
        self.assertEqual(unwired(CHECK_TEXT, STUB_REL), [])

    def test_check_no_runner_drives_is_reported(self) -> None:
        text = CHECK_TEXT.replace(
            """    runner.run(
        StubActionOutcome::Pending,
        check_text_selection_survives_the_poll,
    )
    .await;
""",
            "",
        )
        self.assertEqual(
            [(line, name) for _path, line, name in unwired(text, STUB_REL)],
            [(2, "check_text_selection_survives_the_poll")],
            "a tagged check that no runner calls covers nothing",
        )

    def test_check_outside_the_stub_tier_is_not_checked(self) -> None:
        text = """async fn check_something(page: playwright_rs::Page) {
    let _ = page;
}
"""
        self.assertEqual(unwired(text, HTTP_REL), [])


class TestAttributeAnchorTests(unittest.TestCase):
    """The pre-existing anchor: a tag above a test attribute."""

    def test_tag_anchors_on_the_test_attribute(self) -> None:
        text = """// [docs/specs/story_log.md] SCENARIO: 8.1
#[tokio::test]
async fn test_delete_last_between_actions_http() {
    let _ = 1;
}
"""
        _path, scan = scan_text(text, HTTP_REL.name)
        self.assertEqual(
            scan.annotations, [(1, 2, "docs/specs/story_log.md", "8.1")]
        )
        self.assertEqual(scan.checks, [])
        self.assertEqual(untagged(text, HTTP_REL), [])


if __name__ == "__main__":
    unittest.main()
