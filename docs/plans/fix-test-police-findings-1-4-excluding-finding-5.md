# Fix Test-Police Findings 1–4 (excluding Finding 5)

## Summary
Restore lost error-branch coverage for `retry_user_regen`, clean up unreachable defensive code in `retry.rs` with tests for the reachable branches, and remove the now-write-only per-game posture data (`Game::narrative_perspective/tense`) end-to-end including a v21 DB migration. Per decision, old DBs containing `"Narrator"` message rows are accepted as broken (no migration).

## Key Changes
- **Finding 1**: Restore three unit tests (missing target, narrator fail, cancellation) exercising `retry_user_regen`'s error branches at `src/application/pipeline/action_pipeline/retry.rs:203-208,227-230`, plus the deleted `assert_error_status` helper.
- **Finding 2**: Delete the unreachable `else => return None` arm in `resolve_retry_target` (`retry.rs:140`); add tests for the reachable uncovered branches (`retry.rs:93-94,173-174,190-193`).
- **Finding 3**: No code change — accepted breakage (decision).
- **Finding 4**: Remove `Game`/`NewGame` posture fields, all storage read/write code, the `*_with_posture` names, and the `games` table columns via a `user_version = 21` migration. Also delete the orphaned `settings_defaults::default_narrative_perspective/tense` (review finding, decided: include).

## Implementation

### Phase 1: Restore `retry_user_regen` error-branch coverage

- [ ] #### Task 1.1: Restore error helpers and three user-regen tests (3 SP)
  - [ ] ##### SubTask 1.1.1: Restore `assert_error_status` helper in `retry_tests.rs` (1 SP)
    - Model on the deleted version (`git show HEAD:src/application/pipeline/action_pipeline/retry_tests.rs` lines 1436-1447): reload via `app.message_service.load_or_fresh()`, match `GenerationStatus::Error`, assert substring.
  - [ ] ##### SubTask 1.1.2: `test_retry_user_regen_missing_target` (1 SP)
    - Direct call `app.pipeline.retry_user_regen(state)` with `state.narrative.retry_target = None` and one seeded `Input`. Asserts `retry.rs:203-208`: persisted Error status contains `"missing user-regen target"`.
  - [ ] ##### SubTask 1.1.3: `test_retry_user_regen_narrator_fails` (1 SP)
    - `MockBackend::default().with_fail()` narrator; state with `retry_target = Some(Input)`. Direct call. Asserts `retry.rs:228-230`: Error status contains `"configured_failure"`.
  - [ ] ##### SubTask 1.1.4: `test_retry_user_regen_cancelled` (1 SP)
    - Pre-cancel `app.shutdown_token` before the call (deterministic; same pattern as `test_retry_event_continuation_cancels_before_llm`, retry_tests.rs:363). Direct call. Asserts `retry.rs:227`: returns `Err(PhaseError::Cancelled)` and status is reset to `Idle` (`handle_cancellation` at `pipeline_run.rs:424-432` resets and persists Idle); no new swipe added.
  - Do not restore `room_not_found` / `load_world_bundle_fails` user-regen variants — the shared-prefix failures are covered by `narration_generation_tests.rs`; only the call-site `Err(e) => finalize` arm needed its own test.

### Phase 2: Defensive branches in `retry.rs`

- [ ] #### Task 2.1: Delete unreachable arm in `resolve_retry_target` (1 SP)
  - `retry.rs:140`: `old_target` comes from a Narration|Input filter, so the final `else => return None` cannot execute. Restructure the mode if-chain into an exhaustive match on `message_type` (+ `is_event`) with no dead arm; the compiler then enforces exhaustiveness if a type is ever added to the filter.

- [ ] #### Task 2.2: Tests for reachable uncovered branches (3 SP)
  - [ ] ##### SubTask 2.2.1: ReNarrate no-input edge — `retry.rs:93-94` (1 SP)
    - Seed history `[Narration (plain), Narration + event_header]`, no `Input` anywhere. The plain-Narration anchor needs a snapshot: `save_snapshot` + `set_snapshot_id` + `insert_message_with_swipe`, as in `test_retry_room_not_found_sets_error` (retry_tests.rs:662). `retry_last_response()` → truncation leaves no input. Assert persisted Error status contains `"no input to retry"` (`persist_generation_error` writes Error to persisted state — `core.rs:478-485`).
  - [ ] ##### SubTask 2.2.2: `check_retry_anchor` no-anchor — `retry.rs:173-174` (1 SP)
    - Game with only a plain Narration message (no Input, no event). `app.pipeline.retry(&gate)` → returned `ApplicationError` internal containing `"no anchor message"` plus persisted Error status.
  - [ ] ##### SubTask 2.2.3: `check_retry_anchor` storage error — `retry.rs:190-193` (1 SP)
    - `Storage::with_test_failures()` + `handle.set(<snapshot-load op>, TestOverride::internal(...))` (same override key as `test_retry_event_storage_error_on_pre_event`). `retry(&gate)` → internal error containing the injected message.

### Phase 3: Remove per-game posture data

- [ ] #### Task 3.1: Domain + application removal (3 SP)
  - `src/domain/model/game.rs`: drop `narrative_perspective`/`narrative_tense` from `Game` (lines 20-21) and `NewGame` (lines 35-36); drop now-unused imports.
  - `src/application/games/catalogue.rs` (NewGame at :55, :145) and `src/bootstrap/init_game.rs` (:43): drop the two field initializations each.
  - Rename `create_game_with_posture` → `create_game_from_request` (`games.rs:80`) and `insert_game_with_posture` → `insert_game_from_request` (`db.rs:55`); update callers (`catalogue.rs:69,158`, `init_game.rs:56`).
  - `src/application/games/catalogue_tests.rs`: remove the posture-inheritance assertions (lines 61-62, 85-86).
  - `src/domain/model/utils/settings_defaults.rs`: delete the orphaned `default_narrative_perspective`/`default_narrative_tense` (zero references, including serde `default = "..."` paths).

- [ ] #### Task 3.2: Storage layer removal (3 SP)
  - `src/adapters/driven/storage/models/game.rs`: drop the two `DbGame` fields; shift `from_row` indices 11-13 → 9-11; drop the two parse lines in the to-domain conversion.
  - `src/adapters/driven/storage/games.rs`: drop the columns from both SELECT lists (:19, :134) and both in-memory `Game` constructors (:69-70, :97-98).
  - `src/adapters/driven/storage/db.rs`: drop the two columns/params from the INSERT (:62-73).
  - `src/bootstrap/run_tests.rs`: drop posture from the SELECT tuple and assertions (lines 69-70, 86-87, 94-112); keep the `narrator_mode` assertion.
  - **Compile unit**: Tasks 3.1 and 3.2 must land together — the domain field removal breaks storage constructors until 3.2 lands. `cargo nextest run --lib` is valid only after both.

- [ ] #### Task 3.3: v21 migration + migration test (3 SP)
  - `src/adapters/driven/storage/utils/plumbing.rs`: new `if version < 21` block — `column_exists`-guarded `ALTER TABLE games DROP COLUMN narrative_perspective/tense` (same pattern as the settings posture drop at :421-425); set `user_version` 21.
  - New `src/adapters/driven/storage/plumbing_tests.rs` (sibling unit test file, registered in `storage/mod.rs`) — **unit tier, not `tests/storage/`**: `run_migrations` is `pub(crate)` (`plumbing.rs:17`), invisible to integration tests. Mechanism: `DbPool::new(":memory:")` → `ALTER TABLE games ADD COLUMN` both posture columns back + set `user_version = 20` → call `run_migrations(&conn)` directly → assert columns are gone (`pragma table_info`) and the game row loads. Fresh-DB path (CREATE TABLE never had the columns) is covered by the existing SQLite suite.

## Test Plan
- Per phase: `cargo nextest run retry` (Phases 1-2) and `cargo nextest run --lib` (Phases 1 and 3, after the 3.1+3.2 compile unit).
- Phase 3 additionally: `cargo nextest run --tests` (SQLite-backed integration suite exercises the migration chain and the changed storage layer).
- Final gate: `python build.py` (fmt + clippy + guardrails + tests).
- Optional spot-check: llvm-cov shows `retry.rs` uncovered lines reduced to none beyond genuinely defensive code; `narration_generation.rs` stays ≥98%.

## Per Task/Sub Task Validation Steps
- 1.1.x: `cargo nextest run retry` — the three new tests pass; previously uncovered lines `retry.rs:203-208,227-230` now covered.
- 2.1: `cargo nextest run retry` + `cargo nextest run --test architecture` — no behavior change, no arch violations.
- 2.2.x: new tests pass; `retry.rs:93-94,173-174,190-193` covered.
- 3.1+3.2: `cargo nextest run --lib` — catalogue/bootstrap/storage tests pass after assertion adjustments (run only after both tasks land).
- 3.3: `cargo nextest run --lib plumbing` for the migration test, then `cargo nextest run --tests`; existing storage tests green on both fresh and migrated DB shapes.

## Assumptions
- **Finding 3 (decided)**: accepted breakage — no migration for `"Narrator"` rows; an old DB containing one fails to load with `EngineError::Config` and must be repaired manually. No docs change.
- **World-level posture stays** (`WorldCard`, `TemplateVars`, assembler `apply_posture`) — only the per-game copies go.
- **Phases 1-2 change no production behavior** (Phase 2.1 deletes only by-construction-unreachable code).
- **`Game` serde**: unknown fields are ignored on deserialize, so no snapshot/JSON compatibility risk from field removal.

### NOT in scope
- Finding 5 (browser server-start timeout flake under coverage instrumentation) — excluded by request.
- `"Narrator"` row migration — decided: accepted breakage.
- Other pre-existing dead code, low-coverage files from the <80% list (`bootstrap/`, LLM transport), and the stale `.agents/skills/test-police/TEST_INVENTORY.md` inventory.
- Any per-game posture UI/feature — the data is removed, not relocated; if per-game posture returns later, it returns as a new feature with new fields.

### What already exists (reuse, don't reimplement)
- `assert_error_status` helper shape — restore from HEAD rather than inventing new assertion plumbing.
- Seeding dance (`save_snapshot` + `set_snapshot_id` + `insert_message_with_swipe`) — `retry_tests.rs` helpers, used by `test_retry_room_not_found_sets_error`.
- Cancellation pattern — `test_retry_event_continuation_cancels_before_llm` / `test_pipeline_cancels_when_token_already_cancelled`.
- Failure injection — `Storage::with_test_failures()` + `TestOverride::internal` (used by `test_run_bundle_load_failure_returns_fetch_failed`).
- Migration idioms — `column_exists` guard + `user_version` bump in `plumbing.rs`; `run_migrations(&Connection)` callable directly from unit tests.
- `MockBackend` builders — `with_fail`, `with_delay`, `with_narrations`.

### Failure modes
- Phase 1 tests: error paths persist via `finalize_phase_error`/`handle_cancellation`/`persist_generation_error` — all assert on persisted status, so a swallowed failure (e.g. `save_state` error inside `persist_generation_error`, core.rs:482-484) would surface as a wrong status in the reload assertion.
- Phase 2.1: deleting the arm converts a silent defensive fallback into a compile error if reachability assumptions ever break — the intended failure mode.
- Phase 3 migration: `column_exists` guards make the DROP idempotent; a DB missing the columns (fresh) no-ops. `ALTER TABLE ... DROP COLUMN` requires SQLite ≥3.35 (rusqlite bundled version satisfies this — same operation already used for the settings columns).
- Migration test: if `run_migrations` regresses (e.g. skips v21), `pragma table_info` assertion fails — the test fails loudly, not silently.

### Unresolved decisions
None — all decisions settled (Narrator rows: accept; posture: remove fully; dead arm: delete; orphan defaults: include).
