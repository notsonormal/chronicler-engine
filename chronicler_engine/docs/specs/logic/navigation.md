# Specification: Semantic Navigation

## Objective
Migrate the engine's movement system from rigid, cardinal directions (`go north`, `go south`) to a semantic parsing model (`walk to kitchen`, `go to the grand hall`). 

## Changes to Player Actions
1. **Parser Additions**: The parser will understand the prefix `walk to [target]`, `go to [target]`. 
2. **The `WalkTo` Action**: Navigation is handled via the `Action::WalkTo(Target)` variant. The `Target` string is parsed from natural language input.

### Parsing Examples
- "Go north" -> `Action::WalkTo("north")`
- "Walk to the mansion" -> `Action::WalkTo("the mansion")`
- "Go to front_gates" -> `Action::WalkTo("front_gates")`

3. **Cardinal Directions**: Cardinal directions like "north" will still be supported via `Action::WalkTo("north".to_string())`.

## Resolution Algorithm
When the main engine loop evaluates `Action::WalkTo("target_string")`:
1. It iterates over all exits attached to the `current_room`.
2. For each exit mapped, it retrieves the destination `Room` data block from the overarching `MapDef`.
3. **Condition A: String Name Match:** Does the destination `Room.name.to_lowercase()` contain the user's `target_string`? If so, navigate to this room.
4. **Condition B: Cardinal Fallback:** If the `target_string` happens to match the `Direction` key of the exit map natively (e.g. they typed "north" and there is an `exits: { "north": "room2" }` link), navigate to this room.
5. If neither condition is met across any adjacent room, return `"You don't see a way to go there."`

This hybrid algorithm keeps the `.json` map files tightly connected via directional graphs (great for visual builders) while granting the Text Adventure feel to the end user.
