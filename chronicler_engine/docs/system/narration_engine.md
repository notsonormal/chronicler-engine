# Specification: Game Master Narration System

## Objective
Transform the engine from a strict command parser into a hybrid free-text narrative engine. Player input that does not match a recognized system command is interpreted by an LLM acting as a **Game Master**, who narrates the outcome based on the current game state.

## System Commands vs Free Actions
The engine recognizes two categories of player input:

1. **System Commands**: Hard-coded actions that directly mutate engine state. These are not sent to the LLM. Examples: `look`, `inventory`, `quit`.
2. **Free Actions**: Everything else including navigation attempts. All player input goes to the LLM, which generates narration. The quantifier then detects if movement occurred. The engine must never respond with an error message for non-empty free-text input.

## Game Master Role
The LLM operates as a Game Master / Narrator for the text adventure. Its context window is constructed using the **PromptBuilder** (see `llm_processing.md`) from the current game state:

- **World Lore**: The `WorldCard.global_rules` provide persistent setting and lore context, injected into the **system prompt** (Layer 0) alongside the base rules. They no longer appear in the `<WorldLore>` user data layer.
- **Room Context**: The current `Room.name` and `Room.description` ground the scene.
- **Present NPCs**: All `NpcCard`s located in the current room, including their `personality`, `scenario`, and `description`.
- **Player Identity**: The `PlayerCard.name` and `PlayerCard.description` for reference.
- **Conversation History**: Full narration_history (up to 1000 entries) is sent to maintain continuity.

### Narrative Modes
The Game Master responds to three primary events:
1. **Free Actions**: Responding to non-command text input.
2. **Dialogue**: Responding to the legacy `talk` command.
3. **Arrivals**: Responding to the player entering a new room via quantifier-detected movement. 

### Arrival Logic Flow
1. **State Transition**: The engine validates the move and updates `state.current_room_id` (optional—may not change if player stays in room).
2. **Scene Setup**: The engine prints the standard room dashboard *before* narration to provide system context.
3. **Action Narration**:
   - The engine calls `llm_backend.narrate_action` with the player's action text.
   - The LLM generates a narrative paragraph describing the outcome.
4. **Post-Narration Quantification**:
   - After narration is generated, the engine calls `QuantifierBackendTrait::quantify_room()` to detect:
     - **NPCs**: Which NPCs are present in the generated narration text
     - **Movement**: If the narration indicates player moved to a new room (destination detection)
   - Falls back to `state.scene.npcs_in_area` (previous turn's NPCs) if LLM fails or returns Low confidence.
   - Set `LLM_BACKEND=mock` env var to use MockQuantifierBackend for testing.
5. **Movement Processing**: If movement was detected, `handle_movement()` updates `GameState.current_room_id` and the location header is shown.
6. **Trigger Evaluation**: After quantification, the engine evaluates NPC triggers (see Continuation Narration below).

## Continuation Narration (Auto-Trigger)

After the player moves to a new room and the first narration is generated, the engine checks for NPC triggers based on character state.

**Flow:**
1. Player movement is detected via quantifier → `attempt_semantic_walk` updates `GameState.current_room_id`
2. `evaluate_triggers(state, new_room_id)` is called to find matching triggers
3. For each matching trigger:
   a. Uses unified `PromptBuilder` with continuation context in user message:
      - Full 8-layer SillyTavern prompt structure
      - Trigger text as Layer 6 (User Input)
      - History included for continuity
   b. LLM generates continuation narration
   c. Continuation is appended to the narration log
   d. Non-repeatable triggers are marked as fired
4. `is_generating` is reset to `false` only after ALL trigger narrations complete

**NPC Event Layer:**
After the second quantifier runs (post-narration), the engine computes NPC enter/leave events by comparing previous vs current `npcs_in_area`:
- NPCs in current but not in previous → `Entered` event
- NPCs in previous but not in current → `Left` event

These events drive character state updates:
- `Entered` → `set_currently_meeting(true)` + `increment_times_met()`
- `Left` → `set_currently_meeting(false)`

This means `times_met` only increments on **new encounters** (when an NPC enters the area), not simply when NPCs are present. If Carla follows you through 3 rooms, `times_met` increments once on first entry.

**Key behaviors:**
- `is_generating` stays `true` through both the first narration AND all trigger narrations
- Trigger narrations do NOT cause further movement — quantifier is skipped for them
- Only the first matching trigger is narrated per user action (prevents runaway chains)
- If a trigger LLM call fails, the first narration still displays; error is logged

**Trigger condition example:**
- `TimesMet Eq 0` — fires on first encounter (when `times_met` is 0)
- `TimesMet Lt 3` — fires on encounters 0, 1, 2 (while `times_met < 3`)

## LLM Prompts & Guidance
The Game Master must:
- Narrate outcomes immersive, concisely, and in literary fiction style.
- Voice NPCs that would logically react to the player's presence or actions.
- Never act or speak on behalf of the player.
- **Arrival Instruction**: "The player has just entered the room. Describe their arrival and the scene. If NPCs are present, describe their initial reaction or current activity."

## Boundaries
The Game Master is **narrative only** in this spec. It does not mutate engine state (e.g., it cannot move items into inventory or change the player's room). State mutation via LLM function calling is deferred to a future specification.
