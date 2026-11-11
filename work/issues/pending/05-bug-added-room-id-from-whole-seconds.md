# Two added Rooms in one second get the same id

Source: the sweep of implicit state machines, 2026-11-08 ([catalogue](../../wayfinder-maps/state-machines-next/assets/catalogue.md), row B9)

## The defect

`Room::new_dynamic` (`src/domain/model/map.rs:52`) makes the id of an added Room from the Unix time in whole seconds: `format!("dynamic_{timestamp}")`. `attempt_movement` (`src/domain/model/utils/game_rules.rs:127`) calls it when the Persona walks to a place that the map does not know. It then puts the Room into `SwipeState.movement.dynamic_rooms` under that id.

So two added Rooms that are made in the same second get the same id. The second one overwrites the first in `dynamic_rooms`, and the Persona can no longer go back to the first.

## Impact (inferred)

Low. One generation adds at most one Room, because the Quantifier result has one destination. A Narration with a real model call takes more than one second. A collision needs two generations in one lineage of Swipe states inside one second, which a fast or mocked model call can reach.

## Done when

- The id of an added Room is unique in its Swipe state, whatever the time.
- A test makes two added Rooms with the same clock second and finds both in `dynamic_rooms`.
- A decision about the old ids: the stored Swipe states hold ids of the form `dynamic_<seconds>`. They must still read.
