# Rethink Messages and rooms: what a room on a Message is for, how it shows, and mapless Worlds

Type: grilling (HITL)
Status: open
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
