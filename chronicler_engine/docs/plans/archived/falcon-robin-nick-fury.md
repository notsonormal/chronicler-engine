# Implementation Plan: Extract Application Layer (Option A)

## Overview

Move `engine::game_service` to a new top-level `application` tier, creating a clean architectural boundary: `server` → `application` → `engine` → `model`. This is a pure refactoring — no logic changes, only module paths and imports.

## Architecture Decisions

- **Application tier location**: `src/application/game_service/` (not `src/game_service/`). The `application/` namespace leaves room for future use cases (checkpoint management, settings orchestration) without cluttering the top level.
- **No re-exports from engine**: `engine/mod.rs` will no longer expose `game_service`. All consumers must import from `application::game_service`.
- **arch-lint rules**: `engine` cannot depend on `application` (inner-layer purity); `application` cannot depend on `server` (prevents circularity).

## Dependency Graph (Target)

```
server/          ← HTTP, Axum
    ↓ imports application
tests/           ← integration tests
    ↓ imports application + engine + model
application/     ← DB I/O, LLM coordination, retry orchestration
    ↓ imports engine + narrative + storage + model
engine/          ← pure domain rules (parser, logic, trigger_eval, action_processing)
    ↓ imports model
model/           ← pure data structs
```

---

## Task List

### Phase 1: Create Application Module Structure

#### Task 1: Create `src/application/mod.rs`

**Description:** Create the new application tier module declaration file.

**Acceptance criteria:**
- [ ] File exists at `src/application/mod.rs`
- [ ] Contains only `pub mod game_service;` and module-level doc comment
- [ ] Follows mod.rs purity (no struct/fn/impl definitions)

**Files touched:**
- `src/application/mod.rs` (new)

**Estimated scope:** XS

---

#### Task 2: Move `game_service` directory

**Description:** Move the entire `game_service` submodule from `engine/` to `application/`.

**Acceptance criteria:**
- [ ] All 9 files exist under `src/application/game_service/`
- [ ] No files remain under `src/engine/game_service/`

**Files moved:**
- `src/engine/game_service/mod.rs` → `src/application/game_service/mod.rs`
- `src/engine/game_service/service.rs` → `src/application/game_service/service.rs`
- `src/engine/game_service/context.rs` → `src/application/game_service/context.rs`
- `src/engine/game_service/actions.rs` → `src/application/game_service/actions.rs`
- `src/engine/game_service/retry.rs` → `src/application/game_service/retry.rs`
- `src/engine/game_service/helpers.rs` → `src/application/game_service/helpers.rs`
- `src/engine/game_service/helpers_tests.rs` → `src/application/game_service/helpers_tests.rs`
- `src/engine/game_service/retry_tests.rs` → `src/application/game_service/retry_tests.rs`

**Estimated scope:** XS

---

#### Task 3: Update `src/lib.rs`

**Description:** Add `pub mod application;` to the crate root.

**Acceptance criteria:**
- [ ] `src/lib.rs` declares `pub mod application;`
- [ ] Module ordering follows existing pattern (alphabetical among pub mods)

**Files touched:**
- `src/lib.rs`

**Estimated scope:** XS

---

#### Task 4: Update `src/engine/mod.rs`

**Description:** Remove `pub mod game_service;` and its test declarations from `engine/mod.rs`.

**Acceptance criteria:**
- [ ] `pub mod game_service;` removed
- [ ] `#[cfg(test)] mod helpers_tests;` and `#[cfg(test)] mod retry_tests;` removed (they moved with the directory)

**Files touched:**
- `src/engine/mod.rs`

**Estimated scope:** XS

---

### Checkpoint: Phase 1
- [ ] `cargo check` passes (module structure is valid)

---

### Phase 2: Fix Internal Imports Within Application

#### Task 5: Update cross-references inside `application/game_service/`

**Description:** Fix imports between files that previously referenced `crate::engine::game_service::...`.

**Changes required:**

In `src/application/game_service/context.rs` (line 10):
```rust
// BEFORE
use crate::engine::game_service::helpers::load_messages_into_state;
// AFTER
use crate::application::game_service::helpers::load_messages_into_state;
```

In `src/application/game_service/helpers.rs` (line 3):
```rust
// BEFORE
use crate::engine::game_service::context::GameServiceContext;
// AFTER
use crate::application::game_service::context::GameServiceContext;
```

In `src/application/game_service/retry.rs` (line 3):
```rust
// BEFORE
use crate::engine::game_service::actions::{
    execute_freeaction_pipeline, finish_action, reconcile_post_trigger_npcs,
};
// AFTER
use crate::application::game_service::actions::{
    execute_freeaction_pipeline, finish_action, reconcile_post_trigger_npcs,
};
```

In `src/application/game_service/helpers_tests.rs`:
```rust
// BEFORE
use crate::engine::game_service::context::GameServiceContext;
use crate::engine::game_service::helpers::{...};
// AFTER
use crate::application::game_service::context::GameServiceContext;
use crate::application::game_service::helpers::{...};
```

In `src/application/game_service/retry_tests.rs`:
```rust
// BEFORE
use crate::engine::game_service::context::GameServiceContext;
use crate::engine::game_service::retry::{...};
use crate::engine::game_service::service::DefaultGameService;
// AFTER
use crate::application::game_service::context::GameServiceContext;
use crate::application::game_service::retry::{...};
use crate::application::game_service::service::DefaultGameService;
```

**Acceptance criteria:**
- [ ] All 5 files updated with correct `crate::application::game_service::` paths
- [ ] `super::` references within the module remain unchanged (they still work)
- [ ] `crate::engine::action::...` and `crate::engine::action_processing::...` references in `actions.rs` remain unchanged (those are pure engine imports)

**Files touched:**
- `src/application/game_service/context.rs`
- `src/application/game_service/helpers.rs`
- `src/application/game_service/retry.rs`
- `src/application/game_service/helpers_tests.rs`
- `src/application/game_service/retry_tests.rs`

**Estimated scope:** S

---

### Checkpoint: Phase 2
- [ ] `cargo check` passes (application module compiles internally)

---

### Phase 3: Update All External Consumers

#### Task 6: Update `src/server/` files

**Description:** Update all server-side imports from `engine::game_service` to `application::game_service`.

**Changes required:**

In `src/server/mod.rs`:
- Line 160: `crate::engine::game_service::DefaultGameService::with_storage(...)` → `crate::application::game_service::DefaultGameService::with_storage(...)`
- Line 161: `Arc<dyn crate::engine::game_service::GameService>` → `Arc<dyn crate::application::game_service::GameService>`
- Line 181: `use crate::engine::game_service::{DefaultGameService, GameService};` → `use crate::application::game_service::{DefaultGameService, GameService};`
- Line 243-244: `crate::engine::game_service::GameServiceContext` → `crate::application::game_service::GameServiceContext` (in `as_game_service_context` method)

In `src/server/fragments/actions.rs` (line 81):
- `crate::engine::game_service::persist_new_messages(...)` → `crate::application::game_service::persist_new_messages(...)`

In `src/server/mod_tests.rs` (line 3):
- `use crate::engine::game_service::{DefaultGameService, GameService};` → `use crate::application::game_service::{DefaultGameService, GameService};`

In `src/server/fragments_tests.rs` (lines 29, 32):
- `crate::engine::game_service::DefaultGameService::with_storage(...)` → `crate::application::game_service::DefaultGameService::with_storage(...)`
- `Arc<dyn crate::engine::game_service::GameService>` → `Arc<dyn crate::application::game_service::GameService>`

**Acceptance criteria:**
- [ ] All 5 server files updated
- [ ] No remaining `engine::game_service` references in `src/server/`

**Files touched:**
- `src/server/mod.rs`
- `src/server/fragments/actions.rs`
- `src/server/mod_tests.rs`
- `src/server/fragments_tests.rs`

**Estimated scope:** S

---

#### Task 7: Update `src/test_support/`

**Description:** Update test support utilities that construct `GameServiceContext`.

**Changes required:**

In `src/test_support/context.rs` (line 3):
- `use crate::engine::game_service::GameServiceContext;` → `use crate::application::game_service::GameServiceContext;`

**Acceptance criteria:**
- [ ] `test_support/context.rs` uses `application::game_service::GameServiceContext`

**Files touched:**
- `src/test_support/context.rs`

**Estimated scope:** XS

---

#### Task 8: Update integration tests

**Description:** Update all `tests/` files that import from `chronicler_engine::engine::game_service`.

**Changes required:**

In `tests/game_service.rs`:
- `chronicler_engine::engine::game_service::DefaultGameService` → `chronicler_engine::application::game_service::DefaultGameService`
- `chronicler_engine::engine::game_service::GameServiceContext` → `chronicler_engine::application::game_service::GameServiceContext`

In `tests/game_service/basic.rs`:
- `chronicler_engine::engine::game_service::{DefaultGameService, GameService}` → `chronicler_engine::application::game_service::{DefaultGameService, GameService}`

In `tests/game_service/advanced.rs`:
- Same as basic.rs

In `tests/flow_mock.rs`:
- `chronicler_engine::engine::game_service::GameServiceContext` → `chronicler_engine::application::game_service::GameServiceContext` (multiple occurrences)

In `tests/flow_mock/sequence.rs`:
- `chronicler_engine::engine::game_service::{DefaultGameService, GameService}` → `chronicler_engine::application::game_service::{DefaultGameService, GameService}`

In `tests/flow_mock/retry_main.rs`:
- Same as sequence.rs

In `tests/flow_mock/retry_event.rs`:
- Same as sequence.rs

In `tests/diagnostic.rs`:
- `chronicler_engine::engine::game_service::{DefaultGameService, GameService, GameServiceContext}` → `chronicler_engine::application::game_service::{DefaultGameService, GameService, GameServiceContext}`

In `tests/diagnostic/scenarios.rs`:
- `chronicler_engine::engine::game_service::{DefaultGameService, GameService}` → `chronicler_engine::application::game_service::{DefaultGameService, GameService}`

**Acceptance criteria:**
- [ ] All 9 test files updated
- [ ] No remaining `engine::game_service` references in `tests/`

**Files touched:**
- `tests/game_service.rs`
- `tests/game_service/basic.rs`
- `tests/game_service/advanced.rs`
- `tests/flow_mock.rs`
- `tests/flow_mock/sequence.rs`
- `tests/flow_mock/retry_main.rs`
- `tests/flow_mock/retry_event.rs`
- `tests/diagnostic.rs`
- `tests/diagnostic/scenarios.rs`

**Estimated scope:** M

---

### Checkpoint: Phase 3
- [ ] `cargo check` passes (all imports resolved)
- [ ] `cargo test` passes (no broken tests from import changes)

---

### Phase 4: Guardrails and Documentation

#### Task 9: Update `arch-lint.toml`

**Description:** Add the new `application` scope and enforce dependency rules.

**Changes required:**

Add new scope:
```toml
[[scopes]]
name = "application"
paths = ["src/application/**"]
```

Update existing rules:
```toml
# Engine must not depend on outer layers (server or application).
[[deny-scope-dep]]
from = "engine"
to = ["server", "application"]
message = "Engine layer must not depend on server or application layers."

# Application must not depend on server.
[[deny-scope-dep]]
from = "application"
to = ["server"]
message = "Application layer must not depend on server layer."

# Model (innermost) must not depend on any outer layer.
[[deny-scope-dep]]
from = "model"
to = ["server", "narrative", "engine", "application"]
message = "Model layer must be pure; cannot depend on outer layers."
```

Remove the old `engine` → `server` rule (now covered by the updated `engine` rule above), or keep both if arch-lint supports duplicates.

**Acceptance criteria:**
- [ ] `application` scope defined
- [ ] `engine` cannot depend on `application` or `server`
- [ ] `application` cannot depend on `server`
- [ ] `model` cannot depend on `application`

**Files touched:**
- `arch-lint.toml`

**Estimated scope:** S

---

#### Task 10: Update `docs/architecture/system.md`

**Description:** Insert the new Application Tier into the architecture documentation.

**Changes required:**

Add new section between Engine Tier and Narrative Tier:

```markdown
### 2.5. The Application Tier (`crate::application::*`)
Orchestration layer that coordinates game flow, persistence, and LLM generation.
Sits between the HTTP server and the pure simulation engine.

- **`game_service`**: `GameService` trait and `DefaultGameService` implementation.
  - `execute_freeaction_pipeline()`: Full FreeAction pipeline (narrate → quantify → triggers → event continuation)
  - `retry_last_response_impl()`: Message-aligned retry with snapshot restoration
  - `save_committed_state()`: Snapshot persistence with `committed = true`
  - `load_state()`, `save_state()`, `persist_new_messages()`: Storage I/O helpers
```

Update Engine Tier description to remove `game_service` references and clarify it is now purely domain rules.

**Acceptance criteria:**
- [ ] Application Tier documented with accurate module list
- [ ] Engine Tier description updated to reflect pure-rules-only status
- [ ] Tier ordering in the doc matches actual dependency direction

**Files touched:**
- `docs/architecture/system.md`

**Estimated scope:** S

---

#### Task 11: Update `docs/architecture/guardrails.md`

**Description:** Update the scope/layer enforcement table to include `application`.

**Changes required:**

Update the table in Section 2:

```markdown
| Scope | Cannot depend on |
|-------|-----------------|
| `model` | `server`, `narrative`, `engine`, `application` |
| `engine` | `server`, `application` |
| `application` | `server` |
```

**Acceptance criteria:**
- [ ] Table includes `application` scope
- [ ] Rules match `arch-lint.toml`

**Files touched:**
- `docs/architecture/guardrails.md`

**Estimated scope:** XS

---

### Checkpoint: Phase 4
- [ ] `cargo nextest run --test architecture` passes (arch-lint rules enforced)
- [ ] `cargo nextest run --test guardrails` passes (syn-based conventions)

---

### Phase 5: Final Validation

#### Task 12: Run full validation suite

**Description:** Execute the complete project validation pipeline.

**Verification:**
```bash
cd chronicler_engine
python build.py
```

**Acceptance criteria:**
- [ ] `fmt` passes
- [ ] `clippy` passes
- [ ] All tests pass (including `architecture` and `guardrails`)
- [ ] Coverage report generates without errors

**Estimated scope:** S

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Missed import in a file | Medium | After all edits, run `grep -r "engine::game_service" src/ tests/` to catch stragglers |
| arch-lint rule conflict | Low | The `architecture.rs` test is a single macro invocation (`arch_lint::check!()`); updating `arch-lint.toml` is sufficient |
| Doc anchor references break | Low | Search for `engine::game_service` in doc comments and doc anchors; update any that exist |
| mod.rs purity violation for new `application/mod.rs` | Low | Keep it to one line: `pub mod game_service;` plus doc comment — no definitions |

## Open Questions

- None. All dependencies and import paths are verified.
