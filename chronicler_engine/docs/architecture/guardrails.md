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

| Scope | Cannot depend on |
|-------|-----------------|
| `model` | `server`, `narrative`, `engine`, `application` |
| `engine` | `server`, `application`, `narrative` |
| `application` | `server` |
| `server` | `storage` |

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
**Exemptions**: `src/server/mod.rs` — legacy structural decision.

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

**Standard**: `src/storage/backend/messages.rs` must not reference the `message_swipes` table. Swipe operations belong in `swipes.rs`.

**Severity**: error  
**Scope**: `src/storage/backend/messages.rs` only  
**Checks**: SQL table references (`FROM message_swipes`, `INTO message_swipes`, `UPDATE message_swipes`, `JOIN message_swipes`, `DELETE FROM message_swipes`)

**NOTE**: This is a targeted guardrail for the messages/swipes separation concern. It does not catch dynamic SQL construction or clever abstractions — those should be caught in code review.

### 3.8 Server Layer Boundaries (`guardrails_server_layer_boundaries`)

**Standard**: The server layer (HTTP handlers) must not reference or mutate `GameState` directly. State access goes through the application service layer.

**Severity**: error  
**Scope**: `src/server/` (excluding `mod.rs` and `debug.rs`)  
**Checks**: References to `GameState` (excluding `GameStateSnapshot`)

### 3.9 Handler Return Type Consistency (`guardrails_handler_return_type`)

**Standard**: All HTTP handlers in the server layer must return `Response<Body>` with error mapping via `app_err_to_response()`. The tuple return type `(StatusCode, String)` is forbidden — it bypasses the centralized error-to-HTTP mapping and creates an inconsistent HTTP contract.

**Severity**: error  
**Scope**: `src/server/` (excluding `mod.rs`, `debug.rs`, and `renderers.rs`)  
**Checks**: Function signatures with `-> (StatusCode, String)` return type

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
