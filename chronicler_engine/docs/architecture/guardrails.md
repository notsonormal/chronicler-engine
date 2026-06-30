# Architecture Guardrails

## Overview

The Chronicler Engine enforces coding standards through three complementary layers:

1. **Compile-time**: `clippy` lints (deny-level in `src/lib.rs`)
2. **Test-time (declarative)**: `arch-lint` architecture rules (`arch-lint.toml`)
3. **Test-time (custom)**: `syn`-based convention tests (`tests/guardrails.rs`)

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

User decision (Task 0, Option B): arch-lint 0.4.3 lacks scoped file-level exemptions for `deny-scope-dep`. Pre-existing layer leaks (e.g. `templates.rs`/`view_models.rs` importing `LlmMessage`/`CheckResult`; `ports/llm_provider.rs` default impls reaching into `Storage`) would make every would-be Phase 1.7 rule fail the build at red. Instead these rules are deferred until Phase 2 closes the leaks (or a grep-based guardrail test replaces arch-lint enforcement).

| Deferred rule | Rationale | Blocker | Target phase |
|---------------|-----------|---------|---------------|
| `server` → `storage`, `narrative` | Driving adapters must not import driven adapters directly; route through application ports | Phase 2.3 closed `check_player_input` leaks (now via `TextCheckService`). `templates.rs` + `view_models.rs` import `LlmMessage` + `CheckResult` — these are port types at `application/ports/`, so imports are legal. | Verification needed — may be closed |
| `storage` → `narrative` | Driven adapters must not depend on other driven adapters | None currently (rule would pass today, but paired with the reverse) | After Phase 2 closes the leaks |
| `narrative` → `storage` | Driven adapters must not depend on other driven adapters | Phase 2.1 removed default impls (`LlmProvider` is transport-only). `application/agents/registry.rs` + `application/agents/quantifier/agent.rs` still import `Storage` — these are application→driven leaks, not narrative→storage. Rule may be obsolete. | Verification needed |
| `application` → `adapters/driven` | Application layer must not import driven adapters directly; route through ports | Scoped file-level exemptions needed for `context.rs`, `application_service.rs`, `game_service.rs` (marked `// arch-lint: storage-direct — intentional, see ADR-027`). `action_pipeline/*` no longer imports driven adapters post-Phase-2. | Phase 2.5 (comment-only documentation); enforcement via PR review until then |
| `domain` → anything (explicit) | Already covered by existing `model` scope deny rules; plan repeats for emphasis | Subsumed — no action | Already enforced |
| `application/ports` → anything | Ports must depend only on `domain` and `error` | Subsumed by `application` → `server` rule + (deferred) `application` → `adapters/driven` rule | After Phase 2 closes the leaks |

See [`docs/plans/hexagonal-reorganization-plan.md`](../plans/hexagonal-reorganization-plan.md) Phase 1.7 + Phase 2 for the deferred-leak cleanup items.

### Rules

| Rule | Severity | Notes |
|------|----------|-------|
| `no-unwrap-expect` | error | Tests exempt via `allow_in_tests = true` |
| `require-doc-comments` | warning | Public items should have doc comments |
| `require-tracing` | disabled | Project uses `log` crate instead |
| `no-sync-io` | disabled | Intentional sync I/O during startup |

Run: `cargo nextest run --test architecture`

---

## 3. Custom syn-Based Guardrails (`tests/guardrails.rs`)

AST-parsed convention enforcement. Rules start at `warn` severity; legacy exemptions prevent blocking the build.

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

## 4. Coverage Exclusion Policy

Code coverage is measured via `cargo-llvm-cov` with file-level exclusions configured in `build.py`.

**Approach:** `--ignore-filename-regex` flag instead of `#[coverage(off)]` attributes, because `#[coverage(off)]` requires nightly Rust (feature `coverage_attribute`) and stable Rust compatibility is required.

**Excluded files** (integration-tested wiring/lifecycle, not business logic):
- `server/(router|server_impl|handlers).rs`
- `test_support/.*.rs`
- `bootstrap/run.rs`
- `narrative/llm/(openrouter|ollama|deepseek|backend).rs`

**Reference:** cargo-llvm-cov Issue #453 recommends file-level exclusion for stable Rust.

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
