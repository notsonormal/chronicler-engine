# Plan: Harden the pipeline error-model boundary

**Date:** 2026-08-12  
**Status:** Planned  
**Goal:** Make sure `GenerationStatus::Error` is used only for UI status rendering, not for pipeline control flow, and prevent regression.

## Background
The pipeline has already migrated to `PhaseError` for control flow:

- `src/application/pipeline/phase_error.rs` defines `Cancelled`, `NarratorFailed`, `PersistFailed`, `TriggerMissing`, `SnapshotMissing`, `FetchFailed`.
- `src/application/pipeline/phases.rs` propagates failures via `PhaseError`.
- `src/application/pipeline/pipeline.rs` handles `PhaseError` in `finalize_phase_error` and writes `GenerationStatus::Error` only for the UI/status endpoint.
- The old `error_return` helper no longer exists.

The remaining `status.error_message()` / `GenerationStatus::Error` usages are all in the HTTP/UI layer (`view_models.rs`, `endpoints.rs`, tests that assert UI behavior). That is the desired boundary, but it is not enforced.

## Scope

### Task 1: Audit the boundary
- Grep `src/` and `tests/` for `error_message()` and `GenerationStatus::Error`.
- Confirm every non-test use outside `src/adapters/driving/http/` is gone.
- If any non-UI control flow remains, convert it to `PhaseError` propagation.

### Task 2: Add a guardrail rule
- In `tests/infrastructure/guardrails/structure.rs`, add `check_generation_status_ui_boundary`.
- Flag `error_message()` calls outside `src/adapters/driving/http/` and `src/domain/model/state/game_state_tests.rs` (or other test modules that legitimately assert the variant).
- Keep the rule narrow: allow `is_generating()` everywhere; only `error_message()` is the UI-boundary marker.

### Task 3: Document the contract
- In `docs/diataxis/reference/game_flow.md` (or a new short section), state:
  - `PhaseError` is the pipeline control-flow channel.
  - `GenerationStatus::Error(String)` is a UI status only; HTTP fragments render it to the user.
  - No application/domain code should branch on `status.error_message()`.

## Out of scope
- Removing `GenerationStatus::Error` (UI rendering depends on it).
- Changing `PhaseError` variants.
- Transactional persistence.

## Acceptance criteria

- The guardrail passes on current code (no new violations).
- Introducing a non-UI `error_message()` call makes the guardrail fail.
- `python build.py` passes.
- `docs/diataxis/reference/game_flow.md` documents the boundary.
