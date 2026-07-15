---
diataxis: reference
title: Prompt System
---

> **Diátaxis mode:** Reference. This document describes the layered prompt architecture as it is: the seven prompt layers, the post-history splice for prose and format rules, the system/user split for OpenAI-compatible APIs, the token budget components, the response-length dynamic injection, and the single template variable. The problem it solves for the reader is *look-up*: given a player action, what the LLM sees in each of the system and user message halves. The dynamic-injection shape lives in `./system_prompt.md`; transport and forensics live in `./llm_processing.md`; assembled preset text lives in `data/prompt_presets/`.

## Overview

The engine assembles a structured prompt for every Game Master call from the active system-prompt preset plus the current game state. The system half carries XML-sectioned instruction content. The user half carries XML-wrapped game data, followed by the conversation history, followed by a post-history splice of writing-style and output-format sections, followed by the player's input. Two dynamic sections (`global_rules` from `world.json` and response length from settings) are injected at assembly time. Token budget enforcement fits the assembled prompt into the connection's configured context window by trimming oldest history entries first.

## Layered Prompt Architecture

The prompt is a fixed sequence of seven layers mapped from SillyTavern's Prompt Manager. A post-history splice sits between Layer 5 and Layer 6 (see the next section).

| Layer | Name | SillyTavern Equivalent | Role | Content |
|-------|------|----------------------|------|---------|
| 0 | System | Main Prompt | System | XML-wrapped `<role>`, `<instructions>`, `<global_rules>` sections from the active preset |
| 1 | Game State | Context | User (data) | Current room name, description, present NPCs |
| 2 | NPC Cards | Character Description | User (data) | `<KnownNpcs>` condensed roster for all known NPCs; `<NpcsInRoom>` full cards for NPCs in the current room |
| 3 | Player | Persona Description | User (data) | `<PlayerCharacter>` persona sheet |
| 4 | World Info | World Info / Lorebook | User (data) | `<WorldLore>` world name + description |
| 5 | History | Chat History | User (data) | `<ConversationHistory>` full conversation history |
| 6 | User Input | User Message | User (data) | `<PlayerInput>` sanitized current player input |

Layer 0 is the only system-role layer; Layers 1–6 are user-role data.

### Post-History Splice (Between Layer 5 and Layer 6)

The `<writing_style>` and `<output_format>` sections are rendered into the user message after `<ConversationHistory>` and before `<PlayerInput>`. They are assembled as a separate string and spliced between the history and user-input layers. The splice is not a layer — the seven-layer table is unchanged; the splice is a position in the rendered user message.

## Per-Layer Content

### Layer 0: System

The system half is assembled from the active preset's `role` and `instructions` fields plus `world.json`'s `global_rules` (see `./system_prompt.md` for the section shape and dynamic-injection details). The dynamic response-length text is appended inside `<output_format>` at assembly time and belongs to the post-history splice, not the system message.

### Layer 1: Game State

Current room name, description, and inventory inside `<GameState>`. The room's static description is rendered with the template engine before wrap.

### Layer 2: NPC Cards

Two XML blocks:

- **`<KnownNpcs>`** — condensed roster of every NPC the player has met. Each entry carries the NPC name, an `(in room)` / `(elsewhere)` marker, and a summary drawn from `NpcCard.summary` (falling back to the first three lines of `description` if `summary` is empty). This is the LLM's awareness of off-screen characters.
- **`<NpcsInRoom>`** — full cards for NPCs in the current room only. Each entry carries `Description`, `Personality`, optional `Context` (rendered from `scenario`), and a `Relationships:` subsection listing only the partners that are also in the room.

Full cards are emitted only for present characters; condensed cards carry the rest.

### Layer 3: Player

`<PlayerCharacter>` containing the persona's name, description, personality, and background (rendered from `scenario`).

### Layer 4: World Info

`<WorldLore>` containing the world's name and description.

### Layer 5: Chat History

`<ConversationHistory>` carrying the full conversation history. The history is sent in full and trimmed oldest-first if it exceeds the history token budget.

### Layer 6: User Input

`<PlayerInput>` carrying the player's current message after sanitization (`sanitize_for_prompt` — see `./llm_processing.md`).

## System / User Separation

The `PromptAssembler` separates instructions from data for compatibility with OpenAI-compatible APIs:

- **System half** — XML-sectioned instructions from Layer 0.
- **User half** — XML-wrapped data from Layers 1–5, the post-history splice, and Layer 6.

## Token Budget Management

The budget components live in `src/application/narrative_prompt/budget.rs`; the authoritative cap values are there and are not restated here. Six components make up the budget:

- **Context window** — fallback default, overridden per connection via the connection's `max_context_tokens`.
- **Response cap** — fallback default response length ceiling.
- **History cap** — the conversation history slice ceiling.
- **System cap** — the system prompt ceiling.
- **Safety margin** — reserved against token-estimation error.
- **Minimum input budget** — the minimum reserved on the input side.

`fit_messages_to_context` is the enforcement function. It estimates tokens with character-based counting (approximately four characters per token), trims oldest history entries first if the assembled input exceeds the budget, caps `max_tokens` dynamically, and returns `EngineError::ContextOverflow` if the system message alone does not fit.

## Response Length Control

The user's selected response length (from `AppSettings.response_length`, persisted in the settings singleton) is appended inside the `<output_format>` section at assembly time, as a `Response Length:` heading followed by the configured guidance text. The default guidance is the scene-adaptive text shipped with the engine's default settings. The injection happens in the post-history splice, not in the system message.

## Context Templates

The template engine supports a single variable:

- **`{{user}}`** — substituted from the player persona's name at render time.

No other `GameState`-derived variables are supported. Unknown placeholders are left in place. Substitution of `{{user}}` in author-controlled preset fields happens before user input reaches the assembler; the `{{variable}}` pattern in user input is stripped by `sanitize_for_prompt` (see `./llm_processing.md`).

## Character Card Format

World-author data uses the SillyTavern character-card shape:

```json
{
  "name": "Character Name",
  "description": "Physical appearance, personality",
  "personality": "Behavior traits",
  "scenario": "Setting context",
  "example_dialogue": "Sample conversations"
}
```

`NpcCard` extends this with `id`, `summary` (3-line condensed form), and `relationships` (per-partner dynamic/static text). `PersonaCard` mirrors the same shape on the player side. Cards are stored as JSON files under `data/characters/<world>/` and hydrated at load time.

## Quantifier Prompt (Separate)

The engine also uses a quantifier prompt — a separate secondary LLM call that runs after narration to analyze the scene. It determines which NPCs are present and whether the player moved. It is **not** part of the layered narrative prompt stack. See `./quantifier_prompt.md` for its shape.

## Document References

- [ADR-004: XML-Structured LLM Prompts](../../docs/adr/adr-004-xml-prompt-format.md) — XML-sectioned instructions + XML-wrapped data; tags not objects of analysis.
- [ADR-005: SillyTavern-Style Layered Prompt System](../../docs/adr/adr-005-layered-prompts.md) — layered prompt architecture + post-history splice rationale.
- [ADR-022: PromptAssembler Trait Decoupling](../../docs/adr/adr-022-prompt-assembler.md) — assembly decoupled from transport; preset-driven system prompt.
- [`../explanation/prompt_system_design.md`](../explanation/prompt_system_design.md) — why the prompt system is shaped this way: system/user separation and two-tier NPC cards.
- [`./system_prompt.md`](./system_prompt.md) — assembled system prompt structure + section definitions + dynamic injection points.
- [`./quantifier_prompt.md`](./quantifier_prompt.md) — quantifier prompt architecture (separate secondary prompt).
- [`./llm_processing.md`](./llm_processing.md) — transport + sanitization + forensics; single-user-message mode; response sanitization.
- [`./narration_engine.md`](./narration_engine.md) — how the Game Master uses the layered prompt in the FreeAction flow.