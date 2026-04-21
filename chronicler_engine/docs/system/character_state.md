# System: Character State & Persistence

The `CharacterState` system tracks the player's relationship and history with every NPC in the world.

## Overview
Unlike the volatile `GameState` (which resets frequently), `CharacterState` represents the permanent records of the simulation's actors.

## Tracked Data
- **`times_met`**: An integer incremented every time the player initiates a dialogue or encounter with an NPC.
- **`last_room_id`**: The ID of the room where the player last saw this NPC.
- **`fired_triggers`**: A set of indices representing non-repeatable triggers that have already executed for this NPC.

## Rationale: Why Track Persistence?
The Auto-Trigger system relies on these metrics to prevent repetitive narration. 
- **Example**: A "First Encounter" trigger only fires if `times_met == 0`.
- **Example**: A "Returning Home" trigger only fires if `last_room_id` was the village and the current room is the estate.

## Implementation
- **Storage**: `crate::model::trigger::CharacterState`
- **Mutators**: `crate::engine::trigger_eval::increment_times_met` and `mark_trigger_fired`.
