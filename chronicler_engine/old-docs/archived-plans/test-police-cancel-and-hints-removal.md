# Plan: Test-Police Audit Fixes — Hints Removal + Cancel Test Alignment

**Date:** 2026-06-28
**Status:** Approved
**Scope:** `chronicler_engine/`
**Origin:** `/test-police` skill audit (3 failing tests)

## Context

Test-police audit (`python build.py --target-dir target/test-police`) found 3 failing tests. Investigation classified all three as **test-contract gaps**, not production bugs.

### Failing tests

1. `server::fragments::endpoints_tests::test_hints_handler` (`src/server/fragments/endpoints_tests.rs:44`)
   - Assertion `assert!(!result.0.is_empty())` against `render_action_hints`, which is a stub returning `Ok(String::new())`.
   - Previous version used tautology `assert!(result.0.is_empty() || !result.0.is_empty())` — anti-pattern masking the stub.
   - Hints is an unimplemented feature. `docs/system/dashboard.md` mentions it but no spec defines content. Cleanest fix: remove the feature entirely (code, tests, docs, assets).

2. `pipeline_tests::test_pipeline_cancels_after_main_narration` (`tests/integration/pipeline/pipeline.rs:284`)
   - Asserts narration is discarded on cancel: `assert!(!has_narration, "Narration should be discarded...")`.
   - No production path requires this. Cancel is only triggered by (a) reset (which wipes state separately) and (b) ctrl-C shutdown (process exits).
   - Old test passed due to fast 5ms polling landing cancel before `phase_narrate` persist. Strengthened `wait_for_condition_async` with 50ms interval exposed the gap.
   - Align tests with actual contract: cancel halts pipeline, resets status to Idle. Does not roll back persisted narration (ADR-023 incremental persistence).

3. `pipeline_tests::test_pipeline_cancels_during_trigger_continuation` (`tests/integration/pipeline/pipeline.rs:350`)
   - Same pattern: asserts trigger event discarded on cancel. Same gap, same fix.

## Scope

### Hints removal

**Code:**
- `src/server/router.rs:40` — remove `.route("/hints", get(fragments::hints_handler))`
- `src/server/fragments/endpoints.rs:51-53` — remove `hints_handler`
- `src/server/fragments/endpoints.rs:10` — remove `render_action_hints` from imports
- `src/server/fragments/mod.rs:16` — remove `hints_handler` re-export
- `src/server/fragments/renderers/fragment_renderers.rs:101-103` — remove `render_action_hints`
- `src/server/templates.rs:76` — remove `<div class="action-hints" id="action-hints" hx-get="/hints" hx-trigger="load, every 5s"></div>` from `ActionAreaTemplate`

**Tests:**
- `src/server/fragments/endpoints_tests.rs` — remove `test_hints_handler` + `hints_handler` import
- `tests/http/fragment.rs:111-114` — remove `test_hints_handler`
- `tests/browser/structure.rs:256-265` — remove `test_action_hints_visible`

**Assets:**
- `assets/index.html:69-72` — remove `.action-hints` div
- `assets/styles.css:544` — remove `.action-hints` rule

**Docs:**
- `docs/system/dashboard.md:60` — remove "action hints" from Action Area content list
- `docs/system/dashboard.md:100` — remove "Action hints poll `/hints` every 5 seconds"
- `docs/system/ui_design.md:42` — remove "action hints" from `--font-size-small` row
- `docs/architecture/system.md:111` — remove "hints" from misc utility list
- `docs/adr/adr-002-http-polling.md:49` — remove `/hints` entry

**Index regen:**
- `python scripts/generate_docs_index.py`

### Cancel test alignment (P1)

**Tests:**
- `tests/integration/pipeline/pipeline.rs` `test_pipeline_cancels_after_main_narration` — drop `assert!(!has_narration, ...)`. Keep `assert!(!guard.narrative.input_buffer.status.is_generating())`. Keep `wait_for_condition_async` polling.
- `tests/integration/pipeline/pipeline.rs` `test_pipeline_cancels_during_trigger_continuation` — drop `assert!(!has_event, ...)`. Keep `assert!(!is_generating())` and `assert!(has_narration)`.

**No production code changes.** No doc changes. No new doc add for cancellation (tests were the only false contract; nothing to document).

## Out of scope

- UI "Stop" button functionality (currently inert when generating). Out of scope — separate feature if needed.
- Pipeline rollback semantics (snapshot restore on cancel). Out of scope — no production path requires it.
- Implementing hints feature. Out of scope — removed entirely per user direction.

## Validation

1. `cargo nextest run --target-dir target/test-police test_hints_handler test_pipeline_cancels_after_main_narration test_pipeline_cancels_during_trigger_continuation test_action_hints_visible --nocapture` — first two should be filtered out (removed), pipeline tests pass.
2. `cargo clippy --all-targets --target-dir target/test-police -- -D warnings`
3. `python build.py --target-dir target/test-police` — full gate (fmt + clippy + guardrails + tests + coverage)
4. Manual browser check — dashboard renders without /hints 404 errors in console (optional, browser tests cover most).

## Guardrails

- File sizes: removing code from `templates.rs` and `endpoints.rs` only shrinks files. No size violations introduced.
- Arch-lint: removing one route and one endpoint does not cross module layers.
- Clippy: unused-import lints will flag `hints_handler` import sites if missed.

## CHANGELOG

Add entry under recent changes:
- "Removed unimplemented `/hints` endpoint, `render_action_hints` stub, and related tests/assets/docs. Feature was never implemented; tests were tautological."
- "Pipeline cancellation tests aligned with actual contract: cancel halts pipeline and resets status to Idle; does not roll back persisted narration per ADR-023."

## Archive

Move to `docs/plans/archived/test-police-cancel-and-hints-removal.md` after `python build.py` passes.
