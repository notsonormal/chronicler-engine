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
`assets/architecture-review.html` for the before/after diagram and evidence.

The friction: one orchestration concept is split across `core.rs`, `action.rs`,
`retrigger.rs`, `retry.rs` (and now `options.rs` — see Current state) by
entry path, not behaviour. `PipelineRun` exposes
phase methods (`phase_narrate`, `phase_post_generation`,
`phase_trigger_continuation_llm_call`, `build_trigger_request`,
`phase_finalize`, …) nearly as wide as the orchestration. The real ordering
invariant lives in `run_from_input` (now `core.rs:186-341`, ~155 lines of
manual phase dispatch). `persist_snapshot_or_err` (now
`pipeline_run.rs:57-76`) is duplicated across phases.

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
- Related: ticket 09 (input-buffer transitions) removes the hand-written
  `input_buffer.status` / `.phase` writes from `core.rs`, `pipeline_run.rs`,
  `options.rs` and `retry.rs`. Resolving 09 first shrinks this ticket's
  surface. Not a hard block.
- Related: ticket 10 (arrival narration) decides whether arrival goes through
  the same narration-generation path as this lifecycle.

## Current state (2026-10-04)

Worse. A fifth entry-path file, `action_pipeline/options.rs` (223 lines),
adds the on-demand Options entry and the `OptionsGeneration` Agent dispatch.
Current sizes: `core.rs` 584, `retry.rs` 273, `options.rs` 223, `action.rs`
73, `retrigger.rs` 49, `pipeline_run.rs` 460. `run_from_input` grew from ~110
to ~155 lines and gained an always-on Options rewrite branch. The grilling
must decide how the Options entry fits the consolidated lifecycle. The
`inherent-impl-locality` constraint still applies.
