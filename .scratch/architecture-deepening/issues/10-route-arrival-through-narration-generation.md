# 10 — Route arrival narration through the one narration-generation path

Type: grilling
Status: open
Blocked by: (none)
Assignee: (unclaimed)

## Question

Do we commit to making arrival narration (the opening scene when a player
enters a room) a thin caller of the same narrate-and-persist module that the
Action Pipeline uses — and if so, what does that module's interface take, and
what stays specific to arrival?

## Background

This is **candidate B** of the 2026-10-04 review. See
`assets/architecture-review-2026-10-04.html`, card B.

The friction: `src/application/arrival_service.rs` (190 lines) re-implements
the narration prefix in `ArrivalTaskContext::run_inner`:

1. `PromptContext::new(...)`
2. `build_narration_prompt(...)` → `recorder.complete(AGENT_NARRATOR, ...)`
3. `add_message` + `save_message_and_snapshot`

`NarrationGeneration::run` (`pipeline/narration_generation.rs`, 168 lines)
plus `PipelineRun`'s narrator call and `persist_snapshot_or_err` do the same
sequence for Actions. Arrival also:

- uses its own budget path (`build_narration_prompt` with
  `max_context_tokens` / `max_tokens` captured in `bootstrap/init_game.rs`)
  instead of the storage-backed `PromptAssembler` path;
- takes 10 constructor arguments and stores the `LlmCallRecorder`, so
  `init_game.rs` reaches `pipeline.recorder()` to pass it in;
- writes `input_buffer.status` by hand (see ticket 09);
- has its own fallback to `build_fresh_initial_state` when the stored state
  cannot be loaded.

The budget values are read when the arrival task is spawned, and it runs
straight away. So this is a duplicated path, not a stale-settings bug
(verified 2026-10-04).

Deletion test: removing arrival's prompt → recorder → persist body and
calling the shared path removes about 60 duplicated lines. The room lookup
and the scene context stay arrival-specific.

## What this ticket resolves

- **Commit or reject.** Does arrival differ enough (no player input, no
  Triggers, no Quantifier) to earn its own path?
- **Shared interface.** What the narrate-and-persist module takes (state,
  prompt context, preset, response length?) and returns.
- **Recorder ownership.** Whether `ActionPipeline::recorder()` stays public
  once arrival no longer needs it.
- **Fallback behaviour.** Where the fresh-state fallback lives.
- **What survives.** Arrival tests and narration tests that cross the shared
  interface unchanged.

## Constraints

- Arrival must still run without a player Action and must not run the
  Quantifier or Triggers unless the grilling decides it should.
- Decision ticket, no implementation.

## Notes

- Resolution uses `/grilling` and `/domain-modeling`.
- Related: ticket 06 — arrival is the only production caller of
  `PromptContext::build_narration_prompt`.
- Related: ticket 04 — the narration-generation module sits inside the
  lifecycle that 04 consolidates.
- Related: ticket 05 — arrival is a second caller of
  `save_message_and_snapshot`.
- If this changes `src/application/prompting/`, the LLM-test policy applies
  at implementation time (note in hand-off).
