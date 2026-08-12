# Plan: T9-00 follow-up — verify remaining work is empty

**Date:** 2026-08-12  
**Status:** Verification / close  
**Goal:** Confirm the three original T9-00 fixes are already in place and retire the follow-up.

## Original items and current state

| Original item | Status | Evidence |
|---------------|--------|----------|
| A1 / S2: `seed_test_world_into_storage` error semantics match `TestData::seed_into` | **Done** | `TestData::seed_into` (in `src/test_support/test_data_builder.rs`) already panics via `expect` on every failure path; the return value is simply unused in `seed_test_world_into_storage`. |
| S1: Drop unnecessary `Send` bound on `state_mut` + `game_service_fn` closures | **Done** | `tests/helpers/sqlite_test_app_builder.rs` has `type StateMut = Box<dyn FnOnce(&mut GameState)>;` with no `+ Send`. |
| A3 partial: Delete `BackendSpec::Default` variant | **Done** | `BackendSpec` no longer has a `Default` variant; `backend` is `Option<BackendSpec>` with a `None` panic arm in `build_pipeline`. |

## Remaining question
`src/test_support/context.rs:107` still has `let _ = data.seed_into(storage);`. Because `seed_into` returns `i64` and already panics on failure, this is not a silent error. The only cleanup is to assign the unused id to a named binding to avoid the `let _` pattern, e.g. `let _world_id = data.seed_into(storage);`.

## Scope
- Change `let _ = data.seed_into(storage);` to `let _world_id = data.seed_into(storage);` in `src/test_support/context.rs`.
- Run `cargo build --tests` and `python build.py` to confirm no regression.
- Archive the original `t9-00-follow-up-3-apply-now-review-fixes.md` once this lands.

## Out of scope
- Any other changes to `SqliteTestAppBuilder` or `TestData`.
- Re-architecting seed failure handling (it is already fail-loud).

## Acceptance criteria

- `cargo build --tests` passes.
- `python build.py` passes.
- `rg 'let _ = data.seed_into' src/test_support/` returns no matches.
