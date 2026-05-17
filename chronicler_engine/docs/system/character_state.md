# System: Character State & Persistence

> **Related Decisions**: [ADR-006](../adr/adr-006-quantifier-systems.md)

The `NpcEncounterLog` system tracks the player's relationship and history with every NPC in the world.

## Overview
Unlike the volatile `GameState` (which resets frequently), `NpcEncounterLog` represents the permanent records of the simulation's actors.

## Tracked Data
- **`times_met`**: An integer incremented every time the player encounters an NPC after having left their presence.
- **`trigger_fired`**: A map of trigger indices to booleans representing non-repeatable triggers that have already executed for this NPC.
- **`currently_meeting`**: Whether the player is currently in the same room/session as the NPC. Set to `true` on entry, `false` on exit.

## Rationale: Why Track Persistence?
The Auto-Trigger system relies on these metrics to prevent repetitive narration.
- **Example**: A "First Encounter" trigger only fires if `times_met == 0`.

## Implementation
- **Storage**: `crate::model::trigger::NpcEncounterLog`
- **Mutators**: `crate::engine::trigger_eval::increment_times_met`, `mark_trigger_fired`, and `set_currently_meeting`.
