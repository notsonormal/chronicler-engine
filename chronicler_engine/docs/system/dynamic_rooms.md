# System: Dynamic Pseudo-Rooms

Dynamic Rooms are a safety mechanism used when the LLM's narrative intent contradicts the static game map.

## Overview
Because the engine uses **Quantifier-Driven Movement**, the LLM might decide that the player successfully walked into a "hidden cellar" that was never defined in the `map.json`. 

Instead of failing or teleporting the player to a blank screen, the engine creates a **Pseudo-Room**.

## The Generation Logic
1. **Detection**: The Quantifier detects a movement intent but finds no matching `room_id` in the overworld.
2. **Creation**: `crate::engine::logic::create_dynamic_room` is called.
3. **Data Source**: The quantifier's extracted "Destination Name" is used as the room title.
4. **Description**: A generic or LLM-derived description is applied.
5. **Lifespan**: Dynamic rooms are stored in `state.dynamic_rooms` (a `HashMap`).

## Rationale
This system ensures the "Narrative remains Sovereign." If the LLM narrates that you are in a new place, the engine makes that place a reality, even if only temporarily.

## Limitations
- Dynamic rooms typically have no exits unless the Quantifier detects a way out later.
- They do not persist across full world reloads.
