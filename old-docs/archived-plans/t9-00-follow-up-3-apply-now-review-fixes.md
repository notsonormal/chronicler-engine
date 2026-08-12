# T9-00 Follow-up: 3 Apply-Now Review Fixes

## Summary

Land 3 small, safe fixes from the dual-lens review on the uncommitted T9-00 refactor. All keep T9-00 single-commit "uncommitted for review" shape — these fold into the existing review surface, not a new commit.

1. **A1 / S2** — `seed_test_world_into_storage` error semantics now match `TestData::seed_into` (fail-loud, not silent fake id).
2. **S1** — Drop unnecessary `Send` bound on `state_mut` + `game_service_fn` closures (verified: both used synchronously inside `build_service`, no spawn, no cross-thread share).
3. **A3 partial** — Delete `BackendSpec::Default` variant (dead: 0 callers; all `default_test()` chains invoke `.game_service_fn(...)`).

Rejected: full A3 collapse to 2 helpers, A6 `tests/_support/` crate move, S6 persona-key rename. Deferred: A2 typed setters + A4 `to_transient_state` (follow-up ticket 04), A5 `Message::input(...)` factories (separate domain ticket, touches prod code).

## Key Changes

| File | Change |
|------|--------|
| `src/test_support/context.rs` | `seed_test_world_into_storage`: 5 sites swap silent error suppression → `.expect("test setup: ...")` matching `TestData::seed_into` |
| `tests/helpers/sqlite_test_app_builder.rs` | `type StateMut` drop `+ Send`; `type GameServiceBuilder` drop `+ 'static` explicit (still implicit); delete `BackendSpec::Default` variant + doc comment + match arm + legacy wiring import + `backend: BackendSpec::Default` initializer (→ use any remaining variant; init via `GameServiceFn` placeholder or unwrap default) |

### `with_data` initializer question (single open decision — resolving in plan)

Current: `backend: BackendSpec::Default`. After dropping `Default`, need a different initializer. Options: (a) init to `GameServiceFn` with a closure calling legacy production wiring (defers to same composition path, preserving "if no backend set, use prod wiring" intent without needing enum variant), or (b) make `backend` an `Option<BackendSpec>` + error at `build_service` if unset.

Choosing (a): keeps "default to prod wiring" semantic that `Default` variant provided, without crowding enum. Closure would still require legacy wiring import, so import stays if (a).

Re-choosing (b) — simpler: `backend: Option<BackendSpec>` starting `None`, and `build_service` `match` adds a `None` arm returning `Err` or panicking. Since every existing caller sets a backend, `None` is truly a user error. This drops legacy wiring import from sqlite_test_app_builder.rs entirely (still used by test_app_builder.rs at that time, unaffected).

**Decision locked: (b)** — `backend: Option<BackendSpec>`, new `None` arm panics with clear message `"SqliteTestAppBuilder: no backend set; call .game_service_fn(...) or .mock_backend(...) before .build_service()"`. Legacy wiring import removed from this file.

## Implementation

Single phase — 3 SP total, 3 tasks, sequential.

### Phase 1: Apply 3 fixes + verify

- [ ] #### Task 1.1: Fix `seed_test_world_into_storage` error semantics (1 SP)
  - In `src/test_support/context.rs::seed_test_world_into_storage`:
    - `storage.seed_world(...).unwrap_or(1)` → `.expect("test setup: seed world")` (matches `TestData::seed_into`)
    - `let _ = storage.seed_persona(...)` → `storage.seed_persona(...).expect("test setup: seed persona")`
    - `let _ = storage.seed_character(...)` → `storage.seed_character(...).expect("test setup: seed character")`
    - `.create_game(...).unwrap_or(1)` → `.expect("test setup: create game")`
  - Rationale: aligns with T9 Q2 fail-loud; `arrival_persistence.rs:139` failing-storage test uses `with_test_failures()` which only intercepts `load_latest_snapshot` (git-verified), so seeding calls never hit the failure path — safe to tighten.
  - **Verify**: `cargo build --tests`; `cargo test --test integration arrival_service_tests_falls_back_to_fresh_state_on_load_error` (the one external caller) passes.

- [ ] #### Task 1.2: Drop `Send` bound on `state_mut` + `game_service_fn` closures (1 SP)
  - In `tests/helpers/sqlite_test_app_builder.rs`:
    - `type StateMut = Box<dyn FnOnce(&mut GameState) + Send>` → `type StateMut = Box<dyn FnOnce(&mut GameState)>`
    - `type GameServiceBuilder = Box<dyn FnOnce(&Arc<Storage>) -> Arc<GameService> + 'static>` → `type GameServiceBuilder = Box<dyn FnOnce(&Arc<Storage>) -> Arc<GameService>>` (`'static` stays implicit on `Box<dyn Trait>`; make it explicit if clippy prefers, otherwise drop)
    - `state_mut` method signature: `F: FnOnce(&mut GameState) + Send + 'static` → `F: FnOnce(&mut GameState) + 'static`
    - `game_service_fn` method signature: `F: FnOnce(&Arc<Storage>) -> Arc<GameService> + 'static` (unchanged — keep `'static` since it's stored in `Box<dyn>`; only dropped the redundant `Send`)
  - Reviewer's claim verified: `state_mut` invoked synchronously at line 251 in `build_service`; `game_service_fn` closure invoked synchronously at `f(&storage)` in the match arm. Zero spawn sites.
  - Widens accepted closures (not narrows) — no existing test breaks.
  - **Verify**: `cargo build --tests`; `cargo test --test integration` 223 pass.

- [ ] #### Task 1.3: Delete `BackendSpec::Default` variant (1 SP)
  - In `tests/helpers/sqlite_test_app_builder.rs`:
    - Remove legacy wiring import (line 22)
    - Remove `Default` variant + 2-line doc comment from `enum BackendSpec` (lines ~42-44)
    - Change `backend` field type to `Option<BackendSpec>`; initializer `backend: None`
    - Add `None` arm at top of `match self.backend` in `build_service`: `panic!("SqliteTestAppBuilder: no backend set; call .game_service_fn(...) or .mock_backend(...) before .build_service()")`
    - Remove the `BackendSpec::Default => { ... }` arm (lines 282-291, the `settings_arc` + `preset_storage` block — the simplification work from earlier already moved these inside Default only, so all of it goes)
  - Verified 0 callers hit `Default`: every `default_test()` chain (3 sites in `invariant_contract.rs`, 2 in `retry_event.rs`) chains `.game_service_fn(...)`. All 80+ other sites set a backend.
  - Keeps the other 4 variants (`MockBackend`, `Backends`, `SeparateBackends`, `GameServiceFn`) — each has real callers (37/19/15/4 respectively).
  - **Verify**: `cargo build --tests`; `cargo test --test integration` + `cargo test --lib` + `cargo test --test guardrails` + `cargo test --test infrastructure` all pass.

## Test Plan

- No new tests — these are simplifications of test infrastructure. Behavior of all 1238 tests unchanged.
- Existing tests are the verification: if any of the 25 `state_mut` sites or 80+ backend-setter sites break, build or tests fail.

## Per Task/Sub Task Validation Steps

- **Task 1.1**: `cargo build --tests` green; `cargo test --test integration arrival_service_tests_falls_back_to_fresh_state_on_load_error` passes (failing-storage test still exercises its failure path correctly).
- **Task 1.2**: `cargo build --tests` green (no new type errors); `cargo test --test integration` 223 pass.
- **Task 1.3**: `cargo build --tests` green (no unused import warning, no missing variant in match); `cargo test --test integration` + `cargo test --lib` + `cargo test --test guardrails` + `cargo test --test infrastructure` all pass.
- **Final**: `python build.py` green (fmt + clippy + guardrails + 1238 tests + skipped LLM). Log added to T9-00 `## Answer` deviations section.
- **Docs sync**: no doc updates needed — these are internal simplifications not changing the public `SqliteTestAppBuilder` API surface (only tightening it: `Default` was never advertised in docs).

## Assumptions

- T9-00 stays single-commit uncommitted for review (per map.md `Q6` + ticket `## Answer`); these 3 fixes fold into the same review surface.
- `arrival_persistence.rs:139` failing-storage test's `with_test_failures()` intercepts only `load_latest_snapshot` (git-verified). If future reviewer finds it intercepts seeding calls too, Task 1.1 must be re-evaluated — but current code says no.
- Dispatch: single `general-purpose` subagent (3 SP total, well under 5 SP ceiling); primary verifies + runs `build.py` after.
- If `clippy` complains about implicit `'static` after Task 1.2, add `+ 'static` back explicitly (mint condition: either form compiles).
- If any test breaks from dropping `Send` (unexpected, since this widens accepted closures), individual test closure needs review — but this would be a test capturing non-Send data intentionally, which reviewer verified none do.
