# Settings I/O Centralization — Implementation Plan

**Date:** 2026-05-17
**Scope:** `chronicler_engine/src` — eliminate scattered `load_settings()` calls
**Goal:** Load settings once at bootstrap; pass `Arc<RwLock<AppSettings>>` down through the construction chain. No file I/O in business logic.

---

## Problem Statement

`crate::settings::load_settings()` performs synchronous file I/O from **8 locations** across all layers:

| # | Layer | File | Function | What it reads |
|---|-------|------|----------|---------------|
| 1 | `bootstrap` | `run.rs` | arrival task | narration connection |
| 2 | `server` | `mod.rs` | `run_server_with_config` | full settings → `AppState` |
| 3 | `application` | `game_service/service.rs` | `with_storage` | agents + narration connection |
| 4 | `narrative` | `agents/quantifier/agent.rs` | `from_config_with_storage` | quantifier connection |
| 5 | `narrative/llm` | `backend.rs` | `get_llm_backend` | narration connection |
| 6 | `narrative/llm` | `openrouter.rs` | `narrate_from_context` | `response_length` |
| 7 | `narrative/llm` | `ollama.rs` | `narrate_from_context` | `response_length` |
| 8 | `engine` | `action_processing.rs` | `build_trigger_prompt_parts` | narration connection + `response_length` |

This violates the bootstrap-once principle, makes tests fragile (must set `CHRONICLER_SETTINGS_PATH`), and creates hidden coupling between business logic and the filesystem.

---

## Target State

```
bootstrap/run.rs
    │ load_settings() — ONCE
    ▼
run_server_with_config(settings: Arc<RwLock<AppSettings>>)
    │
    ├──► AppState { settings: Arc::clone(&settings), ... }
    │         │
    │         └──► GameServiceContext { settings: Arc::clone(&settings), ... }
    │                   │
    │                   └──► DefaultGameService::with_storage(storage, &settings)
    │                             │
    │                             ├──► AgentRegistry::from_configs_with_storage(configs, storage, &settings)
    │                             │         └──► QuantifierAgent::from_config_with_storage(config, storage, &settings)
    │                             │
    │                             └──► get_llm_backend_for(connection, storage) — connection from settings
    │
    └──► arrival task uses settings from closure capture
```

After the refactor, `grep -r "load_settings()" src/` returns only:
- `src/settings.rs` — the definition
- `src/bootstrap/run.rs` — the single call site
- `src/settings_tests.rs` — tests of the load function itself

---

## Architecture Decision

**Pass `Arc<RwLock<AppSettings>>` down the construction chain.**

Rationale:
- `AppState` already holds `Arc<RwLock<AppSettings>>` for runtime mutability (settings UI)
- `AppSettings` is `Clone`, but `Arc<RwLock<…>>` preserves the single shared instance
- Backends need `response_length` which can change at runtime; storing the `Arc` lets them read fresh values
- Simpler than a new `SettingsProvider` trait — uses existing Rust patterns

### How Updated Settings Propagate

**Current behavior (pre-existing):**
- Settings handlers update `app_state.settings` in memory AND save to `settings.json`
- `DefaultGameService` is constructed **once** at startup — it never sees connection changes
- So changing the narrator/quantifier connection in the UI currently **requires a server restart** to take effect
- Only `response_length` and `max_context_tokens` were dynamic (because they were re-read from disk per-call via `load_settings()`)

**After this refactor:**
- `response_length` → dynamic: backends hold `Arc<RwLock<AppSettings>>` and read it per-call
- `max_context_tokens` for trigger prompts → dynamic: passed from settings through `FreeActionContext`
- Connection changes → **still require restart** unless we add a rebuild mechanism

### Two Approaches for Connection Changes

| Approach | Behavior | Mechanism | Trade-off |
|----------|----------|-----------|-----------|
| **A. Preserve restart requirement** | Connection changes need restart | None extra | Simple, matches today's behavior |
| **B. Rebuild backends on change** | Connection changes take effect immediately | `DefaultGameService` stores factory inputs (`settings` + `storage`), checks connection IDs before each action, rebuilds `llm_backend` and `agent_registry` if changed | Adds ~2 string comparisons + 1 lock read per action; transparent to callers |

**Recommendation: Approach B.** It eliminates the restart requirement with minimal overhead and makes the settings UI actually work as users expect. The implementation adds interior mutability (`RwLock`) around `llm_backend` and `agent_registry` inside `DefaultGameService`, but the trait interface (`GameService`) stays unchanged.

**What about `response_length`?**
Both `OpenRouterBackend` and `OllamaBackend` call `load_settings()` in `narrate_from_context` solely to read `response_length`. Instead of loading settings, they will store `Arc<RwLock<AppSettings>>` and read `.response_length` from it when building prompts.

**What about `build_trigger_prompt_parts`?**
This `engine/` function reads settings for the narration connection's `max_context_tokens` and `response_length`. It will receive `response_length: &str` as an explicit parameter. The caller (`application/`) reads the value from settings and passes it in.

---

## Task List

### Task 1: Add `settings` to `GameServiceContext`

**Description:**
Add `pub settings: Arc<RwLock<AppSettings>>` to `GameServiceContext`. Update `AppState::as_game_service_context()` to clone it. Update all `GameServiceContext` construction sites (test helpers, `server/mod.rs`).

**Files touched:**
- `src/application/game_service/context.rs`
- `src/server/mod.rs`
- `src/test_support/context.rs`

**Acceptance criteria:**
- [ ] `GameServiceContext` has `settings` field
- [ ] `AppState::as_game_service_context()` populates it
- [ ] `test_support::make_test_context` populates it with `AppSettings::default()`
- [ ] `test_support::make_test_context_with_sqlite` populates it
- [ ] `server/mod.rs` `create_app_with_storage` populates it

**Verification:**
- [ ] `cargo check` passes
- [ ] `cargo test` compiles (tests may fail until later tasks — that's OK)

**Estimated scope:** Small (3 files)

---

### Task 2: Refactor `DefaultGameService` for dynamic backend rebuild

**Description:**
Change `DefaultGameService` to store `settings: Arc<RwLock<AppSettings>>` and `llm_message_storage: Option<Arc<dyn LlmMessageStorage>>` instead of caching `llm_backend` and `agent_registry` permanently. Wrap the cached fields in `std::sync::RwLock` so they can be rebuilt when connections change. Add a `ensure_backends_current(&self)` method that checks connection IDs and rebuilds if needed.

**Files touched:**
- `src/application/game_service/service.rs`
- `src/server/mod.rs` (callers of `DefaultGameService::with_storage`)

**Acceptance criteria:**
- [ ] `DefaultGameService` struct stores:
  - `settings: Arc<RwLock<AppSettings>>`
  - `llm_message_storage: Option<Arc<dyn LlmMessageStorage>>`
  - `llm_backend: std::sync::RwLock<Arc<dyn LlmBackend>>`
  - `agent_registry: std::sync::RwLock<AgentRegistry>`
  - `last_narration_conn_id: std::sync::RwLock<String>`
  - `last_quantifier_conn_id: std::sync::RwLock<String>`
- [ ] `with_storage` signature: `pub fn with_storage(storage: Option<Arc<dyn LlmMessageStorage>>, settings: Arc<RwLock<AppSettings>>) -> Self`
- [ ] `ensure_backends_current(&self)` checks if `settings.narration_connection_id` or `settings.quantifier_connection_id` changed; if so, rebuilds `llm_backend` and `agent_registry`
- [ ] `execute_action` and `retry_last_response` call `ensure_backends_current` before delegating
- [ ] `new()` calls `with_storage(None, Arc::new(RwLock::new(AppSettings::default())))`
- [ ] `run_server_with_config` passes `Arc::clone(&settings)` to `DefaultGameService::with_storage`
- [ ] `create_app_with_storage` passes settings to `DefaultGameService::with_storage`

**Verification:**
- [ ] `cargo check` passes

**Estimated scope:** Small-Medium (2 files)

---

### Task 3: Refactor `AgentRegistry` and `QuantifierAgent`

**Description:**
Change `AgentRegistry::from_configs_with_storage` to accept `&AppSettings`. Pass settings through to `QuantifierAgent::from_config_with_storage` so it can resolve the quantifier connection without loading settings.

**Files touched:**
- `src/narrative/agents/registry.rs`
- `src/narrative/agents/quantifier/agent.rs`
- `src/application/game_service/service.rs` (caller)

**Acceptance criteria:**
- [ ] `AgentRegistry::from_configs_with_storage` takes `settings: &AppSettings`
- [ ] `QuantifierAgent::from_config_with_storage` takes `settings: &AppSettings`
- [ ] `from_config` delegates to `from_config_with_storage` with `&AppSettings::default()`
- [ ] No `load_settings()` calls remain in either file

**Verification:**
- [ ] `cargo check` passes
- [ ] `cargo test` passes (registry and quantifier tests)

**Estimated scope:** Small (3 files)

---

### Task 4: Refactor LLM backend `narrate_from_context`

**Description:**
Add `settings: Arc<RwLock<AppSettings>>` to `OpenRouterBackend` and `OllamaBackend`. Store it at construction time (`from_connection`). In `narrate_from_context`, read `response_length` from stored settings instead of calling `load_settings()`.

**Files touched:**
- `src/narrative/llm/openrouter.rs`
- `src/narrative/llm/ollama.rs`
- `src/narrative/llm/backend.rs` (`get_llm_backend_for` may need a settings parameter)

**Key decision:** `get_llm_backend_for` currently takes `connection: &Connection`. It does not need settings if the backend stores settings separately. The caller (`DefaultGameService`) will pass settings to the backend constructor.

Change `OpenRouterBackend::from_connection` to accept settings:
```rust
pub fn from_connection(
    connection: &Connection,
    storage: Option<Arc<dyn LlmMessageStorage>>,
    settings: Arc<RwLock<AppSettings>>,
) -> Self
```

Same for `OllamaBackend`. Update `get_llm_backend_for` to accept and forward settings.

**Acceptance criteria:**
- [ ] `OpenRouterBackend` stores `settings: Option<Arc<RwLock<AppSettings>>>`
- [ ] `OllamaBackend` stores `settings: Option<Arc<RwLock<AppSettings>>>`
- [ ] `narrate_from_context` reads `response_length` from stored settings
- [ ] `get_llm_backend_for` accepts `settings` and passes to constructors
- [ ] `get_llm_backend()` is removed or marked deprecated (it loads settings internally)
- [ ] `get_llm_backend_with_settings` is updated accordingly

**Verification:**
- [ ] `cargo check` passes
- [ ] `cargo test` passes (LLM backend tests)

**Estimated scope:** Medium (4 files)

---

### Task 5: Refactor `engine/action_processing.rs`

**Description:**
Remove `load_settings()` from `build_trigger_prompt_parts`. Add `response_length: &str` parameter. Update `FreeActionContext` to include `response_length: &'a str`. Update callers in `application/game_service/actions.rs` and tests.

**Files touched:**
- `src/engine/action_processing.rs`
- `src/application/game_service/actions.rs`
- `src/engine/action_processing_tests.rs`

**Acceptance criteria:**
- [ ] `build_trigger_prompt_parts` takes `response_length: &str` instead of loading settings
- [ ] `FreeActionContext` has `response_length: &'a str`
- [ ] `execute_freeaction_impl` passes `ctx.response_length` to `build_trigger_request`
- [ ] `application/game_service/actions.rs` reads `response_length` from settings and passes it in `FreeActionContext`
- [ ] Tests updated to pass `response_length` explicitly
- [ ] No `load_settings()` or `crate::settings::` imports remain in `engine/action_processing.rs`

**Verification:**
- [ ] `cargo check` passes
- [ ] `cargo test` passes (action_processing tests)

**Estimated scope:** Medium (3 files)

---

### Task 6: Update bootstrap `run.rs`

**Description:**
Load settings once at the top of `run()`. Pass `Arc<RwLock<AppSettings>>` to `run_server_with_config`. Update the arrival task to use the pre-loaded settings instead of calling `load_settings()`.

**Files touched:**
- `src/bootstrap/run.rs`
- `src/server/mod.rs` (update `run_server_with_config` signature)

**Acceptance criteria:**
- [ ] `run()` calls `load_settings()` exactly once
- [ ] `run_server_with_config` signature accepts `settings: Arc<RwLock<AppSettings>>`
- [ ] Arrival narration task uses captured settings instead of `load_settings()`
- [ ] `AppState` is built from passed settings, not reloaded

**Verification:**
- [ ] `cargo check` passes

**Estimated scope:** Small (2 files)

---

### Task 7: Verify no stray `load_settings()` calls remain

**Description:**
Run a grep to confirm only allowed call sites remain. Fix any stragglers.

**Acceptance criteria:**
- [ ] `grep -r "load_settings()" src/` returns only `settings.rs`, `bootstrap/run.rs`, and `settings_tests.rs`
- [ ] `grep -r "crate::settings::" src/` returns only legitimate uses (save, path helpers)

**Verification:**
- [ ] Clean grep results

**Estimated scope:** XS

---

### Task 8: Full test run

**Description:**
Run the full test suite to catch any regressions from the refactor.

**Verification:**
- [ ] `cargo test` passes (or failures are pre-existing)
- [ ] `cargo clippy` passes
- [ ] `cargo fmt` is clean

**Estimated scope:** XS

---

## Checkpoint: After Tasks 1–4

- [ ] `cargo check` passes
- [ ] Construction chain carries settings correctly
- [ ] No `load_settings()` in `narrative/` or `application/` layers

## Checkpoint: After Tasks 5–8

- [ ] `cargo test` passes
- [ ] `cargo clippy` passes
- [ ] Grep confirms no stray `load_settings()` in business logic
- [ ] Architecture lint passes (`arch-lint.toml` rules still respected)

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Test constructors need widespread updates | Medium | Task 1 updates test_support helpers; most tests use those helpers |
| `get_llm_backend()` removal breaks external callers | Low | It's an internal crate function; only used in tests |
| Runtime settings mutation stops affecting backends | Medium | Backends store `Arc<RwLock<AppSettings>>`, so they see fresh values |
| `DefaultGameService::new()` defaulting to `AppSettings::default()` loses real settings in tests | Low | Tests should use `with_storage` or `with_backends` anyway; `new()` was already loading real settings which was the bug |
| Deadlock in `ensure_backends_current` | Low | Only acquires one lock at a time; `settings` read-lock → rebuild → `llm_backend` write-lock → done. No nested locking. |
| Connection change mid-action leaves inconsistent state | Low | `ensure_backends_current` runs at the start of `execute_action` before any LLM calls; action is atomic from that point |

---

## Open Questions

None — the approach is straightforward. If any test unexpectedly fails, it likely relied on the hidden `load_settings()` side effect and needs its test context updated.
