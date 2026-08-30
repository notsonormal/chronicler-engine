# 02 — Does the stored-generation-inputs flow need an owning module? (architecture candidate 1)

Type: grilling
Status: open
Blocked by: (none)

> Re-framed 2026-08-29 after ticket 06 resolved: the name **ReplaySteering**
> is avoided, and the candidate's premise was cut — the replay blob is not
> a concept. `GenerationReplay` is data stored on the Swipe; scattered field
> access to plain data is normal data flow, not lost locality. This ticket
> must re-justify the module on other grounds or be rejected.

## Question

Does any module shape still earn its place for the stored-inputs data flow
(stage → attach-to-swipe → read-back-for-generation → read-back-for-new-swipe)
— for example the `GameState::push_message` attachment behaviour, or the
guide/impersonate mutual-exclusion rule — or is candidate 1 rejected?

## Background

**Friction (from `architecture-review.html`, candidate 1, Strong):**
The stored-inputs lifecycle is enforced by no single module. Six files
jointly maintain it:

- `src/domain/model/message.rs` — blob definition (`GenerationReplay`).
- `src/domain/model/state/narrative_state.rs` — `pending_replay` transient
  field.
- `src/domain/model/state/game_state.rs` — `push_message` copies
  `pending_replay` onto the active swipe when a message is created.
- `src/application/pipeline/action_pipeline/action.rs` — builds the blob and
  stages it as `pending_replay`.
- `src/application/pipeline/pipeline_run.rs` — `resolve_guide` /
  `resolve_impersonate` read the blob back from both fresh `PipelineInputs`
  *and* `retry_target.replay()`.
- `src/application/pipeline/action_pipeline/retry.rs` — reconstructs the
  new-swipe mode from `old_target.replay()`.

Understanding the lifecycle requires bouncing between all six. Under the
settled model this is ordinary data flow — the grilling decides whether any
of it rises to behaviour worth an owner.

**Relationship to ticket 01 (narration-generation module).** The report
noted the narration-generation module gives this flow "a single consumer to
call against." The grilling should account for where the stored inputs are
consumed (the deepened module, if 01 is accepted; the current scattered
pipeline reads, if not).

## What to decide

- Re-justify or reject the deepening under the settled model.
- If a module survives: its **interface** (the report sketched
  `stage() · attach_to_swipe() · resolve_for_turn() · resolve_for_retry()` —
  re-skin against the settled model), its **seam** (domain `GameState`,
  application pipeline, or between?), what stays as the value type in
  `message.rs` vs what moves behind the interface, and what **tests** cover
  it at the new seam.
- Whether `GameState::push_message`'s attachment behaviour (copying
  `pending_replay` onto the active swipe) moves behind this module or stays
  on `GameState`.
- Whether the guide/impersonate mutual-exclusion rule lives here or in
  ticket 04's prompt policy — moot if this ticket is rejected (defaults to
  04's grilling).
