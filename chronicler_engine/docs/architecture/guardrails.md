# Architecture Guardrails

## Overview

The Chronicler Engine enforces coding standards through three complementary layers:

1. **Compile-time**: `clippy` lints (deny-level in `src/lib.rs`)
2. **Test-time (declarative)**: `arch-lint` architecture rules (`arch-lint.toml`)
3. **Test-time (custom)**: `syn`-based convention tests in `tests/infrastructure/guardrails/` (21 registered conventions; load-bearing subset documented in §3.1–3.10)

All three run in CI via `build.py`.

---

## 1. Clippy (Compile-Time)

Configured in `src/lib.rs` via `#![deny(...)]`:

| Lint | Rationale |
|------|-----------|
| `clippy::unwrap_used` | Prevent panics in production; propagate errors with `?` |
| `clippy::expect_used` | Same as above; no "should never happen" assumptions |
| `clippy::dbg_macro` | No debug prints in committed code |
| `clippy::todo` | No unfinished code in production |
| `clippy::unimplemented` | No stubs in production |
| `clippy::print_stdout` | No `println!` in library code |
| `clippy::print_stderr` | No `eprintln!` in library code |
| `clippy::panic` | No explicit panics |

Tests are exempt via `#![cfg_attr(test, allow(...))]`.

Additional config in `clippy.toml`:
- `cognitive-complexity-threshold = 25`
- `too-many-arguments-threshold = 7`

---

## 2. arch-lint (Test-Time)

Declarative architecture rules in `arch-lint.toml`.

### Scope/Layer Enforcement

| From Scope | To Scope(s) | Rationale |
|------------|-------------|----------|
| `model` | `server`, `narrative`, `engine`, `application` | Domain layer must be pure; no outer layer dependencies |
| `engine` | `server`, `application`, `narrative` | Engine layer isolated from orchestration and I/O |
| `application` | `server` | Application orchestration independent of driving adapters |
| `server` | `storage` | Driving adapter must not access storage directly; use application ports |

### Deferred arch-lint rules (not yet enforced)

The `server → storage` half of the row below is already enforced by `arch-lint.toml`; only `server → narrative` remains open. Other rows describe pre-existing leaks that arch-lint cannot yet exempt cleanly.

| Deferred rule | Status | Notes |
|---------------|--------|-------|
| `server` → `storage` | **Enforced** by `arch-lint.toml` (`from = "server" to = ["storage"]`); row is paired with `narrative` for symmetry | Driving adapters must not access storage directly; route through application ports |
| `server` → `narrative` | Open | Driving adapters must not import narrative directly; route through application ports |
| `storage` → `narrative` | Open | Driven adapters must not depend on other driven adapters |
| `narrative` → `storage` | Open | Driven adapters must not depend on other driven adapters |
| `application` → `adapters/driven` | Open | Application layer must not import driven adapters directly; route through ports. Three files carry explicit `arch-lint: storage-direct` markers (one intentional persistence boundary, two deferred to T2 reliability plan). Two additional files import `Storage` without a marker and need remediation. |
| `domain` → anything (explicit) | Subsumed | Already covered by existing `model` scope deny rules |
| `application/ports` → anything | Subsumed | Covered by `application` → `server` rule + (deferred) `application` → `adapters/driven` rule |

For the planning history that produced these deferrals, see the decision record (hexagonal-reorganization superplan).

### `DebugPort` exemption

`src/adapters/driving/http/debug.rs` reaches into `ApplicationService` directly. This is an **intentional guardrail exemption** (no `DebugPort` trait) — single debug consumer + single debug surface = phantom port. Documented here for traceability; no code change required.

### Rules

| Rule | Severity | Notes |
|------|----------|-------|
| `no-unwrap-expect` | error | Tests exempt via `allow_in_tests = true` |
| `require-doc-comments` | warning | Public items should have doc comments |
| `require-tracing` | disabled | Project uses `log` crate instead |
| `no-sync-io` | disabled | Intentional sync I/O during startup |

Run: `cargo nextest run --test architecture`

---

## 3. Custom syn-Based Guardrails (`tests/infrastructure/guardrails/`)

AST-parsed convention enforcement. Rules start at `warn` severity; legacy exemptions prevent blocking the build. The full set of registered tests lives in `tests/infrastructure/guardrails/mod.rs`; §3.1–3.9 below document the most-load-bearing subset.

### 3.1 Import Ordering (`guardrails_import_ordering`)

**Standard**: `std` → external → `crate::` → `super` → `self`

Each group must be separated by a blank line. Within a group, sort alphabetically.

**Severity**: error  
**Scope**: `src/` and `tests/`

### 3.2 Module Documentation Standards (`guardrails_doc_standards`)

**Standard**: Every Rust source file in `src/` must have:

1. **Line 1**: DOC anchor comment
   ```rust
   //! [DOC: docs/path/to/domain-doc.md]
   ```

2. **Line 2**: Module summary (human-readable, non-empty)
   ```rust
   //! Character sheet data structures and trigger evaluation types
   ```

The anchor must point to a domain-specific documentation file (e.g., `docs/system/game_flow.md`, `docs/system/navigation.md`), not the generic architecture overview (`docs/architecture/system.md`), except for:

- Cross-cutting infrastructure files: `cli.rs`, `error.rs`, `lib.rs`, `main.rs`, `settings.rs`
- Test support files: `test_support/*`
- Model tier files: `model/*` (model tier IS the architecture)
- Storage tier files: `storage/*` (storage schema IS the architecture)

**Module summary requirements**:
- Must be a `//!` comment on line 2 (after the DOC anchor)
- Must NOT be another `[DOC:]` anchor
- Must be non-empty (not just `//!`)
- Should concisely describe the module's purpose in domain terms

### 3.3 mod.rs Purity (`guardrails_mod_purity`)

**Standard**: `mod.rs` should only contain `pub mod` declarations, `use` / `pub use` statements, and `//!` module-level documentation. No `struct`, `enum`, `fn`, `impl`, or `const` definitions.

**Severity**: error  
**Exemptions**: `src/adapters/driving/http/mod.rs` — legacy structural decision.

### 3.4 Long Comment Run Detection (`guardrails_long_comment_runs`)

**Standard**: No runs of 5 or more consecutive `//` or `///` comment lines. Long explanations belong in external documentation linked via doc anchors.

**Severity**: warn  

### 3.5 Single-Letter Variables (`guardrails_single_letter_vars`)

**Standard**: No single-letter bindings outside tiny scopes (≤3 statements).
**Severity**: warn  

### 3.6 File Length (`guardrails_file_length_src`, `guardrails_file_length_tests`)

**Standard**: No `.rs` file may exceed 2,000 non-blank lines.

**Severity**: error  
**Scope**: `src/` and `tests/`  
**Exemptions**: None

### 3.7 Messages/Swipes Separation (`guardrails_messages_swipes_separation`)

**Standard**: `src/adapters/driven/storage/backend/messages.rs` must not reference the `message_swipes` table. Swipe operations belong in `swipes.rs`.

**Severity**: error  
**Scope**: `src/adapters/driven/storage/backend/messages.rs` only  
**Checks**: SQL table references (`FROM message_swipes`, `INTO message_swipes`, `UPDATE message_swipes`, `JOIN message_swipes`, `DELETE FROM message_swipes`)

**NOTE**: This is a targeted guardrail for the messages/swipes separation concern. It does not catch dynamic SQL construction or clever abstractions — those should be caught in code review.

### 3.8 Server Layer Boundaries (`guardrails_server_layer_boundaries`)

**Standard**: The server layer (HTTP handlers) must not reference or mutate `GameState` directly. State access goes through the application service layer.

**Severity**: error  
**Scope**: `src/adapters/driving/http/` (excluding `mod.rs` and `debug.rs`)  
**Checks**: References to `GameState` (excluding `GameStateSnapshot`)

### 3.9 Handler Return Type Consistency (`guardrails_handler_return_type`)

**Standard**: All HTTP handlers in the server layer must return `Response<Body>` with error mapping via `app_err_to_response()`. The tuple return type `(StatusCode, String)` is forbidden — it bypasses the centralized error-to-HTTP mapping and creates an inconsistent HTTP contract.

**Severity**: error
**Scope**: `src/adapters/driving/http/` (excluding `mod.rs`, `debug.rs`, and `renderers.rs`)
**Checks**: Function signatures with `-> (StatusCode, String)` return type

### 3.10 Enum Variant Docs (`guardrails_enum_variant_docs`, `guardrails_enum_variant_docs_tests`)

**Standard**: Every enum variant carries `///` rustdoc stating what the variant *means* or *when emitted* (semantic, not narration). Trivial enums — variants self-document via name (e.g. `Direction::North`, `Confidence::High`) — may opt out with a `/// [TRIVIAL_ENUM]` marker directly above the `enum` declaration. A trivial-marked enum with any variant `///` is a violation. The rule fires in both directions: missing-doc and trivial-conflict.

**Severity**: error
**Scope**: `src/` and `tests/`
**Checks**: `syn::visit::Visit` walk of every `ItemEnum`; per-variant presence of `doc` attr; per-enum presence of `[TRIVIAL_ENUM]` token in a `doc` attr.
**Exemptions**: Empty enums (`enum Never {}`) pass trivially.

## 4. Coverage Exclusion Policy

Code coverage is measured via `cargo-llvm-cov` with file-level exclusions configured in `build.py`.

**Approach:** `--ignore-filename-regex` flag instead of `#[coverage(off)]` attributes, because `#[coverage(off)]` requires nightly Rust (feature `coverage_attribute`) and stable Rust compatibility is required.

Excluded files are wiring/lifecycle (server bootstrap, test support, LLM backend clients) — covered by integration tests rather than unit coverage. The exclusion list lives in `build.py`'s `--ignore-filename-regex` flag.

---

## 5. Runtime Invariants

Machine-checkable statements about engine runtime behavior. Violations indicate bugs.

### State Mutations

#### INV-001: Generation Status Lifecycle
Every action must end with a free generation slot. The status (mirrored on `state.narrative.input_buffer.status`) returns to `Idle` when the pipeline completes; panics mid-flight heal on the next action via `heal_stale_generating`.
- **Test:** `tests/infrastructure/invariant_contract.rs::test_inv001_generation_guard_resets_on_panic`

#### INV-002: State Mutation Order
Mutations between action start and end follow a fixed sequence: movement → NPC resolution → trigger evaluation → NPC events. State-mutation side effects (message persistence) live in per-phase helpers, not inline in the orchestrator. Out-of-order mutations compile but break silently.
- **Test:** `tests/infrastructure/invariant_contract.rs::test_inv002_state_mutation_order`
- **Test:** `tests/infrastructure/invariant_contract.rs::test_inv002_mutation_order_property` (proptest)


### Concurrency

#### INV-003: No Raw OS Thread Spawning
All concurrent work runs on the tokio runtime. No `std::thread::spawn` or `std::thread::sleep` anywhere in `src/`.
- **Test:** `tests/infrastructure/guardrails/mod.rs::guardrails_no_std_thread`

#### INV-004: LLM Calls Are Cancellable
Long-running generation must be cancellable at phase boundaries. The `ActionPipeline` aborts stale generations at stage boundaries; the LLM transport enforces only a 180-second HTTP timeout (no backend-level cancellation token).
- **Test:** `tests/infrastructure/invariant_contract.rs::test_inv004_cancellable_at_boundaries`

#### INV-004b: No Concurrent Async Actions
Only one `FreeAction` generation in flight at a time. Server rejects overlaps.
- **Test:** `tests/invariant_contract_tests.rs::test_inv004b_no_concurrent_async_actions`

#### INV-005: Lock Poison Recovery
All `Mutex`/`RwLock` sites recover from poison via `into_inner()`.
- **Test:** `tests/poison_recovery.rs::test_settings_recover_from_poisoned_rwlock`

### HTTP Layer

#### INV-006: All Actions Are Async
All player input is parsed as `FreeAction` and offloaded to `spawn_blocking`.
- **Enforced by:** Architecture review (no dynamic test)

#### INV-007: Actions Return Immediately
Handlers return `"Thinking..."` before the LLM call begins.
- **Enforced by:** Architecture review (no dynamic test)

---

## Running Guardrails

```bash
# Individual test files
cargo nextest run --test architecture
cargo nextest run --test guardrails

# Full suite (includes both)
cargo nextest run

# CI pipeline
python build.py
```

## Document References

- [ADR-027: Hexagonal Architecture Migration](../adr/adr-027-hexagonal-architecture-migration.md) — phantom port heuristic + rejected ports + ports/traits collapse + `DebugPort` §3.2 exemption
- [system/triggers.md](../system/triggers.md) — "State Mutation Order" section specifies mutation sequence that INV-002 tests
