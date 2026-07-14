# System: Character State & Persistence

The `NpcEncounterLog` system tracks the player's relationship and history with every NPC in the world.

## Overview
Unlike the volatile `GameState` (which resets frequently), `NpcEncounterLog` represents the permanent records of the simulation's actors.

## Tracked Data

`NpcEncounterLog` records three per-NPC counters: `times_met` (incremented on entry after leaving), `trigger_fired` (set of non-repeatable trigger indices that have already executed), `currently_meeting` (set on room entry, cleared on exit).

## Document References

- [ADR-006: Quantifier-Driven Game Systems](../adr/adr-006-quantifier-systems.md) — quantifier drives `times_met` + `currently_meeting` updates
