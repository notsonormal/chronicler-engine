# Plan: Restructure Engine/Application Boundary in Chronicler Engine

## Context

The `engine/` tier currently contains both **pure domain rules** (`parser`, `logic`, `trigger_eval`, `action_processing`) and **application orchestration** (`game_service` with its DB I/O, LLM coordination, and retry logic). The `docs/architecture/system.md` describes `engine` as "mechanics that drive the simulation," but `game_service` is orchestration, not simulation.

This plan evaluates three restructuring options with different trade-offs between correctness, effort, and disruption.

---

## Current State (Verified)

### Dependency Graph
```
server/          ← HTTP, Axum, AppState
    ↓ imports game_service
engine/game_service/  ← DB I/O, LLM calls, retry orchestration, state persistence
    ↓ imports pure engine
engine/{action,action_processing,logic,parser,trigger_eval,state_diagnostics}  ← pure rules
    ↓ imports model
model/           ← pure data structs
```

### Key Constraints
- `arch-lint.toml` enforces: `model` → cannot import `server`/`narrative`/`engine`; `engine` → cannot import `server`
- `action_processing.rs` is already well-factored: receives `LlmBackend` as a parameter, does not own it
- No pure engine code imports from `game_service` (no circular dependency risk)
- `narrative/`, `storage/`, `bootstrap/` do not import from `engine/`
- ~20 files import `engine::game_service::*` (production + tests)

### Files to Move (in all options)
The `game_service` submodule (9 files):
```
src/engine/game_service/mod.rs
src/engine/game_service/service.rs
src/engine/game_service/context.rs
src/engine/game_service/actions.rs
src/engine/game_service/retry.rs
src/engine/game_service/helpers.rs
src/engine/game_service/helpers_tests.rs
src/engine/game_service/retry_tests.rs
```

---

## Option A: Extract Application Layer (Recommended)

Move `game_service` to a new top-level `application/` tier. This creates a proper Clean Architecture boundary: `server` → `application` → `engine` → `model`.

### Changes

**1. File moves**
```
src/engine/game_service/  →  src/application/game_service/
```

**2. Module declarations**
- `src/lib.rs`: add `pub mod application;`
- `src/engine/mod.rs`: remove `pub mod game_service;` and test declarations
- `src/application/mod.rs`: create with `pub mod game_service;`

**3. Import updates (~20 files)**
All `crate::engine::game_service::*` → `crate::application::game_service::*`
All `chronicler_engine::engine::game_service::*` → `chronicler_engine::application::game_service::*`

Affected files:
- `src/server/mod.rs`
- `src/server/fragments/actions.rs`
- `src/server/fragments_tests.rs`
- `src/server/mod_tests.rs`
- `src/test_support/context.rs`
- `tests/game_service.rs`
- `tests/game_service/basic.rs`
- `tests/game_service/advanced.rs`
- `tests/flow_mock.rs`
- `tests/flow_mock/sequence.rs`
- `tests/flow_mock/retry_main.rs`
- `tests/flow_mock/retry_event.rs`
- `tests/diagnostic.rs`
- `tests/diagnostic/scenarios.rs`

**4. Architecture guardrails**
- `arch-lint.toml`: add `application` scope; add rules:
  - `engine` cannot depend on `application` (inner layer purity)
  - `application` cannot depend on `server` (prevents circularity)
  - `model` cannot depend on `application` (already covered by model→engine ban, but explicit is clearer)
- `tests/architecture.rs`: may need update if it asserts on scope count

**5. Documentation**
- `docs/architecture/system.md`: add "11. The Application Tier" documenting `game_service` as orchestration layer
- `docs/architecture/guardrails.md`: update scope/layer enforcement table
- Update any doc anchors `// [DOC: docs/architecture/system.md]` that reference `engine::game_service`

### Effort
- **Mechanical changes**: ~25 files, all find/replace
- **Validation**: `python build.py` (fmt + clippy + tests + coverage)
- **Time estimate**: 1–2 focused sessions
- **Risk**: Low — no logic changes, only module paths

### Pros
- **Correct layering**: `engine/` becomes purely domain rules; intent is unambiguous
- **Reusability**: Pure engine can be extracted into a separate crate later without dragging in DB/LLM orchestration
- **Scalability**: New use cases (e.g., `checkpoint_service`, `settings_service`) have a clear home in `application/`
- **Test clarity**: Tests that need full orchestration import from `application`; tests that need only rules import from `engine`
- **Aligns with existing docs**: `system.md` already distinguishes "mechanics" from "orchestration"

### Cons
- **Highest disruption** of the three options (~25 files touched)
- **All developers must relearn** import paths
- **External references** (if any scripts/docs outside the repo reference `engine::game_service`) break

---

## Option B: Promote GameService to Top-Level Module

Move `game_service` out of `engine/` to `src/game_service/` (sibling to `engine`, `server`, etc.). Simpler than Option A but less structured for future growth.

### Changes

**1. File moves**
```
src/engine/game_service/  →  src/game_service/
```

**2. Module declarations**
- `src/lib.rs`: add `pub mod game_service;`
- `src/engine/mod.rs`: remove `pub mod game_service;`

**3. Import updates**
All `crate::engine::game_service::*` → `crate::game_service::*`
All `chronicler_engine::engine::game_service::*` → `chronicler_engine::game_service::*`
(Same file list as Option A)

**4. Architecture guardrails**
- `arch-lint.toml`: add `game_service` scope; add `engine` cannot depend on `game_service`

**5. Documentation**
- `docs/architecture/system.md`: update Engine tier description to remove `game_service`; add GameService tier or fold into Server tier description

### Effort
- Same mechanical scope as Option A (~25 files)
- Slightly simpler because no nested `application/` module
- **Time estimate**: 1 focused session

### Pros
- **Achieves separation** without introducing a new container module
- **Shorter import paths** (`crate::game_service::GameService` vs `crate::application::game_service::GameService`)
- **Less conceptual overhead** — one module moves, not a whole new tier

### Cons
- **Poor scalability**: If you later add `checkpoint_service`, `settings_use_cases`, etc., they clutter the top level
- **Less semantic**: `game_service` is just a name; `application` signals an architectural layer
- **Same disruption as A** with fewer long-term benefits
- **Top-level module proliferation**: There are already 10+ top-level modules; adding more hurts navigability

---

## Option C: Internal Rename + Re-document

Keep `game_service` inside `engine/` but rename it to `engine/orchestration/` and update documentation to explicitly acknowledge that `engine/` contains both domain rules and orchestration.

### Changes

**1. Rename directory**
```
src/engine/game_service/  →  src/engine/orchestration/
```

**2. Module declarations**
- `src/engine/mod.rs`: change `pub mod game_service;` → `pub mod orchestration;`

**3. Import updates**
All `crate::engine::game_service::*` → `crate::engine::orchestration::*`
All `chronicler_engine::engine::game_service::*` → `chronicler_engine::engine::orchestration::*`
(Same file list as above)

**4. Documentation**
- `docs/architecture/system.md`: update Engine tier to explicitly state it contains both simulation mechanics and orchestration
- Add note: "The `orchestration` submodule handles DB persistence, LLM coordination, and retry logic"

### Effort
- **Lowest effort**: rename + imports + docs
- **Time estimate**: <1 session

### Pros
- **Minimal disruption**: same number of files touched, but conceptually simple
- **Makes intent explicit** through naming
- **No new modules or scopes** to manage

### Cons
- **Does not solve the problem**: domain and application are still mixed under `engine/`
- **Confusing for new developers**: `engine::orchestration` inside the "mechanics" tier contradicts `system.md`
- **No reusability benefit**: pure engine still cannot be extracted without the orchestration module
- **Future technical debt**: When the codebase grows, you will still need Option A or B eventually

---

## Recommendation

**Option A (Extract Application Layer)** is the only choice that delivers long-term architectural value. The other options save a small amount of effort now but leave the mixed-concerns problem unresolved.

Key reasoning:
1. The codebase already has a clean dependency graph — there are no hidden circular dependencies blocking the move
2. The `action_processing.rs` bridge is already well-factored (receives LLM backend as parameter), so the boundary is natural
3. The project has strong guardrails (`arch-lint`, custom syn tests) — adding a new scope is a one-time config change that pays off forever
4. The test suite uses dependency injection (`MockBackend`, `InMemoryStorage`) — tests will adapt with mechanical import changes only
5. `docs/architecture/system.md` already describes tiers conceptually — adding an Application tier makes the documentation more accurate, not less

The ~25 file changes are 100% mechanical. No logic changes. No test behavior changes. The risk is minimal.

---

## Success Criteria

- [ ] `python build.py` passes (fmt + clippy + tests + coverage)
- [ ] `cargo nextest run --test architecture` passes (arch-lint rules)
- [ ] `cargo nextest run --test guardrails` passes (syn-based conventions)
- [ ] `docs/architecture/system.md` accurately describes the new tier
- [ ] `engine/` contains no DB I/O, LLM coordination, or retry logic
- [ ] `application/` (or chosen location) contains all `game_service` code
