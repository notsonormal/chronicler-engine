# Research: trace the Snapshots each generation path saves

Type: research
Status: resolved
Blocked by: —

## Question

For each entry path that runs a generation, which Snapshots does it save, in which order, with which label, and which Message or Swipe `snapshot_id` does each save set? The seed's S0–S5 diagram covers only `run_from_input`. Check it against the current code and extend it to every path.

## Scope

Entry paths to trace (known from the code; add any you find):

- `ActionPipeline` action entry (`action_pipeline/action.rs`, `run_from_input`), with and without a Trigger continuation, with and without movement.
- Retry (`action_pipeline/retry.rs`): `RetryMode::ReNarrate` (main and event), `ReImpersonate`, `UserRegen`, and guided Retry.
- Retrigger (`action_pipeline/retrigger.rs`).
- On-demand Options and the Options rewrite after a generation (`action_pipeline/options.rs`).
- Narrator-mode Messages, if they take a different path.
- Arrival narration (`application/arrival_service.rs`, `bootstrap/init_game.rs`) and the starting-scenario Narration.
- Swipe switch (`MessageService::switch_swipe`): it saves no new generation, but record what it writes.

For each path, record:

1. The phase sequence (`PipelineRun`, `NarrationGeneration`, `phase_*` functions).
2. Each Snapshot save: the function that saves it (`persist_snapshot_or_err`, `save_message_and_snapshot`, other), the label, and the state it holds (before or after the Quantifier, the engine commit, the Trigger, the Options rewrite).
3. Each pointer it sets: `Message.snapshot_id`, `Swipe.snapshot_id`, and the "set only while `None`" rule in `MessageService::save_message_and_snapshot`.
4. What a failure in each phase leaves behind (saved rows, status, pointers).

## Context

- Seed: [three pipeline defects with one root](../assets/seed-evidence.md) §1.
- Changed since the seed: [Make event Retry restore the trigger context and never roll the game back](../../dashboard-ui-review-phase2/issues/09-fix-event-retry-snapshot.md) moved event Retry to the event Swipe's own Snapshot. Read its Answer.
- [List the game-state writers and the slot each holds](05-research-game-state-writers.md) asks who may write. This ticket asks what each path writes. Do not duplicate the slot analysis.

## Output

An asset `../assets/research-04-generation-snapshots.md`: one sequence per entry path in the seed's text-diagram style, then one table of every save site (path, phase, label, state held, pointer set). Label each fact known (read in the code), inferred, or tested. Cite symbols, not line numbers.

## Feeds

[Model a generation's states, its Snapshot pointers and the game-state writers](01-model-a-generations-states.md).

## Answer

Asset: [research-04-generation-snapshots.md](../assets/research-04-generation-snapshots.md). Facts are known from the code unless marked inferred. None is newly tested.

- **A generation's own pointer is set once, early.** A Narration or Input points at "pre-quantifier narration" (before the Quantifier). An event Message or event Swipe points at "post-trigger snapshot" (before the post-trigger reconciliation). No pointer holds the state after reconciliation or after the Options rewrite. The seed's §1 holds.
- **Seed correction:** a free-action Input points at the claim-time save in `GenerationGate::try_claim`, not at "pre-main snapshot". That Snapshot still holds the previous turn's `last_trigger`. Inferred: a main Retry restores it and never clears it, so Retrigger can re-fire an old Trigger.
- **Seed additions:** a best-effort save after the Quantifier and a best-effort save in the turn-end Options rewrite. 23 save sites in total, in a table in the asset.
- **Per path:** the asset has one sequence for each path: action (free, continue, guide, impersonate), Trigger continuation, the four Retry forms, Retrigger, on-demand Options, starting scenario, arrival and swipe switch. UserRegen runs no pre-main save and no tail. Narrator mode takes the same paths.
- **Arrival (revises seed §3):** it never persists `Generating`. It writes one Snapshot at the end, from the state it loaded at the start. Inferred: it runs on every server start for a World with no scenario text, and appends one more arrival Narration each time.
- **Failures:** a table in the asset. Inferred risk for ticket 05: `handle_cancellation` writes `Idle` into the game the player switched to.
