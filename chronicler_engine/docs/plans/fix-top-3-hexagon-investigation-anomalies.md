# Fix Top 3 Hexagon-Investigation Anomalies

## Summary

Address the three highest-priority anomalies surfaced in `chronicler_engine/tmp/holistic-hexagon-investigation-second-opinion.md`:

1. **Quantifier bypasses forensics** — `QuantifierAgent` strips `LlmCallRecorder` to bare `Arc<dyn LlmProvider>`, so every quantifier LLM call (`orchestration.rs:49`) skips `LlmCallRecorder::complete` and forensics save.
2. **Duplicate `GenerationGuard` struct** — two definitions exist: `adapters/driving/http/fragments/generation_guard.rs:10` (pub, used by tests + invariant contract) and a private one in `application/application_service.rs:395`. The duplicate exists *because* arch-lint denies `application → server` imports — a workaround, not careless duplication.
3. **`bootstrap/run.rs` is a 295-LOC single `fn run`** — does arg-parsing, DB setup, world/player/setting resolution, game-state load, arrival spawn, service wiring, and server start all in one function.

Scope: `chronicler_engine/`. No hex-restructuring. Each fix is mechanical and ≤5 files.

## Key Changes

1. **Quantifier forensics:** thread `Arc<LlmCallRecorder>` through `QuantifierAgent` and change `determine_npcs_in_room` / `quantify_room_with_llm_call` to take `&LlmCallRecorder`. Call `recorder.complete(...)` instead of `backend.complete(...)` in `orchestration.rs:49`.
2. **GenerationGuard unification:** move `GenerationGuard` to `application/` (lowest layer both consumers can import). Delete the private duplicate in `application_service.rs:395`. `adapters/driving/http/fragments/generation_guard.rs` becomes a `pub use` re-export.
3. **`run.rs` split:** decompose `fn run` into 3 helpers — `prepare_data`, `prepare_state`, `start_server`. `run()` becomes a thin orchestrator.

## Implementation

### Phase 1: Quantifier Forensics Restoration (8 SP)

- [ ] #### Task 1.1: Change `QuantifierAgent` to own `Arc<LlmCallRecorder>` (5 SP)
  - [ ] ##### SubTask 1.1.1: Replace `provider: Arc<dyn LlmProvider>` field with `recorder: Arc<LlmCallRecorder>` in `application/agents/quantifier/agent.rs`; update `from_config_with_storage` (agent.rs:37) to store `recorder` directly (no `recorder.provider().clone()`); update test-only `with_provider` to wrap with `test_support/noop_forensics.rs::make_test_recorder`; update `execute` to pass `self.recorder.as_ref()` to `determine_npcs_in_room`. (3 SP)
  - [ ] ##### SubTask 1.1.2: Change `quantify_room_with_llm_call` (orchestration.rs:14) and `determine_npcs_in_room` (orchestration.rs:178) signatures from `backend: &dyn LlmProvider` to `recorder: &LlmCallRecorder`; replace `backend.complete("quantifier", ...)` at `orchestration.rs:49` with `recorder.complete("quantifier", ...)`; replace `backend.name()` / `backend.model()` log calls with `recorder.provider().name()` / `recorder.provider().model()`; update `orchestration_tests.rs:179` to pass recorder. (2 SP)

- [ ] #### Task 1.2: Add integration test asserting quantifier writes to `llm_messages` (3 SP)
  - New file `tests/llm/quantifier_forensics.rs`: configure `QuantifierAgent` with `MockBackend` wrapped in `LlmCallRecorder` over a real-DB `LlmMessageRepository`. Run one quantifier cycle. Assert a row exists in `llm_messages` table (must fail before fix, pass after).

### Phase 2: GenerationGuard Unification (3 SP)

- [ ] #### Task 2.1: Move `GenerationGuard` struct to `application/` layer (3 SP)
  - Create `src/application/generation_guard.rs` with `pub struct GenerationGuard(pub Arc<AtomicBool>)` + Drop impl (moved verbatim from `fragments/generation_guard.rs`); add `//! [DOC: docs/system/dashboard.md]` header. Re-export via `src/application/mod.rs` (`pub use generation_guard::GenerationGuard;`). Delete the private duplicate (`struct GenerationGuard` + `impl Drop`) at `application/application_service.rs:395-403`; the existing `let _guard = GenerationGuard(...)` at `:207` resolves to the new public import automatically. Replace `src/adapters/driving/http/fragments/generation_guard.rs` struct body with `pub use crate::application::GenerationGuard;` — keeps `fragments/mod.rs:19` re-export and all existing test imports (`fragments/generation_guard_tests.rs`, `tests/infrastructure/invariant_contract.rs`) working unchanged.

### Phase 3: `run.rs` Decomposition (5 SP)

- [ ] #### Task 3.1: Extract `prepare_data` helper from `run()` (3 SP)
  - In `bootstrap/run.rs`, define `struct PreparedData { db_pool, data_dir, world_card, map, player, npcs_map }` and `fn prepare_data(args: &Args) -> Result<PreparedData>` covering current `run.rs:14-100` (db setup, `ensure_presets` call, world lookup with fallback, persona lookup, npc load). Replace inline code in `run()` with `let prepared = prepare_data(&args)?;`. Existing free fns (`ensure_presets`, `find_latest_game_for_world`, `list_game_names_for_world`) stay where they are; `prepare_data` composes them. Verify `cargo build` clean.

- [ ] #### Task 3.2: Extract `prepare_state` + `start_server` helpers from `run()` (2 SP)
  - Define `struct PreparedState { storage_arc, settings_arc, game_service, text_check_service, config: ServerConfig }` and `fn prepare_state(prepared: &PreparedData, runtime: &tokio::runtime::Runtime) -> Result<PreparedState>` covering current `run.rs:101-160` (load game state, spawn arrival task, build preset_storage, wire `game_service` and `text_check_service`, build `ServerConfig`). Define `fn start_server(runtime: &Runtime, state: PreparedState) -> Result<()>` covering current `run.rs:160-173` (build `ServerResources`, `run_server_with_config`, `block_on(server)`). Replace inline code in `run()` with `let state = prepare_state(&prepared, &runtime)?;` then `start_server(&runtime, state)?;`. `fn run` body ≤30 LOC.

## Test Plan

### Phase 1
- New integration test `tests/llm/quantifier_forensics.rs` (Task 1.2): asserts quantifier call writes to `llm_messages` table. **Fails before fix, passes after.**
- Existing `orchestration_tests.rs` updated and passes.
- `python build.py` clean.

### Phase 2
- `tests/infrastructure/invariant_contract.rs` passes unchanged (proves RAII semantics preserved).
- `fragments/generation_guard_tests.rs` passes unchanged.
- `grep -rn "^struct GenerationGuard\|^pub struct GenerationGuard" src/` returns exactly ONE match.
- `cargo nextest run --test architecture` passes — deny rules intact, especially `application → server` still denied.
- `python build.py` clean.

### Phase 3
- `cargo nextest run` clean (pure refactor, no new tests).
- Manual smoke test: `cargo run -- --world <default> --persona <default> --port 8080`, hit `GET /` to verify server boots.
- `run.rs::fn run` body ≤30 LOC (down from ~159 lines).
- `python build.py` clean.

## Assumptions

- **A1:** `GenerationGuard` is purely a structural move inward. No arch-lint rule denies `application → application` or `server → application`, so both consumers resolve safely. Verified by reading `arch-lint.toml`: deny rules only block outward cross-layer imports (e.g. `application → server`, `model → …`); inward moves are always permitted.
- **A2:** `LlmCallRecorder::complete` returns `Result<LlmCallResult, EngineError>` — identical success/error shape to `LlmProvider::complete`, so error-handling in `orchestration.rs:49-` requires no logic change beyond the call site.
- **A3:** `run.rs`'s 3-helper split composes existing standalone free fns (`ensure_presets`, `load_game_state`) — no new logic introduced.
- **A4:** Tests in `tests/infrastructure/invariant_contract.rs` and `fragments/generation_guard_tests.rs` import via the `fragments::GenerationGuard` path — keeping the `pub use crate::application::GenerationGuard;` re-export at `fragments/generation_guard.rs` means zero test edits.
- **A5:** Phase order is intentional: Phase 1 first (highest user-facing impact — silent loss of forensics data on every quantifier call), then Phase 2 (mechanical), then Phase 3 (cosmetic). Phases are independent; can be merged separately.
- **A6:** Out of scope, deferred to existing abstraction-fixes super-plan: `ApplicationError::IntoResponse` move (overlaps with T1 error-model plan), `GameServiceContext` method hoisting (overlaps with T2 narration-deepening), `build_initial_state` consolidation (D7 finding), `is_generating` / `GenerationStatus` collapse (blocked on broader state-machine decision in T1 + reliability plan).
