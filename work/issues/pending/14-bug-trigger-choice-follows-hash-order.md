# The Trigger that fires follows the hash order of the Characters

Source: ticket 03 of the state-machine map, the apply ticket of the server Game state ([dispositions](../../wayfinder-maps/state-machines-next/assets/03-dispositions.md), row B3; scan 2 question 9)

## The defect

`docs/diataxis/reference/game_flow.md:137` says: "The first matching Trigger fires." The code does not define "first".

`fired_trigger` (`src/domain/model/utils/game_rules.rs:87-101`) takes the first result of `evaluate_triggers`. `evaluate_triggers` (`game_rules.rs:166-210`) loops over `characters.values()`, and `characters` is a `HashMap<String, CharacterCard>` (`GameWorld.characters`, `game_rules.rs:23`). The Job builds this map again for each read of the World bundle (`src/application/pipeline/job.rs:435-440`). A std `HashMap` gets a random hash seed for each instance. So the loop order of the Characters is random for each Job (known).

When the Triggers of two Characters match in the same generation, either one can fire. A Retry from the same start state can fire the other Trigger. Only one event Narration follows, and the other Trigger stays unfired until a later generation matches it again (inferred from `game_flow.md:137` and `game_rules.rs:87-90`).

## Impact (inferred)

Low to medium. A World with two Characters whose Triggers can match at the same time, such as two first-encounter Triggers in one Room, tells a different story on each run. A test that sets up two matching Triggers can pass or fail by chance.

## Done when

- One defined order picks the Trigger that fires, for example the order of the Characters in the World and then the Trigger index.
- `game_flow.md` names that order.
- A test with two matching Triggers of two Characters gets the same Trigger on every run.
