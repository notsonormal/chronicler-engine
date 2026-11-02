---
diataxis: reference
title: Prompt System
---

## Overview

The engine assembles a structured prompt for every Game Master call from an active preset of the Game plus the current game state. An Impersonate uses the impersonate preset, and every other generation uses the system preset.

The call carries a list of prompt parts. Each prompt part has a role and a text, and it goes to the model as one chat message. The system part holds XML-sectioned instructions. The history goes as one prompt part for each entry. The user parts carry the XML-wrapped game data and the player's input.

## Layered Prompt Architecture

The prompt is a sequence of nine layers mapped from SillyTavern's Prompt Manager. Layers 2, 7 and 8 can be absent. Layer 2 (`<Persona>`) is dropped on an Impersonate generation. Persona data then reaches the prompt through the impersonate preset's template macros. Layer 8 (`<Guide>`) renders only on a Guided Generation.

Layers 7 and 8 render nothing, not even their tags, when their body is blank. A body is blank when it is empty or holds only whitespace.

A post-history splice of `<writing_style>` and `<output_format>` sits between Layer 6 and Layer 7. It joins the last `user` message too.

Every card list renders in Character id order. Layers 1 and 6 each sort their list, and the Quantifier and Options calls sort their own.

The layers sit in these prompt parts, in this order:

1. The system part holds Layer 0.
2. The first user part holds Layers 1 to 3.
3. One prompt part for each history entry holds Layer 4.
4. Layers 5 to 8 and the post-history splice join the last prompt part of the list when it is a `user` part. That part is the last history entry, or the first `user` part when no entry is kept. Otherwise they make one new final `user` part. So the end of the list never has two `user` parts in a row that the blocks made. The assembler does not merge entries of one role. A cut can leave an Input as the first history entry, next to the first `user` part.

| Layer | Name | SillyTavern Equivalent | Prompt part | Content |
|-------|------|----------------------|------|---------|
| 0 | System | Main Prompt | `system` | XML-wrapped `<role>` and `<instructions>` from the active preset, plus `<global_rules>` injected from `world.json` |
| 1 | Known Characters | Character Description | first `user` | `<KnownCharacters>` condensed entry for every World Character: name and summary |
| 2 | Player | Persona Description | first `user` | `<Persona>` persona sheet |
| 3 | World Info | World Info / Lorebook | first `user` | `<WorldLore>` world name + description |
| 4 | History | Chat History | one prompt part for each entry | The conversation history, wrapped and cut as the History section says |
| 5 | Game State | Context | last `user` | `<GameState>` current room name and description |
| 6 | Characters Present | Character Description | last `user` | `<CharactersPresent>` full cards for the Characters in the current room |
| 7 | User Input | User Message | last `user` | `<PlayerInput>` sanitized current player input. Left out when its text is the last history entry already |
| 8 | Guide | — | last `user` | Transient guide text on a Guided Generation. Omitted otherwise |

The narrator makes Layer 7 from the kind of the generation (see Narration System).

## History

Each history entry is one prompt part, as in Marinara Engine. A Narration and an event Narration are `assistant` parts. An Input is a `user` part. A System entry is a `system` part. The text of an entry has no label: no `Narrator:`, no Persona name and no `System:` in front of it.

Tags wrap the entries:

- `<ConversationHistory>` opens at the start of the first history entry.
- `</ConversationHistory>` closes at the end of the second-to-last history entry.
- `<LastMessage>` and `</LastMessage>` wrap the last history entry.

A history of one entry has the last-message tags only. The tags are not stored. Each call builds them again, so the tags move on the next call.

The last history entry of an Input answer is the Input itself. So the assembler leaves out `<PlayerInput>` when its text is that entry. An Impersonate direction and an event continuation are not history entries, so they keep their block.

An empty history has no message. The first `user` message then holds the marker `(No history yet - this is the start of the conversation)` inside `<ConversationHistory>`.

The history cut drops whole Messages from the front of the history, in blocks. The cut walks the history from its first Message. It drops the oldest Messages when the kept Messages pass the history space, the tokens the history may fill. Between two cuts the history start stays where it is, while the history space stays the same.

The history space is the input limit, less every part of the prompt except the `<PlayerInput>` layer and the Guide, less a reserve for those two. A cap limits it. The input limit depends on the window of the Connection and on the response length of the call (see Prompt Presets). A change in the size of another part, such as the room or the Characters present, changes the space and can move the history start. An Input or Guide that still passes the input limit makes the assembler drop more whole Messages, as one block.

Each message also costs a few tokens of chat template (`MESSAGE_OVERHEAD_TOKENS`, an estimate). The cut counts that cost for each history entry and for each fixed message, so a long history of short entries still fits.

The cut knows the blocks it dropped. The marker `(History truncated to fit context window)` shows in the first `user` message, inside `<ConversationHistory>`, when Messages exist and none is kept. The sizes stay in `HistoryCut` and the token budget.

## Section Definitions

| Section | Source | XML Tag | System / User |
|---------|--------|---------|---------------|
| **Role** | Preset `role` field | `<role>` | System |
| **Instructions** | Preset `instructions` field | `<instructions>` | System |
| **Writing Style** | Preset `writing_style` field | `<writing_style>` | User (post-history splice) |
| **Global Rules** | `world.json` `global_rules` array | `<global_rules>` | System |
| **Output Format** | Preset `output_format` field | `<output_format>` | User (post-history splice) |

## Single-User-Message Mode

Some local/quantized models ignore or poorly handle the `system` role. Each connection carries a `single_user_message` toggle that merges every message of the call into one `user` message. A `system` part gets the header `[SYSTEM]` and an `assistant` part gets the header `[ASSISTANT]`. A `user` part has no header. An empty part is left out.

On Ollama with a Gemma 4 model, the provider adds the channel close to the end of the last message.

## The Record of a Call

The transport returns the payload that it sent, on success and on failure. `LlmCallRecorder` stores it in `raw_request_json`. So the LLM message shows the messages as the model got them, with the merged message of the toggle and the channel close. A new row leaves `system_prompt` and `user_prompt` empty. The LLM record dialog shows the messages of `raw_request_json`, and an old row falls back to its two columns. A call that fails before it sends anything stores an empty `raw_request_json`.

## Prompt Injection Sanitization

The assembler passes the `<PlayerInput>` layer through `sanitize_for_prompt`, which replaces each non-empty `{{…}}` pattern with `[FILTERED]`. History entries go to the model unchanged. In a Narration call, the current Input is the last history entry, and the call has no `<PlayerInput>` layer. So the filter does not touch that Input.

## Context Templates

The template engine substitutes template variables — the player persona's fields, the game's posture and the option count — into author-controlled text at render time. In the Narrator call, the text is the preset fields and the World, Room, Persona, Character and rule text. In the Quantifier and Options calls, it is the preset text and the card descriptions.

## Agent Prompts (Separate)

The quantifier and options agents build their own prompts, separate from the layered narrative stack.

## Prompt Presets

The prompt system stores presets of four types: system (the narrator voice), quantifier, impersonate and options. A default preset is read-only, and a copy of it is editable. Each Game references one active preset per type. `PromptPresetService` gives the preset of a Game to the Narrator and to each hook, and every call reads the referenced preset fresh from storage. `PromptPreset::validate` refuses a preset that breaks the rules below. A save and the seed at startup both call it.

A preset carries its generation parameters: the response length (`max_tokens`), the samplers `temperature` and `top_p`, and a free object `custom` for other samplers. The engine owns `model`, `messages`, `stream` and `max_tokens`, so `custom` cannot set them, and the refusal names the key. A key that the engine writes in the request replaces the same key of `custom`.

A Connection can set a ceiling on the response length (`max_tokens_ceiling`). A call asks for the preset value or the default (`DEFAULT_MAX_TOKENS`), and never more than the ceiling. A call of the narrator can ask for less, because the response length cannot pass the free space in the context window.

An Options preset can set `option_count`, the number of Options. The count must be at least 1, and no other preset type can set it. The count fills the `{{option_count}}` placeholder and caps the Options that the parser keeps. An Options preset with no count uses `DEFAULT_OPTION_COUNT`.

## Document References

- [`../../explanation/prompt_system_design.md`](../../explanation/prompt_system_design.md) — why the prompt system is shaped this way: system/user separation, two-tier Character cards, the history cut, and the layer order for the provider cache.
- [`narration_system.md`](narration_system.md) — the prompt rules of each kind of generation.
