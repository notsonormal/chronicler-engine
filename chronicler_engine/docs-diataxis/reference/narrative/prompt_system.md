---
diataxis: reference
title: Prompt System
---

> **Diátaxis mode:** Reference. This document describes the prompt system as it is: the seven-layer narrative prompt architecture, the assembled system message structure (preset sections plus dynamic injection), the post-history splice for prose and format rules, the system/user split for OpenAI-compatible APIs, the token budget components, the response-length dynamic injection, the single-user-message mode toggle, the prompt-injection sanitization, and the single template variable. The problem it solves for the reader is *look-up*: given a player action, what the LLM sees in each of the system and user message halves. The quantifier prompt lives in `./agent_system.md`; LLM transport and forensics live in `./narration_system.md`. Verbatim preset text lives in `data/prompt_presets/`.

## Overview

The engine assembles a structured prompt for every Game Master call from the active system-prompt preset plus the current game state. The system half carries XML-sectioned instruction content. The user half carries XML-wrapped game data, followed by the conversation history, followed by a post-history splice of writing-style and output-format sections, followed by the player's input. Two dynamic sections (`global_rules` from `world.json` and response length from settings) are injected at assembly time. Token budget enforcement fits the assembled prompt into the connection's configured context window by trimming oldest history entries first. Some local/quantized models ignore the `system` role; per-connection `single_user_message` mode merges system and user into one user message. User input is sanitized at render time to strip `{{variable}}` patterns before the prompt reaches the LLM.

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

The system half is assembled from the active preset's `role` and `instructions` fields plus `world.json`'s `global_rules` (see "Assembled System Message" below for the section shape and dynamic-injection details). The dynamic response-length text is appended inside `<output_format>` at assembly time and belongs to the post-history splice, not the system message.

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

`<PlayerInput>` carrying the player's current message after sanitization (see "Prompt Injection Sanitization" below).

## Assembled System Message

The system message is rendered by the assembler from four editable preset sections plus two dynamically injected blocks. Only three of the four preset sections land in the system message; the fourth (`writing_style`) is part of the post-history splice in the user message. Empty sections are omitted from the assembled prompt; if no sections are present, the assembler produces an empty system message.

### Section Definitions

The four preset sections and the two dynamic blocks:

| Section | Source | XML Tag | System / User | Required |
|---------|--------|---------|---------------|----------|
| **Role** | Preset `role` field | `<role>` | System | No |
| **Instructions** | Preset `instructions` field | `<instructions>` | System | No |
| **Writing Style** | Preset `writing_style` field | `<writing_style>` | User (post-history splice) | No |
| **Global Rules** | `world.json` `global_rules` array | `<global_rules>` | System | Dynamic |
| **Output Format** | Preset `output_format` field + response length | `<output_format>` | User (post-history splice) | No |

The system half of the message carries `<role>`, `<instructions>`, and (when present) `<global_rules>`. The user half carries the data layers plus the post-history splice for `<writing_style>` and `<output_format>`.

### Assembled Shape

The system message has the following shape (per-layer prose is from `data/prompt_presets/system/default.json`; reproduce verbatim by opening that file):

```xml
<role>
    You are an interactive fiction author with your own free will...
</role>

<instructions>
    Input validation rules:
    - ...

    State tracking rules:
    - ...

    Narrative rules:
    - ...
</instructions>

<global_rules>
    - Rule from world.json
    - Another rule from world.json
</global_rules>
```

Empty sections are dropped. The `<role>` and `<instructions>` sections render the preset fields through the template engine (which substitutes `{{user}}` — see "Context Templates" below) before wrapping.

### Dynamic Injection: Global Rules

Rules from the world's `global_rules` array are formatted as bullet points and wrapped in `<global_rules>`. They are inserted between `<instructions>` and the post-history splice in the rendered system prompt (i.e. after `<instructions>`, before `<output_format>` is rendered — the actual placement is "the third section of the system half"). An empty `global_rules` array produces no `<global_rules>` block.

### Dynamic Injection: Response Length

The user's selected response length from `AppSettings.response_length` is appended to the `<output_format>` content before wrapping:

```xml
<output_format>
    ...preset content...

    Response Length:
    <configured guidance text>
</output_format>
```

The default value shipped with the engine's default settings is the scene-adaptive guidance. The injection happens in the post-history splice; see "Post-History Splice (Between Layer 5 and Layer 6)" above for the splice position.

## System / User Separation

The `PromptAssembler` separates instructions from data for compatibility with OpenAI-compatible APIs:

- **System half** — XML-sectioned instructions from Layer 0.
- **User half** — XML-wrapped data from Layers 1–5, the post-history splice, and Layer 6.

## Single-User-Message Mode

Some local/quantized models ignore or poorly handle the `system` role. Each connection carries a `single_user_message` toggle:

- **`false` (default).** The system prompt is sent as the `system` message; the user text is sent as the `user` message.
- **`true`.** The system and user prompts are merged into a single `user` message with a `[SYSTEM]` prefix. The `system` field is omitted from the API payload.

The mode is per-connection, so different backends can use different strategies within the same `AppSettings`. The merge helper lives in `LlmProvider` (`merge_single_user_message`) and is invoked identically by the OpenRouter and Ollama adapters.

## Prompt Injection Sanitization

User input enters the engine as `<PlayerInput>` content. The assembler passes it through `sanitize_for_prompt`, which replaces any `{{variable}}` pattern (double curly braces enclosing an identifier) with `[FILTERED]`. Legitimate text passes through unchanged; single braces and empty/unclosed brace pairs are preserved.

Sanitization runs at render time only. Substitution of `{{user}}` in author-controlled preset fields happens before user input reaches the assembler. Output-side handling (response sanitization, including the Gemma 4 thinking-channel suffix workaround) lives in `narration_system.md`.

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

No other `GameState`-derived variables are supported. Unknown placeholders are left in place. Substitution of `{{user}}` in author-controlled preset fields happens before user input reaches the assembler; the `{{variable}}` pattern in user input is stripped by `sanitize_for_prompt` (see "Prompt Injection Sanitization" above).

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

The engine also uses a quantifier prompt — a separate secondary LLM call that runs after narration to analyze the scene. It determines which NPCs are present and whether the player moved. It is **not** part of the layered narrative prompt stack; see `./agent_system.md` for the quantifier's prompt shape and execution phases.

## Prompt Presets

The four editable sections are stored on `PromptPreset` records. The active preset id is held on `AppSettings.active_system_prompt_preset_id`. At assembly time, the assembler reads the preset fresh from storage; `AppSettings` holds only the active-id reference.

Default presets ship as `data/prompt_presets/system/default.json` and are protected from edit or delete. The dashboard's Prompt Presets tab provides the create/copy/set-active surface.

## Document References

- [ADR-004: XML-Structured LLM Prompts](../../../docs/adr/adr-004-xml-prompt-format.md) — XML-sectioned instructions + XML-wrapped data; tags not objects of analysis.
- [ADR-005: SillyTavern-Style Layered Prompt System](../../../docs/adr/adr-005-layered-prompts.md) — layered prompt architecture + post-history splice rationale.
- [ADR-022: PromptAssembler Trait Decoupling](../../../docs/adr/adr-022-prompt-assembler.md) — assembly decoupled from transport.
- [`../../explanation/prompt_system_design.md`](../../explanation/prompt_system_design.md) — why the prompt system is shaped this way: system/user separation and two-tier NPC cards.
- [`./agent_system.md`](./agent_system.md) — the quantifier prompt as a separate secondary prompt, hosted by the `QuantifierAgent`.
- [`./narration_system.md`](./narration_system.md) — LLM transport, sanitization (response side + Gemma 4 workaround), forensics, and runtime tracing.
