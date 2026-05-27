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

### 3.2 Doc Anchor Requirements (`guardrails_doc_anchors`)

**Standard**: Public functions with >5 statements or containing control flow must contain a doc anchor comment:

```rust
// [DOC: docs/path/to/spec.md]
```

Exemptions: getters/setters, `From`/`Into` impls, test functions.

**Severity**: warn  
**Goal**: zero warnings, then promote to error.

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
