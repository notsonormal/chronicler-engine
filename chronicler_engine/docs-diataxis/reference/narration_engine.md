---
diataxis: reference
title: Narration Engine
---

> **Diátaxis mode:** Reference. This document describes the Game Master's role and behavior as it is: how the LLM narrates a FreeAction, what context it sees, and what boundaries it must respect. The problem it solves for the reader is *look-up*: given a player action, what does the narrator receive, what does it produce, and which engine boundaries does the LLM not cross. Prompt-layer details live in `src/application/prompts/` and `data/prompt_presets/`.

## Overview

The narrator is an LLM acting as a Game Master for a text adventure. All non-empty player input that does not match a recognized system command is treated as a **Free Action** and sent to the LLM for narration. The narrator produces a single paragraph describing the outcome; the engine then runs the post-generation quantifier to detect NPCs and movement, evaluates NPC triggers, and may generate a continuation narration.

The narrator is narrative-only. State mutation is the engine's job, run through the action pipeline after the LLM has spoken.

## Game Master Context

The Game Master's prompt is built from the current game state at action receipt:

- **World lore** — `WorldCard.global_rules` injected into the system prompt; `<WorldLore>` user data layer carries only `world.name` and `world.description`.
- **Room context** — `Room.name` and `Room.description` for the player's current room.
- **Present NPCs** — `NpcCard`s located in the current room, with `personality`, `scenario`, and `description`.
- **Player identity** — `PersonaCard.name` and `PersonaCard.description`.
- **Conversation history** — full narrative history, up to the FIFO cap.

History is sent in full and trimmed oldest-first if it exceeds the cap. Token budget enforcement is deterministic and happens during assembly.

## Narrative Modes

The Game Master responds to three primary events:

1. **Free Actions** — non-command text input. The default mode for any non-empty input.
2. **Dialogue** — NPC speech embedded within a free-action narration. There is no separate "speak" command; NPC lines appear inside the narrator's paragraph when context calls for them.
3. **Arrivals** — the player entering a new room via quantifier-detected movement. The room dashboard is rendered before narration to provide system context.

## Per-Action Flow

For a single FreeAction:

1. **State transition** — the engine validates the move and updates `state.movement.current_room_id`. Optional; the room may not change if the player stays put.
2. **Scene setup** — the engine prints the standard room dashboard before narration.
3. **Action narration** — the engine loads the active preset, calls `assembler.assemble()` to build the prompt, then dispatches to the configured `LlmProvider` transport through `LlmCallRecorder::complete()`. The recorder runs forensics + postprocessing and returns a narrative paragraph.
4. **Post-processing** — the post-generation quantifier detects NPCs and movement, then triggers are evaluated.

## Continuation Narration (Auto-Trigger)

After main narration, the engine evaluates NPC triggers and may generate a continuation narration. The continuation uses the same layered prompt shape, but the trigger's `narration_prompt` is placed in the user input layer and the trigger's stored `StoredTriggerContext` provides the surrounding context. Only the first matching trigger fires per action. The game's `GenerationSlot` stays `Generating` through both narrations.

## Document References

- [ADR-005: SillyTavern-Style Layered Prompt System](../../docs/adr/adr-005-layered-prompts.md) — layered prompt architecture and per-layer placement.
- [ADR-006: Quantifier-Driven Game Systems](../../docs/adr/adr-006-quantifier-systems.md) — dual-LLM architecture; quantifier detects NPCs and movement after narration.
- [ADR-022: PromptAssembler Trait Decoupling](../../docs/adr/adr-022-prompt-assembler.md) — assembly decoupled from transport; preset-driven system prompt.
- [`./game_flow.md`](./game_flow.md) — phase pipeline + status display + retry flow.
- [`./agent_system.md`](./agent_system.md) — post-generation agent that detects NPCs and movement.
- [`./triggers.md`](./triggers.md) — trigger evaluation rules that produce continuation narration.
