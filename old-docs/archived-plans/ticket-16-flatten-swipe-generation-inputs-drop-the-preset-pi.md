# Ticket 16 — Flatten swipe generation inputs, drop the preset pin, remove the replay blob

## Summary

Execute ticket 15's resolution: a Swipe carries only the player-typed inputs of its generation as direct properties — `impersonated: bool` + `direction: Option<String>` — and never game configuration. `GenerationReplay`, the `replay` field/column, and the entry-time preset pin leave the codebase. Impersonate generation (first run and redo) always resolves the game's current active preset at generation time. Migration v22 converts existing rows from the replay JSON and drops the column. Build-green via `python build.py`, then close the ticket and advance the map.

## Key Changes

- **Domain** (`src/domain/model/message.rs`): delete `GenerationReplay`; `Swipe` gains `impersonated` + `direction` (both `#[serde(default)]`). Active-swipe accessors reshape: `replay()` → `impersonated()` / `direction()` / `is_guided()` (the one helper, `!impersonated && direction.is_some()`, on `Message`); `set_replay()` → `set_stored_inputs(impersonated, direction)`; module doc drops "replay blob".
- **GameState** (`game_state.rs`): `add_message_with_inputs(text, type, impersonated, direction)`; `push_message`'s retry-swipe inheritance copies the two flat fields from the retry target instead of cloning the blob.
- **Pipeline**: `action.rs` drops the entry-time `active_impersonate_preset_id` storage read; the action arm builds flat inputs. `narration_generation.rs`: `ImpersonateInputs` loses `preset_id`; `resolve_preset_choice` collapses to the active-preset path (former fallback becomes the only path); `stored_inputs_from` returns the flat pair. `core.rs`: `run_from_input(state, input, impersonated, direction)` rebuilds `GenerationInputs` from flat fields with fresh-then-target precedence; "impersonate wins over guide" goes (mutual exclusivity is structural); `execute_action_with_replay` → `execute_action_with_inputs`. `retry.rs`: mode classification reads `old_target.impersonated()`; guided-anchor reads use `is_guided()`. `message_service.rs::find_retry_anchor_msg`: same `is_guided()` swap.
- **Storage + migration v22** (`plumbing.rs`): `ALTER TABLE message_swipes ADD COLUMN impersonated INTEGER NOT NULL DEFAULT 0` and `direction TEXT` (column_exists-guarded, file pattern); backfill `impersonated ← COALESCE(json_extract(replay,'$.impersonate'),0)`, `direction ← COALESCE(json_extract(replay,'$.impersonate_direction'), json_extract(replay,'$.guide'))` with `WHERE replay IS NOT NULL AND json_valid(replay)` (corrupt blobs degrade to plain — mirroring today's runtime behavior); `DROP COLUMN replay` (guarded), with a comment recording the coalesce order and the degrade rationale. `DbSwipe`, `model_swipes_to_db`, the `TryFrom` mapper, `swipes.rs` INSERT/SELECT lists reshape; `parse_swipe_replay` deleted.
- **Rename sweep**: no `replay`/`GenerationReplay` identifier survives in the message domain (struct, field, column, accessors, comments, fixtures, tests). The AGENTS.md structure line regenerates via the pre-commit hook. Compile-driven: `python build.py check` flushes out any caller not in the enumerated lists (in-memory backend needs no change — it stores domain `Swipe`s).

## Implementation

### Phase 1: Flatten the domain, pipeline, and storage

- [ ] #### Task 1.1: Domain model flatten (3 SP)
  - Edit `src/domain/model/message.rs` per Key Changes; reshape `game_state.rs` (`push_message`, `add_message_with_inputs`, retry-swipe inheritance).
- [ ] #### Task 1.2: Pipeline reshape (3 SP)
  - `action_pipeline/action.rs` (drop pin read, flat input triple, rename `execute_action_with_inputs`), `core.rs` (`run_from_input` signature + rebuild + fresh-then-target precedence), `retry.rs` (mode classification + guided reads), `narration_generation.rs` (`ImpersonateInputs` minus `preset_id`, `resolve_preset_choice` collapse, flat `stored_inputs_from`), `message_service.rs` (anchor read).
- [ ] #### Task 1.3: Storage reshape + migration v22 (3 SP)
  - `models/swipe.rs` (`impersonated: i64`, `direction: Option<String>`, `from_row` indices), `mappers/message.rs` (mapper builds flat fields; delete `parse_swipe_replay`; `model_swipes_to_db`), `swipes.rs` (INSERT/SELECT lists), `utils/plumbing.rs` v22 block per Key Changes.
- [ ] #### Task 1.4: Unit-test sweep (3 SP)
  - Update: `retry_tests.rs` (913, 1319, 1368, 1394, 1446, 1488), `action_tests.rs` (580, 647, 618-area helper), `game_state_tests.rs` (122–231), `swipes_tests.rs` (433–491: `sample_replay` → flat sample), `db_tests.rs` (140–152: assert `impersonated`+`direction` exist and `replay` is gone), `mappers/message_tests.rs` (delete the `parse_swipe_replay` suite; mapper round-trip on flat fields), `models/message_tests.rs` (59, 84), `message_service_tests.rs` (83, 121), `test_support/fixtures.rs` (`dummy_swipe`, `seed_swipe_with_stored_inputs` signature → `(impersonated, direction)` + callers).
  - New: `is_guided()` truth table (4 states) in `domain/model/message_tests.rs`; v22 backfill test in `utils/plumbing_tests.rs` mirroring the existing simulated-old-DB pattern — v15-shape table at user_version 21 with a guide row, a directed-impersonate row whose blob carries a preset id (assert the id is DISCARDED — this is the deleted-pin coverage), a bare impersonate row, a plain row, and a corrupt-JSON row; assert flat values, dropped column, user_version 22.

### Phase 2: Integration tests, docs, gate

- [ ] #### Task 2.1: Integration tests (3 SP)
  - Restyle: `tests/storage/message_storage.rs` (5 literals), `tests/http/requires_migration/fragment.rs` (4 literals), `tests/http/actions.rs` (717–731, 765–775 — drop the preset-pin assertion at 730), `tests/http/swipe_new.rs` (755, 805, 881, 903–913 — drop pin assertions; keep the direction re-application assertion as the ReImpersonate stored-input coverage).
  - New in `tests/http/swipe_new.rs`: redo of an impersonated swipe after updating the active impersonate preset's CONTENT in place (via the existing update-preset path) generates with the CURRENT preset — assert the new content appears in the recorded LLM system prompt. Same id, new configuration: the sharpest available proof that no pin survives and that resolution happens at generation time. (Decided in plan review: no game-slot writer exists, and the ticket's deleted-pin e2e premise is structurally destroyed by this change — its intent is held by the backfill unit test's "preset id discarded" row plus this test.)
- [ ] #### Task 2.2: Docs — dead-concept removal (1 SP)
  - `docs/diataxis/reference/narrative/ai_steering.md`: delete the "Replay Blob" section; overview table's "Retry re-applies via" column and transient rows; the Impersonate line "...re-impersonate using the same preset" → stored direction + the game's current impersonate preset (the preset is NOT pinned — the behavior change, stated positively); Mutual Exclusivity and Retry sections' blob references; storage.md cross-ref's "replay column".
  - `CONTEXT.md`: Swipe entry's stored-inputs clause rewritten around the two properties.
  - `docs/diataxis/reference/storage.md`: line 67's replay-blob clause → the two swipe fields; line 181's cross-ref reworded (per Assumptions).
- [ ] #### Task 2.3: Gate, commit, tracker bookkeeping (1 SP)
  - `python build.py` green. Commit flags the behavior change: a redo of an impersonated swipe after a preset change generates with the NEW active preset; a migrated row whose pinned preset was deleted no longer fails.
  - Ticket 16: `Status: resolved` + `## Answer` (ticket 14's shape: what shipped / deviations — including the test-reshape deviation above / facts for later tickets). Map: one Decisions-so-far line; ticket 06's `Blocked by: 16` → `(none)`.

## Test Plan

- Unit: `is_guided()` truth table; v22 backfill (guide / directed-impersonate-with-pin-discarded / bare-impersonate / plain / corrupt rows); schema assertion; reshape of all existing replay-touching suites listed in Task 1.4.
- Integration: existing redo coverage restyled to flat fields; new redo-after-preset-content-update test.
- Gate: `python build.py` (fmt + clippy + guardrails + unit + integration).

## Per Task/Sub Task Validation Steps

- After 1.1–1.3 (one compile unit): `python build.py check` compiles (tests may still reference old APIs — if so, land 1.4 before checking).
- After 1.4: `python build.py unit` green, incl. the two new tests.
- After 2.1: `python build.py nextest swipe_new` and `python build.py nextest actions` green.
- After 2.2: `python build.py validate-docs` green.
- After 2.3: `python build.py` fully green; ticket/map edits done.

## Assumptions

- `storage.md` joins the doc sweep (ticket listed only ai_steering.md + CONTEXT.md): its replay-blob + pinned-preset description is dead-concept removal, the same rule the ticket applies elsewhere. No new reference content added.
- Migration v22 confirmed at execution time (v21 = posture drop already in `plumbing.rs`); plan re-checks the max version before writing the block per the ticket's standing instruction.
- Pin-specific assertions in `tests/http/actions.rs:730` and `tests/http/swipe_new.rs:913` are deleted, not rewritten — the concept they asserted is gone.
- Plan-review ruling (user-picked): the ticket's "deleted pinned preset" e2e test is dropped in favor of the backfill unit test's pin-discard row + the content-update redo test; no test-only storage writer is added. The deviation goes on record in ticket 16's Answer.
- The ticket is claimed (status/assignee per the tracker doc) at implementation start, from the ready menu's implement step.
