# Decide where the room a generation arrives in lives

Type: grilling
Status: resolved
Blocked by: 01, 07, 08, 23

**Waiting on the clean-room route (ticket 17).** Where the room lives is a choice inside the target design. After [Compare the target design with the code](23-compare-target-design-with-code.md), this ticket gets a new question, is closed, or is ruled out of scope.

## Question

The room name (the location header) is stored on the Swipe of the Narration that moved the player, and it is written back onto that Swipe after the Quantifier runs. The Swipe's Snapshot is from before the Quantifier, so a swipe switch shows the arrival room while it restores the room the generation started from. Where must the room a generation arrives in live, and how is it shown, so that the room name on screen and the state a swipe switch restores cannot disagree?

## User framing (2026-10-11)

- The user deferred the mismatch out of [Put the room name on the Narration that arrives in the room](../../dashboard-ui-review-phase2/issues/15-room-name-on-arriving-narration.md) (option B) and is not sure what the solution is.
- "Would be updating the previous message … that seems like a more complex issue." The user asks whether the way rooms work now is the best way.
- The user sees an overlap with [Replace `System` Messages with an error channel outside the Message history](02-replace-system-messages.md).

## Context

**How it works now** (known, from the code after ticket 15 of the phase-2 map):

- `GameState::log_movement_completion` (`src/domain/model/state/game_state.rs`) sets the header on the arriving Message and queues an `ArrivingLocationWrite` (`NarrativeState::arriving_location_write`). `MessageService::save_message_and_snapshot` writes it onto the already-saved Swipe row with `Storage::update_swipe_location_header`.
- `pending_location` remains only for the starting scenario's Narration.
- No prompt reads `location_header`. The room name is display-only, so unlike a `System` notice it does not reach an LLM.
- Before ticket 15, the header landed one Message late: the next Message took it (finding S6 of the [2026-10-11 re-review](../../dashboard-ui-review-phase2/re-review-2026-10-11.md)).

**The mismatch** (known, confirmed by a probe in ticket 15 and by its spec review):

- A Narration's Swipe points at S1, the Snapshot from before the Quantifier ([seed](../assets/seed-evidence.md) §1). A swipe switch restores S1, so the player is back in the start room while the header names the arrival room.
- Before ticket 15 that Swipe had no header, so the two did not disagree. Ticket 15 made the old pointer defect visible.
- Scenario 9.8 (`docs/specs/swipe_new.md`) checks the header after a swipe switch, but not the restored room.
- `docs/diataxis/explanation/storage_design.md` §Per-swipe state binding says each Swipe keeps the state its own narration produced. The code does not do this.

**Options to weigh** (starting points, not a decision):

| Option | Effect | Cost |
|---|---|---|
| A. Keep the header on the Swipe, and point the Swipe at its result Snapshot | Swipe switch restores the arrival room; the stored header agrees | The write-back onto a saved row stays; it needs ticket 01's two pointers, because guided Retry needs S1 |
| B. Derive the header from the Swipe's result Snapshot when the story log renders (show it when the room differs from the room before) | No stored header and no write-back; the header and the restored state cannot disagree | Needs a result pointer; the render reads Snapshots; old rows keep stored headers |
| C. Show the current room outside the Message history (status line or scene panel), as ticket 02 weighs for notices | The story log stores no room; nothing to keep in step | The story log loses where each scene took place |

**Overlap with ticket 02** (inferred): both are display data stored inside the Message history. A `System` notice and a location header are each written as part of a Message, but neither is story text. "[System] Entered unknown location" is both a movement artifact and a `System` notice. The two decisions can share one rule: what belongs to a Message, and what belongs to a channel next to the history.

Questions for the grilling: does the player need the room in the story log, or only the current room? If ticket 01 gives a generation a result pointer, is a stored header still needed? Do the event Message and the starting scenario follow the same rule? Does `pending_location` survive (`docs/plans/subplan-c-mapless-enablement.md` plans to delete it)? What do old Games with stored headers need?

## Related

- Waits for [Model a generation's states, its Snapshot pointers and the game-state writers](01-model-a-generations-states.md): options A and B need its answer about the rewind and result pointers.
- [Replace `System` Messages with an error channel outside the Message history](02-replace-system-messages.md): the overlap above. Record the dependency if one of the two decides the other.

## Done when

- The decision is recorded under `## Answer`, with the option chosen and the reason.
- The fix is graduated into the plan that this map ends in, or into a ticket.
- `storage_design.md` §Per-swipe state binding and `game_flow.md` §Movement Branch agree with the decision, or a ticket owns making them agree.

## Source

Ticket 15 of the phase-2 map ("Open (known)" in its Answer), the spec review of tickets 15 and 16 (2026-10-11), and the user's decision B.

## Answer

Closed by the owner on 2026-10-12, in the grilling of [Decide where the new design of the game-state lifecycle makes sense](24-decide-fix-or-rewrite.md) (round 2, Q3), as [Compare the target design with the code](23-compare-target-design-with-code.md) recommended. In the [target design](../assets/target-design.md) the room report is on the Swipe and is written with its Swipe state (D-19), so a swipe switch gives back the room and the report together. That is this ticket's option A without pointers. The doc follow-ups (`storage_design.md` §Per-swipe state binding, `game_flow.md` §Movement Branch) go to whatever builds the result. Whether this design replaces today's is ticket 24's verdict.
