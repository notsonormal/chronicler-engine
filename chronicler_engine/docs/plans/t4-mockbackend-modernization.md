# T4: MockBackend Modernization

**Parent Plan:** [abstraction-fixes-followup-superplan.md](./abstraction-fixes-followup-superplan.md)
**Status:** Planning — ready
**Date:** 2026-06-28
**Depends on:** none
**Blocks:** none
**Priority:** P2
**Findings owned:** C6, N6, N7

---

## Summary

`MockBackend` (`narrative/llm/mock.rs:22-35`) builders exist (`::failing()`, `::with_empty_response()`, `::with_failing_trigger_narration()`, `::with_delay()`, `::with_trigger_delay()`) but all 6 AtomicBool/AtomicU64 + 2 Vec fields are still `pub` — tests bypass builders via direct mutation (`backend.should_fail.store(true, ...)`). New tests reach for `field.store(...)` not builders, propagating flag-bag smell.

Source plan originally specified Option C (minimal prune + builders); migration of ~100 `::default()` call sites was Task 6.6b — never done.

## Key Changes

1. Privacy: change `pub should_fail: AtomicBool` → `pub(crate)` for all 6 flags + Vec fields. Forces builder use externally.
2. Audit and remove 2 unused flags (likely `trigger_started` if no test checks it; `narration_started` likely still used).
3. Add `::succeeding()` builder (explicit symmetrical to `::failing()`).
4. Migrate ~100 `::default()` sites in test code — opportunistic, no deadline. Skip migration if step 1 lands; old code still works.

## Decisions to Lock

- Migrate 100 `MockBackend::default()` test sites, or leave (fields go `pub(crate)` so old code still works)?

## Blast Radius

`narrative/llm/mock.rs` + tests touching fields directly. Cosmetic migration can run alongside anything.

## Verification

- `python build.py` — fmt + clippy + tests + coverage must pass clean.
- Verify all existing tests using `MockBackend` still compile after `pub` → `pub(crate)` change.
- Add at least one new test using `::succeeding()` builder to lock the API.

## Pre-Implementation Checklist

- [ ] Grep all `MockBackend` field accesses (`should_fail`, `trigger_started`, `narration_started`, etc.) and confirm they are all in test code (no prod callers).
- [ ] Verify which of the 6 flags have zero test readers (candidates for removal).
