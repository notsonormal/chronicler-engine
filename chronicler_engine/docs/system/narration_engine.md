# Specification: Game Master Narration System

> **Related Decisions**: [ADR-005](../adr/adr-005-layered-prompts.md), [ADR-006](../adr/adr-006-quantifier-systems.md)

**Scope:** This document specifies the **Game Master's role and behavior** — what the LLM should do, how it should narrate, and what boundaries it must respect. For the pipeline that triggers the GM (phase sequence, status display, retry), see [`game_flow.md`](game_flow.md). For LLM infrastructure (backends, configuration, logging), see [`llm_processing.md`](llm_processing.md). For prompt composition and layer definitions, see [`prompt_system.md`](prompt_system.md).

## Objective
Transform the engine from a strict command parser into a hybrid free-text narrative engine. Player input that does not match a recognized system command is interpreted by an LLM acting as a **Game Master**, who narrates the outcome based on the current game state.

## Free Actions
All player input is treated as a **Free Action** and sent to the LLM for narration. The quantifier then detects if movement occurred. The engine must never respond with an error message for non-empty free-text input.

## Game Master Role
The LLM operates as a Game Master / Narrator for the text adventure. Its context window is constructed using the **PromptAssembler** (see `llm_processing.md`) from the current game state:

- **World Lore**: The `WorldCard.global_rules` provide persistent setting and lore context, injected into the **system prompt** (Layer 0) alongside the base rules. They no longer appear in the `<WorldLore>` user data layer.
- **Room Context**: The current `Room.name` and `Room.description` ground the scene.
- **Present NPCs**: All `NpcCard`s located in the current room, including their `personality`, `scenario`, and `description`.
- **Player Identity**: The `PlayerCard.name` and `PlayerCard.description` for reference.
- **Conversation History**: Full narrative history (up to `MAX_MESSAGES` = 1000 entries) is sent to maintain continuity.

### Narrative Modes
The Game Master responds to three primary events:
1. **Free Actions**: Responding to non-command text input.
2. **Dialogue**: NPC dialogue embedded within narration (no separate command).
3. **Arrivals**: Responding to the player entering a new room via quantifier-detected movement. 

### Arrival Logic Flow
1. **State Transition**: The engine validates the move and updates `state.movement.current_room_id` (optional—may not change if player stays in room).
2. **Scene Setup**: The engine prints the standard room dashboard *before* narration to provide system context.
3. **Action Narration**: The engine loads the active preset, calls `assembler.assemble()` to build the prompt, then calls `llm_backend.complete()` with the assembled system and user prompts. The LLM generates a narrative paragraph describing the outcome.
4. **Post-Processing**: The quantifier detects NPCs and movement, then triggers are evaluated — see [`game_flow.md`](game_flow.md) for the full phase pipeline and [`triggers.md`](triggers.md) for trigger evaluation rules.

## Continuation Narration (Auto-Trigger)

After main narration, the engine evaluates NPC triggers and may generate a continuation narration. This uses the same [7-layer prompt (with post-history splice)](prompt_system.md) with the trigger's `narration_prompt` in Layer 6 (User Input). Only the first matching trigger fires per action; `is_generating` stays true through both narrations. For the full trigger evaluation flow, `times_met` timing, NPC event semantics, and mutation order invariant, see [`triggers.md`](triggers.md).

## LLM Prompts & Guidance
The Game Master must:
- Narrate outcomes immersively and concisely, in literary fiction style.
- Voice NPCs that would logically react to the player's presence or actions.
- Never act or speak on behalf of the player.
- **Arrival Instruction**: "The player has just entered the room. Describe their arrival and the scene. If NPCs are present, describe their initial reaction or current activity."

## Boundaries
The Game Master is **narrative only** in this spec. It does not mutate engine state (e.g., it cannot move items into inventory or change the player's room). State mutation via LLM function calling is out of scope for this specification.
