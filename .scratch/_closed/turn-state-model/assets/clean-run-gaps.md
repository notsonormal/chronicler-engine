# Gaps from the three walled clean designs

Ticket: [Run three walled clean designs from the spec](../issues/20-run-walled-clean-designs.md). Input of [Answer the clean agents' questions and close the spec gaps](../issues/21-close-clean-spec-gaps.md).

Designs: [GLM](clean-design-glm.md) (G), [Kimi](clean-design-kimi.md) (K), [Opus](clean-design-opus.md) (O). Spec: [`clean-spec.md`](clean-spec.md), as signed off in ticket 19.

This file holds no owner answers. Ticket 21 writes them in `clean-spec-rulings.md`.

## How to read a row

- **Raised by** names the designs that made the point an assumption (A) or a question (Q). A design that chose without a word is not listed there. Its choice is still in its column.
- **Agree** means that all three designs chose the same observable behaviour. **Split** means that at least two designs differ in what a player or an HTTP client sees.
- O is from the spec writer's family (Opus). A gap that only G or K shows still counts (ticket 20, Models).
- Section numbers in the design columns point into that design.
- Each cell states what the player or the client sees. Records and mechanisms are named only where the behaviour depends on them.

## Summary

| Group | Rows | Agree | Split, spec conflict, open, or in one design only |
|---|---|---|---|
| A. The 17 open questions | 17 | 6 | 11 |
| B. Other gaps | 21 | 9 | 12 |

Two rows are conflicts inside the spec that only O found: B-04 (R-17 against R-65) and B-16 (R-01 against R-06).

## A. The open questions of spec §6

Each design answered every OQ. Only G left some open (OQ-05, OQ-06, OQ-11), and it gave a default for each.

| # | OQ | Raised by | G | K | O | Result |
|---|---|---|---|---|---|---|
| A-01 | OQ-01. Offered set after a swipe switch (R-36 against R-38) | G A, K A, O A + Q4 | R-38 wins: the dock is empty | R-38 wins | R-38 wins. O notes that the conflict shows only when the last Message is an Input, or a Narration with the always-on toggle on | Agree |
| A-02 | OQ-02. A Retry whose redo fires a Trigger | G A, K A, O A + Q3 | An event Narration is added as a new Message | Same | Same. O asks to confirm, because this drops the source rule "a Retry never adds a Message" | Agree |
| A-03 | OQ-03. Notices for an analysis fallback and for a new room; does an Options failure fail the generation? | G A, K A, O A | Fallback: a notice. New room: no notice. Options failure: the status stays idle | Fallback: no notice. New room: no notice. Status idle | Fallback: no notice (the header banner shows a degraded Quantifier). New room: no notice. Status idle | Split on the fallback notice |
| A-04 | OQ-04. Is a notice a Message? | G A, K A, O A | No. It never changes the last Message or the controls | Same | Same | Agree |
| A-05 | OQ-05. A Retry when the story holds no Input (R-44 against R-40) | G Q, K A, O A + Q5 | Default: a Retry is allowed whenever the last Message holds a generation or a typed Input. 400 only for an empty story or a story that ends in the opening. This departs from R-44 as written | R-44 as written: 400. With an Input anywhere in the story, a continue or a guided Narration can be redone | Same as K | Split |
| A-06 | OQ-06. An Input whose Narration failed | G Q, K A, O A | The Input stays. A Retry on it is a user regen. G offers a special case if the owner wants a Retry to try the Narration again | Same, no special case | Same. A continue writes the Narration that answers it | Agree |
| A-07 | OQ-07. `POST /status/reset-generating` | G A, K A, O A | Kept. Busy: "failed", and the work goes on. Idle: "reset", and the failure line is cleared | Kept. Busy: the work is cancelled, and the Game becomes idle. No Game: "failed" | Kept. Busy: "failed", and the work goes on. No Game: "failed". Otherwise "reset", and the failure line is cleared | Split: K cancels running work |
| A-08 | OQ-08. Edit, delete and swipe switch on a busy Game | G A, K A, O A | All three refused with `200` "Still thinking..." | All three refused with `409 Conflict`. Spec §2 gives no `409` | Edit and delete: `200` "Still thinking...". Swipe switch: the unchanged story log | Agree to refuse. Split on the answer |
| A-09 | OQ-09. The player leaves a busy Game | G A, K A, O A | Switch or create: the work finishes for its own Game, and coming back shows its status. Reset: the work's writes are dropped | Same | Same | Agree |
| A-10 | OQ-10. A World with no opening text | G A, K A, O A | An opening Narration with empty text. No LLM opening | Same as G. A Retry then answers 400 | An empty story, with no opening Message. No LLM opening. A continue writes the first Narration. R-77 says that the fresh story holds the opening Narration | Split: an empty Message or no Message |
| A-11 | OQ-11. Delete and the game state | G Q, K A, O A | Default: a delete changes only the story. The room, encounters, fired Triggers, trigger context and offered set stay. Any Message kind, the opening too | The game state goes back to the state after the new last Message, the offered set included. Any Message kind | The game state goes back to the state after the new last Message. The offered set is emptied. Any Message kind, the opening too | Split: G keeps the state, K and O roll it back. K and O also split on the offered set |
| A-12 | OQ-12. Cut-off work after a restart | G A, K A, O A + Q7 | Not reported. The story shows what was saved, for example an Input with no Narration | Same | Same. O asks whether it must be reported | Agree |
| A-13 | OQ-13. Which requests clear a failure on the status | G A, K A, O A | Cleared by: new work, a swipe switch, the idle-reset route. Not by: an edit, a delete, a switch of Game, a restart (the status is saved) | Cleared by: new work, an edit, a delete, a swipe switch, a switch of Game, a reset, the idle-reset route | Cleared by: new work, a swipe switch, a delete, the idle-reset route, a restart (the status is in memory only). Not by: an edit, a switch of Game | Split, three ways. Edit, delete and restart differ |
| A-14 | OQ-14. How long a trigger context stays stored | G A, K A, O A | Until another Trigger fires, or the Game is removed. A saved event, a later Action and a delete do not clear it. So a Retrigger can write an event again after that event was saved and a later Narration came | Until another Trigger fires, or a delete or a swipe switch rolls the state back. A later Action does not clear it. So the same repeat as G is possible | The next analysed generation sets it to its own Trigger or to none. A saved event clears it. A delete of the event brings it back. So no repeat after a saved event | Split. G and K let a Retrigger write an event that already exists in the story (see B-05) |
| A-15 | OQ-15. The room report of an Impersonate that moves the player | G A, K A, O A | The Impersonate Input reports the room | The Input reports no room. The next Narration that moves the player reports a room. So the move of an Impersonate is never shown when no later move comes | The Impersonate Input reports the room | Split: K |
| A-16 | OQ-16. Does the opening report the starting room? | G A, K A, O A | Yes | Yes | No: no generation moved the player (R-22) | Split: O |
| A-17 | OQ-17. A swipe switch to the first Swipe of a typed Input | G A, K A, O A | Nothing is restored. The game state stays as the last work left it | The game state goes back to the state before the Input | The game state goes back to the state in which the player typed (the same as K) | Split: G |

## B. Other gaps

| # | Spec | Gap | Raised by | G | K | O | Result |
|---|---|---|---|---|---|---|---|
| B-01 | §1 Game state, R-36 | Does a swipe switch give back the trigger context and the fired one-shot Triggers? The §1 term puts both in the game state | — | Unclear. G keeps the trigger context in a separate slot. Its saved state has "the game state without the offered set" | Yes, both | Yes, both. So whether Retrigger is offered follows the active Swipe | Split (G unclear) |
| B-02 | §3.4, R-16, R-58 | When does a one-shot Trigger count as fired: when it fires, or when its event Narration is saved? | O A-1 | When it fires. After a failed event, a later Narration cannot fire it again. Only Retrigger brings the event back | Not stated | When its event Narration is saved. After a failed event, a later Narration can fire it again (§3.4: "can fire again while no event Narration for it has been written") | Split |
| B-03 | R-16 | Is "at most once in a Game" read against the current story? After a delete of the event Narration, may the same one-shot event be written again? | O A-10 + Q8 | Yes, through Retrigger: the trigger context stays | Yes: the delete rolls the fired Triggers back | Yes: the delete rolls the fired Triggers back. O asks to confirm | Agree on yes. The way differs |
| B-04 | R-17, R-65 | An Impersonate fires a Trigger, and its event Narration fails. R-17 offers Retrigger "on the last Narration", but the last Message is an Input, so R-65 refuses it | O Q1 | Not seen. G's §5.7 says that the last Message after an Impersonate redo is a Narration, which is wrong | Retrigger refused after a Retry on an Input | Default: R-65 holds. The event can come back when a later Narration fires the Trigger again (B-02) | Spec conflict. Only O saw it |
| B-05 | R-62, R-16 | May a Retrigger write the event of a Trigger whose event Narration is already in the story? R-62 asks only for a stored context and a last Message that is a Narration and not an event Narration | — | Yes (A-14) | Yes (A-14) | No (A-14) | Split |
| B-06 | R-07, R-64, §3.4 | The analysis of an event Narration reports a Trigger that fires. Does a second event follow? | G A3, O A-3 | No. The reported Trigger is ignored | No | No | Agree |
| B-07 | R-14, R-57, §3.2, §3.4 | How the analysis of an event Narration is kept from moving the player | G A3, O A-2 | The movement is removed before the world rules run | The movement is ignored | The movement is removed before the world rules run | Agree. The black-box contracts allow it |
| B-08 | R-11, R-18 | Is a user regen analysed? R-11 does not list it. R-18 says only that it does not move the player | G A2, K A21 | No analysis | No analysis | No analysis | Agree |
| B-09 | R-31, R-32, R-17 | The event Narration fails after a Narration was saved. Do R-31 and R-32 apply to the saved Narration? | — | No. The work ends in failure, and the offered set stays as it was, also with the toggle off | No. The work ends in failure | Yes. With the toggle on, an Options run makes a new set. The failure is still reported | Split |
| B-10 | R-30, R-31, R-32 | An Impersonate fires a Trigger, and its event Narration is saved. Do R-31 and R-32 apply to that event Narration? | K A18 | No. An Input-writing generation skips the Options step | Yes: an event Narration is a Narration | Yes | Split: G |
| B-11 | R-31 | After a Narration that fires a Trigger: one Options run after the event, or a run after each Narration? | O A-6 + Q9 | One run, after the last Narration | One run | One run | Agree. O asks to confirm |
| B-12 | R-31, R-32, R-40 | Do R-31 and R-32 apply to the Narration that a Retry writes? | G A9 | Yes | Yes | Yes | Agree |
| B-13 | R-20, R-02 | A free-text Action meets an invalid room. Does its Input stay in the story? | O A-7 | No. The room check runs before the Input is saved | Not stated | No. The room check runs before the Input is saved | Agree (K silent) |
| B-14 | R-44, R-45, R-46, R-65, R-70 | The order of the request checks. When a Game is busy and the request is also a user error, which answer comes first? | O A-11 | The 400 and 500 checks run first. Then "Still thinking..." | "Still thinking..." first | "Still thinking..." first | Split: G |
| B-15 | §2 | The answer of a request that acts on the active Game when no Game exists. R-45 and R-65 give 400 for Retry and Retrigger only | G A5, O A-12 | 400 for every route that acts on the active Game | Not stated | 400 for an Action and a swipe switch. The idle-reset route answers "failed" | Agree on 400 where stated |
| B-16 | R-01, R-06 | An offered option whose text starts with "/" (for example "/options") is a command by R-01. R-06 says that it goes in as free text | O Q2 | Not seen | Not seen | Default: the dock's request marks the text as an option, and the engine treats it as free text | Spec conflict. Only O saw it |
| B-17 | R-01 | Command parsing: is the text trimmed? Must "/options" match exactly? | O A-14 | Not stated | Not stated | Trimmed. "/options" exact. "/guide " and "/impersonate " are prefixes | O only |
| B-18 | R-27 | What "nothing to work from" means for `/options` | G A10, O A-12 | No Messages, or only an opening with empty text | Not stated | No Message has non-empty text | Agree in effect |
| B-19 | R-83, OQ-03 | Where a notice shows, and how long it stays. R-83 lets the design choose the place | G A10, K A, O | In a strip next to the story log. Cleared when the next work starts. Saved, so a restart keeps it | In a rail above the story log, and near the dock. Cleared by the next successful generation, or by a dismiss control that the spec does not ask for | In the dock. Cleared when the offered set is next written (a new set, a Narration, a swipe switch, a delete or a reset). Saved | Split. K adds a control |
| B-20 | §3.7 | What the header banner reads, and when an LLM role counts as degraded | G Q-4, O Q6 | Asks. Default: a role-health note that the LLM call log side gives | Not stated | Asks. Default: the last outcome of each role (failed, fell back or not available), in memory | Open in two designs |
| B-21 | §1 Game, §3.2, R-77, R-20 | Assumptions about the opening state and the black-box inputs | G A4, G A7b, G A8, O A-8, O A-9, O A-15 | A reset keeps the World and the Persona (A4). For a room that play added, "the Characters that can be in that room" is every Character of the World (A7b). A contradiction after the text was saved ends the work with a failure, and the text stays (A8) | Not stated | A reset keeps the World and the Persona (A-8). In the opening state, each starting Character was met once and is met now (A-9). An internal error ends the work with its own failure line (A-15) | Each point is in one or two designs only |

## Points that are not gaps

- **How much story a call reads.** G sends the whole story to the narrator and 8 Messages to the Quantifier and the Options call (G A1). K and O send the story before the redone Message. Spec §3.2, §3.3 and §3.5 leave the window to the design. Not a gap.
- **Where each design keeps the status and the busy mark.** G and K save them. O keeps them in memory. The observable difference is the restart row of A-13.
- **The failure lines.** O writes a line for each failure kind (O §3.3). R-21 leaves the words to the design.
- **G §2.3 against G §6.3.** G says that a Swipe text never changes, and then that an edit changes it. This is an error inside G, not a gap in the spec.
