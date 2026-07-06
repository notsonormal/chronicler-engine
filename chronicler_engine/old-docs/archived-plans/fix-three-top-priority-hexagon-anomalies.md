# Plan: Fix Three Top-Priority Hexagon Anomalies

**Date:** 2026-07-05
**Status:** ✅ All 3 anomalies completed (verified 2026-07-06)
**Scope:** `chronicler_engine/`
**Source:** Findings from `chronicler_engine/tmp/holistic-hexagon-investigation-second-opinion.md`

## Objective

Address the three top-priority anomalies surfaced by the second-opinion audit. These are concrete, low-risk, mechanical fixes — not architectural rework. Each ≤5 SP, each independently shippable.

## Scope (NOT in scope)

Out of scope for this plan:
- Splitting `ApplicationError` from `IntoResponse` (C8) — separate plan
- Collapsing `is_generating` + `GenerationStatus` (B5) — separate plan; risky design tradeoff
- Merging `arrival_service` + `phase_narrate` (C1) — separate plan
- Collapsing the two `build_initial_state` paths (D7) — separate plan
- Storage exemption cleanup (5 → 3 files; ADR staleness) — separate plan; ADR-027 needs revision first
- Documentation regeneration (chronicler-after-plan-workflow) — runs after merge

In scope:
1. Quantifier agent bypasses `LlmCallRecorder` (forensics lost)
2. Duplicate `GenerationGuard` struct definition
3. `run.rs` 295-LOC single-function monolith

---

## Already exists — reuse, don't write

| Need | Existing code to reuse |
|---|---|
| RAII flag-clear semantics | `GenerationGuard` impl at `adapters/driving/http/fragments/generation_guard.rs:10` |
| `LlmCallRecorder::complete` API | `application/llm_recorder.rs:18` — already does provider call + sanitize + forensic save |
| `Arc<dyn LlmProvider>` extraction | `LlmCallRecorder::provider()` accessor (`llm_recorder.rs:43`) — keep for Mock-test path |
| `spawn_pipeline_task` pattern | `application/spawn.rs:9` — already passed `Arc<GameService>` not bare deps |
| Build/test runner | `python build.py` (fmt + clippy + tests + coverage) |

---

## Anomaly 1 — Quantifier Agent Bypasses Forensics ✅ COMPLETED

**Story points:** 3
**Status:** ✅ Completed (verified 2026-07-06) — `agent.rs:24,40` now hold `recorder: Arc<LlmCallRecorder>` instead of bare `Arc<dyn LlmProvider>`. Recorder used directly. No `recorder.provider().clone()` flatten.
**Findings:** `holistic-hexagon-investigation-second-opinion.md` §C2

### Problem

`application/agents/quantifier/agent.rs:37-46` accepts `Arc<LlmCallRecorder>` from registry wiring but immediately flattens it:
```rust
provider: recorder.provider().clone(),  // throws away the recorder
```
`execute()` at `agent.rs:99-105` calls `determine_npcs_in_room(..., self.provider.as_ref(), ...)`. This routes to `orchestration.rs:49 backend.complete(...)` — a bare `LlmProvider` call. Forensic save (the recorder's reason for existing) is silently skipped for every quantifier LLM call.

### Fix

Change `QuantifierAgent` to hold `Arc<LlmCallRecorder>` instead of `Arc<dyn LlmProvider>`. Route LLM calls through the recorder.

**Files touched (~3):**

1. `src/application/agents/quantifier/agent.rs`
   - Struct field: `provider: Arc<dyn LlmProvider>` → `recorder: Arc<LlmCallRecorder>`
   - `from_config_with_storage` constructor: store `recorder` directly (no `recorder.provider().clone()`)
   - `with_provider` (test-only `#[cfg(feature = "testing")]`): wrap bare provider in `LlmCallRecorder::new(provider, Arc::new(NoopForensics))` — `NoopForensics` already exists at `test_support/noop_forensics.rs`
   - `execute()`: pass `self.recorder.as_ref()` instead of `self.provider.as_ref()`

2. `src/application/agents/quantifier/orchestration.rs`
   - `determine_npcs_in_room` signature: `backend: &dyn LlmProvider` → `recorder: &LlmCallRecorder`
   - `quantify_room_with_llm_call` signature: same change
   - Body `backend.complete("quantifier", ...)` → `recorder.complete("quantifier", ...)` (same args; `LlmCallRecorder::complete` already produces `LlmCallResult` with `.text`)

3. `src/application/agents/quantifier/orchestration_tests.rs`
   - Update the test call site (`orchestration_tests.rs:179`) — instead of `&provider` pass `&recorder` (or built via `make_test_recorder(provider)` from `test_support/noop_forensics.rs`)

Note: `QuantifierAgent::from_config_with_storage` no longer needs `_config: &AgentConfig` parameter to extract the provider — but **leave the signature unchanged**; unrelated drift. Out of scope for this fix.

### Tests

- `noop_forensics.rs` already provides `make_test_recorder(provider)` — reuse it
- Add unit test in `agent.rs` (mirror existing constructor test) asserting that calling `execute()` triggers ONE forensic save through the recorder. Use a `SpyForensics` mock that counts `save_llm_message` calls — model pattern exists in `test_support/noop_forensics.rs:NoopForensics`
- Keep existing `orchestration_tests.rs` assertions passing

### Failure modes

None new. If `LlmCallRecorder::complete` fails on the forensic save (storage error) it returns `EngineError` already — `determine_npcs_in_room` already propagates errors via `QuantifierResult` → higher-level `EngineError` plumbing unaffected. Forensic failure now blocks quantifier (same behavior as every other LLM call site); previously silent. **This is the intended side effect of the fix.**

### Verification

```
rg "quantifier_recorder" src/ — should find zero `drop(...)` calls after fix
rg "self.provider.as_ref" src/application/agents/quantifier/ — should be zero
rg "make_test_recorder" src/ — should be used in quantifier tests
python build.py  # fmt + clippy + tests + coverage clean
```

---

## Anomaly 2 — Duplicate GenerationGuard Struct ✅ COMPLETED

**Story points:** 2
**Status:** ✅ Completed (verified 2026-07-06) — `rg "struct GenerationGuard" src/` returns exactly 1 hit at `application/generation_guard.rs:10`. Private dup at `application_service.rs:395` removed. Fragments layer re-exports via `pub use`.
**Findings:** `holistic-hexagon-investigation-second-opinion.md` §C3

### Problem

Two `struct GenerationGuard(pub Arc<AtomicBool>)` definitions, identical body:
- `adapters/driving/http/fragments/generation_guard.rs:10` (the proper one, pub)
- `application/application_service.rs:395` (private dup)

Duplication exists **because** `arch-lint.toml` denies `application → server` — `application_service.rs:207` cannot import from `adapters/driving/http`. The dup is a workaround, not an accident.

### Fix

Move `GenerationGuard` to `application/` layer. Both `application` and `adapters/driving/http` can then import from there.

**Files touched (~5):**

1. **New file** `src/application/generation_guard.rs`
   - Move struct + Drop impl from `fragments/generation_guard.rs:1-19` verbatim
   - Same doc header `[DOC: docs/system/dashboard.md]`

2. `src/application/mod.rs`
   - Add `pub mod generation_guard;`
   - Add `pub(crate) use generation_guard::GenerationGuard;` if pattern matches other exports

3. `src/application/application_service.rs`
   - Delete private `struct GenerationGuard(Arc<AtomicBool>)` + `impl Drop` at lines 395-401
   - Remove unused `use std::sync::atomic::AtomicBool` if no other use in file
   - Add `use crate::application::generation_guard::GenerationGuard;` if not already in scope via `pub(crate) use` re-export

4. `src/adapters/driving/http/fragments/generation_guard.rs`
   - Replace struct + Drop impl with: `pub use crate::application::generation_guard::GenerationGuard;` (re-export preserves the `adapters::driving::http::fragments::GenerationGuard` import path)
   - Keep the doc-comment header

5. `src/adapters/driving/http/fragments/generation_guard_tests.rs`
   - No code change (tests import via `use super::GenerationGuard;` which still resolves)

### arch-lint check

- `application/generation_guard.rs` imports only `std::sync::{Arc, atomic::{AtomicBool, Ordering}}` — zero cross-layer deps. `arch-lint` `application → server` rule unaffected. ✓
- `adapters/driving/http/fragments/generation_guard.rs` re-export of `application::GenerationGuard` — `server → application` dependency is allowed (only `application → server` is denied). ✓

### Tests

- `generation_guard_tests.rs` stays green (re-export means same symbol)
- `tests/infrastructure/invariant_contract.rs` imports `chronicler_engine::adapters::driving::http::fragments::GenerationGuard` — path still resolves ✓

### Failure modes

None. Pure module relocation.

### Verification

```
rg "struct GenerationGuard" src/  — should return exactly 1 result
python build.py
cargo nextest run --test architecture  # arch-lint still passes
```

---

## Anomaly 3 — `run.rs` 295-LOC Single-Function Monolith ✅ COMPLETED

**Story points:** 5
**Status:** ✅ Completed (verified 2026-07-06) — `run.rs` now exposes `pub fn run` + `fn prepare_data` (line 55) + `fn prepare_state` (line 128) + `fn start_server` (line 196). Decomposition landed.
**Findings:** `holistic-hexagon-investigation-second-opinion.md` §C17 (size corrected: 295 LOC, not 130)

### Problem

`src/bootstrap/run.rs` is one `pub fn run(args: Args)` spanning lines 13-188 (~175 LOC body). Mixes:
1. Early exits + data_dir + db_pool + preset seeding (~lines 15-35)
2. World + persona + NPC lookup (~lines 36-70)
3. State load + settings load (~lines 76-127)
4. Service wiring (preset_storage, game_service, text_check_service) (~lines 128-145)
5. Server start (~lines 146-160)
6. existing helpers: `find_latest_game_for_world`, `list_game_names_for_world`, `ensure_presets` (already extracted)

### Fix

Extract 3 fns following the data-prep/state-prep/server-start split suggested by the original audit (TL;DR item 7). Existing helpers stay. **Guarded by Anomaly 3a smoke tests.**

**Files touched: 1** (`src/bootstrap/run.rs`)

**New internal fns (all `pub(crate)` not required — use `fn` private unless already called elsewhere):**

1. `fn prepare_data(args: &Args, data_dir: &Path) -> Result<LoadedData, EngineError>`
   - DB pool creation + db_path resolution
   - Game data seeding call (`load::seed_game_data`)
   - World + map + player + NPC lookup via `lookup_storage`
   - Returns `LoadedData { db_pool, storage, world_arc, map_arc, player_arc, npcs_arc, world_id, world_card, map, player }` (struct in `run.rs`)

2. `fn prepare_state(args: &Args, data: &LoadedData, runtime: &Runtime) -> Result<StateResources, EngineError>`
   - `init_game::load_game_state`
   - `npcs_map`/`nearby_npcs`/`all_npcs` extraction
   - Settings load + `Arc<RwLock<>>` wrap
   - `spawn_arrival_task_if_needed` call
   - Build `preset_storage`, `game_service`, `text_check_service`
   - Returns `StateResources { storage, preset_storage_arc, settings, game_service, text_check_service }`

3. `fn start_server(resources: ServerResources, config: ServerConfig, runtime: &Runtime) -> Result<(), EngineError>`
   - `run_server_with_config` call
   - `runtime.block_on(server)`

`pub fn run(args: Args)` becomes ~30 LOC: parse args / early exits / call `prepare_data` / `prepare_state` / `start_server`. Settings import path (`--settings-file`) stays in `prepare_state`.

### Local structs (in `run.rs`)

```rust
struct LoadedData {
    db_pool: DbPool,
    storage: Arc<Storage>,
    storage_arc_for_wiring: Arc<Storage>,
    world_arc: Arc<WorldCard>,
    map_arc: Arc<MapDef>,
    player_arc: Arc<PlayerCard>,
    npcs_arc: Arc<HashMap<String, NpcCard>>,
    room_id: String,
    nearby_npcs: Vec<NpcCard>,
    all_npcs: Vec<NpcCard>,
}

struct StateResources {
    storage: Arc<Storage>,
    preset_storage_arc: Arc<Storage>,
    settings: Arc<RwLock<AppSettings>>,
    game_service: Arc<GameService>,
    text_check_service: Arc<TextCheckService>,
}
```

Existing `ServerResources` (built in `start_server` step currently) is constructed inline where `run_server_with_config` is invoked.

### Constraints (do not break)

- `--list-worlds` early exit at the top of `run()` must stay before `prepare_data` (avoids `DbPool::new` when listing worlds)
- Error variants stay `EngineError::Config` — no error-shape drift
- Existing helpers `find_latest_game_for_world`, `list_game_names_for_world`, `ensure_presets` stay in `run.rs`; do not move

### Tests

No new tests added in this task. Refactor is guarded by Anomaly 3a's smoke tests + existing integration suite (especially `tests/integration/flow/retry_main.rs`, `tests/infrastructure/invariant_contract.rs`, and `tests/test_utils/server.rs:117` which covers `--settings-file`).

If extraction changes break startup on (a)/(c)/(d), the 3a smoke tests fail. If they pass, refactor is structurally safe.

### Failure modes

No new failure paths — all error variants and propagation unchanged. Each new fn returns `Result<_, EngineError>`. The early `list_available_worlds()` path was already early-returning.

### Verification

```
wc -l src/bootstrap/run.rs  # should be ~300-330 (no size drop; fns + structs add boilerplate)
grep -n "^fn \|^pub fn " src/bootstrap/run.rs # should show run + prepare_data + prepare_state + start_server + 3 existing helpers
python build.py
```

Note: this is NOT a LOC reduction. It is a semantic decomposition — `run.rs` will likely grow slightly due to struct definitions. The win is grep-ability: each step has a name and an entry in the file's function list.

---

## Execution Order

Independent. Can land in any order, but recommended:

1. **Anomaly 2 first (2 SP)** — smallest, fastest, zero risk. Confirms `arch-lint` still passes after a layer move.
2. **Anomaly 1 second (3 SP)** — touches the agent system. Verify with unit test.
3. **Anomaly 3 last (5 SP)** — largest surface. Verify with integration tests.

Total: **10 SP across 3 atomic tasks.** Each shippable independently. No task >5 SP — no subtasks required per AGENTS.md.

---

## Failure modes summary

| Anomaly | New failure paths? | Mitigation |
|---|---|---|
| 1 (quantifier forensics) | Yes — quantifier LLM now fails if forensic save fails | Already the contract for `LlmCallRecorder::complete`; matches every other LLM call site. Intended behavior. |
| 2 (GenerationGuard move) | None | Pure relocation |
| 3 (run.rs refactor) | None | Behavior-preserving extraction |

## Unresolved decisions

1. **Anomaly 1 — test mock shape:** Use `NoopForensics` for the existing test path (zero assertions on save count), or add a new `SpyForensics` that counts saves? **Recommended:** add `SpyForensics` in `test_support/noop_forensics.rs` (alongside `NoopForensics`) — single md5-hash-counter implementation, ~20 LOC. Without it, the regression has no test guard.
2. **Anomaly 3 — struct naming:** `LoadedData` / `StateResources` are placeholder names. If there are existing conventions for bootstrap-time data carriers (didn't find any in `bootstrap/mod.rs`), rename to match.
3. **Anomaly 3 — keep `ServerConfig` arg in `run()` or push into `prepare_data`?** Currently built from `args.port`. Leaving outside `prepare_data` keeps `prepare_data` focused on DB/world/player load. **Recommended:** build `ServerConfig` in `run()` before calling `start_server`.

## Reversibility

- Anomaly 1: reversible — wrapping back to `provider.clone()` is a one-line change
- Anomaly 2: reversible — re-duplicating the struct is trivial
- Anomaly 3: reversible — reorder `run()` body and inline the new fns

None of the three changes touch the database schema, public API, or ADR-pinned architecture decisions.

## What already exists (do not reimplement)

- `NoopForensics` and `make_test_recorder` at `test_support/noop_forensics.rs` — use for Anomaly 1
- `LlmCallRecorder` API at `application/llm_recorder.rs` — already has the right shape; just call `.complete()`
- `GenerationGuard` impl at `fragments/generation_guard.rs` — the single source of truth post-move
- Existing helpers in `run.rs` (`find_latest_game_for_world`, `list_game_names_for_world`, `ensure_presets`) — leave in place during Anomaly 3
