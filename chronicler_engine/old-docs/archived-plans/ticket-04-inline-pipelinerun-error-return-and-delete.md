# Ticket 04: Inline `PipelineRun::error_return` and delete

## Summary
`PipelineRun::error_return` in `chronicler_engine/src/application/pipeline/phases.rs` (lines 89–97) is a one-line wrapper around `Err(self.set_error(state, msg))`. Five call sites still spell it `return self.error_return(state, msg)`; every other error site in the file uses `Err(self.set_error(...))` directly (see `phase_post_generation` at lines 256, 265). Inline the five call sites and delete the wrapper so the file has one consistent spelling.

No behavior change. No public-API change (wrapper is `pub(super)`). Zero test references, zero rustdoc references — pure mechanical edit.

## Key Changes
- Replace 5 `return self.error_return(state, …)` calls in `phases.rs` with `return Err(self.set_error(state, …))`.
- Delete the `error_return` method (lines 89–97).

## Implementation

### Phase 1: Inline and delete

- [ ] #### Task 1.1: Inline `error_return` and delete the wrapper (1 SP)
  - File: `chronicler_engine/src/application/pipeline/phases.rs`
  - Replace at line 112: `return self.error_return(state, "Room not found".to_string());` → `return Err(self.set_error(state, "Room not found".to_string()));`
  - Replace at line 118: `Err(msg) => return self.error_return(state, msg),` → `Err(msg) => return Err(self.set_error(state, msg)),`
  - Replace at line 140: `Err(e) => return self.error_return(state, e.llm_error_string()),` → `Err(e) => return Err(self.set_error(state, e.llm_error_string())),`
  - Replace at line 151: same pattern as 140
  - Replace at line 159: `return self.error_return(state, "LLM Error: empty response".to_string());` → `return Err(self.set_error(state, "LLM Error: empty response".to_string()));`
  - Delete the `pub(super) fn error_return(&self, state: &mut GameState, msg: String) -> Result<(String, String, String), PhaseError> { Err(self.set_error(state, msg)) }` block (lines 89–97).
  - Resolve ticket: append answer to `.scratch/pipeline-review-hygiene/issues/04-inline-error-return.md` (under `## Answer`: one-line summary + verification log path), close the issue, add a one-line Decisions-so-far entry to `.scratch/pipeline-review-hygiene/map.md` linking the closed ticket.

## Test Plan
- `cargo check --all-targets --all-features` from `chronicler_engine/`: must be green.
- `python chronicler_engine/build.py`: must be green (cargo fmt + clippy + nextest full suite).
- No new tests — pure deletion of a private wrapper, existing pipeline + guardrail suites exercise the behavior through the public `PipelineRun::phase_*` API.

## Per Task/Sub Task Validation Steps
- Task 1.1:
  1. `grep -n error_return chronicler_engine/src/application/pipeline/phases.rs` → 0 matches.
  2. `cd chronicler_engine && cargo check --all-targets --all-features` → exit 0.
  3. `python chronicler_engine/build.py` → exit 0; record log path under ticket's `## Answer`.

## Assumptions
- Doc references in `docs/adr/adr-032-phaseerror.md` (Negative + Trade-offs sections) and `docs/CHANGELOG.md` (3 historical entries) remain as-is: per map's `## Out of scope` (cosmetic) and closed-tickets 02/03 precedent (pure refactors skip doc churn). Stale ADR prose is a separate ticket if anyone cares. **Locked: skip per standing precedent.**
- Live plans under `chronicler_engine/docs/plans/*` referencing `error_return` are historical plans-in-flight — not edited here.
- The 5 call sites are the full set: confirmed via `grep -n error_return phases.rs` → 5 call sites + 1 def, plus 1 set_error call inside the wrapper itself (deleted with the wrapper).
- `set_error` (line 81) is retained — it's the underlying helper that does the real work (set `GenerationStatus::Error`, persist snapshot, return `PhaseError::NarratorFailed`).
