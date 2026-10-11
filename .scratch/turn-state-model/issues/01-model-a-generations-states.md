# Model a generation's states, its Snapshot pointers and the game-state writers

Type: grilling
Status: open
Blocked by: —

## Question

Which states does one generation pass through (Input, Narration, Quantifier, engine commit, Trigger continuation, Options rewrite), which Snapshot is its rewind point and which is its result, whether those are two separate pointers, what a Trigger continuation owns, and who may write game state while it runs? The answer is the written model the map's Destination asks for, at a level that lets the next tickets (retry tails, the writer rule, fix or rewrite) be stated precisely.

## Context

- **Seed:** [three pipeline defects with one root](../assets/seed-evidence.md). Read it first. Its S0–S5 diagram and its pointer table are the starting point.
- **Changed since the seed (2026-10-11).** [Make event Retry restore the trigger context and never roll the game back](../../dashboard-ui-review-phase2/issues/09-fix-event-retry-snapshot.md) changed one row of the seed's pointer table: event Retry now starts from the event Swipe's own Snapshot (S5), not the Narration's S1, and a failed redo restores the state from before the click. Swipe switch still reads S1. Read that ticket's Answer before you grill §1.
- **Related open question:** [Replace `System` Messages with an error channel outside the Message history](02-replace-system-messages.md). It changes what counts as the last Message of a generation, which the retry rules read. The two tickets can run in either order; record any dependency you find.
- **Terms:** `CONTEXT.md` deprecates "turn"; use Message, Swipe, generation, Action. The map's title keeps "turn" for history.
- **Skills:** `/grilling` and `/domain-modeling` (one question at a time; update `CONTEXT.md` if a term changes).

## Done when

- The model is written as a linked asset under `../assets/` and checked against the code (each state and pointer cites the function that writes it).
- The map's remaining fog patches "Retry tails" and "The writer rule" are graduated into tickets, or kept as fog with a reason.
