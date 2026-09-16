# Task: Flatten swipe generation inputs, drop the preset pin, remove the replay blob

Type: task
Status: resolved
Blocked by: (none)

## Question

Execute [ticket 15's resolution](15-grill-allowed-modes-pinned-preset-edge.md)
(2026-09-06). A Swipe carries only the player-typed inputs of its generation,
as direct properties — never game configuration. Implementation per the map's
Notes override.

### Scope

1. **Domain.** `GenerationReplay` and `Swipe.replay` are deleted
   (`src/domain/model/message.rs`). `Swipe` gains `impersonated: bool` and
   `direction: Option<String>` (guide text and impersonate direction merge
   into the one field). One helper — `is_guided()` =
   `!impersonated && direction.is_some()` — serves the retry/anchor reads.
   `Message::replay()` / `set_replay()` / `add_message_with_inputs` reshape
   to the flat fields. `GenerationInputs` / `ImpersonateInputs`
   (`narration_generation.rs`) lose `preset_id`.

2. **Pipeline.** The impersonate arm of the `process_action` dispatcher
   (`action_pipeline/action.rs`) drops the entry-time storage read — no
   settings/storage coupling at entry. `resolve_preset_choice`
   (`narration_generation.rs`) collapses to the per-game active-preset
   resolution for impersonate (the former fallback becomes the only path).
   `run_from_input` (`action_pipeline/core.rs`) rebuilds generation inputs
   from the flat fields; the "impersonate wins over guide" priority logic
   goes (mutual exclusivity is structural now). `retry.rs` mode
   classification and the guided-input reads move to the flat fields /
   `is_guided()`.

3. **Storage + migration.** Take the next free migration number at
   execution time (expected v22 — ticket 09 is slated for v21; re-check
   before writing the migration). Add `impersonated INTEGER NOT NULL
   DEFAULT 0` and `direction TEXT NULL` columns; backfill from the `replay`
   JSON column (`impersonated ← $.impersonate`;
   `direction ← coalesce($.impersonate_direction, $.guide)` — producers
   never set both, but if a hand-written row does, impersonate wins); drop
   the `replay` column. `parse_swipe_replay` and `DbSwipe.replay` go.

4. **Rename sweep.** No `replay` / `GenerationReplay` identifiers survive
   in the message domain — struct, field, column, accessors, comments.
   `RetryMode` stays `pub(crate)` per guided-generations ticket 06.

5. **Tests.** Update `retry_tests.rs` (913, 1368, 1446),
   `action_tests.rs` (580, 647), `game_state_tests.rs` (122-231),
   `swipes_tests.rs` (453-491), `db_tests.rs` (152 schema assertion), and
   `test_support/fixtures.rs`. New coverage: a redo after a preset change
   generates with the game's CURRENT preset; a migrated row whose pinned
   preset was deleted no longer fails into `GenerationStatus::Error`;
   migration backfill correctness (guide and directed-impersonate rows);
   `is_guided()` truth table; `direction` re-applied on ReImpersonate.

6. **Docs to touch in this ticket** (dead-concept removal only — full
   feature docs stay in the map's doc fog):
   `docs/diataxis/reference/narrative/ai_steering.md` (drop the
   "Replay Blob" section; fix the overview table's "Retry re-applies via"
   column and the transient-persistence rows) and `CONTEXT.md` (the Swipe
   entry's stored-inputs clause rewritten around the two properties).

## Notes for the session

- **Behavior change to flag in the commit message:** a redo of an
  impersonated swipe after a preset change generates with the NEW active
  preset; a migrated row whose pinned preset was deleted no longer fails.
- Read before implementing: `src/domain/model/message.rs`,
  `src/application/pipeline/narration_generation.rs`,
  `src/application/pipeline/action_pipeline/{action,core,retry}.rs`,
  `src/application/message_service.rs`,
  `src/domain/model/message_history.rs`,
  `src/adapters/driven/storage/mappers/message.rs`,
  `src/adapters/driven/storage/swipes.rs`,
  `src/adapters/driven/storage/utils/plumbing.rs` (migrations).
- Build-green: `python build.py`.

## Answer

Implemented per ticket 15's resolution, plan-reviewed (improve-ai-plan) before coding. `python build.py` fully green (fmt, clippy, guardrails, 119 unit + 1481 integration tests; log `logs/build_20260906_191139.log`).

### What shipped

1. **Domain** — `GenerationReplay` deleted; `Swipe` carries `impersonated: bool` + `direction: Option<String>` (both `#[serde(default)]`). Message accessors reshape: `impersonated()` / `direction()` / `is_guided()` (the one helper — `!impersonated && direction.is_some()`, on `Message`) replace `replay()`; `set_stored_inputs(impersonated, direction)` replaces `set_replay()`; `add_message_with_inputs(text, type, impersonated, direction)` takes the flat pair. `push_message`'s retry-swipe inheritance copies the two flat fields from the retry target.
2. **Pipeline** — the impersonate arm of `process_action` no longer reads `active_impersonate_preset_id` at entry (the entry-path→storage coupling is gone). `resolve_preset_choice` collapses to the active-preset resolution for first run and redo alike. `run_from_input(state, input, impersonated, direction)` rebuilds `GenerationInputs` with fresh-then-target precedence; "impersonate wins over guide" is structural (one `direction` field, the bool discriminates). `execute_action_with_replay` → `execute_action_with_inputs`. `retry.rs` mode classification reads `old_target.impersonated()`; the guided reads in `retry.rs` and `find_retry_anchor_msg` use `is_guided()`.
3. **Migration v22** (`plumbing.rs`) — adds `impersonated INTEGER NOT NULL DEFAULT 0` + `direction TEXT` (column_exists-guarded), backfills from the replay JSON (`impersonated ← $.impersonate`; `direction ← coalesce($.impersonate_direction, $.guide)`; `json_valid` guard degrades corrupt blobs to plain, mirroring the old parse-time behavior), drops the `replay` column, bumps `user_version` to 22. `parse_swipe_replay`, `DbSwipe.replay`, and the JSON serialization in `insert_swipe` are gone; INSERT/SELECT lists carry the flat columns.
4. **Tests** — all replay-touching suites restyled (`retry_tests`, `action_tests`, `core_tests`, `game_state_tests`, `swipes_tests`, `messages_tests`, `mappers/message_tests`, `models/message_tests`, `message_service_tests`, `db_tests`, fixtures). New: `is_guided()` truth table (4 states) in `message_tests.rs`; v22 backfill test + fresh-DB noop test in `plumbing_tests.rs` (guided / directed-impersonate-with-pin-discarded / bare-impersonate / plain / corrupt rows); the `db_tests` schema assertion now checks both new columns and the absence of `replay`.
5. **Integration** — `swipe_new`, `actions`, `message_storage`, `fragment` restyled to flat fields; the preset-pin assertions deleted. New `test_retry_re_impersonate_uses_current_preset_content_http`: updates `impersonate_default`'s instructions in place, redoes an impersonated swipe, asserts the marker in the recorded LLM prompt.
6. **Docs** — dead-concept removal: `ai_steering.md` (Replay Blob section deleted; table, Impersonate, Mutual Exclusivity, Retry sections, and the storage cross-ref rewritten around stored inputs — the preset-resolved-at-generation-time behavior stated positively), `CONTEXT.md` (Swipe entry's stored-inputs clause names the two properties), `storage.md` (swipe-bullet + AI Steering cross-ref).

### Deviations from the plan (review-justified)

- The ticket's "migrated row whose pinned preset was deleted no longer fails" e2e test was dropped (user-approved during plan review): the pin's deletion makes the scenario structurally impossible, and its intent is held by the backfill test's pin-discarded row plus the content-update redo test. No test-only game-preset storage writer was added (none exists — the per-game picker is ticket 07).
- `db_tests` and the older migration tests assert `user_version` 22 (the chain's new landing version) rather than 21.
- Clippy required `then_some` over `then` in `run_from_input`; the v22 comment was compressed to four lines for the long-comment-run guardrail.

### Facts for later tickets

- `TestAppBuilder` rebinds an injected pipeline to the **builder's** storage (`build_app_graph_for_tests` → `rebind_for_test` with `wired.storage`). A test that wants to mutate storage the pipeline reads must pass it via `.storage(...)` — otherwise the mutation hits an orphaned instance (this bit the content-update test on first run).
- The behavior change to flag in the commit message: a redo of an impersonated swipe after a preset change generates with the NEW active preset; a migrated row whose pinned preset was deleted no longer fails.
- Ticket 06 (narration-pipeline posture) is unblocked by this ticket's close; ticket 16's flat `GenerationInputs`/`ImpersonateInputs` (no `preset_id`) is the pin-free resolver shape ticket 06 threads game preset-ids through.
