# Fix review findings on `testing-refactor`

## Summary

Implement every valid finding from the pre-merge review of branch `testing-refactor` vs `main` in this session. Scope covers the one real blocker (duplicate spec scenario IDs), all high-priority cleanup, and every medium item: flaky cancellation tests, `handle_retry_outcome` name/behaviour drift, `browser.md` 16.7 spec/test mismatch, duplicated `ProcessActionResult` mapping in chat-window handlers, inconsistent `fetch_body` helper signature, and stale validator docstring. The "broken guardrail + missing headers" finding is dropped — ADR-028 scopes `check_test_module_header` to `tests/` only; `src/**/*_tests.rs` sibling modules are intentionally exempt. The already-chartered `split-pipeline-rs-into-entry-modules.md` plan is **not** part of this session.

## Key Changes

- Renumber `games_create.md`/`games_switch.md`/`games_delete.md` scenarios to `17.x`/`18.x`/`19.x` and update `SCENARIO:` tags in `tests/http/games_*.rs`.
- Teach `validate_feature_spec.py` to reject duplicate scenario IDs across `docs/specs/*.md`; refresh its docstring to mention `tests/browser/`.
- Remove dead `failing_pipeline()`/`working_pipeline()` from `tests/integration/mod.rs`.
- Archive the duplicate `ticket-8-execute-the-browser-tier-changes.md` plan.
- Remove AI-style step-narration comments from `tests/http/actions.rs` and `tests/http/story_log.rs`.
- Unify `fetch_body` signature with `post_action`/`post_empty` (`&Router`) and update call sites.
- Extract a shared `ProcessActionResult` → response mapper in the chat-window handler module.
- Rename/inline `handle_retry_outcome` to match its actual responsibility.
- Update `browser.md` 16.7 to describe the synthetic `htmx:beforeSwap` event the test exercises.
- Replace the 50 ms real-sleep race in `retry_tests.rs` cancellation tests with deterministic channel coordination.
- Update `CHANGELOG.md` and regenerate AGENTS.md structure index.

## Implementation

### Phase 1: Spec and validator blocker

- [ ] #### Task 1.1: Renumber games scenarios to unique ranges (3 SP)
  - [ ] ##### SubTask 1.1.1: Move `games_create.md` scenarios to `17.x`, `games_switch.md` to `18.x`, `games_delete.md` to `19.x`; update headings.
  - [ ] ##### SubTask 1.1.2: Update `// […] SCENARIO:` tags in `tests/http/games_*.rs` to the new IDs.
  - [ ] ##### SubTask 1.1.3: Run validator and confirm all scenario IDs are unique with no gaps/orphans.

- [ ] #### Task 1.2: Harden `validate_feature_spec.py` against duplicate IDs and stale docstring (2 SP)
  - [ ] ##### SubTask 1.2.1: Add duplicate-ID detection across `docs/specs/*.md`; fail with a clear message naming both files.
  - [ ] ##### SubTask 1.2.2: Update module docstring to say tags live in `tests/http/` and `tests/browser/`, not HTTP-only.

### Phase 2: Cleanup and consistency

- [ ] #### Task 2.1: Remove dead pipeline helpers from `tests/integration/mod.rs` (1 SP)
  - [ ] Delete `failing_pipeline()` and `working_pipeline()`; confirm no callers via `grep`.

- [ ] #### Task 2.2: Archive duplicate `ticket-8` plan file (1 SP)
  - [ ] Move `chronicler_engine/docs/plans/ticket-8-execute-the-browser-tier-changes.md` to `chronicler_engine/old-docs/archived-plans/`.
  - [ ] Keep `chronicler_engine/docs/plans/ticket-8-browser-tier-execution.md` as canonical.

- [ ] #### Task 2.3: Remove "what" comments from HTTP tests (1 SP)
  - [ ] Delete step-narration comments in `tests/http/actions.rs` and `tests/http/story_log.rs`.
  - [ ] Keep any comments that explain *why* a step exists.

- [ ] #### Task 2.4: Unify `fetch_body` signature in `test_helpers.rs` (2 SP)
  - [ ] Change `fetch_body(app: Router, …)` to `fetch_body(app: &Router, …)`.
  - [ ] Update all call sites; adjust mutable borrows where needed.

- [ ] ] #### Task 2.5: Extract shared result mapper for chat-window handlers (2 SP)
  - [ ] Add a helper in `src/adapters/driving/http/chat_window/handlers/` that maps `ProcessActionResult` → HTTP response/status HTML.
  - [ ] Replace duplicated `match` arms in `retry_handler` and `retrigger_handler` with the helper.

- [ ] #### Task 2.6: Rename/inline `handle_retry_outcome` (2 SP)
  - [ ] Verify `handle_retry_outcome` only logs `PhaseError::Cancelled`.
  - [ ] Either rename it to `log_cancellation` and update callers, or inline the log at the call sites and remove the function.

- [ ] #### Task 2.7: Update `browser.md` 16.7 spec text (1 SP)
  - [ ] Rewrite the When/Then of 16.7 to describe a synthetic `htmx:beforeSwap` event with `isError=true`, matching the actual test.

### Phase 3: Test reliability

- [ ] #### Task 3.1: Make retry cancellation tests deterministic (3 SP)
  - [ ] ##### SubTask 3.1.1: Replace the 50 ms `thread::sleep` with an async/await channel or atomic flag so the game-id flip happens only after the retry has started.
  - [ ] ##### SubTask 3.1.2: Use a long `with_trigger_delay` (or equivalent) so timing no longer gates the cancellation window.
  - [ ] ##### SubTask 3.1.3: Run the two cancellation tests many times to confirm stability.

### Phase 4: Verify and record

- [ ] #### Task 4.1: Run full validation (2 SP)
  - [ ] `cargo check -p chronicler_engine --tests`
  - [ ] `python chronicler_engine/build.py`
  - [ ] `python chronicler_engine/scripts/validate_feature_spec.py`
  - [ ] `cargo nextest run -p chronicler_engine guardrails`

- [ ] #### Task 4.2: Update CHANGELOG and regenerate AGENTS.md index (1 SP)
  - [ ] Add a dated entry to `chronicler_engine/docs/CHANGELOG.md` summarizing the fixes.
  - [ ] Run `python chronicler_engine/scripts/generate_structure_index.py` so docstring/plan changes are reflected in `AGENTS.md`.

## Test Plan

- `cargo nextest run -p chronicler_engine` passes with 0 new failures.
- `python chronicler_engine/scripts/validate_feature_spec.py` reports no gaps, no orphans, no duplicate IDs.
- The two retry cancellation tests pass reliably across repeated runs.
- `python chronicler_engine/build.py` exits 0.

## Per Task/Sub Task Validation Steps

- Task 1.1: `grep` shows no duplicate `Scenario X.Y` IDs across specs.
- Task 1.2: Introducing a duplicate ID makes the validator exit 1 with a clear duplicate report.
- Task 2.1: `grep failing_pipeline\\|working_pipeline` in `tests/integration/` returns no matches.
- Task 2.2: Only one `ticket-8*.md` remains in `docs/plans/`.
- Task 2.4: `cargo check --tests` passes and all `fetch_body` call sites compile.
- Task 2.5: `chat_window.rs` no longer contains two identical `match ProcessActionResult` blocks.
- Task 2.6: `handle_retry_outcome` either no longer exists or its name matches its only action.
- Task 2.7: `browser.md` 16.7 no longer references `POST /action` returning 500.
- Task 3.1: Cancellation tests pass under `cargo nextest run --test retry_tests` repeated 10 times.
- Task 4.1: Build script summary is green.

## Assumptions

- No production behaviour changes; this is cleanup, spec renumbering, and test hardening.
- The `split-pipeline-rs-into-entry-modules.md` refactor is intentionally out of scope.
- ADR-028's `check_test_module_header` is correctly scoped to `tests/` only; `src/**/*_tests.rs` files do not require `//!` headers.
- New games scenario IDs will be `17.x`, `18.x`, `19.x` to avoid collision with existing `1.x`–`16.x`.
- The browser 16.7 test's synthetic-event approach is acceptable; the spec will be adjusted to match.
