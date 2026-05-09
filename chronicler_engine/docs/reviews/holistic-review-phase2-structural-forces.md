# Phase 2: Structural Forces — Findings

**Date:** 2026-05-09  
**Scope:** Cross-cutting architectural decisions and their interactions  
**Method:** Import graph analysis, state access pattern counts, lock duration tracing, error flow audit

---

## Executive Summary

The engine has **five load-bearing architectural decisions** that are mostly well-aligned but create two significant tensions:

1. **`Arc<Mutex<GameState>>` + async LLM calls** → Forces extensive data cloning and a split-state pattern (lock → clone → drop → LLM → re-lock → mutate). This works but is fragile.
2. **Layer boundaries + narrative/engine coupling** → `engine/` directly imports 9 types from `narrative/`, and `narrative/quantifier/core.rs` imports from `engine/` (upward violation). The boundary is porous.

**Severity:** The structural tensions are **moderate** — they don't cause bugs today, but they make the architecture resistant to change (especially multiplayer or non-LLM backends).

**One concrete bug surface:** `map_llm_error` discards `ParseError.raw_response`, making JSON parse failures undebuggable from the UI.

---

## 1. The Five Load-Bearing Decisions

### Decision 1: Centralized Mutable State

**What:** Single `Arc<std::sync::Mutex<GameState>>` held by `AppState`. All state mutations go through this lock.

**Enforced by:** Type system (`GameState` is not `Clone` for production; only test fixtures clone it).

**Access patterns:**

| Pattern | Count | Where |
|---------|-------|-------|
| `Arc<Mutex<GameState>>` | 15 | `GameService` trait, `AppState`, `GeneratingGuard` |
| `&mut GameState` | 13 | `action_processing`, `logic`, `bootstrap`, `fragments` |
| `&GameState` | 20 | `logic` helpers, `trigger_eval`, `server` renderers, diagnostics |
| `MutexGuard<GameState>` | 1 | `AppState::lock_state()` |

**Why it was chosen:** Simplicity. One lock, one state object, no distributed consistency problems.

**Escape hatches:** `with_state_lock` helper for brief re-locks during async work.

### Decision 2: Layer Boundaries

**What:** `model` → `engine` → `narrative` → `server` → `bootstrap`.

**Enforced by:** `arch-lint.toml` (`model/` cannot import `engine/`, `narrative/`, or `server/`).

**Actual dependency graph:**

```
error, model
   ↓
settings (root), test_support, cli
   ↓
narrative  ←── engine::logic::get_current_room (VIOLATION)
   ↓
engine
   ↓
server
   ↓
bootstrap
```

**Finding:** The graph is a DAG, but with one upward edge: `narrative/quantifier/core.rs:162` calls `crate::engine::logic::get_current_room(state)`. This is the only production-code layering violation.

### Decision 3: Sync Action Processing Inside Async HTTP

**What:** HTTP handlers are async (Axum). Game action processing is sync. LLM calls are sync-blocking. Bridge: `tokio::task::spawn_blocking`.

**Enforced by:** Convention + review. No `std::thread::spawn` allowed (guardrail).

**Pattern:**
```rust
// async HTTP handler
let mut guard = state.lock()?;      // brief lock
guard.add_log(...);
drop(guard);                         // drop before spawn

tokio::task::spawn_blocking(move || {
    // sync blocking thread
    game_service.execute_action(state_clone, cmd, name);
});
```

**Count:** 3 uses of `spawn_blocking` (bootstrap arrival, fragments action, fragments retry).

### Decision 4: LLM as Black-Box Dependency

**What:** All LLM interaction goes through traits (`LlmBackend`, `QuantifierBackendTrait`). Production uses OpenRouter/Ollama/DeepSeek. Tests use mocks.

**Enforced by:** Trait system + `MockBackend` / `MockQuantifierBackend`.

**Leakage:** Despite traits, 9 narrative types are hard-imported into `engine/`:

| Narrative Type | Used In Engine File | Purpose |
|----------------|---------------------|---------|
| `PromptBuilder` | `action_processing.rs` | Build trigger continuation prompt |
| `PromptContext` | `action_processing.rs` | Trigger prompt context |
| `make_prompt_context` | `game_service.rs` | Build narration prompt |
| `NpcEvent` | `action_processing.rs` | Apply NPC enter/leave events |
| `QuantifierResult` | `action_processing.rs` | Read quantifier output |
| `compute_npc_events` | `action_processing.rs` | Diff NPC presence |
| `QuantifierBackendTrait` | `game_service.rs` | Owned by `DefaultGameService` |
| `QuantifierConfidence` | `game_service.rs` | Check quantifier confidence |
| `determine_npcs_in_room` | `game_service.rs` | Call quantifier |

**Assessment:** The trait abstraction is real (you can swap backends), but the *orchestration* is deeply coupled. `DefaultGameService` owns `Arc<dyn LlmBackend>` and `Arc<dyn QuantifierBackendTrait>` as fields and directly calls `make_prompt_context`, `determine_npcs_in_room`, etc.

### Decision 5: No `.unwrap()` in Production Code

**What:** `clippy::unwrap_used` is denied. All error paths must use `Result`.

**Enforced by:** Clippy lint in `src/lib.rs` + arch-lint.

**Escape hatches:** `#[allow(clippy::expect_used)]` for infallible operations (static regex, hardcoded HTTP responses).

**Finding:** Clean. No `.unwrap()` in production code. However, `.ok()` is used extensively to swallow `Result`s (see Section 4).

---

## 2. Force Interactions: Reinforcements (+) and Tensions (−)

### Matrix

| | Centralized State | Layer Boundaries | spawn_blocking | LLM Traits | No unwrap |
|---|:---:|:---:|:---:|:---:|:---:|
| **Centralized State** | — | − | + | − | + |
| **Layer Boundaries** | − | — | + | − | + |
| **spawn_blocking** | + | + | — | + | + |
| **LLM Traits** | − | − | + | — | + |
| **No unwrap** | + | + | + | + | — |

### Key Tensions Explained

#### Tension A: Centralized State ↔ LLM Traits (−)

**Problem:** The LLM traits want to be abstract and swappable. But `DefaultGameService` must clone 8+ fields out of `GameState` before calling `backend.narrate_action()`, then re-lock to apply results. This makes the LLM backend feel like an *implementation detail* of state mutation, not an independent service.

**Concrete evidence:**
- `game_service.rs:155-167` clones `world`, `map`, `player`, `room_id`, `history`, `room_npc_ids`, `nearby_npcs`, `all_npcs` before dropping the lock.
- `FreeActionContext` (`action_processing.rs:19`) embeds `&'a dyn crate::narrative::llm::LlmBackend` — the engine's core mutation function carries a raw LLM backend reference.

**What this means:** You cannot easily replace the LLM backend without touching `game_service.rs` and `action_processing.rs`. The trait is abstract at the *call* site but not at the *orchestration* site.

#### Tension B: Layer Boundaries ↔ LLM Traits (−)

**Problem:** `engine/` is supposed to be above `narrative/` in the stack. But `engine/` imports 9 narrative types, and `narrative/quantifier/core.rs` imports `engine::logic::get_current_room` (upward).

**Concrete evidence:**
- `engine/action_processing.rs` lines 15-16 import `PromptBuilder`, `PromptContext`, `NpcEvent`, `QuantifierResult`, `compute_npc_events`.
- `narrative/quantifier/core.rs:162` calls `crate::engine::logic::get_current_room(state)` to resolve the current room for quantifier context.

**What this means:** The narrative tier is not a clean "below engine" layer. It is a peer that the engine embeds. This is acceptable for a single-player text adventure but would block clean extraction of the narrative system into a separate crate or service.

#### Tension C: Centralized State ↔ Layer Boundaries (−)

**Problem:** `GameState` lives in `model/`, but its lifecycle (lock → clone → drop → re-lock) is managed by `engine/` and `server/`. The model tier is supposed to be pure data, yet it contains `GeneratingGuard` (`model/state.rs:310`) — a synchronization primitive.

**Concrete evidence:**
- `GeneratingGuard` is defined in `model/state.rs` but is conceptually engine orchestration.
- `with_lock_or_recover` (poison recovery) is also in `model/state.rs`.

**What this means:** `model/` is not quite "pure data." It contains concurrency primitives that exist only because of the centralized state pattern.

#### Reinforcement A: spawn_blocking ↔ Centralized State (+)

**Why they work well together:** `spawn_blocking` lets the sync mutex be dropped for seconds while LLM I/O happens. The lock is only held for microseconds at a time. No deadlocks, no scheduler blocking.

**Evidence:** FreeAction timeline:
- Lock held: ~μs (clone data)
- Lock free: ~1-10s (LLM call)
- Lock held: ~μs (apply quantifier result)
- Lock free: ~1-10s (trigger continuation LLM call)
- Lock held: ~μs (commit results)

**Ratio:** ~20 seconds of LLM work with ~3× μs of lock time. Excellent.

#### Reinforcement B: No unwrap ↔ Centralized State (+)

**Why they work well together:** Poison recovery (`with_lock_or_recover`) returns `Result` and never panics. If a `spawn_blocking` task panics, the mutex is poisoned, but `GeneratingGuard::drop` recovers and resets status. The "no unwrap" rule forces explicit handling of this edge case.

---

## 3. Centralized State: Deep-Dive

### 3.1 Lock Duration by Action Type

| Action | Lock Duration | Work Done Under Lock |
|--------|--------------|----------------------|
| **Sync (Look, Inventory, Quit, Talk)** | Full function | Parse, log, read room, render — all sync, fast |
| **FreeAction — Phase 1** | ~μs | Parse, clone 8 fields, drop |
| **FreeAction — Phase 2 (LLM)** | **None** | Network I/O to LLM provider |
| **FreeAction — Phase 3 (Quantifier)** | ~μs | Call quantifier, apply result |
| **FreeAction — Phase 4 (Trigger LLM)** | **None** | Network I/O for trigger continuation |
| **FreeAction — Phase 5 (Commit)** | ~μs | Log trigger result, reset status |
| **Retry** | 2× brief locks | Read input, clone data; later: replace response |

### 3.2 State Mutation Sites

**Functions that mutate `GameState` (called under lock):**

| Function | File | Mutates |
|----------|------|---------|
| `handle_movement` | `engine/action_processing.rs` | `movement`, `character_state`, `narrative` |
| `execute_freeaction_impl` | `engine/action_processing.rs` | `narrative`, `scene`, `character_state` (via helpers) |
| `apply_npc_events` | `engine/action_processing.rs` | `character_state` |
| `commit_trigger_narration` | `engine/action_processing.rs` | `narrative`, `character_state` |
| `evaluate_and_narrate_triggers` | `engine/action_processing.rs` | `narrative`, `character_state` |
| `process_sync_action` | `server/fragments.rs` | `narrative` |
| `add_log` / `edit_log` / `delete_log` | `model/state.rs` | `narrative` |
| `replace_last_ai_response` | `model/state.rs` | `narrative` |
| `inject_scenario_logs` | `bootstrap.rs` | `narrative` |

**Observation:** Every mutation touches `narrative` (history or generation status). No mutation is *purely* movement or *purely* character state. This means `NarrativeState` is the true "hot" sub-struct.

### 3.3 Test Fixture Bypass

`test_support/fixtures.rs` has two constructors (`with_npc_raw`, `with_npc_in_named_room_raw`) that bypass `GameState::new()` entirely. They construct raw struct literals with `character_state: Default::default()`.

**Impact:** Tests using these fixtures skip the starting-room encounter tracking that `GameState::new` performs (`times_met = 1`, `currently_meeting = true` for NPCs in starting room). This is intentional for test control but means tests don't exercise the real initialization path.

---

## 4. Error Handling: Silent Losses

### 4.1 `map_llm_error` Destroys Structure

**Location:** `engine/game_service.rs:83-101`

**What it does:** Converts `EngineError` → `String` for display in the UI.

**What it loses:**

| Input | Kept | Lost |
|-------|------|------|
| `ParseError { raw_response, expected_format }` | `expected_format` | **`raw_response`** — the actual LLM response body |
| `DataLoad { path, source }` | Display string | `path` and `source` as separate fields |
| `ContextOverflow { requested, max }` | Display string | Numeric values for debugging |
| `Internal(InternalError)` | Display string | Invariant name / location |
| `RoomNotFound(id)` | Display string | The actual room ID |

**Impact:** When the LLM returns unparseable JSON, the user sees "unexpected response format (expected valid JSON)" but the raw response (the only evidence of what the model actually said) is **gone**. It is logged by `llm_client.rs` at the narrative layer, but not forwarded to the engine/service layer.

**Severity:** Moderate. Debugging LLM parse failures requires reading server logs, not just the UI or debug endpoint.

### 4.2 `.ok()` Swallow Pattern

**Production code uses `.ok()` to silently drop Results in 10 locations:**

| File | Line | What Is Swallowed |
|------|------|-------------------|
| `engine/action_processing.rs` | 93 | `assert_state_consistency` after `handle_movement` |
| `engine/action_processing.rs` | 110 | `assert_state_consistency` after `apply_npc_events` |
| `engine/action_processing.rs` | 137 | `assert_state_consistency` after `commit_trigger_narration` |
| `engine/action_processing.rs` | 283 | `assert_state_consistency` after `evaluate_and_narrate_triggers` |
| `engine/game_service.rs` | 60 | `with_state_lock` on poisoned mutex |
| `engine/game_service.rs` | 124 | `get_current_room` error in `Action::Look` |
| `engine/game_service.rs` | 291 | `assert_state_consistency` after trigger commit |
| `server/fragments.rs` | 192 | `state.lock()` poison → returns default status |
| `server/fragments.rs` | 215 | `state.lock()` poison → returns "failed" |

**Impact:** Low in production (these are edge cases). High in testing — diagnostic failures don't fail tests.

### 4.3 Dead Error Variant

`EngineError::NpcNotFound(String)` exists in `error.rs` but is **never constructed** anywhere in the codebase.

---

## 5. Concurrency: The Missing `GeneratingGuard`

`GeneratingGuard` is only used in `bootstrap.rs` (arrival narration). It is **not** used in:
- `server/fragments.rs` action handler
- `engine/game_service.rs` `execute_action`
- `engine/game_service.rs` `retry_last_response`

Instead, these paths manually set `status = Generating` / `Idle` via `set_phase`, `reset_generating`, and `set_error_and_reset`.

**Impact:** A panic in `execute_action` or `retry_last_response` will **not** auto-reset `generation.status` to `Idle`. The stale comment at `game_service.rs:111` ("Guard will still reset on drop") is misleading — there is no guard in that path.

**Severity:** Low. Panics are rare (no unwrap in production). But it's an inconsistency in the reliability model.

---

## 6. Recommendations

### Critical

1. **Fix `map_llm_error` to preserve `ParseError.raw_response`**
   - Include the first 500 chars of `raw_response` in the error string shown to users.
   - This is the single biggest debuggability gap in the error flow.

### Important

2. **Remove or use `EngineError::NpcNotFound`**
   - Either construct it somewhere or delete the variant.

3. **Unify poison handling**
   - `GeneratingGuard` recovers poison. `with_state_lock` silently skips on poison. Pick one strategy and apply it everywhere.

4. **Fix the `narrative → engine` upward dependency**
   - `narrative/quantifier/core.rs` calls `engine::logic::get_current_room`. Move the room lookup out of the quantifier, or pass `&Room` as a parameter.

### Suggestions

5. **Consider `GeneratingGuard` for action paths**
   - Replace manual `set_phase` / `reset_generating` with RAII guard for automatic cleanup.

6. **Document why `FreeActionContext` carries `&dyn LlmBackend`**
   - This is the tightest coupling point between engine and narrative. A comment explaining the trade-off would help future readers.

---

## 7. Appendix: Raw Metrics

| Metric | Value |
|--------|-------|
| Functions taking `Arc<Mutex<GameState>>` | 15 |
| Functions taking `&mut GameState` | 13 |
| Functions taking `&GameState` | 20 |
| `spawn_blocking` sites | 3 |
| `std::sync::Mutex` usages | All state locks |
| `tokio::sync::Mutex` usages | 0 |
| Narrative types imported by engine/ | 9 |
| Engine types imported by server/ | 5 |
| Layer violations (production) | 1 (`narrative/quantifier/core.rs` → `engine::logic`) |
| `EngineError` variants | 16 |
| Dead `EngineError` variants | 1 (`NpcNotFound`) |
| `.ok()` swallow sites (production) | 10 |
| `.unwrap()` sites (production) | 0 |
| `.expect()` sites (production) | 19 (all infallible: regex, static headers) |
| `GeneratingGuard` usage sites | 1 (`bootstrap.rs`) |
