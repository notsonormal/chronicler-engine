# 09 — Give the input buffer named generation transitions

Type: grilling
Status: open
Blocked by: (none)
Assignee: (unclaimed)

## Question

Do we commit to moving generation-state changes onto the input buffer (or
`GameState`) as named transitions — so callers stop writing
`input_buffer.status` and `input_buffer.phase` by hand — and if so, what is the
transition set, and which invariants does the deepened module own?

## Background

This is **candidate A** of the 2026-10-04 review and its **top
recommendation**. See `assets/architecture-review-2026-10-04.html`, card A.

The friction: generation state is two fields, `status: GenerationStatus` and
`phase`, three levels deep (`state.narrative.input_buffer`). Callers set them
by hand. A 2026-10-04 count found 39 non-test assignment lines across 8 files:

- `src/application/generation/gate.rs`
- `src/application/pipeline/pipeline_run.rs`
- `src/application/pipeline/action_pipeline/core.rs`
- `src/application/pipeline/action_pipeline/options.rs`
- `src/application/pipeline/action_pipeline/retry.rs`
- `src/application/message_service.rs`
- `src/application/arrival_service.rs`
- `src/test_support/test_app_builder.rs`

Many test files also write the fields. `generation_status.rs` documents
status and phase as independent axes, but no method enforces how they relate.
The crash-recovery invariant in
`docs/diataxis/explanation/two-state-channels.md` (persisted `Generating`
with no live slot heals to `Idle`) depends on these writes being consistent,
and today that consistency lives in every caller.

The deepening: named transitions (for example `begin(phase)`,
`advance(phase)`, `fail(err)`, `finish()`) with the two fields private to the
domain module. Pure domain code, so it can be tested without a runtime.

Deletion test: inlining the sub-structs does not shorten call sites; adding
transitions removes the repeated two-line writes and concentrates the
invariant in one place.

## What this ticket resolves

- **Commit or reject.** Do the independent axes need an owner, or is direct
  field access fine?
- **Transition set.** The names, what each sets, which transitions are legal
  from which states, and where the error message is captured.
- **Home.** `InputBuffer` itself, `NarrativeState`, or `GameState`.
- **Persistence and serde.** The fields are persisted in Snapshots. Can they
  go private without changing the snapshot format?
- **Test access.** How tests set up a specific state (for example a stale
  `Generating`) once the fields are private.

## Constraints

- Domain purity: no `tokio`, no I/O in the domain module.
- Snapshot format must stay restorable (CONTEXT.md: every Snapshot is
  immediately valid for restore).
- Decision ticket, no implementation.

## Notes

- Resolution uses `/grilling` and `/domain-modeling`. CONTEXT.md has no entry
  for generation status or phase; add one if the transitions need a name.
- Blocks ticket 08 (gate fold): the gate writes these fields in `heal_stale`
  and `try_claim`.
- Shrinks tickets 04 (pipeline lifecycle), 05 (MessageService) and 10
  (arrival narration), which all write the fields.
