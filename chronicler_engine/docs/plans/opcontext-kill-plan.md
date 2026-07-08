# Title

OpContext + WorldSnapshot Removal + Tier 1+2 Cleanup (v3)

## Summary

Kill OpContext grab-bag AND WorldSnapshot bundle entirely. Move 5 service singletons (storage, preset_storage, settings, cancel_token, is_generating) onto `DefaultApplicationService`. Per-op world data (world+map+player+npcs) loads INSIDE ApplicationService methods — no external per-op bundle passed around. Pipeline/arrival_service read `state.world` etc. directly from `&mut GameState` (already holds these 4 `pub` fields). ArrivalTaskContext takes `Arc<DefaultApplicationService>` + loads state internally via `app.load_or_fresh()` — no bundle.

3 scout investigations verified feasibility. Plan refined through 19 issue reviews (improve-ai-plan skill).

Total: ~21 SP across 11 tasks in 2 phases. All sequential per AGENTS hard constraint. Each task ends with build green (per Issue 1 option A). No ADR (per Issue 8 — refactor, not significant decision).

## Key Changes

**Phase 1 — OpContext + WorldSnapshot kill (~16 SP across 9 tasks):**
- `DefaultApplicationService` grows from bare `Arc<GameService>` wrapper to owning 6 fields (5 singletons + game_service). Add `#[derive(Clone)]`. New fields `pub(crate)` matching existing OpContext visibility pattern.
- 7 free fns in `context.rs` added as `DefaultApplicationService` methods (additive first; old free fns stay alive during migration). Each fetches world+map+player+npcs from storage using `current_game_id`.
- 5 OpContext helper methods added as ApplicationService methods.
- 13 free fns across `query_handlers.rs` (9) + `message_editing.rs` (5) become ApplicationService methods — migrated per-method-group (Issue 9 option B).
- 14 ApplicationService methods signature change: drop `ctx: OpContext`, take operation-specific args.
- `spawn_pipeline_task(app: Arc<DefaultApplicationService>, f)` — closures take `&ApplicationService`.
- `PipelineRun<'a>` struct — replace `ctx: &'a OpContext` with `app: &'a DefaultApplicationService`.
- `ArrivalTaskContext` holds `Arc<DefaultApplicationService>` (NOT `&'a` — Issue 3 fix for spawn_blocking 'static). `run(self)` calls `self.app.load_or_fresh()`.
- `execute_action_impl(app: &DefaultApplicationService, ...)` + `retry_last_response_impl` + `retrigger_event_impl` updated.
- `process_action` spawn site: `Arc::new(self.clone())` (Issue 4).
- HTTP handlers: drop OpContext extractor. Game-management handlers pass `world_key`/`persona_key` explicit params.
- HTTP extractor `FromRequestParts<AppState> for OpContext` deleted.
- `async-trait` dependency may be unused post-kill (cleaned in A7 if no other user).

**Phase 2 — Tier 1+2 cleanup (~5 SP across 3 tasks):**
- B1: `process_action` split (3 SP).
- B2: `arrival_service.run` dedup (1 SP).
- B4: `is_generating_invariant_tests.rs` property test fail-fast (1 SP). (B3 folded into A6 per Issue 10.)

## Implementation

### Phase 1: OpContext + WorldSnapshot kill

- [x] #### Task A0: Scout investigations (DONE)

- [x] #### Task A1: Add singletons to DefaultApplicationService (1 SP)
  - [ ] ##### SubTask A1.1: Edit `src/application/application_service.rs` — add 5 `pub(crate)` fields to `DefaultApplicationService`: `storage: Arc<Storage>`, `preset_storage: Arc<Storage>`, `settings: Arc<RwLock<AppSettings>>`, `cancel_token: CancellationToken`, `is_generating: Arc<AtomicBool>`. Keep `game_service: Arc<GameService>`. Add `#[derive(Clone)]`. Update constructor with 6 params.
  - [ ] ##### SubTask A1.2: Update `src/adapters/driving/http/wiring.rs` — constructor: `DefaultApplicationService::new(game_service, storage, preset_storage, settings, cancel_token, is_generating)`. AppState keeps its own `Arc` clones of same 5 singletons (shared Arc source per H2 reflection 7b35d73de97a).
  - [ ] ##### Validate: `cargo check` clean. AppState accessors still compile.

- [x] #### Task A2: Add new methods on `impl DefaultApplicationService` (3 SP, additive — old OpContext-based free fns stay alive)
  - [ ] ##### SubTask A2.1: Add 7 methods mirroring context.rs free fns: `load_or_fresh`, `load_expecting_valid_state`, `save_state`, `save_message_and_snapshot`, `delete_and_remove_message`, `load_messages_with_swipes`, `load_messages_into_state`, `build_fresh_initial_state`. Internally use `self.storage`, `self.current_game_id()`.
  - [ ] ##### SubTask A2.2: Add 5 methods mirroring OpContext helper methods: `load_messages`, `update_message_text`, `active_quantifier_prompt`, `find_retry_anchor<'a>`, `set_game_id`.
  - [ ] ##### SubTask A2.3: Move `map_llm_error` helper from `src/application/context.rs` to `application_service.rs`. Leave context.rs OpContext + WorldSnapshot + old free fns alive (deletion in A7).
  - [ ] ##### Validate: `cargo check` clean. New methods unused (suppress via `#[allow(dead_code)]` until A3-A6 wire up).

- [ ] #### Task A3: Migrate callers per-method-group (5 SP)

  **Dependency chain: A3a → A3b → A3c → A3d → A4 → A5 → A6 → A7 → A8**

  - [x] ##### SubTask A3a: Game-management methods + HTTP handler callers (1.5 SP). Migrate 11 ApplicationService methods (`list_worlds`, `create_world`, `update_world`, `delete_world`, `get_world`, `list_games`, `current_game_id`, `switch_game`, `delete_game`, `create_game`, `reset`) to drop `ctx: OpContext`, take operation-specific args. Fetch world+player data internally via storage. Update HTTP handler callers in `games_fragment/handlers.rs` + `worlds_fragment/handlers.rs` to pass explicit params. Route `games_fragment/handlers.rs:56` direct `storage.list_personas()` call through ApplicationService method. Build green.
  - [x] ##### SubTask A3b: Pipeline-touching methods + spawn signature (1.5 SP). Migrate `process_action`, `continue_narration`, `persist_initial_state_with_swipes` signatures. Update `spawn_pipeline_task` signature per Scout C Q2. Update `execute_action_impl`, `retry_last_response_impl`, `retrigger_event_impl` signatures. Update `process_action` spawn site per Scout C Q3. Update `PipelineRun<'a>` struct — replace `ctx: &'a OpContext` with `app: &'a DefaultApplicationService`. Update `ActionPipeline::run_from_input`. Build green.
  - [x] ##### SubTask A3c: message_editing free fns → methods (1 SP). Move `switch_swipe`, `edit_history`, `delete_last`, `retry`, `retrigger` into `impl DefaultApplicationService`. `retry`/`retrigger` signatures become `(&self) -> Result<(), ApplicationError>` using `Arc::new(self.clone())` for spawn closure. Update all callers. Build green.
  - [x] ##### SubTask A3d: query_handlers free fns → methods (1 SP). Move 9 fns into `impl DefaultApplicationService`. Fns reading `ctx.world_snapshot.world.X` pivot to `let state = self.load_or_fresh(); state.world.X`. Update all callers. Build green.
  - [x] ##### Validate: `cargo check` clean after each sub-task. `grep -rn "ctx: OpContext\|ctx: &OpContext\|ctx.world_snapshot" src/application/` returns 0 after A3d.

- [x] #### Task A4: ArrivalTaskContext restructure (1.5 SP) — implemented via Option A (Phase D builds app, Phase E reuses)
  - [x] ##### SubTask A4.1: Update `ArrivalTaskContext` per Scout C Q4 option (c) + Issue 3 fix — drop `ctx: OpContext` field, add `app: Arc<DefaultApplicationService>`. `run(self)` calls `self.app.load_or_fresh()` or `self.app.load_expecting_valid_state()` for state. Reads `state.world`, `state.map`, `state.player`, `state.npcs`. Update `init_game.rs:187` caller to construct ArrivalTaskContext with `Arc::clone(&app)` instead of OpContext. (Implemented via Option A: Phase D `prepare_state` builds `Arc<DefaultApplicationService>`, passes `&app` to spawn_arrival_task_if_needed; Phase E `start_server` reuses `state.app.game_service()`.)
  - [ ] ##### Validate: `cargo check` clean. `grep -rn "OpContext" src/application/arrival_service.rs src/bootstrap/init_game.rs` returns 0.

- [ ] #### Task A5: HTTP handlers + extractor + handler test call sites (2.5 SP)
  - [ ] ##### SubTask A5.1: Delete `impl FromRequestParts<AppState> for OpContext` from `src/adapters/driving/http/op_context_loader.rs`. Delete file entirely.
  - [ ] ##### SubTask A5.2: Update 22 `load_op_context_for_active_game` call sites in 8 `src/adapters/driving/http/*_tests.rs` files: `debug_tests.rs` (2), `games_fragment/handlers_tests.rs` (2), `fragments/actions_tests.rs` (9), `fragments/history_tests.rs` (2), `fragments/endpoints_tests.rs` (4), `fragments/misc/retrigger_tests.rs` (1), `fragments/misc/swipe_tests.rs` (1), `fragments/misc/retry_tests.rs` (1). Each `let ctx = ...; handler(ctx, ...)` line → handler takes `State(state)` directly. Where tests need state, call `state.application_service.load_or_fresh().expect(...)`.
  - [ ] ##### Validate: `cargo check --all-targets` clean. `grep -rn "OpContext\|WorldSnapshot\|FromRequestParts.*OpContext\|load_op_context" src/adapters/driving/http/` returns 0.

- [ ] #### Task A6: Test fixtures + pipeline_helpers + is_generating tests (3 SP)
  - [ ] ##### SubTask A6.1: Replace 4 helper builders of `OpContext` with `make_test_app` family returning `Arc<DefaultApplicationService>`: `tests/helpers/fixtures.rs:348-358 make_test_ctx`, `src/test_support/context.rs:11-23 make_test_context`, `src/test_support/context.rs:26-38 make_test_context_without_snapshot`, `src/test_support/context.rs:70-87 make_test_context_with_sqlite`. Includes B3 fixture (folded per Issue 10): `make_test_app_with_default_preset(storage, world, player) -> Arc<DefaultApplicationService>` — extends `make_test_app` by seeding "system_default" PromptPreset into preset_storage.
  - [ ] ##### SubTask A6.2: Replace 5 inline OpContext literals: `tests/integration/application/action_pipeline/retry.rs:172-195, 264-287, 353-376, 443-466` (4 sites) + `tests/integration/flow/retry_main.rs:440-458` (1 site). Each → single `make_test_app_with_default_preset` call.
  - [ ] ##### SubTask A6.3: Restructure 3 pipeline_helpers sites: `tests/helpers/pipeline_helpers.rs:124-130 wait_for_generation_complete`, `tests/helpers/pipeline_helpers.rs:142-148 latest_state`, `tests/integration/flow/retry_main.rs:469-474` inline. Take `app: &DefaultApplicationService` + use `app.load_or_fresh()` or `app.storage.load_latest_snapshot()` + fetch world data via `app.storage.load_world(app.storage.current_game_id()?)` inside helper.
  - [ ] ##### SubTask A6.4: Rewrite `is_generating_invariant_tests.rs`: helpers `cached_flag`, `persisted_flag`, `invariant_holds`, `wait_until_idle` switch from `ctx: &OpContext` to `app: &DefaultApplicationService`. `cached_flag(app)` reads `app.is_generating.load(...)`. `persisted_flag(app)` loads state via `app.load_or_fresh()`, reads status field. 3 existing tests adapt.
  - [ ] ##### Validate: `cargo check --all-targets` clean. `grep -rn "WorldSnapshot\|OpContext" src/ tests/` returns 0.

- [ ] #### Task A7: Delete OpContext + WorldSnapshot + final cleanup (1 SP) [POINT OF NO RETURN — Issue 15]
  - [ ] ##### SubTask A7.1: Delete `OpContext` + `WorldSnapshot` struct definitions from `src/application/context.rs`. Delete remaining old free fns in context.rs (replaced by ApplicationService methods in A2). Delete file entirely if `map_llm_error` already moved (per A2.3).
  - [ ] ##### SubTask A7.2: Clean unused imports across `src/` + `tests/`. Drop `async-trait` from Cargo.toml if no other user. Run `cargo clippy --all-targets --all-features -- -D warnings` clean.
  - [ ] ##### Validate: `cargo check` + `cargo clippy --all-targets --all-features -- -D warnings` clean. **A7 is point of no return — defer if post-A6 review surfaces design issue.**

- [ ] #### Task A8: Final build.py (0.5 SP)
  - [ ] ##### SubTask A8.1: Run `python build.py` from `cd chronicler_engine`. All guardrail steps green (fmt, clippy, structure, docstring, storage-direct, tests). Run `python3 scripts/validate_adrs.py` (expect 24 ADRs unchanged — no ADR-031 per Issue 8).
  - [ ] ##### Validate: `python build.py` green. ADR validator passes.

### Phase 2: Deferred Tier 1+2 cleanup

- [ ] #### Task B1: process_action split (3 SP)
  - [ ] ##### SubTask B1.1: Extract `Self::heal_stale_generating(&self, state: &mut GameState)` — encapsulates stale-Generating check + reset + warn log. (1 SP)
  - [ ] ##### SubTask B1.2: Extract `Self::claim_generation_slot(&self, state: &mut GameState, player_name: &str, input: &str) -> Result<ProcessActionResult, EngineError>`. Three outcomes: (a) CAS won + save ok → `Ok(Started)`, AtomicBool=true, persisted=Generating; (b) CAS lost → `Ok(ConcurrentGeneration)`, no rollback; (c) CAS won + save failed → `Err(EngineError)`, AtomicBool=true, caller MUST call `release_generation_slot` then propagate. Helper does NOT call release itself. (1 SP)
  - [ ] ##### SubTask B1.3: Extract `Self::release_generation_slot(&self)` — `self.is_generating.store(false, Ordering::SeqCst)`. Rewrite process_action body linearly: load_state → heal_stale → claim_generation_slot (match outcome) → cancel check → spawn pipeline. (1 SP)
  - [ ] ##### Validate: `python build.py` passes; process_action body ≤30 lines.

- [ ] #### Task B2: arrival_service::run dedup (1 SP)
  - [ ] ##### SubTask B2.1: Replace state-construction lines (60-110 of `arrival_service.rs::run`) with `let mut state = self.app.load_expecting_valid_state()?;`. Preserve scenario-logs injection on fresh-state branch (`was_fresh` flag return, or `state.narrative.history.is_empty()` check).
  - [ ] ##### Validate: `python build.py` passes; arrival_service tests pass unchanged.

- [ ] #### Task B4: Property test fail-fast (1 SP)
  - [ ] ##### SubTask B4.1: Rewrite `wait_until_idle` in `src/application/is_generating_invariant_tests.rs` to fail-fast on divergence. Each poll: check `invariant_holds(app)`. Allowed transient: `(cached=true, persisted=Idle)`. Forbidden at any poll: `(cached=false, persisted=Generating)`. Fail immediately. Returns true when both reach `(false, false)`. Add new test: inject `(cached=false, persisted=Generating)` during flight by manually `app.is_generating.store(false)` while generation in progress; assert `wait_until_idle` returns false within 1-2 poll cycles.
  - [ ] ##### Validate: `python build.py` passes; 3 existing tests + 1 new test pass.

### Story point rules

- Sizes: 1, 3, 5, 8, 13
- 8 SP or larger → must break into subtasks
- 5 SP = single worker session; primary agent must verify output
- SubTasks optional for atomic tasks ≤5 SP; required for tasks >5 SP
- SP mandatory on every Task line

## Test Plan

- A1: `cargo check` clean after struct change
- A2: `cargo check` clean. New methods unused (suppressed via `#[allow(dead_code)]`)
- A3a-d: `cargo check` clean after each sub-task. After A3d: 0 grep hits for `ctx: OpContext|ctx: &OpContext|ctx.world_snapshot` in `src/application/`
- A4: `cargo check` clean. 0 grep hits for `OpContext` in arrival_service.rs + init_game.rs
- A5: `cargo check --all-targets` clean. 0 grep hits for `OpContext|WorldSnapshot|FromRequestParts.*OpContext|load_op_context` in `src/adapters/driving/http/`
- A6: `cargo check --all-targets` clean. 0 grep hits for `WorldSnapshot|OpContext` in src/ + tests/
- A7: `cargo check` + `cargo clippy --all-targets --all-features -- -D warnings` clean. OpContext + WorldSnapshot deleted. Point of no return.
- A8: `python build.py` green. All 6 guardrail steps pass. ADR validator 24 ADRs.
- B1: process_action body ≤30 lines; 3 helpers present
- B2: arrival_service tests pass unchanged
- B4: 3 existing + 1 new is_generating invariant test pass

Final verification: `python build.py` from `cd chronicler_engine`. All guardrail steps green. Pre-existing `run_branches.rs` SQLite WAL flake acceptable — passes on retry under regular build.py, not caused by this work.

## Assumptions

- **GameState is the per-op data carrier**: holds 4 `pub` fields (`world: Arc<WorldCard>`, `map: Arc<MapDef>`, `player: Arc<PlayerCard>`, `npcs: HashMap<String, NpcCard>` — NOT Arc-wrapped, not blocker since HashMap clone is cheap). Fns pivot `ctx.world_snapshot.world.X` → `state.world.X`. Per-op loading pushed inside ApplicationService methods.
- **ArrivalTaskContext uses `Arc<DefaultApplicationService>`** (Issue 3 fix — NOT `&'a` borrow): satisfies `spawn_blocking` `'static` bound. `run(self)` calls `self.app.load_or_fresh()` internally. Caller (`init_game.rs:187`) passes `Arc::clone(&app)` instead of OpContext.
- **PipelineRun keeps `&'a DefaultApplicationService`** (Issue 3 — verified stays inside spawn_blocking closure, no cross-spawn storage).
- **`DefaultApplicationService::clone()` + `Arc::new(self.clone())` per spawn** (Issue 4 — matches existing `Arc::clone(game_service)` pattern at `spawn.rs:9-15`). Cheap (~50ns, 5 Arc clones + 1 CancellationToken clone per spawn).
- **No data duplication**: ApplicationService's 5 singleton fields share Arc source with AppState (H2 reflection 7b35d73de97a). Same DB, same RwLock, same AtomicBool.
- **New DefaultApplicationService fields `pub(crate)`** (Issue 2 — no accessors, matches existing OpContext visibility pattern at `context.rs:33-40`).
- **T2.2 work unwinds entirely**: 22 `load_op_context_for_active_game` call sites in 8 `src/*_tests.rs` files deleted (Issue 12 ground-truth grep). async-trait dep may be unused after kill — cleaned in A7 if no other user.
- **13 free fns become ApplicationService methods**: 9 in query_handlers.rs + 5 in message_editing.rs (per Scout A).
- **14 ApplicationService methods migrate** (per Scout A section 4 table).
- **Per user hard constraint**: all tasks strictly sequential with `python build.py` (or `cargo check`+`cargo clippy` for intermediate green per Issue 1 option A); no parallel code editing.
- **Per user preference**: no ADR-031 (Issue 8 — refactor, not significant architectural decision; ADRs are stable decision records, not prescriptive constraints).
- **Plan corrections from scouts**: 5 inline OpContext literals (not 4); 2 test blockers (is_generating_invariant helpers + pipeline_helpers latest_state/wait_for_generation_complete); 13 free fns move (not 11); 14 ApplicationService methods (not 13); WorldSnapshot dies entirely (no ArrivalTaskContext residual bundle).
- **Plan corrections from 19 issue reviews**:
  - Issue 1 option A: keep-build-green per step via additive A2 + per-method-group A3 migration.
  - Issue 2 option A: new fields `pub(crate)`, no accessors.
  - Issue 3 option A: ArrivalTaskContext holds `Arc<DefaultApplicationService>` (NOT `&'a`).
  - Issue 4 option A: `Arc::new(self.clone())` spawn pattern preserved.
  - Issue 8: ADR-031 dropped entirely (refactor, not decision).
  - Issue 9 option B: A3-A6 collapsed into per-method-group atomic migrations (A3a/b/c/d).
  - Issue 10 option A: B3 folded into A6.
  - Issue 12 option A: A5.2 cites 22 call sites in 8 `src/*_tests.rs` files (Scout B was wrong — searched tests/ only).
  - Issue 13 option A: explicit chain A3a→A3b→A3c→A3d→A4→A5→A6→A7→A8.
  - Issue 15 option A: A7 marked point-of-no-return, deferrable.
- **Out of scope**: further process_action decomposition beyond B1's 3 helpers; ADR-029/030 prescriptive-tone audit; HTTP handler API design changes; GameState struct refactor; `spawn_pipeline_task` JoinHandle return (pre-existing polling in is_generating_invariant_tests left as-is, renamed only); test_support/context.rs deletion (may survive as make_test_app fixture location); B3 standalone task (folded into A6).
- **Pre-existing flake** (`run_branches.rs` SQLite WAL race): not caused by this work, passes on retry under regular build.py.
- **Total SP**: Phase 1 ~16 SP (A1 1 + A2 3 + A3 5 + A4 1.5 + A5 2.5 + A6 3 + A7 1 + A8 0.5 = 17.5 SP). Phase 2 ~5 SP (B1 3 + B2 1 + B4 1 = 5 SP). Total ~21 SP.
