# Async Concurrency & Codebase Hygiene

## Problem

The Chronicler Engine suffers from three diagnosability and maintainability problems:

1. **Raw `std::thread::spawn` for LLM work** — creates OS threads that are invisible to the tokio runtime, cannot be cancelled, and silently poison mutexes on panic.
2. **Monolithic files** — `quantifier.rs` (2,129 lines), `llm.rs` (1,752 lines), and `prompt.rs` (1,447 lines) mix unrelated concerns, making any mechanical change risky.
3. **Silent failures in `GeneratingGuard`** — mutex poison errors are swallowed with `if let Ok(...)`, leaving the engine permanently stuck in `Generating` status.

## Approach

**Keep backend traits synchronous.** Use `tokio::spawn` + `tokio::task::spawn_blocking` to run blocking LLM work inside the async runtime. This avoids the `#[async_trait]` + `dyn Trait` incompatibility in Rust 2024 edition while delivering all the observability and cancellation benefits of structured concurrency.

Split the three oversized files into directory modules before doing any mechanical work. Smaller files mean safer refactors.

---

## Phase 1: Audit & Map

**Goal:** Quantify the problem. No production code changes.

### Task 1: Catalog String-Based Error Emissions

**Description:** Enumerate every `EngineError` construction with string literals. Focus on `game_service.rs` string-match sites (`msg.contains(...)`).

**Acceptance criteria:**
- [ ] Markdown table: file, line, variant, string literal
- [ ] `game_service.rs` string-match sites flagged for targeted fix

**Files touched:** None (read-only)
**Estimated scope:** Small

---

### Task 2: Map Concurrency Boundaries

**Description:** Find and document every `std::thread::spawn`, `Mutex<GameState>` usage, and `Arc` clone. Document the `GeneratingGuard` poisoned-mutex failure mode.

**Acceptance criteria:**
- [ ] List of all spawn sites with file/line
- [ ] Data flow diagram for each spawn
- [ ] `GeneratingGuard` behavior documented

**Files touched:** None (read-only)
**Estimated scope:** Small

---

### Task 3: Map `main.rs` Responsibilities

**Description:** Document what `main.rs` does. List public functions in `parser.rs`, `logic.rs`, `trigger_eval.rs`, `llm_client.rs` that lack unit tests.

**Acceptance criteria:**
- [ ] `main.rs` responsibility map
- [ ] Untested public API list with testability notes

**Files touched:** None (read-only)
**Estimated scope:** Small

---

## Checkpoint 1

- [ ] Audit tables complete
- [ ] No production code changed
- [ ] `python build.py` passes

---

## Phase 2: Bootstrap Extraction & Core Tests

**Goal:** Make `main.rs` testable and add fast unit tests to untested modules.

### Task 4: Extract CLI Parsing

**Description:** Move `Args` and `resolve_engine_data_path()` into `src/cli.rs`.

**Acceptance criteria:**
- [ ] `src/cli.rs` with `Args`, `resolve_engine_data_path()`, `list_available_worlds()`
- [ ] `main.rs` imports from `cli`
- [ ] Unit tests for CLI parsing

**Files touched:** `src/main.rs`, `src/cli.rs`
**Estimated scope:** Small

---

### Task 5: Extract Bootstrap Logic

**Description:** Move world loading, validation, state creation, server startup into `src/bootstrap.rs`.

**Acceptance criteria:**
- [ ] `src/bootstrap.rs` with `initialize_world_from_manifest()` and `run_app()`
- [ ] `main.rs` reduced to ~30 lines
- [ ] No behavior change

**Files touched:** `src/main.rs`, `src/bootstrap.rs`
**Estimated scope:** Medium

---

### Task 6: Add Unit Tests for `parser.rs`

**Description:** Test `parse_command` for all `Action` variants.

**Acceptance criteria:**
- [ ] Tests for: Look, Quit, Inventory, Talk, FreeAction, empty input

**Files touched:** `src/engine/parser.rs`
**Estimated scope:** Small

---

### Task 7: Add Unit Tests for `logic.rs`

**Description:** Test navigation and room resolution.

**Acceptance criteria:**
- [ ] Tests for: get_current_room, find_room_in_map, attempt_semantic_walk

**Files touched:** `src/engine/logic.rs`
**Estimated scope:** Small

---

### Task 8: Add Unit Tests for `trigger_eval.rs`

**Description:** Test trigger evaluation and state mutations.

**Acceptance criteria:**
- [ ] Tests for: times_met, room scoping, non-repeatable triggers

**Files touched:** `src/engine/trigger_eval.rs`
**Estimated scope:** Small

---

### Task 9: Add Unit Tests for `llm_client.rs`

**Description:** Test response parsing and sanitization without network calls.

**Acceptance criteria:**
- [ ] Tests for: extract_content, sanitize_llm_output, malformed JSON handling

**Files touched:** `src/narrative/llm_client.rs`
**Estimated scope:** Small

---

## Checkpoint 2

- [ ] `main.rs` < 50 lines
- [ ] `cli.rs` and `bootstrap.rs` extracted
- [ ] 4 previously untested modules have unit tests
- [ ] `python build.py` passes

---

## Phase 3: Split Oversized Files

**Goal:** Break `quantifier.rs`, `llm.rs`, and `prompt.rs` into directory modules. This makes all subsequent mechanical work safer.

### Task 10: Split `llm.rs` into `llm/` directory

**Description:** Convert file module to directory module.

**Target structure:**
```
src/narrative/llm/
├── mod.rs          # Trait + backend selection helpers
├── openrouter.rs   # OpenRouterBackend
├── ollama.rs       # OllamaBackend
├── deepseek.rs     # DeepSeekBackend
├── mock.rs         # MockBackend
```

**Acceptance criteria:**
- [ ] `mod.rs` re-exports preserve public API
- [ ] Tests distributed into per-backend `#[cfg(test)]` blocks
- [ ] `cargo check` passes
- [ ] `python build.py` passes

**Files touched:** `src/narrative/llm.rs` (deleted), `src/narrative/llm/*.rs` (new)
**Estimated scope:** Medium

---

### Task 11: Split `quantifier.rs` into `quantifier/` directory

**Description:** Convert file module to directory module.

**Target structure:**
```
src/narrative/quantifier/
├── mod.rs          # Re-exports + get_quantifier_backend()
├── types.rs        # All structs/enums
├── parser.rs       # Response parsing logic
├── prompt.rs       # QuantifierPromptBuilder
└── backends.rs     # Trait + all backend impls
```

**Acceptance criteria:**
- [ ] `mod.rs` re-exports preserve public API
- [ ] Tests distributed into submodule test blocks
- [ ] `cargo check` passes
- [ ] `python build.py` passes

**Files touched:** `src/narrative/quantifier.rs` (deleted), `src/narrative/quantifier/*.rs` (new)
**Estimated scope:** Medium

---

### Task 12: Split `prompt.rs` into `prompt/` directory

**Description:** Convert file module to directory module.

**Target structure:**
```
src/narrative/prompt/
├── mod.rs          # Re-exports + sanitize_for_prompt
├── budget.rs       # estimate_tokens, truncate_to_budget
├── layers.rs       # PromptLayer enum
├── context.rs      # PromptContext
└── builder.rs      # PromptBuilder + render methods
```

**Acceptance criteria:**
- [ ] `mod.rs` re-exports preserve public API
- [ ] Tests distributed into submodule test blocks
- [ ] `cargo check` passes
- [ ] `python build.py` passes

**Files touched:** `src/narrative/prompt.rs` (deleted), `src/narrative/prompt/*.rs` (new)
**Estimated scope:** Medium

---

## Checkpoint 3

- [ ] `llm.rs`, `quantifier.rs`, `prompt.rs` split into directories
- [ ] Public APIs unchanged (verified by `cargo check`)
- [ ] `python build.py` passes

---

## Phase 4: Replace `std::thread::spawn` with `tokio::spawn`

**Goal:** Eliminate raw OS thread spawning. Use tokio tasks with `spawn_blocking` for blocking LLM work.

### Task 13: Replace Thread Spawning in `game_service.rs`

**Description:** `execute_action` and `retry_last_response` currently spawn `std::thread`. Replace with `tokio::spawn(async move { tokio::task::spawn_blocking(...).await })`.

**Acceptance criteria:**
- [ ] No `std::thread::spawn` in `game_service.rs`
- [ ] No `std::thread::sleep` in `game_service.rs`
- [ ] LLM call runs inside `spawn_blocking`
- [ ] Quantifier call runs inside `spawn_blocking`
- [ ] State updates happen after blocking work completes

**Files touched:** `src/engine/game_service.rs`
**Estimated scope:** Medium

---

### Task 14: Replace Thread Spawning in `fragments.rs`

**Description:** HTTP handlers spawn threads to call `game_service`. Replace with `tokio::spawn`.

**Acceptance criteria:**
- [ ] Action handler: `tokio::spawn(async move { game_service.execute_action(...) })`
- [ ] Retry handler: `tokio::spawn(async move { game_service.retry_last_response(...) })`
- [ ] No `std::thread::spawn` in `fragments.rs`
- [ ] No `std::thread::sleep` in `fragments.rs`

**Files touched:** `src/server/fragments.rs`
**Estimated scope:** Small

---

### Task 15: Replace Thread Spawning in `bootstrap.rs`

**Description:** Arrival narration currently spawns in `thread::spawn`. Use tokio task.

**Acceptance criteria:**
- [ ] Arrival narration runs as tokio task
- [ ] No `std::thread` usage in `bootstrap.rs`

**Files touched:** `src/bootstrap.rs`
**Estimated scope:** Small

---

## Checkpoint 4

- [ ] No `std::thread::spawn` in production code
- [ ] All blocking LLM/quantifier work uses `tokio::task::spawn_blocking`
- [ ] `python build.py` passes

---

## Phase 5: Harden `GeneratingGuard` and Add Cancellation

**Goal:** Eliminate silent mutex failures and make LLM work cancellable.

### Task 16: Harden `GeneratingGuard`

**Description:** The current guard silently swallows `Mutex` poison errors. Replace with explicit recovery.

**Acceptance criteria:**
- [ ] `GeneratingGuard::new` uses `Mutex::clear_poison()` if lock fails, then sets status
- [ ] `GeneratingGuard::drop` logs `error!` and attempts recovery on poison
- [ ] OR: Replace guard with explicit `set_phase` / `reset_generating` calls in each task
- [ ] If mutex is poisoned, status is reset to `Idle` (not left stuck on `Generating`)

**Files touched:** `src/model/state.rs`
**Estimated scope:** Small

---

### Task 17: Add Cancellation Tokens

**Description:** Pass `CancellationToken` into spawned LLM tasks so they can be aborted on shutdown or duplicate requests.

**Acceptance criteria:**
- [ ] `tokio_util::sync::CancellationToken` added to `Cargo.toml`
- [ ] `AppState` holds a `CancellationToken`
- [ ] `tokio::spawn` closures check `token.is_cancelled()` before and after LLM calls
- [ ] On server shutdown, token is cancelled and all in-flight LLM tasks stop cleanly
- [ ] Test: spawn slow mock backend, cancel token, verify state resets to `Idle`

**Files touched:** `Cargo.toml`, `src/engine/game_service.rs`, `src/server/mod.rs`, `src/narrative/llm/mock.rs`
**Estimated scope:** Medium

---

## Checkpoint 5

- [ ] `GeneratingGuard` no longer silently fails on poison
- [ ] Cancellation tokens abort in-flight LLM work
- [ ] `python build.py` passes

---

## Phase 6: Invariants, Guardrails & Build System

### Task 18: Write `docs/architecture/invariants.md`

**Description:** Codify runtime invariants as machine-checkable statements.

**Acceptance criteria:**
- [ ] Invariant: State mutation order in `execute_freeaction_impl`
- [ ] Invariant: `generation_state.status` returns to `Idle` after every action
- [ ] Invariant: No `std::thread::spawn` in production code
- [ ] Invariant: LLM backend calls must be cancellable

**Files touched:** `docs/architecture/invariants.md`
**Estimated scope:** Small

---

### Task 19: Update `tests/guardrails.rs`

**Description:** Add custom syn-based checks for patterns we eliminated.

**Acceptance criteria:**
- [ ] Rule: `no-std-thread` — ban `std::thread::spawn` in `src/` (allow in `tests/`)
- [ ] Rule: `require-generating-guard-docs` — any spawn site must document cleanup strategy

**Files touched:** `tests/guardrails.rs`
**Estimated scope:** Small

---

### Task 20: Update `build.py`

**Description:** Add dependency checks and `--strict` mode.

**Acceptance criteria:**
- [ ] Check for `cargo-nextest` before using it
- [ ] Check Rust version >= 1.85
- [ ] `--strict` mode runs with debug assertions enabled

**Files touched:** `build.py`
**Estimated scope:** Small

---

## Checkpoint 6

- [ ] `invariants.md` written
- [ ] Guardrails ban `std::thread::spawn`
- [ ] `python build.py` passes
- [ ] `python build.py --strict` passes

---

## Dependency Graph

```
Phase 1: Audit
├── Task 1: Error catalog
├── Task 2: Concurrency map
└── Task 3: main.rs map
    │
    ▼
Phase 2: Bootstrap & Tests
├── Task 4: Extract cli.rs
├── Task 5: Extract bootstrap.rs
├── Task 6: Tests for parser.rs
├── Task 7: Tests for logic.rs
├── Task 8: Tests for trigger_eval.rs
└── Task 9: Tests for llm_client.rs
    │
    ▼
Phase 3: Split Files
├── Task 10: Split llm.rs
├── Task 11: Split quantifier.rs
└── Task 12: Split prompt.rs
    │
    ▼
Phase 4: Async Spawn
├── Task 13: game_service.rs
├── Task 14: fragments.rs
└── Task 15: bootstrap.rs
    │
    ▼
Phase 5: Guard & Cancel
├── Task 16: Harden GeneratingGuard
└── Task 17: Cancellation tokens
    │
    ▼
Phase 6: Guardrails & Build
├── Task 18: invariants.md
├── Task 19: guardrails.rs
└── Task 20: build.py
```

---

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| File splitting breaks `use` statements | Medium | `mod.rs` re-exports preserve public API; zero call-site changes |
| `spawn_blocking` panics don't propagate | Low | `JoinHandle.await` + `catch_unwind` in task wrapper; log on panic |
| Total scope is too large | High | Explicit checkpoints; stop for approval after each phase |

---

## What This Plan Does NOT Include

- **Structured error migration** — moved to separate plan
- **`async fn` in traits** — intentionally avoided; `spawn_blocking` achieves the same concurrency benefits without dyn-compatibility issues
- **`Arc<tokio::sync::Mutex<GameState>>`** — out of scope; `std::sync::Mutex` is fine for short-held locks in `spawn_blocking` tasks

---

*Revised: 2026-05-04*
*Original: Approach 3 — Preventive Architecture*
*Revision reason: `#[async_trait]` incompatible with `dyn Trait` in Rust 2024; structured errors split to separate plan*
