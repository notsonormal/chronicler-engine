# Seed evidence: three pipeline defects with one root

Found by the Phase 4 reference-doc review (commit `28841e6f`) and the review of plan `docs/plans/doc-spec-boundary-follow-ups.md`. Each item says whether it is known from the code, inferred, or tested. None is tested yet.

## 1. A Swipe's Snapshot is set before the quantifier runs

**Known from the code.** `MessageService::save_message_and_snapshot` (`src/application/message_service.rs:62-90`) sets a Message's and a Swipe's `snapshot_id` only while it is `None`. The first save after narration sets it. Every narration Message and Swipe therefore points at the state before the quantifier.

```text
run_from_input  (one normal turn)
  phase_pre_main_snapshot     → S0  Input.snapshot_id = S0
  NarrationGeneration::run    → S1  Narration.snapshot_id = swipe.snapshot_id = S1   (set once)
  phase_post_generation       → S2  "pre-quantifier phase update"; then the quantifier runs
  engine commit               → S3  "post-engine snapshot": first state with the quantifier result
  trigger continuation?       → S4 "pre-event", S5 "post-trigger" (the event Message gets S5,
                                    before post-event reconciliation)
  turn-end Options rewrite, finalize
```

| Reader of the pointer | Gets now | Needs |
|---|---|---|
| `MessageService::switch_swipe` | S1 ✗ | S3 |
| Event retry (`find_retry_anchor_msg`, `is_event` branch: the narration Message is the anchor) | S1 ✗ (inferred: the continuation runs without the turn's NPCs and movement) | S3 |
| Guided retry (the guided Narration is target and anchor; history is cut before it) | S1 ✓ | S1 |
| Main retry (the Input is the anchor) | S0 ✓ | S0 |

One pointer cannot give guided retry S1 and swipe switch S3. Moving the pointer to S3 breaks guided retry: the redo starts in the room the old turn moved to.

The design says each Swipe holds the state its own narration produced, including the quantifier's updates: `docs/diataxis/explanation/storage_design.md` §Per-swipe state binding and `docs/diataxis/reference/game_flow.md` §Retry Flow ("Each Message and Swipe points to the Snapshot representing its resulting state"). The owner ruled the code wrong, not the docs.

Tests that would go red: a swipe switch after a retry whose quantifier moves the player (cover swipe 0 and the retry swipe); an event retry whose Trigger needs an NPC the main quantifier detected.

## 2. Re-impersonate retry and the quantifier

**Known conflict, behaviour not tested.** `docs/specs/swipe_new.md` I.6 and Scenario 22.1 say a re-impersonate retry does not re-run the quantifier. `ActionPipeline::retry` (`src/application/pipeline/action_pipeline/retry.rs`, `RetryMode::ReImpersonate`) calls `retry_main_narration`, whose comment says "Redo = fresh Impersonate … with the full tail (quantifier → engine commit → trigger)". 22.1 asserts "the player has not moved", which may pass only because its mock quantifier returns no movement (inferred).

## 3. Arrival narration writes game state without the generation slot

**Known from the code; the race is inferred.** At startup, for a Game whose World has no scenario text, `bootstrap::run` calls `spawn_arrival_task_if_needed` (`src/bootstrap/init_game.rs`), which runs `ArrivalTaskContext::run` with `spawn_blocking`. It persists `Generating` but claims no `GenerationGate` slot.

```text
ArrivalTaskContext::run_inner
  load state, persist Generating        (no slot)
  … LLM call …
                                        POST /action
                                          claim_and_spawn
                                            heal_stale_if_changed  ← Generating with no slot → reset to Idle
                                            try_claim → Started; the turn runs and saves
  save_message_and_snapshot(state)      ← saves a Snapshot from the state loaded before the turn
```

The status display reads the registry, so it shows idle during arrival. The last save carries a state older than the player's turn: a lost update. The test harness (`src/test_support/`) never runs arrival, so no HTTP test covers this path.

## The shared root

Game state is saved from 17 sites (`persist_snapshot_or_err`, `save_message_and_snapshot`). No document states:

- which states a turn has (its rewind point and its result) and which pointer holds each;
- which part of a turn each retry form re-runs (main retry, event retry, guided retry, re-impersonate, user regen, Retrigger);
- who may write game state, and when (the slot owner only?).
