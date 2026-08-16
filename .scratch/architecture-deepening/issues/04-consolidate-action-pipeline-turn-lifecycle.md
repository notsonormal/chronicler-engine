# 04 — Consolidate the ActionPipeline turn lifecycle

Type: grilling
Status: open
Blocked by: (none)
Assignee: (unclaimed)

## Question

Do we commit to consolidating the `ActionPipeline` / `PipelineRun` split —
currently sliced across four files by entry path — into one deep turn-lifecycle
module with phase ordering made internal — and if so, what is the shape of the
deepened module?

## Background

This is **candidate 3** of the architecture review. See
`architecture-review.html` for the before/after diagram and evidence.

The friction: one orchestration concept is split across `core.rs`, `action.rs`,
`retrigger.rs`, `retry.rs` by entry path, not behaviour. `PipelineRun` exposes
phase methods (`phase_narrate`, `phase_post_generation`,
`phase_trigger_continuation_llm_call`, `build_trigger_request`,
`phase_finalize`, …) nearly as wide as the orchestration. The real ordering
invariant lives in `run_from_input` (`core.rs:226-335`, ~110 lines of manual
phase dispatch). `persist_snapshot_or_err` (`pipeline_run.rs:41-55`) is
duplicated across phases.

The deepening: merge the four `impl ActionPipeline` blocks into one
turn-lifecycle module; make phase ordering internal; expose entry variants
(action/retrigger/retry) as constructors, not separate files.

The deletion test splits: the file split *vanishes* (merging reappears no
complexity); the phase ordering *reappears* — it genuinely belongs in one deep
module.

## What this ticket resolves

- **Commit or reject.** Does the split earn its locality, or does it scatter
  one concept?
- **Interface shape.** What the turn-lifecycle module exposes; which phase
  methods go private; how entry variants are represented.
- **What survives.** Which tests cross the lifecycle interface unchanged; how
  phase-level unit tests (if any) are re-homed behind an internal seam.

## Constraints

- Must respect the existing `action_pipeline/` type-split folder (decided in
  `.scratch/inherent-impl-locality/` ticket 04) — the consolidation must not
  regress the inherent-impl-locality rule.
- Decision ticket, no implementation.

## Notes

- Resolution uses `/grilling` and `/domain-modeling`.
- Domain term: Action Pipeline (CONTEXT.md) — "ordered sequence of phases that
  validates and resolves an Action."
