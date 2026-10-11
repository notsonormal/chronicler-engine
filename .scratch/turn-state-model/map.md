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

## Decisions so far

<!-- none yet -->

## Not yet specified

<!-- These patches come from the seed. "A turn's states" graduated into the first ticket. -->

- **Retry tails.** For each retry form, what it restores and what it re-runs. The re-impersonate conflict (seed §2) is one case.
- **The writer rule.** Is the slot owner the only writer of game state? How does arrival fit (seed §3)? What does `heal_stale_if_changed` assume?
- **Fix or rewrite.** Decided after the model is checked against the code.

## Out of scope

- The doc and spec follow-ups in `docs/plans/doc-spec-boundary-follow-ups.md`.
- Options turn-end rules. That plan's Phase B pins them with tests.
