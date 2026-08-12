# Plan: Storage test-seam investigation (refresh)

**Date:** 2026-08-12  
**Status:** Draft — investigation only, no code changes  
**Goal:** Decide whether the current `TestOverride` / `#[cfg(feature = "testing")]` seam in `Storage` is still the right trade-off.

## Current seam
- `src/adapters/driven/storage/backend/core.rs` holds the production `Storage` type.
- Test-only fields/methods are guarded by `#[cfg(feature = "testing")]`:
  - `with_failure`, `add_failure`, `with_test_failures`, `test_override_for`.
- `TestOverride` and `TestFailureHandle` live in `src/adapters/driven/storage/backend/test_support.rs`.
- `Cargo.toml` sets `default = ["testing"]`, so the feature is effectively always on for normal builds.

## Investigation questions

### Q1. Maintenance tax
- Count `#[cfg(feature = "testing")]` blocks in `backend/core.rs` and related storage code.
- Count `with_failure` / `TestOverride` call sites in `src/` tests and integration tests.
- Identify any backend operation that does *not* route through the override mechanism (miss-risk).

### Q2. Does the feature gate do anything useful?
- Confirm whether `default = ["testing"]` means production builds and test builds compile the same `Storage` shape.
- Check if any CI/build step explicitly disables the `testing` feature.
- Determine whether `#[cfg(test)]` on the test-only methods would be sufficient.

### Q3. Alternatives
Sketch three options with rough diff counts:
- **Option A — keep current:** document the trade-off.
- **Option B — move to `#[cfg(test)]`:** hide test seam from non-test builds.
- **Option C — `TestStorage` wrapper:** move failure injection into a test-only wrapper that owns a `Storage`; production `Storage` loses the `Test` fields entirely.

### Q4. Recommendation
- GO / NO-GO / DEFER, with the decision recorded in a short note under `docs/diataxis/explanation/storage_design.md` (ADR directory no longer exists).

## Out of scope
- Implementing any refactor.
- Touching `src/` code.

## Deliverables
1. `tmp/storage-test-seam-findings.md` with Q1/Q2 numbers, Q3 write-ups, and a recommendation.
2. If GO: a follow-up implementation plan.
3. If NO-GO/DEFER: a short note added to `docs/diataxis/explanation/storage_design.md`.

## Verification

- `python build.py` remains green (investigation produces no diff).
- `git status` shows only new `tmp/` docs.
