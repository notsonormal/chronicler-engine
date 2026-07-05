# System: Dynamic Pseudo-Rooms

> **Related Decisions**: [ADR-006](../adr/adr-006-quantifier-systems.md)

Dynamic Rooms are a safety mechanism used when the LLM's narrative intent contradicts the static game map.

## Overview
Because the engine uses **Quantifier-Driven Movement**, the LLM might decide that the player successfully walked into a "hidden cellar" that was never defined in the `map.json`. 

Instead of failing or teleporting the player to a blank screen, the engine creates a **Pseudo-Room**. The narrative remains sovereign: if the LLM narrates a new place, the engine makes that place real, even if only temporarily.

## The Generation Logic
1. **Detection**: The Quantifier detects a movement intent but finds no matching `room_id` in the overworld.
2. **Creation**: `crate::domain::engine::logic::create_dynamic_room` is called.
3. **Data Source**: The quantifier's extracted "Destination Name" is used as the room title.
4. **Description**: A generic or LLM-derived description is applied.
5. **Lifespan**: Dynamic rooms are stored in `state.dynamic_rooms` (a `HashMap`).

## Limitations
- Dynamic rooms typically have no exits unless the Quantifier detects a way out later.
- They do not persist across full world reloads.
