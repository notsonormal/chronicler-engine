# Rethink Messages and rooms: what a room on a Message is for, how it shows, and mapless Worlds

Type: grilling (HITL)
Status: resolved
Blocked by: —

## Question

What is the link between a Message and a room, what is it for, and how does the dashboard show it? Decide, as one model:

1. **One fact or two.** Today every Swipe state holds the room (`current_room_id` and the added rooms), and every Swipe also stores a room report (a copied room name). The story log shows the room report as a change marker. Keep the stored marker (target design D-19), derive the header from the Swipe states (the room differs from the room of the Message before), show the room on every Message, or another model.
2. **The exceptions.** The opening always reports its room (R-25a), and an event Narration never does (R-24). Do they stay rules, or do they follow from the model? The "is this a Retry of the opening?" fact is computed three times today (`opening.rs`, `gate.rs` `tail.count == 1`, `job.rs` `is_the_opening`). Ticket 19 Q10 handed it here.
3. **"Moved" versus "named a destination".** Today the Game rules report a room whenever the Quantifier names a destination, also the room the player is already in (`movement()` in `game_rules.rs`). Is that a move?
4. **Mapless Worlds.** Subplan A of the mapless effort shipped (`c27b8f65`). Subplans B (`docs/plans/subplan-b-quantifier-field-split.md`: the destination as an id and a name) and C (`docs/plans/subplan-c-mapless-enablement.md`: `has_map`, an optional room, no added rooms) were not built, but both still say "Decision-complete". Are mapless Worlds still a goal? If yes, what does a room mean in a World with no map, and what happens to the added rooms? If no, retire the two plans.
5. **What the room is for** apart from the header: the narrator prompt, the Trigger room condition, the Quantifier's Characters list, the sidebar location image.

The answer is the model, the requirement changes (R-22 to R-25a, D-19) and the spec scenarios. The build is a later slice.

## Context

- Ticket 19 (the shape of a generation) leaves the room report as it is today. It only makes one room lookup ("map room, else added room"). The Game rules interface of ticket 19 then gains or loses the room report by this answer.
- Prior work: `.scratch/turn-state-model/issues/03-decide-where-the-room-lives.md`; `.scratch/turn-state-model/assets/research-08-location-header.md` (the header is a change marker; subplan C is stale); target design D-19 and its rejected option ("a room report taken from the Swipe state cannot tell 'moved to the same room' from 'did not move'"); clean spec R-22 to R-25a.
- Old mapless parent plan: `old-docs/archived-plans/mapless-worlds-plan.md`.
- Skills: `/grilling`, `/domain-modeling`. Terms: `CONTEXT.md`. Design: `.scratch/turn-state-model/assets/target-design.md`.

## Answer

Resolved with the owner, 2026-10-20 (grilling, two rounds).

**The model.** The room is one fact: the room of the Swipe state. The stored room report (`message_swipes.room_report`, `Swipe.room_report`, its writers in `gate.rs` and `job.rs`, and `opening.rs`) is deleted. D-19 is replaced.

- **Q1 (one fact or two): derived change marker (option A).** The story log shows the room name on a Message when the room name of its active Swipe state differs from the room name of the Message before it.
- **Q2 (what is compared): the room name (option A),** not the room id. A name is the one fact that every World kind gives. Accepted cost: a move between two map rooms with the same name shows no header.
- **Q3 (the story window): the oldest Message of the story window always shows its room name (option A).** No read of the hidden Message before it.
- **Q4 (mapless Worlds): not followed up (owner).** The owner mentioned mapless Worlds, a growing map and the added rooms only as context. Subplans B and C stay as they are. Recorded under *Out of scope* on the map.

**What follows from the model, with no rule of its own:**

- The opening shows its room, because no Message is before it, also after a Retry of the opening. R-25a is no longer a rule. The three "Retry of the opening?" checks (`opening.rs`, `gate.rs` `tail.count == 1`, `job.rs` `is_the_opening`) go away, for the room.
- An event Narration cannot move the player, so it shows no room (R-24 follows).
- A swipe switch shows the room of the new active Swipe (R-25 follows).
- An Input holds the state of before the Action, so it shows no room. An Impersonate Input that moved the player shows its room.
- "Moved versus named a destination" (question 3) is moot: naming the room the player is in changes no name, so nothing shows. The Game rules do not change for it; `GameRulesOutcome::arrived_room_name` loses its room-report reader.
- What the room is for (question 5): the narrator prompt, the Quantifier prompt, the Trigger `room_id` condition and the sidebar read the Swipe state, not the room report. None of them changes.

**Behaviour that changes from today:** (1) naming the current room shows no header; (2) two added rooms with the same name show one header; (3) a move between two map rooms with the same name shows no header; (4) the top of the story window shows the room.

**Requirement changes.** R-22 to R-25a become two rules: the derived change marker, and the oldest Message of the story window. D-19 is replaced by the derived marker (its rejected option fell because "moved to the same room" is not a move). Target design §4.3 rows "Room shown on a Message", "Create, reset", "Input save", "Text save", "State update", and the §4.4 room-report column lose the room report.

**Spec scenarios.** 1.14, 9.2, 9.8, 10.8, 22.5 and the `swipe_switch.md` room scenario keep their results; steps on a Swipe that is not active ("its first swipe reports no room") become "after a switch to that Swipe, the Message shows …". New scenarios: the four behaviour changes above.

The build is a slice of the build plan for the response.
