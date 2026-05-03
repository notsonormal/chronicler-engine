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
| `model` | `server`, `narrative`, `engine` |
| `engine` | `server` |

### Rules

| Rule | Severity | Notes |
|------|----------|-------|
| `no-unwrap-expect` | error | Tests exempt via `allow_in_tests = true` |
| `require-doc-comments` | warning | Public items should have doc comments |
| `require-tracing` | disabled | Project uses `log` crate instead |
| `no-sync-io` | disabled | Intentional sync I/O during startup |

Run: `cargo test --test architecture`

---

## 3. Custom syn-Based Guardrails (`tests/guardrails.rs`)

AST-parsed convention enforcement. Rules start at `warn` severity; legacy exemptions prevent blocking the build.

### 3.1 Import Ordering (`guardrails_import_ordering`)

**Standard**: `std` → external → `crate::` → `super` → `self`

Each group must be separated by a blank line. Within a group, sort alphabetically.

```rust
use std::collections::HashMap;
use std::fs;

use walkdir::WalkDir;

use crate::model::state::GameState;
```

**Severity**: error  
**Exemptions**: None

### 3.2 "What" Comment Detection (`guardrails_what_comments`)

**Standard**: No explanatory comments starting with action verbs (`Loop`, `Check`, `Build`, `Handle`, `Process`, `Create`, `Get`, `Set`, `Update`, `Remove`, `Add`, `Call`, `Apply`, `Evaluate`, `Determine`, `Spawn`, `Validate`).

Instead: use semantic variable names or add a doc anchor.

```rust
// BAD
// Loop through NPCs and check if they are in the room
for npc in all_npcs { ... }

// GOOD
let residents = find_npcs_in_current_location(all_npcs, current_room);
// [DOC: docs/system/navigation.md]
```

**Severity**: warn (14 existing instances)  
**Goal**: zero warnings, then promote to error.

### 3.3 Doc Anchor Requirements (`guardrails_doc_anchors`)

**Standard**: Public functions with >5 statements or containing control flow (`if`, `for`, `while`, `match`, `loop`) must contain a doc anchor comment:

```rust
// [DOC: docs/path/to/spec.md]
```

Exemptions:
- Getters/setters
- `From`/`Into` impls
- Test functions
- Functions already containing a doc anchor

**Severity**: warn (20 existing instances)  
**Goal**: zero warnings, then promote to error.

### 3.4 mod.rs Purity (`guardrails_mod_purity`)

**Standard**: `mod.rs` should only contain:
- `pub mod` declarations
- `use` / `pub use` statements
- `//!` module-level documentation

No `struct`, `enum`, `fn`, `impl`, or `const` definitions.

**Severity**: error  
**Exemptions**:
- `src/server/mod.rs` — legacy structural decision; centralizes Axum router, `AppState`, and server entry points.

### 3.5 Long Comment Run Detection (`guardrails_long_comment_runs`)

**Standard**: No runs of 5 or more consecutive `//` or `///` comment lines. Long explanations belong in external documentation linked via doc anchors.

Exemptions:
- Visual dividers (`////` or more slashes)
- Doc anchors (`// [DOC: ...]`)
- Empty comment lines
- Comments inside `#[cfg(test)]` modules

**Severity**: warn  
**Goal**: zero warnings, then promote to error.

### 3.6 Single-Letter Variables (`guardrails_single_letter_vars`)

**Standard**: No single-letter bindings (`a`, `b`, `i`, `x`, etc.) outside tiny scopes (≤3 statements).

**Severity**: warn  
**Current status**: zero violations.

---

## Running Guardrails

```bash
# Individual test files
cargo test --test architecture
cargo test --test guardrails

# Full suite (includes both)
cargo test

# CI pipeline
python build.py
```

---

## Adding New Rules

1. Implement the check as a standalone function in `tests/guardrails.rs`
2. Return `Vec<Violation>` with appropriate `severity`
3. Wire it into `main_runner()`
4. Add a test function that calls `main_runner()` and asserts no `error` severity violations
5. Update this document
6. If the rule affects legacy code, start at `warn` and file a cleanup task
