# DOC anchor cutover → strict full-path with validator enforcement (no suffix form)

## Summary

All ~190 `//! [DOC: ...]` line-1 anchors in `src/` point at legacy `docs/system|reference|architecture/<name>.md` paths that no longer exist after the diátaxis folder cutover. `validate_docs.py` only scans `docs/diataxis/`, so the drift is invisible, and the Rust guardrail at `tests/infrastructure/guardrails/structure.rs` rejects any anchor not starting with `docs/` (blocking the planned full-path rewrite).

This plan:

1. **Build the validator first.** Add three new rules to `validate_docs.py`:
   - `BROKEN_DOC_ANCHOR` — fires when a `[DOC: ...]` target file doesn't exist, OR the path is missing the `chronicler_engine/` prefix (two message variants of one rule).
   - `TEST_SUPPORT_ANCHOR_FORBIDDEN` — fires when any `src/test_support/*.rs` carries a `[DOC: ...]` line.
   - `TEST_FILES_ANCHOR_FORBIDDEN` — fires when any `tests/**/*.rs` carries a `[DOC: ...]` line (mirrors `tests/infrastructure/guardrails/structure.rs` ADR-028 rule from Rust side into Python).
   Plus a `TEST_SUPPORT_SUMMARY_REQUIRED` rule that fires when a `src/test_support/*.rs` file's line 1 is missing/empty. The validator does NOT support the `— section "..."` suffix form — anchors are path-only. `--anchors` flag defaults ON; no `--no-anchors` (YAGNI).
2. **Apply the rewrite.** Use a small ~40-line `scripts/migrate_doc_anchors.py` with hardcoded mapping tuples that asserts every new target resolves on disk BEFORE writing any file. Run, watch validator violation count collapse to zero.
3. **Strip anchors from `src/test_support/*.rs`** (6 files: `context.rs`, `fixtures.rs`, `noop_forensics.rs`, `test_app_builder.rs`, `test_data_builder.rs`, `recording_forensics.rs`) — delete line 1, also delete any now-dangling empty `//!` lines, ensure the resulting line 1 is a non-empty `//! <summary>`. Existing `— section "..."` suffixes in those 4 files go away with the anchor; no special handling needed.
4. **Update the existing Rust guardrail predicate** so it accepts the new `chronicler_engine/docs/...` full-path form (replaces the `docs/...` short form). Section-suffix form is removed entirely.
5. Convention updates (skill, CHANGELOG, archive).

## Anchor path form (locked — single canonical form)

```
//! [DOC: chronicler_engine/docs/diataxis/<reference|explanation|how-to>/<name>.md]
//! [DOC: chronicler_engine/docs/diataxis/<subdir>/<name>.md]            // subdir: coding_standards | frontend | narrative
```

- Leading `chronicler_engine/` is **required**. Short form `docs/diataxis/...` is rejected by the validator.
- **No section suffix** (`— section "Foo"`). The four `src/test_support/*.rs` files that previously used the suffix will lose the anchor entirely (Phase 2.3); no other source files use it.
- `src/test_support/*.rs` MUST NOT have a `[DOC: ...]` line; line 1 must be `//! <summary>` (non-empty).
- `tests/**/*.rs` MUST NOT have a `[DOC: ...]` line (matches ADR-028).

## Key Changes

| Where | What |
|---|---|
| `chronicler_engine/scripts/validate_docs.py` | Add `check_doc_anchors(report, anchor_path, engine_root)` enforcing `BROKEN_DOC_ANCHOR`, `TEST_SUPPORT_ANCHOR_FORBIDDEN`, `TEST_FILES_ANCHOR_FORBIDDEN`, `TEST_SUPPORT_SUMMARY_REQUIRED`. Add `TARGET_GLOBS = ("src/**/*.rs", "tests/**/*.rs", "*.toml")` enumeration in `main()`. New CLI flag `--anchors` (default ON; no opt-out). Test-support path detection via `Path.parts` containing `"test_support"` segment under `src/`. |
| `chronicler_engine/scripts/tests/test_validate_docs.py` | New fixtures A–L (see Phase 1). Plus an explicit fixture asserting `--anchors` is the implicit default. Fixture E removed (no suffix variant). |
| `chronicler_engine/scripts/migrate_doc_anchors.py` | ~40-line one-off: hardcoded `MAPPING: list[tuple[str, str]]` of `(legacy, new)` tuples; CLI = `python migrate_doc_anchors.py [--apply]`; iterates `engine_root.rglob("*.rs")` + `engine_root.glob("*.toml")` (default-skips `**/test_support/**`; asserts each `new_target` resolves on disk before ANY write; emits `path:line: <old> -> <new>` report). Asserts path-form too: rejects any anchor that doesn't already match `legacy` exactly. Deleted at task end. |
| `chronicler_engine/src/**/*.rs` (~185 production files) | Line-1 `[DOC: <legacy>]` → `[DOC: <full diataxis path>]` via `migrate_doc_anchors.py`. |
| `chronicler_engine/src/test_support/*.rs` (6 files) | Manual edit per Phase 2 below. |
| `chronicler_engine/arch-lint.toml` | Header comment line 2 rewritten by the script. |
| `chronicler_engine/tests/infrastructure/guardrails/structure.rs` | Update predicate `!anchor.starts_with("docs/")` (line ~99) to `!anchor.starts_with("chronicler_engine/docs/")`. Update corresponding error message. Section-suffix code path (if any) removed. |
| `.agents/skills/chronicler-comment-fixer/SKILL.md` | Replace example with full-path form; drop any section-suffix mention. Add the canonical rules + test_support/test_files exclusions. |
| `chronicler_engine/docs/CHANGELOG.md` | Top-of-file entry. |
| `chronicler_engine/docs/plans/doc-anchor-cutover-plan.md` | This plan; archived to `chronicler_engine/old-docs/archived-plans/` at task close. |

## Legacy → diátaxis mapping (script has these hardcoded)

```python
MAPPING = [
    ("docs/system/storage.md",            "chronicler_engine/docs/diataxis/reference/storage.md"),
    ("docs/system/game_flow.md",           "chronicler_engine/docs/diataxis/reference/game_flow.md"),
    ("docs/system/startup.md",            "chronicler_engine/docs/diataxis/reference/startup.md"),
    ("docs/system/dashboard.md",          "chronicler_engine/docs/diataxis/reference/frontend/dashboard.md"),
    ("docs/system/agent_system.md",       "chronicler_engine/docs/diataxis/reference/narrative/agent_system.md"),
    ("docs/system/prompt_system.md",      "chronicler_engine/docs/diataxis/reference/narrative/prompt_system.md"),
    ("docs/system/worlds.md",             "chronicler_engine/docs/diataxis/reference/game_flow.md"),
    ("docs/system/navigation.md",         "chronicler_engine/docs/diataxis/reference/game_flow.md"),
    ("docs/system/triggers.md",           "chronicler_engine/docs/diataxis/reference/game_flow.md"),
    ("docs/system/character_state.md",    "chronicler_engine/docs/diataxis/reference/narrative/agent_system.md"),
    ("docs/system/llm_processing.md",     "chronicler_engine/docs/diataxis/explanation/prompt_system_design.md"),
    ("docs/system/text_check.md",         "chronicler_engine/docs/diataxis/reference/coding_standards/testing.md"),
    ("docs/architecture/system.md",       "chronicler_engine/docs/diataxis/reference/architecture_system.md"),
    ("docs/architecture/guardrails.md",   "chronicler_engine/docs/diataxis/reference/coding_standards/guardrails.md"),
]
# NOTE: docs/reference/test_support.md not in MAPPING — anchors with that target
#       are stripped under src/test_support/, not rewritten.
```

Pre-flight: every entry's `new_target` has been verified to exist on disk under `chronicler_engine/docs/diataxis/`.

## Implementation

### Phase 1: Build validator (3 SP)

- [ ] #### Task 1.1: Implement three new rules + tests (3 SP)
  - [ ] ##### SubTask 1.1.1: `check_doc_anchors` (1 SP)
    - Regex `DOC_ANCHOR = re.compile(r"\[DOC: ([a-zA-Z0-9_/.\\-]+\.md)\s*\]")`. (No suffix group; suffix form is unsupported.)
    - On each match: extract `target`. If doesn't start with `chronicler_engine/` → `BROKEN_DOC_ANCHOR` (path-form variant message: `"DOC anchor must start with chronicler_engine/"`). Resolve against `engine_root`; missing file → `BROKEN_DOC_ANCHOR` (target-missing variant: `"DOC anchor target file does not exist"`).
    - Two message variants under one rule name.
  - [ ] ##### SubTask 1.1.2: `check_test_support_rules` (1 SP)
    - Function takes `(report, anchor_path, engine_root)`. Determines if path is under `src/test_support/` or `tests/`. If under either and any line matches `DOC_ANCHOR` → emit the appropriate forbidden rule (`TEST_SUPPORT_ANCHOR_FORBIDDEN` or `TEST_FILES_ANCHOR_FORBIDDEN`).
    - For `src/test_support/` files ONLY: assert line 1 starts with `//!` AND is non-empty (after the prefix). If empty/missing → `TEST_SUPPORT_SUMMARY_REQUIRED`. (`tests/` files get only the forbidden rule, no summary check.)
  - [ ] ##### SubTask 1.1.3: Wire + CLI (0.5 SP)
    - In `main()`: collect `anchor_files = chain(rglob("*.rs"), glob("*.toml"))` filtered to engine-root-relative paths. Add `scan_anchor_file(path, engine_root)` parallel to `scan_file()`.
    - `--anchors` flag, default ON. No `--no-anchors`.
  - [ ] ##### SubTask 1.1.4: Tests (0.5 SP)
    - Fixture A: valid `//! [DOC: chronicler_engine/docs/diataxis/reference/storage.md]` in `tmp/.../foo.rs` → 0 violations.
    - Fixture B: bogus target → `BROKEN_DOC_ANCHOR` (target-missing variant).
    - Fixture C: short-form `docs/diataxis/reference/storage.md` (no prefix) → `BROKEN_DOC_ANCHOR` (path-form variant).
    - Fixture D: `// [DOC: ...]` (no `!`) in toml-shaped fixture → still detected.
    - Fixture E (replaces old section-suffix test): a non-section-suffix form is detected and parsed normally. (Old section-suffix test deleted.)
    - Fixture F: `tmp/.../src/test_support/x.rs` with `[DOC: ...]` → `TEST_SUPPORT_ANCHOR_FORBIDDEN` (1).
    - Fixture G: same file without anchor but with non-empty `//!` line 1 → 0 violations.
    - Fixture H: same file with empty `//!` line 1 → `TEST_SUPPORT_SUMMARY_REQUIRED`.
    - Fixture I: `tmp/.../tests/x.rs` with `[DOC: ...]` → `TEST_FILES_ANCHOR_FORBIDDEN` (1).
    - Fixture J: explicit `--anchors` defaults; no flag = scans anchors anyway. Catches future regressions.
    - Fixture K: `(line_number, anchor_path)` printed in violation message includes the full path so failures are actionable.
    - Fixture L: scan against live engine-root should report ~185 `BROKEN_DOC_ANCHOR` (target-missing) on FIRST RUN (pre-rewrite) and ZERO after Phase 2.2.

### Phase 2: Apply rewrite + test_support manual edit (3 SP)

- [ ] #### Task 2.1: Migration script (1 SP)
  - [ ] ##### SubTask 2.1.1: Implement `scripts/migrate_doc_anchors.py` (1 SP)
    - ~40 lines. Hardcoded `MAPPING` constant (14 tuples above). `if __name__ == "__main__":` reads argv directly; no argparse.
    - For each `(legacy, new)`: assert `(engine_root / new).exists()` before any write — abort with clear error if not.
    - Walk `engine_root.rglob("*.rs")` + `engine_root.glob("*.toml")`. Skip any path under `src/test_support/`. (Defence-in-depth even though Phase 2.3 handles those manually.)
    - For each file: iterate lines; for each match, replace `legacy` with `new`. (No suffix preservation; the four `src/test_support/*.rs` files with `— section` form are skipped by the path filter.) Default = dry-run; `--apply` writes. Exit code = number of files changed.
- [ ] #### Task 2.2: Run script + observe validator collapse (1 SP)
  - [ ] ##### SubTask 2.2.1: Dry-run + apply (1 SP)
    - `python chronicler_engine/scripts/migrate_doc_anchors.py` — dry-run report; expect ~185 src/ files + 1 arch-lint.toml.
    - `python chronicler_engine/scripts/validate_docs.py --anchors` BEFORE apply → confirm ~185 `BROKEN_DOC_ANCHOR` violations, count = baseline.
    - `python chronicler_engine/scripts/migrate_doc_anchors.py --apply` → apply.
    - `python chronicler_engine/scripts/validate_docs.py --anchors` AFTER apply → expect zero `BROKEN_DOC_ANCHOR` for production files (excluding test_support violations still pending).
    - `grep -rn 'docs/system\|docs/reference/[a-z]\|docs/architecture/[a-z]' chronicler_engine/src chronicler_engine/arch-lint.toml` returns zero hits.
    - `cargo check --all-targets` exits 0.
- [ ] #### Task 2.3: Manual edit of 6 `src/test_support/*.rs` files (1 SP)
  - [ ] ##### SubTask 2.3.1: Per file (1 SP)
    - For each of: `context.rs`, `fixtures.rs`, `noop_forensics.rs`, `test_app_builder.rs`, `test_data_builder.rs`, `recording_forensics.rs`:
      1. Read first 5 lines.
      2. Delete the `[DOC: ...]` line (line 1). The `— section "X"` suffix where present goes with it.
      3. Examine remaining lines; if the new line 1 is a non-empty `//! <summary>` (already exists in some files), great. If the new line 1 is empty `//!` or starts with `#![...]` attribute (special case: `noop_forensics.rs:3`, `fixtures.rs:3`, `test_app_builder.rs:3`, `test_data_builder.rs:3` all have `#![allow(...)]`), keep as-is and ensure SUMMARY comes from a non-attribute `//!` line OR insert a one-liner based on file purpose.
      4. Delete any trailing empty `//!` lines orphaned by the deletion.
    - After all 6 edits: `grep -rn '\[DOC:' chronicler_engine/src/test_support/` returns zero hits. `python chronicler_engine/scripts/validate_docs.py --anchors` reports zero `TEST_SUPPORT_*` violations. `cargo check --all-targets` exits 0.

### Phase 3: Rust guardrail predicate + skill (1 SP)

- [ ] #### Task 3.1: Align existing Rust guardrail (0.5 SP)
  - [ ] ##### SubTask 3.1.1: Update `tests/infrastructure/guardrails/structure.rs:99` (0.5 SP)
    - Change `if !anchor.starts_with("docs/")` to `if !anchor.starts_with("chronicler_engine/docs/")`. Update the error message string to say "must start with `chronicler_engine/docs/...`".
    - If any `— section "..."` handling exists in this file, simplify/remove it (the suffix form is unsupported going forward).
    - `cargo nextest run -E 'test(/structure/)'` green.
- [ ] #### Task 3.2: Skill update (0.5 SP)
  - [ ] ##### SubTask 3.2.1: `chronicler-comment-fixer/SKILL.md` (0.5 SP)
    - Replace the example with `//! [DOC: chronicler_engine/docs/diataxis/reference/startup.md]` (full path, no suffix).
    - Drop any mention of section suffix.
    - Add canonical rules:
      - "Anchor target must be the full repo path (`chronicler_engine/docs/diataxis/<...>.md`)."
      - "No section suffix. Path-only anchors."
      - "`src/test_support/*.rs` MUST NOT carry a `[DOC: ...]` line — shared test helpers are organised by fixture weight (ADR-028); a `//! <summary>` line on line 1 suffices."
      - "`tests/**/*.rs` MUST NOT carry a `[DOC: ...]` line."

### Phase 4: Persistence (0.5 SP)

- [ ] #### Task 4.1: CHANGELOG + archive + script deletion (0.5 SP)
  - [ ] ##### SubTask 4.1.1: `docs/CHANGELOG.md` top entry (0.25 SP)
    - One-line summary referencing the plan.
  - [ ] ##### SubTask 4.1.2: Move plan to `old-docs/archived-plans/` (0.125 SP)
    - Move `chronicler_engine/docs/plans/doc-anchor-cutover-plan.md` to `chronicler_engine/old-docs/archived-plans/doc-anchor-cutover-plan.md`.
  - [ ] ##### SubTask 4.1.3: Delete one-off artefacts (0.125 SP)
    - Delete `chronicler_engine/scripts/migrate_doc_anchors.py`.

## Test Plan

Run from `chronicler_engine/`:

1. `python scripts/tests/test_validate_docs.py` — fixtures A–L pass alongside existing 14 fixtures.
2. `python scripts/validate_docs.py --anchors` post-Phase-2 — zero violations across all rules.
3. `python scripts/validate_docs.py` (no `--anchors` flag) — same zero-violation outcome (asserts the default is ON; catches regression).
4. `grep -rn '\[DOC:' chronicler_engine/src/test_support/` → 0 hits.
5. `grep -rn 'docs/system\|docs/reference/[a-z]\|docs/architecture/[a-z]' chronicler_engine/src chronicler_engine/arch-lint.toml` → 0 hits.
6. `grep -rn '\— section\|-- section' chronicler_engine/src chronicler_engine/tests chronicler_engine/arch-lint.toml` → 0 hits (suffix form gone from production code).
7. `cargo check --all-targets` → exits 0.
8. `cargo nextest run -E 'test(/structure/)'` → green.
9. `python build.py` → full pipeline green.
10. `git grep -n '\[DOC:' chronicler_engine/src | wc -l` → ≈185 (all under production src/, none under test_support/).
11. `git grep -n '\[DOC:' chronicler_engine/tests | wc -l` → 0.
12. Spot-check: pick 3 random `src/**/*.rs` files → confirm anchor is `[DOC: chronicler_engine/docs/diataxis/...]` (no suffix).

## Per Task Validation Steps

| Sub | Check |
|---|---|
| 1.1.1 | Unit-run `check_doc_anchors` against synthetic inputs — full-path valid passes; missing target fires; missing prefix fires. |
| 1.1.2 | Unit-run `check_test_support_rules` — anchor present → forbidden fires; missing summary → `TEST_SUPPORT_SUMMARY_REQUIRED` fires; tests/ with anchor → `TEST_FILES_ANCHOR_FORBIDDEN` fires. |
| 1.1.3 | `python scripts/validate_docs.py --help` shows `--anchors`; no `--no-anchors`. Default scan path includes `src/`, `tests/`, `*.toml`. |
| 1.1.4 | All 12 new fixtures pass; pre-rewrite live-tree snapshot shows the expected ~185 baseline violations (recorded as a constant for the test). |
| 2.1.1 | `python scripts/migrate_doc_anchors.py --help` (or `--apply` flag visible). Assert that introducing a non-existent target causes the script to abort BEFORE writing anything. |
| 2.2.1 | Pre/post counts match: validator reports ~185 broken before, 0 after. `cargo check` green. Greps return zero hits. |
| 2.3.1 | `grep -rn '\[DOC:' src/test_support/` = 0 hits. Validator reports 0 `TEST_SUPPORT_*` violations. `cargo check` green. |
| 3.1.1 | `cargo nextest run -E 'test(/structure/)'` green; spot-check that one full-path anchor file passes the new predicate. |
| 3.2.1 | Skill doc shows full-path example + all three exclusion rules + no suffix mention. |
| 4.1.1–3 | `head -3 docs/CHANGELOG.md` shows entry. `ls old-docs/archived-plans/doc-anchor-cutover-plan.md` exists. `ls scripts/migrate_doc_anchors.py` does not. |
| Final | `python build.py` green. |

## Failure Modes

| Codepath | Failure | Plan handling |
|---|---|---|
| `BROKEN_DOC_ANCHOR` (target-missing) | File at target path doesn't exist | Emit violation with path + lineno + variant message. `validate_docs.py` exits non-zero in `--strict` mode. |
| `BROKEN_DOC_ANCHOR` (path-form) | Anchor missing `chronicler_engine/` prefix | Same rule, message variant `"DOC anchor must start with chronicler_engine/"`. Catches new contributors using the old short form. |
| `TEST_SUPPORT_ANCHOR_FORBIDDEN` | `src/test_support/*.rs` has a `[DOC: ...]` line | Emit with file:line. CI red until fixed. |
| `TEST_SUPPORT_SUMMARY_REQUIRED` | `src/test_support/*.rs` line 1 is empty/missing `//! <summary>` | Emit with file:line. CI red. |
| `TEST_FILES_ANCHOR_FORBIDDEN` | `tests/**/*.rs` has a `[DOC: ...]` line | Emit. ADR-028 enforcement mirrored. |
| Migration script — non-existent target | `(engine_root / new).exists()` returns False | Abort with clear error, exit 1, NO writes performed. |
| Migration script — interrupted mid-flight | `read_text` → `write_text` not atomic | Acceptable for one-off; `git checkout` restores. Documented in plan. |
| Validator scan against pre-rewrite tree | Will produce ~185 `BROKEN_DOC_ANCHOR` errors | Expected, useful baseline. Plan uses this as the proof-of-correctness signal in 2.2.1. |

## What Already Exists (Reuse, Don't Rewrite)

- `tests/infrastructure/guardrails/structure.rs:14`: `MODULE_DOC_EXEMPTIONS = &["lib.rs", "main.rs", "test_support/"]` — already exempts `test_support/` from line-1 anchor check. Plan does NOT add a redundant exemption; only updates the path-prefix predicate.
- `tests/infrastructure/guardrails/structure.rs:34-37`: `extract_doc_anchor_path()` — same regex shape we use in the Python validator (path-only now; suffix-stripping branch removed). We define ours independently because the Python regex engine is different, but the predicate semantics are kept identical.
- `scripts/generate_structure_index.py:23`: already grep-anchors via `line1.startswith("//! [DOC:")`. No change needed.
- `scripts/validate_docs.py` `Violation` / `FileReport` / `render_reports` types — the new rules reuse these verbatim.
- The `MAPPING` tuples are the SAME mapping the di-taxis-folder-cutover plan produced for live scripts/skills; the legacy-target strings match exactly.

## NOT in scope

- Creating any new diátaxis docs. Closest-fit mappings only.
- Supporting the `— section "..."` suffix form. The four test_support files that used it lose the anchor entirely; no other sources used it. Any future need for section-specific anchoring is a separate ticket.
- `AGENTS.md`, `CHANGELOG.md`, ADR-028, plan docs that contain literal example text like `[DOC: docs/path/to/domain-doc.md]`. Those are documentation artifacts, not anchors consumed by the validator.
- Renaming `validate_docs.py --no-anchors` or any opt-out path. YAGNI.
- Touching `src/` line-2 `//! <summary>` content beyond what Phase 2.3.1 dictates.
- Migrating `tests/` files that don't currently have anchors (none do; the validator just enforces no-new-ones).
- `lib.rs` / `main.rs` / `mod.rs` doc-comment rewriting beyond what the script does automatically (already exempted).
