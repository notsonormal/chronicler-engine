# Specification: Core Architecture

**Status:** Completed

## Objective
Establish the foundational data structures and execution loop for the Chronicler Engine, built in Rust.

## Data Structures
1. **WorldCard (`world.rs`)**:
   - `name`, `description`, `global_rules` 
2. **MapDef (`map.rs`)**:
   - Contains a single `Overworld`, consisting of `Regions`, consisting of `Room`s.
   - `Room`s define `Direction` mapping exactly to adjacent `Room.id`s. Contains `items` and `npcs`.
3. **Character Cards (`character.rs`)**:
   - `PlayerCard`: `name`, `description`, `inventory`. 
   - `NpcCard`: Includes modern chat attributes (`personality`, `scenario`, `example_dialogue`), `name`, `inventory`.

## State & Execution
1. **GameState (`state.rs`)**: Aggregates all loaded cards, holds a Map of NPCs, and tracks the current loaded room ID.
2. **Actions & Parser (`action.rs`, `parser.rs`)**: Parses `look`, `go <direction>`, `talk <name>`, and `quit`.

## Testing Rules
Any extension to this structure (e.g., adding an `inventory` action, adding a `combat` module) must be accompanied by relevant unit tests in their local files or within a `/tests/` integration directory prior to implementation.
