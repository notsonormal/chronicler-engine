# Model a generation's states, its Snapshot pointers and the game-state writers

Type: grilling
Status: resolved
Blocked by: 04, 05, 06, 07, 11, 13, 14, 23

**Waiting on the clean-room route (ticket 17).** The owner ruled that this ticket's questions are implementation decisions, and a design from the spec may go another way. After [Compare the target design with the code](23-compare-target-design-with-code.md), this ticket gets a new question, is closed, or is ruled out of scope.

## Question

Which states does one generation pass through (Input, Narration, Quantifier, engine commit, Trigger continuation, Options rewrite), which Snapshot is its rewind point and which is its result, whether those are two separate pointers, what a Trigger continuation owns, and who may write game state while it runs? The answer is the written model the map's Destination asks for, at a level that lets the next tickets (retry tails, the writer rule, fix or rewrite) be stated precisely.

## Context

- **Seed:** [three pipeline defects with one root](../assets/seed-evidence.md). Read it first. Its S0–S5 diagram and its pointer table are the starting point.
- **Changed since the seed (2026-10-11).** [Make event Retry restore the trigger context and never roll the game back](../../dashboard-ui-review-phase2/issues/09-fix-event-retry-snapshot.md) changed one row of the seed's pointer table: event Retry now starts from the event Swipe's own Snapshot (S5), not the Narration's S1, and a failed redo restores the state from before the click. Swipe switch still reads S1. Read that ticket's Answer before you grill §1.
- **Related open question:** [Replace `System` Messages with an error channel outside the Message history](02-replace-system-messages.md). It changes what counts as the last Message of a generation, which the retry rules read. The two tickets can run in either order; record any dependency you find.
- **Waiting on this ticket:** [Decide where the room a generation arrives in lives](03-decide-where-the-room-lives.md). Its options need this ticket's answer about the rewind and result pointers.
- **Terms:** `CONTEXT.md` deprecates "turn"; use Message, Swipe, generation, Action. The map's title keeps "turn" for history.
- **Skills:** `/grilling` and `/domain-modeling` (one question at a time; update `CONTEXT.md` if a term changes).

## Done when

- The model is written as a linked asset under `../assets/` and checked against the code (each state and pointer cites the function that writes it).
- The map's remaining fog patches "Retry tails" and "The writer rule" are graduated into tickets, or kept as fog with a reason.

## Answer

Closed by the owner on 2026-10-12, in the grilling of [Decide where the new design of the game-state lifecycle makes sense](24-decide-fix-or-rewrite.md) (round 2, Q3), as [Compare the target design with the code](23-compare-target-design-with-code.md) recommended. The [target design](../assets/target-design.md) is the model: its records (§2), the states of a generation and of a work (§4.3, §4.4) and the writer rule (§3). [Research 23](../assets/research-23-design-vs-code.md) §4 names the function that writes each state today. The owner ruled the pointer questions to be implementation (ticket 17). The fog patches "Retry tails" and "The writer rule" were cleared by ticket 23.
