# Map: turn state model

Labels: wayfinder:map

## Destination

A written model of a turn's states, the writers of game state, and the part of a turn each retry form re-runs, checked against the code. It ends with one decision: targeted fixes or a rewrite of the turn pipeline. The way is clear when a plan can be written from the model with nothing left to decide.

## Notes

- **Plan, don't do.** Tickets produce decisions. The fixes go to a plan written after the map.
- **Domain:** the action pipeline (`src/application/pipeline/`), `MessageService`, `GenerationGate`, `arrival_service.rs`, and their storage of Messages, Swipes and Snapshots.
- **Seed:** [three pipeline defects with one root](assets/seed-evidence.md), from the doc-spec boundary work. Read it before charting.
- **Current docs:** `docs/diataxis/reference/game_flow.md`, `docs/diataxis/explanation/storage_design.md`, `docs/diataxis/explanation/two-state-channels.md`, and the Message, Swipe, Retry and Snapshot entries in `CONTEXT.md`. Specs: `docs/specs/swipe_new.md`, `swipe_switch.md`, `retrigger.md`, `actions.md`.
- **Prior work:** the closed map [guided generations architecture](../_closed/guided-generations-architecture/map.md) reshaped narration turns and steering replay.
- **Skills:** `/grilling` and `/domain-modeling` for decisions and terms; `/codebase-design` for module shape.
- **Research first.** The research tickets (04–14) collect the code facts before the grilling tickets. Each writes `assets/research-NN-<slug>.md` and labels each fact known, inferred, or tested. A grilling session reads the assets of the tickets that block it, and does not grill the user on facts that the code answers.

## Decisions so far

- [Research: trace the Snapshots each generation path saves](issues/04-research-trace-generation-snapshots.md) — a generation's pointer is set once, before the Quantifier (Narration) or before reconciliation (event); a free-action Input points at the claim save with a stale `last_trigger`; arrival never persists `Generating`. [Asset](assets/research-04-generation-snapshots.md).
- [Research: list the game-state writers and the slot each holds](issues/05-research-game-state-writers.md) — many writers hold no slot, and every write goes to the process-global `Storage::game_id()`; nine concurrent pairs can lose an update; `heal_stale_if_changed` assumes the slot owner is the only writer of `Generating` and that its copy is current. [Asset](assets/research-05-game-state-writers.md).
- [Research: map what each retry form restores and re-runs](issues/06-research-retry-tails.md) — five Retry forms share one entry; re-impersonate runs the full tail by an owner decision, so `swipe_new.md` I.6 and 22.1 are stale; the main-Retry anchor is the last Input row in the history, which rewinds a continue or post-impersonate Retry to an earlier Action. [Asset](assets/research-06-retry-tails.md).
- [Research: list the readers of the Message history and its Snapshot pointers](issues/07-research-message-history-readers.md) — only three sites read a `snapshot_id` (`switch_swipe`, `retry_last_response`, `check_retry_anchor`); no HTTP path reads one, so every render shows the latest Snapshot; every history-carrying prompt reads `text` and `message_type` only, so all `System` Messages reach the LLM; `check_retry_anchor` validates a pointer the event redo does not restore. [Asset](assets/research-07-message-history-readers.md).
- [Research: trace the location header from write to render](issues/08-research-location-header-lifecycle.md) — the header is a change marker on the Swipe of the generation that moved, display-only; the mismatch is exactly that one Swipe class (its Snapshot is the pre-move state), reachable only through a Retry that left 2+ Swipes. Option B needs four missing pieces, not one: no Swipe points at the result Snapshot, there is no "room before" to diff against, name resolution is unreachable from HTTP, and the story log loads no per-Swipe Snapshot. Option B fixes the display only — option A is the only one that changes what `switch_swipe` restores. `pending_location` is a bootstrap seed only; `docs/plans/subplan-c-mapless-enablement.md` is stale and contradicts the shipped ticket-15 behaviour. [Asset](assets/research-08-location-header.md).

## Not yet specified

<!-- These patches come from the seed. "A turn's states" graduated into the first ticket. -->

- **Retry tails.** Facts in [research 06](assets/research-06-retry-tails.md). Open parts: the rewind point of a Retry (per Action, not the last Input row: continue and post-impersonate Retries break today); whether a main Retry clears `last_trigger`; the spec edits for re-impersonate (owner already chose the full tail) and for event Retry's reconciliation Quantifier; whether a Retry may add Messages (spec I.2 vs 9.5).
- **The writer rule.** Is the slot owner the only writer of game state? How does arrival fit (seed §3)? What does `heal_stale_if_changed` assume? Facts in [research 05](assets/research-05-game-state-writers.md). Open parts: which game a write belongs to (today the global `Storage::game_id()`, not the slot's game); whether handler writers (`edit_history`, `delete_last`, `switch_swipe`, `/status/reset-generating`) must take the slot; whether arrival must run on every boot of a World with no scenario text.
- **Fix or rewrite.** Decided after the model is checked against the code.

## Out of scope

- The doc and spec follow-ups in `docs/plans/doc-spec-boundary-follow-ups.md`.
- Options turn-end rules. That plan's Phase B pins them with tests.
- The story-log display cut. `AppState::render_story_log` takes the **first** 50 entries of an oldest-first history, so the Delete, Retrigger and Swipe controls attached to `loop.last` land on entry 50 once a Game passes 50 Messages and the newest entries render bare. Found by [research 07](assets/research-07-message-history-readers.md) and [research 08](assets/research-08-location-header.md). A read-side display defect, not a turn state or a writer — it does not belong on this route.
