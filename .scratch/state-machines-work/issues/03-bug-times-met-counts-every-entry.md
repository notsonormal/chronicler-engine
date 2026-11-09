# `times_met` counts every entry, but its doc comment says the first one only

Status: needs-triage
Source: ticket 02 of the state-machine map, the second scan (scan 2, 2026-11-09): `scan2-domain.md`, *Encounter* ([catalogue](../../state-machines-next/assets/catalogue.md), row B4)

## The defect

The doc comment of `Encounter` (`src/domain/model/trigger.rs:39`) says: "`times_met`: Increments on first encounter (Entered from !currently_meeting)".

The code does not do this. `apply_character_transitions` (`src/domain/model/utils/game_rules.rs:147-155`) calls `increment_times_met` on every `Entered` event. A Character that leaves the Room and comes back gets a second count. The Scenario start sets `times_met = 1` directly (`src/domain/model/state/swipe_state.rs:76`), which is a second writer.

`docs/diataxis/reference/game_flow.md:146` says only that enter and leave events "update `currently_meeting` and `times_met`". It does not settle the count.

## Impact (inferred)

Low. A Trigger condition reads `times_met` (`trigger.rs:95-99`). A condition "`Eq` 1" fires only on the first meeting in both readings. A condition with a threshold above 1 fires after the second entry under the code, and after the second "first encounter" under the comment, which cannot happen. So the code is probably the intended rule and the comment is wrong. A World author who trusts the comment writes a Trigger that never fires.

## Done when

- One rule for `times_met`: it counts every entry, or it counts first meetings only.
- The doc comment at `trigger.rs:39` and `game_flow.md` state that rule.
- A test moves a Character out and back in, and checks `times_met` against the rule.
