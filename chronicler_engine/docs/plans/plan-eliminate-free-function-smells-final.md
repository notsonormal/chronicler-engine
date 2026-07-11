# Plan: Eliminate Free-Function Smells (Final)

## Summary
Move 13 module-level `pub fn`s into `impl` blocks on the right receiver, plus a structural deepening: introduce an `ApplicationQueries` seam so view fns don't pollute `PersistenceGate`. Delete 3 dead duplicates; add 1 `From` impl. Defer 32+ smells as documented exceptions (engine layer, mappers, Arc-self orchestrators, factories, guardrails).

- **Scope:** 13 fns moved + 3 deletions + 1 From impl + 5 test helpers, across 6 phases
- **Effort:** 16 SP
- **Validation:** `python chronicler_engine/scripts/build.py` after each phase + `find_free_fn_smells.py` to confirm count drops

## Key Changes

- 6 render fns → `impl AppState` (Phase 1)
- 8 query_handlers view fns → new `ApplicationQueries` seam over `PersistenceGate` (Phase 2A) — **deepening**, not relocation
- 3 message_editing CRUD fns → `impl PersistenceGate` (Phase 2B) — fits persistence concern
- 3 mapper fns → `From` impls (Phase 3)
- 1 `PromptContext` method + 6 bootstrap moves + `GenerationGate::release_owned_slot` delegator (Phase 4)
- 5 test helpers → `TestAppBuilder` methods (Phase 5)
- 3 fns deleted (dead duplicates) + 1 `From<&EngineError> for ApplicationError` added (Phase 0)
- `chronicler_engine/docs/architecture/system.md` gains "Free fn Doctrine" section (table-style, matching existing doc voice)

## Doctrine (accepted exceptions — not fixed)

| Category | Fns | Reason |
|----------|-----|--------|
| `domain/engine/action_processing.rs` ×7 | consume `GameState` by value | functional pipeline style (Issue 7) |
| `domain/engine/{logic,state_diagnostics,trigger_eval}.rs` ×6 | engine-layer logic over state (Issue 7) | engine's job |
| `adapters/driving/http/locks.rs` ×2 | `&RwLock<T>` | generic utility, no single receiver |
| `application/agents/quantifier/orchestration.rs::determine_npcs_in_room` ×1 | over `GameState` (Issue 7) | engine logic |
| `application/scenario.rs::inject_scenario_logs` ×1 | over `GameState` (Issue 7) | engine logic |
| `application/narrative_prompt/{assembler,context}.rs` ×2 | builder fns over domain (Issue 8) | idiomatic mapper pattern |
| `adapters/driven/storage/mappers/state_snapshot.rs::snapshot_to_db` ×1 | mapper (Issue 8) | idiomatic |
| `bootstrap/wiring.rs::build_*` ×4 | composition root factories | CONSTRUCTOR pattern |
| **`application/message_editing.rs::{retry, retrigger}` ×2** | take `Arc<app>` by value | spawn_blocking closure needs owned Arc |
| **`application/action_pipeline/retry.rs::{retry_last_response_impl, retrigger_event_impl}` ×2** | take `Arc<app>` | spawn_blocking closure |
| **`application/action_pipeline/actions.rs::execute_action_impl` ×1** | takes `Arc<app>` | spawn_blocking closure |
| Phase 7 guardrails | contradicts exceptions policy | removed |

## Implementation

### Phase 0: Dead code cleanup + `From` impl (1 SP)

- [ ] #### Task 0.1: Delete duplicates + add `From` impl + preserve `get_input_status` test (1 SP)

  **Files modified:**
  - `src/application/application_service.rs` — delete `load_messages_with_swipes` free fn (line 30); canonical is `PersistenceGate::load_messages_with_swipes` at `persistence_gate/gate.rs:201`
  - `src/application/query_handlers.rs` — delete `get_input_status` (line 58, byte-identical duplicate of `get_generating_status` above)
  - `src/application/mappers.rs` — delete `map_llm_error`; add `impl From<&EngineError> for ApplicationError` in `src/application/errors.rs`
  - `src/application/mod.rs` — remove `pub use mappers::map_llm_error;` re-export (verify with grep)
  - `src/application/query_handlers_tests.rs` — **rewrite** `test_get_input_status_delegates_to_generating_status` to assert on `get_generating_status` directly (preserves the delegation contract test)

  **Automation:**
  - Use sed for call-site renames: `sed -i 's/map_llm_error(\(.*\))/(\1).into()/g'` — review each resulting diff before commit (map_llm_error took a single arg; verify sed produces valid Rust)

  **Validation:**
  - `cd chronicler_engine && cargo fmt && cargo clippy -- -D warnings && cargo nextest run --lib`
  - `python scripts/find_free_fn_smells.py | head -3` — smell count drops by 3

### Phase 1: AppState renderers (3 SP)

- [ ] #### Task 1.1: Move 6 render fns into `impl AppState` (3 SP)

  **Free fns (move):**
  - `render_header`, `render_story_log`, `render_visual_sidebar`, `render_action_area`, `render_character_headshots`, `render_llm_messages`

  **Source:** `src/adapters/driving/http/fragments/renderers/fragment_renderers.rs`
  **Target:** `impl AppState` block in `src/adapters/driving/http/app_state.rs` (keep methods close to struct definition; implementer may prefer keeping them in `fragment_renderers.rs` as `impl AppState { ... }` if the file stays slim — file placement is a code-simplification call)

  **Files modified:**
  - `src/adapters/driving/http/fragments/renderers/fragment_renderers.rs` — internal cross-calls between the 6 render fns (7 sites)
  - Same file — 1 external call: `query_handlers::get_current_game_name(&state.application_service)` at line 34 (will be rewritten in Phase 2A; for now leave as-is to avoid conflating phases)

  **Automation:** sed pattern `render_header(&\(state\)\)/\1.render_header()/g` — verify diff

  **Validation:**
  - `cargo fmt && cargo clippy -- -D warnings && cargo nextest run --lib`
  - `python scripts/find_free_fn_smells.py | head -3` — smell count drops by 6

### Phase 2A: Deepening — `ApplicationQueries` seam (3 SP)

- [ ] #### Task 2A.1: Spawn `ApplicationQueries` module + move 8 query_handlers fns (3 SP)

  **Free fns (move):**
  - `get_generating_status`, `reset_generating_status`, `get_current_game_name`, `list_latest_llm_messages`, `get_story_log_entries`, `get_current_room_view`, `get_npc_headshots`, `get_debug_state`

  **Source:** `src/application/query_handlers.rs`
  **Target:** new `src/application/application_queries.rs`:

  ```rust
  //! [DOC: docs/system/game_flow.md]
  //! ApplicationQueries — read surface over the current game session.
  //! Wraps PersistenceGate's load_or_fresh for view/read paths; \
  //! does NOT own persistence (load/save stays on PersistenceGate).

  use crate::application::errors::ApplicationError;
  use crate::application::persistence_gate::PersistenceGate;
  use crate::application::DebugStateView;
  use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
  use crate::domain::model::state::message_types::MessageEntry;
  use crate::error::EngineError;
  use crate::application::ports::llm_message_repository::LlmMessage;

  pub struct ApplicationQueries<'a> {
      gate: &'a PersistenceGate,
  }

  impl<'a> ApplicationQueries<'a> {
      pub fn new(gate: &'a PersistenceGate) -> Self { Self { gate } }

      pub fn get_generating_status(&self) -> Result<(GenerationStatus, GenerationPhase), ApplicationError> {
          let state = self.gate.load_or_fresh();
          Ok((state.narrative.input_buffer.status.clone(),
              state.narrative.input_buffer.phase.clone()))
      }
      // ... 7 more methods
  }
  ```

  **Façade accessor** on `DefaultApplicationService`:
  ```rust
  pub fn queries(&self) -> ApplicationQueries<'_> {
      ApplicationQueries::new(&self.persistence_gate)
  }
  ```

  **Files modified:**
  - `src/application/application_queries.rs` (new module, `+impl ApplicationQueries` with 8 methods)
  - `src/application/application_queries_tests.rs` (new — move bodies from `query_handlers_tests.rs`; update call form)
  - `src/application/query_handlers.rs` (delete file)
  - `src/application/query_handlers_tests.rs` (delete file — bodies moved to `application_queries_tests.rs`)
  - `src/application/mod.rs` — drop `pub use query_handlers::*;` (line 38), drop `mod query_handlers_tests` if present; add `pub mod application_queries;` + `#[cfg(test)] mod application_queries_tests;`
  - `src/application/application_service.rs` — add `pub fn queries(&self)` accessor

  **Call sites (12 files, use sed):**
  - `src/adapters/driving/http/debug.rs:19` — `query_handlers::get_debug_state(&state.application_service)` → `state.application_service.queries().get_debug_state()`
  - `src/adapters/driving/http/fragments/endpoints.rs:102` — `reset_generating_status` → `state.application_service.queries().reset_generating_status()`
  - `src/adapters/driving/http/fragments/renderers/fragment_renderers.rs:34,41,52,55,71,85,100` — 7 sites (Phase 1 touched the file; subagent should have fresh context)
  - `src/application/application_service.rs:123` — internal call to `get_generating_status` → `self.queries().get_generating_status()` or `ApplicationQueries::new(&self.persistence_gate).get_generating_status()`
  - `tests/integration/application/game_service.rs` — verify (likely already uses `app.foo()` via façade; update to `app.queries().foo()`)
  - `tests/llm/flow_llm_tests.rs` — verify

  **Automation:**
  - Use sed for the regular pattern: `s/query_handlers::get_\([a-z_]*\)(&\([^.]*\)\.application_service)/\2.application_service.queries().get_\1()/g`
  - Verify diff per file before commit
  - Per-phase diff cap: if total LOC edited exceeds 500, decompose into sub-commits

  **Validation:**
  - `cargo fmt && cargo clippy -- -D warnings && cargo nextest run --lib`
  - `python scripts/find_free_fn_smells.py | head -3` — smell count drops by 8

### Phase 2B: `PersistenceGate` — message_editing CRUD (2 SP)

- [ ] #### Task 2B.1: Move 3 mutation fns onto `impl PersistenceGate` (2 SP)

  **Free fns (move):**
  - `switch_swipe`, `edit_history`, `delete_last`

  **Source:** `src/application/message_editing.rs`
  **Target:** `src/application/persistence_gate/gate.rs` `impl PersistenceGate` block

  **Note:** `retry` and `retrigger` in the same file (taking `Arc<DefaultApplicationService>` by value) are DEFERRED — they spawn_blocking and need owned Arc. The file does NOT get deleted; it keeps `retry` and `retrigger` as documented free fns.

  **Files modified:**
  - `src/application/persistence_gate/gate.rs` (+3 methods)
  - `src/application/message_editing.rs` — keep `retry` and `retrigger` (add `// Doctrine: Arc<app> needed for spawn_blocking` comment); delete the 3 moved fns
  - `src/application/mod.rs` — remove `delete_last, edit_history, switch_swipe` from the `pub use message_editing::{...};` line (line 37); KEEP `retry, retrigger`

  **Call sites:**
  - `src/adapters/driving/http/fragments/history.rs:25,32` — `message_editing::edit_history(&state.application_service, ...)` → `state.application_service.persistence_gate().edit_history(...)`
  - `src/adapters/driving/http/fragments/misc/swipe.rs:21` — similar
  - `tests/integration/application/game_service.rs:208,236,259,281,297,321` — similar (update each)

  **Automation:** sed pattern `s/message_editing::\([a-z_]*\)_\?\(&\([^.]*\).application_service\(.*\)\)\)/\3.application_service.persistence_gate().\1(\4)/g` — review diff per-site; some have trailing args (e.g. `edit_history(&app, id, form)`)

  **Validation:**
  - `cargo fmt && cargo clippy -- -D warnings && cargo nextest run --lib`

### Phase 3: Adapter `From` impls + `PromptContext` method (2 SP)

- [ ] #### Task 3.1: Convert 3 message mapper free fns to `From` impls + `PromptContext::build_narration_prompt` (2 SP)

  **Free fns (convert):**
  - `db_message_to_model(&DbMessage)` → `impl From<&DbMessage> for Message` in `adapters/driven/storage/mappers/message.rs`
  - `model_message_to_db(&Message)` → `impl From<&Message> for DbMessage` in same file
  - `model_swipes_to_db(&Message)` → keep as free fn in same file (mapper module, idiomatic per Issue 8 doctrine; flag as exception in `system.md`)
  - `build_narration_prompt(&PromptContext)` → `impl PromptContext { fn build_narration_prompt(&self) -> ... }` in `application/narrative_prompt/types.rs`

  **Files modified:**
  - `src/adapters/driven/storage/mappers/message.rs` — 2 fns become `From` impls; 1 stays as free fn with doctrine comment
  - `src/application/narrative_prompt/types.rs` — add `impl PromptContext { fn build_narration_prompt(&self) -> ... }`
  - `src/application/narrative_prompt/assembler.rs` — delete the free fn `build_narration_prompt`; OR keep as one-line delegator if other code still imports the free name (implementer preference; deletion preferred)

  **Call sites:**
  - `rg "db_message_to_model\|model_message_to_db\|model_swipes_to_db\|build_narration_prompt" chronicler_engine/src chronicler_engine/tests`
  - Convert: `db_message_to_model(&x)` → `Message::from(&x)` (idiomatic) or `(x as &DbMessage).into()`

  **Validation:**
  - `cargo fmt && cargo clippy -- -D warnings && cargo nextest run --lib`

### Phase 4: Bootstrap moves + `GenerationGate` release delegator (4 SP)

- [ ] #### Task 4.1: Move 6 bootstrap fns + add `GenerationGate::release_owned_slot` delegator (4 SP)

  **Free fns (move):**
  - `application/generation_gate/slot.rs::release_owned_slot` — **Option C**: keep free fn, add `impl GenerationGate { pub fn release_owned_slot(&self, game_id, gen_id) { release_owned_slot(&self.registry, &self.is_generating, game_id, gen_id) } }` delegator. Document why free fn stays (used by `GenerationGuard::drop` which doesn't hold `&GenerationGate`)
  - `bootstrap/llm_factory.rs::get_llm_recorder_for(&LlmProviderConfig)` → `impl LlmProviderConfig`
  - `bootstrap/load.rs::seed_game_data(&Storage, &Path)` → `impl Storage { fn seed_game_data(&self, &Path) }`
  - `bootstrap/text_check_factory.rs::create_text_check_service(&AppSettings)` → `impl AppSettings { fn create_text_check_service(&self) }`
  - `bootstrap/validate.rs::validate_loaded_data(&WorldCard)` → `impl WorldCard { fn validate(&self) }`
  - `settings.rs::load_settings(&Storage)` → `impl AppSettings { fn load(storage: &Storage) }`
  - `src/test_support/context.rs::seed_test_world_into_storage(&Storage, &GameState)` → `impl Storage { fn seed_test_world_into(&self, &GameState) }` (or `StorageExt` under `#[cfg(feature = "testing")]` if added to src/ doesn't compile cleanly into the Storage impl)

  **Files modified:**
  - `src/application/generation_gate/gate.rs` (+1 delegator method)
  - `src/application/generation_gate/slot.rs` (free fn stays, add doctrine comment)
  - `src/adapters/driven/storage/storage.rs` or wherever `Storage` impl lives (+2 methods: `seed_game_data`, `seed_test_world_into`)
  - `src/domain/model/settings.rs` (+2 methods on `AppSettings`: `create_text_check_service`, `load`)
  - `src/domain/model/world.rs` (+1 method on `WorldCard`: `validate`)
  - `src/domain/model/llm_provider_config.rs` or actual location (+1 method on `LlmProviderConfig`)
  - `src/test_support/context.rs` (delete `seed_test_world_into_storage` after move)

  **Chesterton's Fence check (mandatory before each move):**
  - Subagent MUST run `git log -p --follow <file>` on each fn being moved and record any rationale found in commit messages. If a commit message explains why the fn is a free fn (e.g. "extracted as free fn because X"), subagent stops and reports back before moving.

  **Call sites:**
  - `rg "release_owned_slot\|get_llm_recorder_for\|seed_game_data\|create_text_check_service\|validate_loaded_data\|load_settings\|seed_test_world_into_storage" chronicler_engine/src chronicler_engine/tests`

  **Validation:**
  - `cargo fmt && cargo clippy -- -D warnings && cargo nextest run --lib`

### Phase 5: Test helpers → `TestAppBuilder` (3 SP)

- [ ] #### Task 5.1: Convert 5 pipeline_helpers fns to `TestAppBuilder` methods + write Free fn Doctrine section (3 SP)

  **Free fns (move):**
  - `wait_for_generation_complete`, `latest_state`, `save_state`, `add_input_and_save`, `latest_snapshot`

  **Source:** `tests/helpers/pipeline_helpers.rs`
  **Target:** `impl TestAppBuilder { ... }` (already exists in `src/test_support/test_app_builder.rs`); OR extension trait `PipelineTestExt for Arc<DefaultApplicationService>` in the test module. Implementer picks based on existing `TestAppBuilder` shape (subagent should read `TestAppBuilder` first to decide).

  **Files modified:**
  - `tests/helpers/pipeline_helpers.rs` (delete file after move, or keep as extension-trait definition site)
  - `src/test_support/test_app_builder.rs` (+5 methods if routing into `TestAppBuilder`)
  - `chronicler_engine/docs/architecture/system.md` (+ new "Free fn Doctrine" section — see below)

  **MANDATORY first step:** Subagent runs `rg "pipeline_helpers::" chronicler_engine/tests` to enumerate every call site in `tests/`. The plan does NOT pre-enumerate these.

  **Chesterton's Fence check before each move:**
  - `git log -p --follow tests/helpers/pipeline_helpers.rs` — confirm no architectural rationale for free-fn shape.

  **Validation:**
  - `cargo fmt && cargo clippy -- -D warnings && cargo nextest run` (full — includes integration tests)

  **"Free fn Doctrine" section to add to `system.md`** (terse, table-style — matches existing doc voice):

  ```markdown
  ## Free fn Doctrine

  A module-level `pub fn` taking `&DomainType` / `&mut DomainType` is a **smell** (should be a method on its primary receiver), **unless** it falls into one of these categories:

  | Category | Where | Reason |
  |----------|------|--------|
  | Engine logic over state | `domain/engine/*` over `&GameState` / `&MapDef` | engine's job to act on state |
  | Generic utility | `&RwLock<T>` etc. | no single concrete receiver |
  | Adapter mapper / builder | `application/narrative_prompt/*`, `adapters/driven/storage/mappers/*` | idiomatic layer convention; one-to-one conversions use `From` impls |
  | Composition root factory | `bootstrap/wiring.rs::build_*` taking `Arc<RwLock<AppSettings>>` | factory = owned input, not borrow |
  | Arc-self spawn orchestrator | `application/message_editing.rs::{retry, retrigger}`, `application/action_pipeline/{retry.rs::retry_last_response_impl, retrigger_event_impl, actions.rs::execute_action_impl}` | `spawn_blocking` closure needs owned `Arc<app>` for task lifetime |

  Examples (deferred, not converted):
  - `domain/engine/logic.rs::{find_room_in_map, find_room_in_world_map, attempt_semantic_walk}` — category 1
  - `domain/engine/state_diagnostics.rs::assert_state_consistency`, `trigger_eval.rs::evaluate_triggers`, `action_processing.rs::execute_freeaction_impl` — category 1
  - `adapters/driving/http/locks.rs::{read_lock_or_recover, write_lock_or_recover}` — category 2
  - `application/narrative_prompt/{assembler,context}.rs`, `adapters/driven/storage/mappers/state_snapshot.rs::snapshot_to_db` — category 3
  - `application/scenario.rs::inject_scenario_logs`, `application/agents/quantifier/orchestration.rs::determine_npcs_in_room` — category 1

  **Read surface over the current game session** lives on `ApplicationQueries` (`src/application/application_queries.rs`), not on `PersistenceGate` (persistence concern) or `DefaultApplicationService` (orchestration concern).
  ```

## Test Plan

After each phase:
1. `cd chronicler_engine && cargo fmt`
2. `cargo clippy -- -D warnings`
3. `cargo nextest run --lib` (Phase 5: `cargo nextest run` — full)
4. `python scripts/find_free_fn_smells.py | head -3` — confirm smell count drops by expected amount
5. `git diff --stat` — review scope matches the phase
6. **Diff cap:** if the phase exceeded 500 LOC changed, subagent decomposed into sub-commits before validation runs (per code-simplification Rule of 500)
7. **Chesterton's Fence log:** if any fn being moved had a commit-message rationale surfacing in `git log -p --follow`, that rationale is in `chronicler_engine/docs/architecture/system.md` "Free fn Doctrine" section before the move is committed

After all phases:
8. `python build.py` — full validation (fmt + clippy + tests + coverage)
9. `python scripts/find_free_fn_smells.py` — total SMELL ≤ 32 (accepted-exception set)

## Per Task/Sub Task Validation Steps

Every task:
```
1. cd chronicler_engine && cargo fmt
2. cargo clippy -- -D warnings
3. cargo nextest run --lib
4. python scripts/find_free_fn_smells.py | head -3
   # Confirm SMELL count decreased by N (the task's fn count)
5. git diff --stat | head -20
   # Confirm no files outside the task's listed scope were touched
6. git log --follow <src-file> | head -5
   # Confirm Chesterton's Fence check ran; no hidden rationale missed
```

## Assumptions

- **No port trait for `ApplicationService`.** Confirmed via grep — `DefaultApplicationService` is a concrete struct, not a trait; adding methods + a `queries()` accessor does not affect adapter boundary.
- **`ApplicationQueries` deepening.** New module borrows `&PersistenceGate` (arbitrary lifetime, no `Arc`). `DefaultApplicationService::queries(&self)` returns `ApplicationQueries<'_>` borrowing from `self.persistence_gate`. Callers cannot outlive the borrow; Rust's borrow checker enforces.
- **`message_editing` file stays.** `retry` and `retrigger` (Arc-self fns for spawn_blocking) remain as free fns in `src/application/message_editing.rs`; the file is NOT deleted. Only the 3 mutation fns move to `PersistenceGate`.
- **`map_llm_error` conversion choice.** Implementer picks `impl From<&EngineError> for ApplicationError` (preferred) vs `EngineError::to_application(&self)` (named method). Both acceptable.
- **`model_swipes_to_db` stays as free fn.** Per Issue 8 doctrine; flagged in `system.md`.
- **`release_owned_slot` Option C.** Free fn stays for `GenerationGuard::drop`; `impl GenerationGate::release_owned_slot` is a one-line delegator. Documented in `slot.rs` with rationale.
- **Phase 5 routing deferred to implementer.** `TestAppBuilder` methods vs `PipelineTestExt` extension trait — depends on existing `TestAppBuilder` shape. Subagent reads the file first and decides.
- **Phase 5 call sites not pre-enumerated.** Implementer MUST run `rg "pipeline_helpers::" chronicler_engine/tests` first.

## Behavioral preservation (observable-level claims)

HTTP responses, persisted state, and test outcomes preserved. Acknowledged contract changes:

- **`get_input_status`** — removed from public surface; delegation contract preserved by rewriting its test to assert on `get_generating_status` directly (test still exists, no behavioral change documented)
- **`map_llm_error`** — removed from public surface; callers use `.into()`. If `map_llm_error` had tracing/logging calls beyond pure conversion, those are lost. Subagent verifies via `git log -p` before deleting.
- **`retry` / `retrigger` / `retry_last_response_impl` / `retrigger_event_impl` / `execute_action_impl`** — DEFERRED, not moved. Free-fn shape is correct because these take `Arc<app>` by value for spawn_blocking closure lifetimes; converting to `&self` would force Arc::clone at call sites AND break the spawn contract. This is the load-bearing reason for the doctrine exception.
- **Test path in Phase 2A** — tests moved from `query_handlers_tests.rs` to `application_queries_tests.rs`. Test bodies unchanged except the call form (`get_foo(&app)` → `app.queries().get_foo()`).

## Automation mandate (code-simplification Rule of 500)

Each phase:
- Use `sed` / `codemod` for mechanical call-site renames where the pattern is regular. Example:
  ```bash
  # Phase 2A call-site rewrite (after methods exist):
  sed -i 's|query_handlers::get_\([a-z_]*\)(&\([^.]*\)\.application_service)|\2.application_service.queries().get_\1()|g' \
    src/adapters/driving/http/debug.rs \
    src/adapters/driving/http/fragments/endpoints.rs \
    src/adapters/driving/http/fragments/renderers/fragment_renderers.rs
  ```
- Review each sed diff before commit. Rust syntax occasionally breaks the regex (lifetimes, multiline); fall back to manual edit for those cases.
- Per-phase diff cap: if a phase exceeds 500 LOC edited, subagent MUST decompose into sub-commits before running the phase's validation step. Mixed renaming + unreverted logic changes in one commit violate the incremental-over-revolutionary principle.
- Chesterton's Fence check: subagent runs `git log -p --follow <file>` on each file where fns are being DELETED (Phase 0 deletes, Phase 1 file moves, Phase 2A file delete). If commit messages reveal a rationale for the free-fn shape, that rationale is added to `system.md` BEFORE the move lands.
