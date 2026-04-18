# Specification: Game Master Narration System

## Objective
Transform the engine from a strict command parser into a hybrid free-text narrative engine. Player input that does not match a recognized system command is interpreted by an LLM acting as a **Game Master**, who narrates the outcome based on the current game state.

## System Commands vs Free Actions
The engine recognizes two categories of player input:

1. **System Commands**: Hard-coded actions that directly mutate engine state. These are not sent to the LLM. Examples: `look`, `walk to <target>`, `inventory`, `quit`.
2. **Free Actions**: Everything else. The player's raw text is forwarded to the Game Master LLM for narrative interpretation. The engine must never respond with an error message for non-empty free-text input.

## Game Master Role
The LLM operates as a Game Master / Narrator for the text adventure. Its context window is constructed using the **PromptBuilder** (see `llm_processing.md`) from the current game state:

- **World Lore**: The `WorldCard.global_rules` provide persistent setting and lore context.
- **Room Context**: The current `Room.name` and `Room.description` ground the scene.
- **Present NPCs**: All `NpcCard`s located in the current room, including their `personality`, `scenario`, and `description`.
- **Player Identity**: The `PlayerCard.name` and `PlayerCard.description` for reference.
- **Conversation History**: Full narration_history (up to 1000 entries) is sent to maintain continuity.

### Narrative Modes
The Game Master responds to three primary events:
1. **Free Actions**: Responding to non-command text input.
2. **Dialogue**: Responding to the legacy `talk` command.
3. **Arrivals**: Responding to the player entering a new room (`Action::WalkTo`). 

### Arrival Logic Flow
1. **Movement Intent**: The player issues a navigation command.
2. **State Transition**: The engine validates the move and updates `state.current_room_id`.
3. **Scene Quantification** (NEW):
   - The engine calls `QuantifierBackend::quantify_room()` with a secondary LLM.
   - The quantifier uses a separate model (`QUANTIFIER_MODEL` env var, defaults to free model).
   - It dynamically determines which NPCs are present based on: previous room NPCs, recent conversation history, and the player's action.
   - Falls back to static `room.npcs` from map.json if LLM fails or returns Low confidence.
4. **Arrival Narration**:
   - The engine calls `llm_backend.narrate_arrival` with the dynamic `npcs_in_area`.
   - The LLM generates a narrative paragraph describing the entrance and NPC reactions.
5. **Scene Setup**: The engine prints the standard room dashboard *after* the narration to provide system context.

## LLM Prompts & Guidance
The Game Master must:
- Narrate outcomes immersive, concisely, and in literary fiction style.
- Voice NPCs that would logically react to the player's presence or actions.
- Never act or speak on behalf of the player.
- **Arrival Instruction**: "The player has just entered the room. Describe their arrival and the scene. If NPCs are present, describe their initial reaction or current activity."

## Boundaries
The Game Master is **narrative only** in this spec. It does not mutate engine state (e.g., it cannot move items into inventory or change the player's room). State mutation via LLM function calling is deferred to a future specification.
