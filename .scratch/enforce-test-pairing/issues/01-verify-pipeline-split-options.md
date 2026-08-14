# Verify pipeline.rs split options

Type: research
Status: closed
Resolution: see [research summary](01-verify-pipeline-split-options-summary.md). Recommendation: modified 4-way split (core/action/retry/retrigger) with retry_event_continuation in core.rs. This unblocks [Execute pipeline split](issues/07-execute-pipeline-split.md).
Blocked by: —

## Question

`src/application/pipeline/pipeline.rs` (802 lines) must be split so `retry_tests.rs` gets a matching source module. The existing plan (`old-docs/archived-plans/split-pipeline-rs-into-entry-modules.md`) proposes a 4-way split: `core.rs` (state/constructors/shared orchestration), `action.rs` (action entry), `retry.rs` (retry entry), `retrigger.rs` (retrigger entry). **Verify whether this is the best decomposition, and present multiple viable options** (more than one if practical) rather than a single answer.

Read `pipeline.rs` fully. For each candidate decomposition, judge against three criteria — **coupling is the gate**:

1. **Coupling** — each module depends only on `core`'s public surface; no back-edges; minimal `pub(crate)` exposure. A decomposition that forces exposing private internals or creates circular deps is rejected.
2. **Cohesion** — each module has one reason to change (one entry path).
3. **Testability** — each module's tests can move to a matching `_tests.rs` with no shared private helpers leaking across files.

Method inventory (clusters already identified):
- **state/constructors**: `with_storage`, `with_backends`, `with_mock_quantifier`, `backend_info`, `recorder`, `prompt_assembler`, `rebind_for_test`, `is_shutting_down`, `reset_persisted_status`
- **action entry**: `process_action`, `continue_narration`, `execute_action`
- **retry entry**: `retry_last_response`, `retry`, `retry_main_narration`, `retry_event_continuation`
- **retrigger entry**: `retrigger`, `retrigger_event`
- **shared orchestration**: `run_from_input`, `persist_generation_error`

Consider at minimum:
- The proposed 4-way (`core`/`action`/`retry`/`retrigger`).
- A 3-way where `retry` + `retrigger` merge (both are "re-run an existing generation" paths).
- Any other decomposition the coupling analysis surfaces.

Deliver a markdown summary (linked as an asset from the resolution) with: per-option coupling map (which `self.` fields / cross-calls cross the seam, what must become `pub(crate)`), cohesion verdict, testability verdict, and a recommendation with reasoning. The chosen option drives [Execute pipeline split](issues/07-execute-pipeline-split.md).
