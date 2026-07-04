# T7 Sub-Plan (Archived): Split `Backend` enum into `Backend` + `LayeredBackend`

**Date:** 2026-06-27
**Status:** Implemented (uncommitted in working tree)
**Parent:** `docs/plans/abstraction-fixes-followup-superplan.md` → Track T7
**Scope:** `chronicler_engine/src/storage/backend/` + 3 doc files

## Summary

Fix root cause: `Backend` enum conflated real storage impls (`Sqlite`, `InMemory`) with a decorator (`Test { base, overrides }`). Split into two types — `Backend` (2 real variants only) + `LayeredBackend` (decorator, non-recursive `Test { base: Box<Backend>, overrides }`). Enabled removal of 40 dead `Backend::Test { .. } => unreachable!()` arms across 10 storage files (compiler-enforced 2-variant match) while keeping test failure-injection working unchanged. Also isolated Test variant into `LayeredBackend` (preparatory step for future `#[cfg(test)]` gating, separate sub-plan) and preserved existing non-stacking invariant (at most one Test layer, now structurally type-enforced). Test-infra types (`TestOverride`, `TestFailureHandle`, `ErrorKind`) relocated to dedicated `storage::backend::test_support` module. Public `Storage` API preserved → all existing tests pass without modification.

## Architecture Decisions

1. **Non-recursive `LayeredBackend::Test { base: Box<Backend>, ... }`** — at most one Test layer per `Storage`, mirrors actual invariant (`with_overrides` already replaces, never nests). Prevents infinite Test chain failure mode.
2. **Exhaustive match in `with_backend_mut` + `set_game_id`** (not `while let` + `unreachable!()`) — zero runtime panics; adding 3rd `LayeredBackend` variant is compile error not silent panic.
3. **Test-infra types moved now** (`storage::backend::test_support` module), not deferred — keeps this plan's blast radius tight while structurally preparing future `#[cfg(test)]` gating.
4. **Minimal doc touch** on 3 files (`storage.md`, `adr-020`, `CHANGELOG.md`) — narrow scope, only directly-falsified claims. Doc-debt catchall T9 stays separate.
5. **2 micro-tests** pin the load-bearing invariant — `with_failure_replaces_does_not_nest`, `add_failure_reuses_map_does_not_nest`.
6. **No commits** — all changes uncommitted in working tree for review.

## Key Changes

### 1. `src/storage/backend/core.rs` — enum split

```rust
pub enum Backend {
    Sqlite { pool: DbPool },
    InMemory(Box<InMemoryData>),
}

pub enum LayeredBackend {
    Direct(Backend),
    Test {
        base: Box<Backend>,                                    // non-recursive
        overrides: Arc<Mutex<HashMap<&'static str, TestOverride>>>,
    },
}
```

### 2. `Storage` struct field type change

```rust
pub struct Storage {
    game_id: AtomicU64,
    backend: Mutex<LayeredBackend>,    // was: Mutex<Backend>
}
```

### 3. Constructor updates

- `new_sqlite`: wraps in `LayeredBackend::Direct(Backend::Sqlite { pool })`
- `new_in_memory`: wraps in `LayeredBackend::Direct(Backend::InMemory(Box::new(InMemoryData::empty())))`
- `InMemoryData::empty()` constructor dedupes 2 literals (was 3 — `add_failure` throwaway now uses `empty()`)

### 4. `with_backend_mut` rewrite — exhaustive 2-arm match

```rust
match &mut *backend {
    LayeredBackend::Direct(inner) => f(inner),
    LayeredBackend::Test { overrides, base } => {
        if let Some(override_) = overrides.lock()...get(method) {
            return Err(override_.to_error());
        }
        f(base.as_mut())
    }
}
```

### 5. `set_game_id` update — exhaustive match unwraps Test layer

```rust
let target = match &*backend {
    LayeredBackend::Direct(b) => b,
    LayeredBackend::Test { base, .. } => base.as_ref(),
};
if let Backend::Sqlite { pool } = target { ... }
```

### 6. `add_failure` + `with_overrides` — match on `LayeredBackend` variants

`add_failure` uses `LayeredBackend::Direct(Backend::InMemory(...))` as `mem::replace` throwaway; existing Test arm reuses map; Direct arm wraps in Test.

`with_overrides` extracts `base: Box<Backend>` from existing Test layer (unwraps single decorator) or wraps Direct in Box, then re-wraps in fresh Test. Replace-not-nest preserved.

### 7. Test-infra types relocated to `src/storage/backend/test_support.rs`

`TestOverride`, `ErrorKind`, `TestFailureHandle` + all impls + `Drop` moved from `core.rs` to new module. `core.rs` imports them via `use super::test_support::{TestFailureHandle, TestOverride};`. Re-export shim in `storage/backend/mod.rs` preserves `crate::storage::{TestOverride, TestFailureHandle, ErrorKind}` import path → zero test code changes.

### 8. Dead arm deletion — 40 arms across 10 files

`Backend::Test { .. } => unreachable!(),` deleted from: characters.rs (3), games.rs (4), llm_messages.rs (2), messages.rs (8), personas.rs (3), presets.rs (4), settings.rs (2), snapshots.rs (3), swipes.rs (5), worlds.rs (6).

### 9. Tests added

2 micro-tests in `src/storage/backend/core_tests.rs`:

- `with_failure_replaces_does_not_nest`
- `add_failure_reuses_map_does_not_nest`

Both use test-only `Storage::backend_layer_info()` inspector returning `(top_layer, base_variant)`.

### 10. Documentation

- `docs/system/storage.md` (3 passages updated)
- `docs/adr/adr-020-storage-consolidation.md` (annotation appended, ADR not rewritten)
- `docs/CHANGELOG.md` (new entry at top of Unreleased)

## Out of Scope (Deferred)

- D3: `with_backend_mut` signature change to `Option<u64>` API — cosmetic miss, current signature achieves same goal
- D11: `from_row` consistency audit across 9 Db* models — separate sub-plan
- D2: `empty_to_none` inlining — separate sub-plan
- `#[cfg(test)]` gating of `LayeredBackend::Test` variant — M1-cleanup crossover, future sub-plan
- Trait `StorageBackend` reconsideration (original Decision 1 Option C) — storage.md explicitly prohibits `dyn Trait`/custom mocks

## Verification

- `cargo check`: clean
- `build.py`: fmt ✓, clippy ✓, tests ✓ (1250 pass, 2 skipped, 0 fail — was 1248, +2 new)
- Public `Storage` API unchanged — existing tests pass without modification

## Decisions Log (Findings 1-8)

- **F1** Locked: non-recursive `Box<Backend>` for `Test.base`
- **F2** Locked: scope claims reworded (M1 "isolates" not "resolves"; M2 "preserves" not "fixes")
- **F3** Locked: `InMemoryData::empty()` made required, dedupes 2 literals
- **F4** Locked: exhaustive match, no `unreachable!()` runtime panics
- **F5** Locked: 2 micro-tests added pinning replace-not-nest invariant
- **F6** Locked (option B): test-infra types moved now, not deferred
- **F7** Locked (revised): 2-phase implementation with `build.py` between phases, no commits
- **F8** Locked (option A): minimal doc touch on 3 files, NOT defer to T9

## Adherence Notes

Worker Task 3 deviation occurred mid-implementation (subagent deleted public API methods `with_failure` / `with_test_failures` / `add_failure` / `with_shared_overrides` / `with_overrides` and the `LayeredBackend` enum itself, rewrote `with_backend_mut` as bare `f(&mut backend)` with `_method` unused). Caught during main-agent review per Plan Adherence rule, all modified storage files reverted via `git checkout`, Task 3 redone directly by main agent via `edit` tool. All other subagent work (Task 1 relocate types, Task 2 add enum + `empty()`, Task 5 delete dead arms, Task 6 micro-tests, Task 7 docs, build.py checkpoints) verified before next step.
