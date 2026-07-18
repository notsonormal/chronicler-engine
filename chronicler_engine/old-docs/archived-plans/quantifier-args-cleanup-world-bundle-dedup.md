# Quantifier Args Cleanup + World Bundle Dedup

## Summary

Two independent refactors:
1. **World bundle deduplication** — `pipeline.rs::load_world_bundle` and `gate.rs::fetch_world_data_for_fresh_state` contain character-identical 6-line fetch sequences. Push assembly onto `Storage::world_bundle_for(game_id)` returning the same `Arc`-wrapped tuple. Reverses the prior "no bundle helper" decision from `storage-require-helpers.md` strictly at the fetch boundary; the tuple never propagates across layers (caller destructures immediately), so the original coupling concern doesn't apply.
2. **`determine_npcs_in_room` argument reduction** — 10 → 4 args via three cuts across three phases: drop `previous_room_npcs` (redundant snapshot of `state.scene.npcs_in_area`), drop `room_npc_ids` (always `&[]` in prod, tests seed `state.scene.npcs_in_area`), and replace the five world-context args with `&AgentContext` (the struct callers already have). Return type becomes `Result<QuantifierResult, EngineError>` to propagate the `ctx.current_room` absence cleanly (no `.expect`, no new `#[allow]`).

## Key Changes

- `src/adapters/driven/storage/backend/worlds.rs`: add `pub fn world_bundle_for(&self, game_id: u64) -> Result<(Arc<WorldCard>, Arc<MapDef>, Arc<PersonaCard>, HashMap<String, NpcCard>), EngineError>` — single site with `#[allow(clippy::type_complexity)]`. Body composes existing `require_game` → `require_world` → `require_persona` → `list_characters`. `PersonaCard` returned as `Arc<PersonaCard>` to match both current call sites (zero migration).
- `src/application/action_pipeline/pipeline.rs`: `load_world_bundle` becomes 1-line delegate to `app.storage().world_bundle_for(started_for)`. Drop `#[allow(clippy::type_complexity)]` and now-unused imports. Caller `pipeline.rs:54` unchanged (destructures same tuple).
- `src/application/action_pipeline/retry.rs:124`: second caller of `load_world_bundle`, unchanged (destructures `(_, map, persona, npcs_map)`, drops `_` world card; delegate-through-helper preserves API).
- `src/application/persistence_gate/gate.rs`: `fetch_world_data_for_fresh_state` becomes 1-line delegate to `self.storage.world_bundle_for(self.storage.current_game_id())`. Drop `#[allow(clippy::type_complexity)]`. May inline into `build_fresh_initial_state` if it's the sole caller — in-task decision.
- `src/application/agents/quantifier/orchestration.rs`: `determine_npcs_in_room` new signature `(ctx: &AgentContext, main_response: &str, recorder: &LlmCallRecorder, quantifier_prompt_override: Option<String>) -> Result<QuantifierResult, EngineError>`. Drops `#[allow(clippy::too_many_arguments)]`. Reads `previous_room_npcs` from `ctx.state.scene.npcs_in_area`. Low-conf fallback uses `ctx.state.scene.npcs_in_area` directly (kills empty-vs-non-empty `room_npc_ids` branch). `ctx.current_room.ok_or_else(...)?` returns `EngineError::RoomNotFound` (moved from `agent.rs::execute` — single source of truth).
- `src/application/agents/quantifier/agent.rs`: `execute` deletes its own `ctx.current_room.ok_or_else(...)?` (now redundant — handled in `determine_npcs_in_room`). Deletes `previous_room_npcs` snapshot. Drops `&[]` arg. Call site becomes `determine_npcs_in_room(ctx, main_response, self.recorder.as_ref(), quantifier_prompt_override)?`. Separate `main_response` `Option<&str>` resolution stays in `execute` (different concern).
- `src/application/agents/quantifier/orchestration_tests.rs`: `determine_npcs_with_room` helper rewritten to construct `AgentContext` from existing test inputs. Param rename: `npc_pool_ids` (world pool seed only, no longer a prod param) and `scene_npcs` (seeds `state.scene.npcs_in_area`, low-conf fallback source). Brief comment explains the split.

## Implementation

### Phase 1: World Bundle Dedup (5 SP)

- [ ] #### Task 1.1: Add `Storage::world_bundle_for(game_id)` (3 SP)
  - [ ] ##### SubTask 1.1.1: Add method to `impl Storage` in `backend/worlds.rs`. Body: `require_game(id) → require_world(&game.world_key) → require_persona(&game.persona_key) → list_characters(world_with_map.world_id) → HashMap collect → Arc-wrap world_card, map, persona`. Single `#[allow(clippy::type_complexity)]` on this method only. (1 SP)
  - [ ] ##### SubTask 1.1.2: Add unit test in `worlds_tests.rs` covering positive hit (returns Arc tuple with correct contents) + missing-game propagation (`GameNotFound`). Reuse existing fixtures. (2 SP)
  - Verify: `cargo test -p chronicler_engine --lib storage::backend::worlds`; `cargo clippy --lib -- -D warnings` clean.

- [ ] #### Task 1.2: Migrate `pipeline.rs::load_world_bundle` (1 SP)
  - Replace 6-line body with `app.storage().world_bundle_for(started_for)`. Drop `#[allow(clippy::type_complexity)]` and unused imports (`WorldCard`, `MapDef`, `PersonaCard`, `NpcCard`, `HashMap` if no longer directly used in this file).
  - Verify: `cargo build`; `cargo clippy --lib -- -D warnings` shows no new warning.

- [ ] #### Task 1.3: Migrate `gate.rs::fetch_world_data_for_fresh_state` (1 SP)
  - Replace body with `self.storage.world_bundle_for(self.storage.current_game_id())`. Drop `#[allow(clippy::type_complexity)]`. If `build_fresh_initial_state` is the sole caller (it is, per grep), inline the call directly there and delete the private method.
  - Verify: `cargo build`; `cargo test -p chronicler_engine --lib persistence_gate`.

### Phase 2: Drop `previous_room_npcs` param (3 SP)

- [ ] #### Task 2.1: Drop `previous_room_npcs` from `determine_npcs_in_room` (3 SP)
  - [ ] ##### SubTask 2.1.1: In `orchestration.rs::determine_npcs_in_room`, remove the param. Build `let previous_room_npcs: Vec<NpcCard> = state.scene.npcs_in_area.clone();` locally; pass `&previous_room_npcs` to `QuantifierPromptContext`. (1 SP)
  - [ ] ##### SubTask 2.1.2: In `agent.rs::execute`, delete the `let previous_room_npcs = state.scene.npcs_in_area.clone();` snapshot and drop the arg from the call. (0.5 SP)
  - [ ] ##### SubTask 2.1.3: In `orchestration_tests.rs::determine_npcs_with_room`, drop the `previous_room_npcs` param. Tests that previously passed `&[npc_a, npc_b]` now seed `state.scene.npcs_in_area` before calling. Rename the retained scene-seed param to `scene_npcs` (downstream of Phase 3 this becomes the low-conf-fallback seed). Add brief comment on helper explaining seeding role. (1.5 SP)
  - Verify: `cargo test -p chronicler_engine --lib agents::quantifier`. Lint still has `#[allow(too_many_arguments)]` (9 args remaining) — expected, removed in Phase 4.

### Phase 3: Drop `room_npc_ids` param (3 SP)

- [ ] #### Task 3.1: Drop `room_npc_ids` from `determine_npcs_in_room` (3 SP)
  - [ ] ##### SubTask 3.1.1: In `orchestration.rs::determine_npcs_in_room`, remove `room_npc_ids: &[String]`. In `static_npc_result` (called from `process_quantifier_result`), remove the `room_npc_ids` param and the empty-vs-non-empty branch — always read `state.scene.npcs_in_area` (non-empty branch becomes unconditional). Update `quantify_room_with_llm_call` fallback: derive `fallback_npc_ids` from `state.scene.npcs_in_area` (IDs) inside `determine_npcs_in_room` and pass as before. (2 SP)
  - [ ] ##### SubTask 3.1.2: In `agent.rs::execute`, drop the `&[]` arg. (0 SP — trivial)
  - [ ] ##### SubTask 3.1.3: In `orchestration_tests.rs::determine_npcs_with_room`, drop the `room_npc_ids` param. Rename `npc_pool_ids` (was `room_npc_ids`) — world pool seed only. Tests that asserted specific low-conf rosters (e.g. `test_determine_npcs_low_confidence_fallback`) now pass both `scene_npcs = [a,b]` (seeds `state.scene.npcs_in_area`) AND `npc_pool_ids = [a,b]` (world pool must contain them for lookup). Helper comment:
    ```rust
    // npc_pool_ids seeds the world lookup map; scene_npcs seeds state.scene.npcs_in_area
    // (low-conf fallback source). Both must contain asserted-IDs in low-conf tests.
    ```
    (1 SP)
  - Verify: `cargo test -p chronicler_engine --lib agents::quantifier`; existing assertions still hold (same NPCs now come from state).

### Phase 4: Replace world-context args with `&AgentContext` (5 SP)

- [ ] #### Task 4.1: `determine_npcs_in_room` takes `&AgentContext` (5 SP)
  - [ ] ##### SubTask 4.1.1: In `orchestration.rs::determine_npcs_in_room`, change signature to `(ctx: &AgentContext, main_response: &str, recorder: &LlmCallRecorder, quantifier_prompt_override: Option<String>) -> Result<QuantifierResult, EngineError>`. First line of body:
    ```rust
    let current_room = ctx.current_room
        .ok_or_else(|| EngineError::RoomNotFound("current room not set in AgentContext".into()))?;
    let state = ctx.state;
    ```
    Replace `map`, `persona`, `npcs` with `ctx.map`, `ctx.persona`, `ctx.npcs`. Add `use crate::domain::model::agent::AgentContext;`. Drop `#[allow(clippy::too_many_arguments)]`. (2 SP)
  - [ ] ##### SubTask 4.1.2: In `agent.rs::execute`, delete the existing `let current_room = ctx.current_room.ok_or_else(...)?` early-return (now redundant — `determine_npcs_in_room` enforces it). Change call site to:
    ```rust
    let result = determine_npcs_in_room(
        ctx,
        main_response,
        self.recorder.as_ref(),
        quantifier_prompt_override,
    )?;
    ```
    The `main_response` `Option<&str>` resolution (`ok_or_else(|| EngineError::Config(...))`) stays in `execute`. (0.5 SP)
  - [ ] ##### SubTask 4.1.3: In `orchestration_tests.rs`, rewrite `determine_npcs_with_room` to construct `AgentContext { state, main_response, player_input, current_room: Some(&room), map, persona, npcs }` from existing test inputs, then call `determine_npcs_in_room(&ctx, main_response, recorder, override)`. All 6 call sites updated. (2.5 SP)
  - Verify: `cargo test -p chronicler_engine --lib agents::quantifier`; `cargo clippy -p chronicler_engine --all-targets -- -D warnings` shows no `too_many_arguments` warning in `orchestration.rs`.
- [ ] #### Task 4.2: Final validation (primary agent — 5 SP task requires primary verification per AGENTS.md) (implicit)
  - Run `python build.py` from `chronicler_engine/`. Green required.
  - `grep -rn "allow(clippy::too_many_arguments)" src/application/agents/quantifier/` returns no matches.
  - `grep -rn "allow(clippy::type_complexity)" src/` returns exactly one match (on `Storage::world_bundle_for`).

## Test Plan

- Existing quantifier unit tests (`orchestration_tests.rs`, `prompt_tests.rs`, `agent_tests.rs`) cover all behavior paths (high/medium/low confidence, backend error, unknown ID filtering, retry). All must pass unchanged in assertions after the agent.rs and test-helper rewrites.
- Existing `pipeline.rs::load_world_bundle` callers (pipeline happy path, retry continuation) covered by `action_processing_tests.rs` and integration tests.
- Existing `persistence_gate::build_fresh_initial_state` covered by gate tests.
- New: one positive-hit + one missing-game test for `Storage::world_bundle_for`.
- `python build.py` green final validation (run by primary agent after Phase 4 completion; required because Task 4.1 is 5 SP).

## Per Task/Sub Task Validation Steps

- After each task: `cargo build -p chronicler_engine` clean.
- After each task touching `orchestration.rs` or `agent.rs`: `cargo test -p chronicler_engine --lib agents::quantifier` all pass.
- After each task touching storage: `cargo test -p chronicler_engine --lib storage::backend::worlds` all pass.
- After each task: `cargo clippy -p chronicler_engine --all-targets -- -D warnings` shows no new warnings.
- After Phase 4 completes: `grep -rn "allow(clippy::too_many_arguments)" src/application/agents/quantifier/` returns no matches.
- After Phase 1 completes: `grep -rn "allow(clippy::type_complexity)" src/` returns exactly one match (on `Storage::world_bundle_for`).
- Final: `python build.py` green from `chronicler_engine/`.

## Assumptions

- The prior architectural decision (`storage-require-helpers.md`: "do not recreate fetch_world_bundle") was about a bundle that **threaded across layers**. This plan's `Storage::world_bundle_for` returns a tuple that is **destructured at every call site within ~5 lines** — it never becomes a parameter that crosses modules. Coupling profile unchanged vs. today's duplicated code; only duplication goes away.
- `PersonaCard` returned as `Arc<PersonaCard>` to match both current call sites (both `pipeline.rs` and `gate.rs` currently `Arc::new()` the result of `require_persona`). Consumers (`pipeline.rs:69` `Arc::clone(&persona)`, `retry.rs:139` `&persona` typed as `&Arc<PersonaCard>`) require Arc form. Zero migration at call sites — destructuring code unchanged.
- `main_response: &str` stays a separate parameter (not pulled from `ctx.main_response` via `Option::unwrap`) because `agent.rs::execute` already resolves the `Option<&str>` before calling. Tests pass a literal `&str`. The `main_response` option resolution in `execute` is a different concern from `current_room` and stays there.
- `determine_npcs_in_room` returns `Result<QuantifierResult, EngineError>` and propagates the `ctx.current_room` absence (`EngineError::RoomNotFound`) via `ok_or_else(...)?`. No `.expect` needed, no new `#[allow(clippy::expect_used)]` added. `agent.rs::execute`'s duplicate `ok_or_else` for `current_room` is deleted — single source of truth inside the function.
- `state.scene.npcs_in_area: Vec<NpcCard>` is `pub` on `SceneState` (`scene_state.rs:9`) and already mutated directly in tests (`state_tests.rs:193, 228`). Test seeding requires no new accessor.
- `determine_npcs_with_room` test helper gets simpler, not more complex — it currently forwards many args; new shape builds `AgentContext` + state, calls fn. Helper param rename (`npc_pool_ids` vs `scene_npcs`) keeps the semantic split visible to future readers.
- `fetch_world_data_for_fresh_state` likely inlines into its sole caller `build_fresh_initial_state` during Phase 1 Task 1.3; in-task decision left to subagent based on grep verification of sole-caller assumption.
- The 4 `#[allow]` sites this plan addresses (`type_complexity` x2, `too_many_arguments` x1 in `orchestration.rs`; the `too_many_arguments` in `init_game.rs::spawn_arrival_task_if_needed` is NOT addressed — separate debt site) are the scope. The remaining `#[allow]`s in test-support files and the HTTP layer (warranted per the audit) are out of scope.
- `spawn_arrival_task_if_needed` (4th debt site from the audit) is out of scope. Its fix (`&StateResources` instead of unpacked args) is a separate ~3 SP task to be planned later if desired.
