---
diataxis: reference
title: Prompt System
---

## Overview

The engine assembles a structured prompt for every Game Master call from the active system-prompt preset plus the current game state. The system half carries XML-sectioned instructions. The user half carries XML-wrapped game data and the player's input.

## Layered Prompt Architecture

The prompt is a sequence of eight layers mapped from SillyTavern's Prompt Manager. Layers 3 and 7 are conditional — Layer 3 (`<PlayerCharacter>`) is dropped on an Impersonate generation, and persona data then reaches the prompt through the impersonate preset's template macros; Layer 7 (`<Guide>`) renders only on a Guided Generation. The remaining six layers are always present. A post-history splice of `<writing_style>` and `<output_format>` sits between Layer 5 and Layer 6.

| Layer | Name | SillyTavern Equivalent | Role | Content |
|-------|------|----------------------|------|---------|
| 0 | System | Main Prompt | System | XML-wrapped `<role>` and `<instructions>` from the active preset, plus `<global_rules>` injected from `world.json` |
| 1 | Game State | Context | User (data) | Current room name and description |
| 2 | NPC Cards | Character Description | User (data) | `<KnownNpcs>` condensed entry for every World NPC; `<NpcsInRoom>` full cards, only for NPCs in the current room |
| 3 | Player | Persona Description | User (data) | `<PlayerCharacter>` persona sheet |
| 4 | World Info | World Info / Lorebook | User (data) | `<WorldLore>` world name + description |
| 5 | History | Chat History | User (data) | `<ConversationHistory>` conversation history |
| 6 | User Input | User Message | User (data) | `<PlayerInput>` sanitized current player input |
| 7 | Guide | — | User (data) | Transient guide text on a Guided Generation; omitted otherwise |

## Section Definitions

| Section | Source | XML Tag | System / User |
|---------|--------|---------|---------------|
| **Role** | Preset `role` field | `<role>` | System |
| **Instructions** | Preset `instructions` field | `<instructions>` | System |
| **Writing Style** | Preset `writing_style` field | `<writing_style>` | User (post-history splice) |
| **Global Rules** | `world.json` `global_rules` array | `<global_rules>` | System |
| **Output Format** | Preset `output_format` field + response length | `<output_format>` | User (post-history splice) |

## Single-User-Message Mode

Some local/quantized models ignore or poorly handle the `system` role. Each connection carries a `single_user_message` toggle that merges the system and user prompts into one user message.

## Prompt Injection Sanitization

User input enters the engine as `<PlayerInput>` content. The assembler passes it through `sanitize_for_prompt`, which replaces any `{{…}}` pattern with `[FILTERED]`.

## Context Templates

The template engine substitutes template variables — the player persona's fields, the game's posture and the option count — into author-controlled preset fields at render time.

## Agent Prompts (Separate)

The quantifier and options agents build their own prompts, separate from the layered narrative stack.

## Prompt Presets

The prompt system stores editable presets of four types: system (the narrator voice), quantifier, impersonate and options. Each Game references one active preset per type, and every call reads the referenced preset fresh from storage.

## Document References

- [`../../explanation/prompt_system_design.md`](../../explanation/prompt_system_design.md) — why the prompt system is shaped this way: system/user separation and two-tier NPC cards.
