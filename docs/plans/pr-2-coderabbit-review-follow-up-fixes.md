# PR #2 CodeRabbit Review Follow-up Fixes

## Summary

Apply the 18 accepted fixes from the PR #2 CodeRabbit triage (`tmp/pr2_review_issues.md`, all decisions recorded 2026-08-31). Ordered by risk: a migration that can abort startup, a vacuous test, a user-facing retry dead end, an unreachable IF preset bundle, a stale new reference doc, then a mechanical cleanup batch. Excluded: N1 (deferred pending Q3 of `docs/plans/replay-blob-storage-investigation-plan.md`), M2 (permission allowlist — removed from scope by user, decision recorded as unscheduled), T4 (skipped).

## Key Changes

- **M1**: `plumbing.rs` migration v19 `UPDATE games` subqueries wrapped in `COALESCE(..., 'novel'/'third'/'past')`; post-migration `tracing::warn!` for orphan `games.world_key` rows (logging is initialized before `DbPool::new`, `main.rs:15` → `run.rs:59`).
- **N6**: `test_run_game_changed_returns_cancelled` asserts history length unchanged; comment and assertion message corrected.
- **M3**: `retry.rs` guard accepts a guide-bearing retry target (guard and resolver both read `game_state.narrative.history`); `ReNarrate` passes `String::new()` when the target swipe replay carries a guide.
- **M4**: preset cards get per-mode activation buttons gated by `preset.allowed_modes`, posting `mode` via `hx-vals` to the existing handler; view model gains `allowed_modes` and per-mode active ids (replacing the single `active_impersonate_id`).
- **N3**: `ai_steering.md` swept for ticket-17 fallout (Impersonate output `Input`, no `sender`, no `Dialogue`).
- **Docs/preset batch**: N2 (quantifier example → `null`), N4 (Layers 3/7 conditional), N5 (I.4 reword).
- **Cleanup batch**: N7, N8, N9, D1, D2, D3-D6, T1 (rename), T2 (delete), T3 (block-scope guard).

## Implementation

### Phase 1: Stability

- [ ] #### Task 1.1: M1 — Migration v19 orphan-world hardening (3 SP)
  - `plumbing.rs:380-385`: wrap the three scalar subqueries in `COALESCE` with the column defaults (`'novel'`, `'third'`, `'past'`).
  - After the update, add a Rust-side check: `SELECT key FROM games WHERE world_key NOT IN (SELECT key FROM worlds)`; `tracing::warn!` per orphan row.
  - Test: storage test with a game referencing a missing world — assert the data outcome (migration succeeds, game row takes defaults). The warn emission is verified by inspection, not assertion (tracing capture needs subscriber plumbing).

### Phase 2: Retry correctness

- [ ] #### Task 2.1: N6 — Fix the vacuous cancellation test (1 SP)
  - `narration_generation_tests.rs`: capture `state.narrative.history` length before `run`, assert unchanged after; remove the `"MockNarration"` contains-assertion.
  - Fix the comment ("A game switch **before** run construction") and assertion message ("create_game must produce a distinct game id").
  - Sanity: temporarily make `run` append a message on cancellation (scratch edit, reverted) and confirm the test now fails.

- [ ] #### Task 2.2: M3 — Guide-only retry (3 SP)
  - [ ] ##### SubTask 2.2.1: Guard accepts guide-bearing target (1 SP)
    - `retry.rs:44` guard: in `game_state.narrative.history` (same source the resolver's pick derives from), find the last `Narration`/`Input` message; retryable when `last_input_text()` is `Some` **or** that message's replay carries `guide.is_some()`. Error message unchanged for the genuinely-empty case.
  - [ ] ##### SubTask 2.2.2: ReNarrate passes empty input for guide turns (1 SP)
    - `retry.rs:90-101`: when `target.old_target`'s replay has a guide, call `retry_main_narration(state, String::new())` instead of `last_input_text()`.
  - [ ] ##### SubTask 2.2.3: Tests (1 SP)
    - Retry a guide-only turn with no prior Input: succeeds, guide re-applied (assert narration generated with empty `<PlayerInput>`).
    - Retry a guided turn with an older Input in history: prompt user-message is empty, not the older input.
    - Guide-free narrative retry unchanged (existing tests keep passing).

### Phase 3: Feature completion

- [ ] #### Task 3.1: M4 — Per-mode preset activation (3 SP)
  - **View model** (`prompt_presets` view structs): add `allowed_modes` to the preset view struct; replace the single `active_impersonate_id` with per-mode active ids (e.g. `active_novel_id`, `active_if_id`) derived from `settings.mode_preset_registry` via `bundle_for(mode).impersonate_prompt_preset_id`.
  - **Template** `prompt_presets.rs:126-188`: replace the single "Set Active" button with "Set Active (Novel)" / "Set Active (IF)", each `hx-post` to `/prompt-presets/{id}/activate` with `hx-vals='{"mode": "novel"|"interactive_fiction"}'`; render each only when `allowed_modes` includes that mode; hide when the preset is already active for that bundle. The "Active" badge shows which mode(s) the preset is active in.
  - Handler unchanged (already accepts `mode` via `ActivateQuery`).
  - Tests: HTTP integration — activating an IF-only preset via the IF button succeeds and populates the IF bundle; via the Novel button still errors. Template/Browser test — buttons gated correctly by `allowed_modes`.

### Phase 4: Docs and preset data

- [ ] #### Task 4.1: N3 — ai_steering.md ticket-17 sweep (2 SP)
  - Table: Impersonate output message type `Narration` → `Input`.
  - Rewrite the "player-voiced dialogue entry / sender is the persona name" prose for the speaker/role model (Input row; no sender field).
  - Sweep the whole doc for remaining `Dialogue`/`sender` language.

- [ ] #### Task 4.2: N2, N4, N5 — data and doc corrections (1 SP)
  - N2: `data/prompt_presets/quantifier/default.json:5` — foyer/Carla example expected output → `{"movement": {"type": null}}`.
  - N4: `prompt_system.md:12` — "Layers 3 and 7 are conditional (Layer 3 drops on impersonated turns; Layer 7 renders only on guided turns); the remaining six are always present."
  - N5: `swipe_new.md` I.4 — "Retry never modifies the text of any prior swipe."

### Phase 5: Cleanup batch

- [ ] #### Task 5.1: Test infrastructure fixes (N7, N8, N9, T1, T2, T3) (3 SP)
  - N7: `context.rs` `seed_default_preset` also seeds `system_if_default`.
  - N8: `fixtures.rs:657` — `storage.insert_swipe(...)?`.
  - N9: `build.py:143-146` — progress-counter group optional in `_NEXTEST_RESULT_RE`.
  - T1: rename `test_migration_v20_reshapes_registry_and_backfills_flags` → `test_migration_v20_v21_reshapes_registry_and_backfills_flags`.
  - T2: delete the `null_sender` test in `models/message_tests.rs:37-49`.
  - T3: `view_query.rs:189-191` — block-scope the settings guard around `preset_id` resolution.

- [ ] #### Task 5.2: Standards and agent-file fixes (D1, D2, D3-D6) (1 SP)
  - D1: `chronicler-after-plan-workflow-plus-review/SKILL.md` step 12 — name the three no-test skills explicitly.
  - D2: `CODING_STANDARDS.md:11-14` — DOC-anchor requirement excludes `src/test_support/*.rs`; summary-line requirement kept.
  - D3: `explictly` → `explicitly`. D4: `test/` → `tests/` (AGENTS.md:216). D5: `500 line file` → `500-line file`. D6: ``#[ignore]'`` → ``#[ignore]`` (tests/AGENTS.md:9).

## Test Plan

- New tests: M1 orphan-world migration (data outcome); M3 guide-only retry (3 cases); M4 IF activation (HTTP + gating); N6 rewritten assertion.
- Regression: existing retry, migration, preset, and prompt-preset suites unchanged.
- Gate: `python build.py` green.

## Per Task/Sub Task Validation Steps

| Task | Validation |
|---|---|
| 1.1 | `cargo nextest run plumbing` — orphan-world test passes; `cargo clippy` clean |
| 2.1 | `cargo nextest run test_run_game_changed_returns_cancelled` — fails if reverted to vacuous assertion (scratch-verified) |
| 2.2 | `cargo nextest run retry` — new guide-retry tests + existing retry suite |
| 3.1 | `cargo nextest run prompt_presets` + browser slash/preset tests |
| 4.1 | `python scripts/validate_docs.py`; grep `ai_steering.md` for `Dialogue`/`sender` — zero hits |
| 4.2 | `python scripts/validate_data.py` (quantifier JSON); docs validation |
| 5.1 | `cargo test --lib` + `cargo nextest run settings message view_query` |
| 5.2 | `cargo nextest run --test guardrails`; `python scripts/generate_docs_index.py` if docs index affected |

## Assumptions

- N1 stays untouched, blocked on Q3 of `docs/plans/replay-blob-storage-investigation-plan.md`.
- M2 is out of scope — decision recorded as unscheduled in `tmp/pr2_review_issues.md`; no permission-config changes in this plan.
- Column defaults `'novel'` / `'third'` / `'past'` are the correct COALESCE fallbacks (they match the v19 column defaults).
- The M4 handler needs no change — it already accepts and honors `mode`.
