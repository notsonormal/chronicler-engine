---
diataxis: reference
title: Prompt System
---

## Overview

The engine assembles a structured prompt for every Game Master call from the active system-prompt preset plus the current game state. The system half carries XML-sectioned instructions. The user half carries XML-wrapped game data and the player's input.

## Layered Prompt Architecture

The prompt is a sequence of eight layers mapped from SillyTavern's Prompt Manager. Layers 3, 6 and 7 can be absent. Layer 3 (`<Persona>`) is dropped on an Impersonate generation. Persona data then reaches the prompt through the impersonate preset's template macros. Layer 7 (`<Guide>`) renders only on a Guided Generation.

A layer with a blank body renders nothing, not even its tags. A body is blank when it is empty or holds only whitespace.

A post-history splice of `<writing_style>` and `<output_format>` sits between Layer 5 and Layer 6.

Every card list renders in Character id order. Layer 2 sorts both of its lists, and the Quantifier and Options calls sort their own.

| Layer | Name | SillyTavern Equivalent | Role | Content |
|-------|------|----------------------|------|---------|
| 0 | System | Main Prompt | System | XML-wrapped `<role>` and `<instructions>` from the active preset, plus `<global_rules>` injected from `world.json` |
| 1 | Game State | Context | User (data) | Current room name and description |
| 2 | Character Cards | Character Description | User (data) | `<KnownCharacters>` condensed entry for every World Character, and `<CharactersPresent>` full cards for the Characters in the current room |
| 3 | Player | Persona Description | User (data) | `<Persona>` persona sheet |
| 4 | World Info | World Info / Lorebook | User (data) | `<WorldLore>` world name + description |
| 5 | History | Chat History | User (data) | `<ConversationHistory>` conversation history, cut as the History section says |
| 6 | User Input | User Message | User (data) | `<PlayerInput>` sanitized current player input |
| 7 | Guide | — | User (data) | Transient guide text on a Guided Generation. Omitted otherwise |

The narrator makes Layer 6 from the kind of the generation (see Narration System).

## History

The history cut drops whole Messages from the front of the history, in blocks. The cut walks the history from its first Message. It drops the oldest Messages when the kept Messages pass the history space, the tokens the history may fill. Between two cuts the history start stays where it is, while the history space stays the same.

The history space is the input limit, less every part of the prompt except the `<PlayerInput>` layer and the Guide, less a reserve for those two. A cap limits it. The input limit depends on the window of the Connection and on the response length of the call (see Prompt Presets). A change in the size of another part, such as the room or the Characters present, changes the space and can move the history start. An Input or Guide that still passes the input limit makes the assembler drop more whole Messages, as one block.

The cut knows the blocks it dropped. The marker `(History truncated to fit context window)` shows when Messages exist and none is kept. The sizes stay in `HistoryCut` and the token budget.

## Section Definitions

| Section | Source | XML Tag | System / User |
|---------|--------|---------|---------------|
| **Role** | Preset `role` field | `<role>` | System |
| **Instructions** | Preset `instructions` field | `<instructions>` | System |
| **Writing Style** | Preset `writing_style` field | `<writing_style>` | User (post-history splice) |
| **Global Rules** | `world.json` `global_rules` array | `<global_rules>` | System |
| **Output Format** | Preset `output_format` field | `<output_format>` | User (post-history splice) |

## Single-User-Message Mode

Some local/quantized models ignore or poorly handle the `system` role. Each connection carries a `single_user_message` toggle that merges the system and user prompts into one user message.

## Prompt Injection Sanitization

User input enters the engine as `<PlayerInput>` content. The assembler passes it through `sanitize_for_prompt`, which replaces any `{{…}}` pattern with `[FILTERED]`.

## Context Templates

The template engine substitutes template variables — the player persona's fields, the game's posture and the option count — into author-controlled text at render time. In the Narrator call, the text is the preset fields and the World, Room, Persona, Character and rule text. In the Quantifier and Options calls, it is the preset text and the card descriptions.

## Agent Prompts (Separate)

The quantifier and options agents build their own prompts, separate from the layered narrative stack.

## Prompt Presets

The prompt system stores editable presets of four types: system (the narrator voice), quantifier, impersonate and options. Each Game references one active preset per type. `PromptPresetService` gives the preset of a Game to the Narrator and to each hook, and every call reads the referenced preset fresh from storage. The preset service refuses a preset that breaks the rules below, in a save and in a seed file.

A preset carries its generation parameters: `max_tokens`, `temperature`, `top_p` and a free object `custom` for other samplers. A value that is not set is not sent. The free object cannot set `model`, `messages`, `stream` or `max_tokens`, and the refusal names the key. The engine keys always win over the free object in the request.

The Connection keeps a ceiling on `max_tokens` (`max_tokens_ceiling`). A call sends the lower of the preset value and the ceiling. A preset that sets no `max_tokens` asks for the default of 2048 tokens, and a Connection with no ceiling has no upper limit. A Narrator-role call can send less, because the response length cannot pass the free space in the context window. The Quantifier and Options calls have no such clamp.

An Options preset can set `option_count`, the number of Options. It must be at least 1, and no other preset type can set it. The count fills the `{{option_count}}` placeholder and caps the Options that the parser keeps. An Options preset with no count offers 3.

An OpenRouter call also sends a `session_id`, the same value for every call of one Game. Other providers do not read it, and a call with no Game sends none.

## Document References

- [`../../explanation/prompt_system_design.md`](../../explanation/prompt_system_design.md) — why the prompt system is shaped this way: system/user separation and two-tier Character cards.
- [`narration_system.md`](narration_system.md) — the prompt rules of each kind of generation.
