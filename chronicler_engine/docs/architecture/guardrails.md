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

### 3.2 Module-Level DOC Anchor Requirements (`guardrails_module_doc_anchors`)

**Standard**: Every Rust source file in `src/` must have a module-level DOC anchor comment as its first line:

```rust
//! [DOC: docs/path/to/domain-doc.md]
```

The anchor must point to a domain-specific documentation file (e.g., `docs/system/game_flow.md`, `docs/system/navigation.md`), not the generic architecture overview (`docs/architecture/system.md`), except for:

- Cross-cutting infrastructure files: `cli.rs`, `error.rs`, `lib.rs`, `main.rs`, `settings.rs`
- Test support files: `test_support/*`
- Model tier files: `model/*` (model tier IS the architecture)
- Storage tier files: `storage/*` (storage schema IS the architecture)

**Mapping by module**:

| Module | Target Doc |
|--------|-----------|
| `application/*` | `docs/system/game_flow.md` |
| `engine/mod.rs`, `engine/logic.rs` | `docs/system/navigation.md` |
| `engine/trigger_eval.rs` | `docs/system/triggers.md` |
| `engine/state_diagnostics.rs` | `docs/architecture/invariants.md` |
| `model/character.rs` | `docs/system/character_state.md` |
| `model/trigger.rs` | `docs/system/triggers.md` |
| `model/agent.rs` | `docs/system/agent_system.md` |
| `model/llm*` | `docs/system/llm_processing.md` |
| `narrative/agents/*` | `docs/system/agent_system.md` |
| `narrative/prompt/*` | `docs/system/prompt_system.md` |
| `narrative/llm/*`, `narrative/llm_client/*` | `docs/system/llm_processing.md` |
| `narrative/text_check/*` | `docs/system/text_check.md` |
| `narrative/mod.rs` | `docs/system/narration_engine.md` |
| `server/*` | `docs/system/dashboard.md` |
| `bootstrap/*` | `docs/system/startup.md` |

**Rationale**: Module-level anchors provide a clear link from code to its domain documentation without cluttering individual functions. If a specific code block needs documenting, extract it into a separate method.

**Severity**: warn  
**Goal**: zero warnings.

### 3.3 mod.rs Purity (`guardrails_mod_purity`)

**Standard**: `mod.rs` should only contain `pub mod` declarations, `use` / `pub use` statements, and `//!` module-level documentation. No `struct`, `enum`, `fn`, `impl`, or `const` definitions.

**Severity**: error  
**Exemptions**: `src/server/mod.rs` — legacy structural decision.

### 3.4 Long Comment Run Detection (`guardrails_long_comment_runs`)

**Standard**: No runs of 5 or more consecutive `//` or `///` comment lines. Long explanations belong in external documentation linked via doc anchors.

**Severity**: warn  
**Goal**: zero warnings, then promote to error.

### 3.5 Single-Letter Variables (`guardrails_single_letter_vars`)

**Standard**: No single-letter bindings outside tiny scopes (≤3 statements).

**Severity**: warn  
**Current status**: zero violations.

### 3.6 File Length (`guardrails_file_length_src`, `guardrails_file_length_tests`)

**Standard**: No `.rs` file may exceed 2,000 non-blank lines.

**Severity**: error  
**Scope**: `src/` and `tests/`  
**Exemptions**: None

### 3.7 One Table Per Storage Module (`guardrails_one_table_per_storage`)

**Standard**: Each `src/storage/*_storage.rs` module may reference exactly one physical SQLite table. No storage module may touch more than one table.

**Severity**: error  
**Scope**: `src/storage/*_storage.rs`  
**Exemptions**: Temporary migration tables (`*_new` suffix), `sqlite_*` internal tables

See ADR-019 for the rationale.

**NOTE**: This guardrail was removed after ADR-020 unified storage into a single `Storage` struct with no `*_storage.rs` modules.

### 3.8 Messages/Swipes Separation (`guardrails_messages_swipes_separation`)

**Standard**: `src/storage/backend/messages.rs` must not reference the `message_swipes` table. Swipe operations belong in `swipes.rs`.

**Severity**: error  
**Scope**: `src/storage/backend/messages.rs` only  
**Checks**: SQL table references (`FROM message_swipes`, `INTO message_swipes`, `UPDATE message_swipes`, `JOIN message_swipes`, `DELETE FROM message_swipes`)

**NOTE**: This is a targeted guardrail for the messages/swipes separation concern. It does not catch dynamic SQL construction or clever abstractions — those should be caught in code review.

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
