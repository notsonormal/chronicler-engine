# Specification: Game Master Narration System

**Scope:** This document specifies the **Game Master's role and behavior** — what the LLM should do, how it should narrate, and what boundaries it must respect.

## Objective
Transform the engine from a strict command parser into a hybrid free-text narrative engine. Player input that does not match a recognized system command is interpreted by an LLM acting as a **Game Master**, who narrates the outcome based on the current game state.

## Free Actions
All player input is treated as a **Free Action** and sent to the LLM for narration. The quantifier then detects if movement occurred. The engine must never respond with an error message for non-empty free-text input.

## Game Master Role
The LLM operates as a Game Master / Narrator for the text adventure. Its context window is constructed using the **PromptAssembler** (see `llm_processing.md`) from the current game state:

- **World Lore**: The `WorldCard.global_rules` provide persistent setting and lore context, injected into the **system prompt** alongside the base rules. The `<WorldLore>` user data layer carries only `world.name` and `world.description`.
- **Room Context**: The current `Room.name` and `Room.description` ground the scene.
- **Present NPCs**: All `NpcCard`s located in the current room, including their `personality`, `scenario`, and `description`.
- **Player Identity**: The `PersonaCard.name` and `PersonaCard.description` for reference.
- **Conversation History**: Full narrative history (up to `MAX_MESSAGES` = 1000 entries) is sent to maintain continuity.

### Narrative Modes
The Game Master responds to three primary events:
1. **Free Actions**: Responding to non-command text input.
2. **Dialogue**: NPC dialogue embedded within narration (no separate command).
3. **Arrivals**: Responding to the player entering a new room via quantifier-detected movement. 

### Per-Action Flow
1. **State Transition**: The engine validates the move and updates `state.movement.current_room_id` (optional—may not change if player stays in room).
2. **Scene Setup**: The engine prints the standard room dashboard *before* narration to provide system context.
3. **Action Narration**: The engine loads the active preset, calls `assembler.assemble()` to build the prompt, then calls `LlmCallRecorder::complete()` with the assembled system and user prompts. The recorder routes through the configured `LlmProvider` transport, runs forensics + postprocessing, and returns a narrative paragraph describing the outcome.
4. **Post-Processing**: The quantifier detects NPCs and movement, then triggers are evaluated.

## Continuation Narration (Auto-Trigger)

After main narration, the engine evaluates NPC triggers and may generate a continuation narration. This uses the same layered prompt (with post-history splice) with the trigger's `narration_prompt` placed in the user input layer. Only the first matching trigger fires per action; this game's `GenerationSlot` stays `Generating` through both narrations.

## LLM Prompts & Guidance
The Game Master must:
- Narrate outcomes immersively and concisely, in literary fiction style.
- Voice NPCs that would logically react to the player's presence or actions.
- Never act or speak on behalf of the player.
- **Arrival Instruction (planned)**: Per-arrival room description guidance. Not currently present in the active system prompt preset (`data/prompt_presets/system/default.json`); to be added to the preset.

## Boundaries
The Game Master is **narrative only** in this spec. It does not mutate engine state (e.g., it cannot move items into inventory or change the player's room). State mutation via LLM function calling is out of scope for this specification.

## Document References

- [ADR-005: SillyTavern-Style Layered Prompt System](../adr/adr-005-layered-prompts.md) — layered prompt architecture
- [ADR-006: Quantifier-Driven Game Systems](../adr/adr-006-quantifier-systems.md) — quantifier detects NPCs + movement after narration
- [system/game_flow.md](./game_flow.md) — phase pipeline + status display + retry flow
- [system/llm_processing.md](./llm_processing.md) — LLM backends + configuration + logging
- [system/prompt_system.md](./prompt_system.md) — layered prompt composition + per-layer definitions
- [system/triggers.md](./triggers.md) — trigger evaluation rules + `NpcEncounterLog` + mutation order
