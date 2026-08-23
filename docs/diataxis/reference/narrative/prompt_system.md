---
diataxis: reference
title: Prompt System
---

## Overview

The engine assembles a structured prompt for every Game Master call from the active system-prompt preset plus the current game state. The system half carries XML-sectioned instruction content. The user half carries XML-wrapped game data, followed by the conversation history, followed by a post-history splice of writing-style and output-format sections, followed by the player's input. Two dynamic sections (`global_rules` from `world.json` and response length from settings) are injected at assembly time. Token budget enforcement fits the assembled prompt into the connection's configured context window by trimming oldest history entries first. Some local/quantized models ignore the `system` role; per-connection `single_user_message` mode merges system and user into one user message. User input is sanitized at render time to strip `{{variable}}` patterns before the prompt reaches the LLM.

## Layered Prompt Architecture

The prompt is a sequence of eight layers mapped from SillyTavern's Prompt Manager. Layer 7 is conditional; the remaining seven layers are always present. A post-history splice sits between Layer 5 and Layer 6.

| Layer | Name | SillyTavern Equivalent | Role | Content |
|-------|------|----------------------|------|---------|
| 0 | System | Main Prompt | System | XML-wrapped `<role>` and `<instructions>` from the active preset, plus `<global_rules>` injected from `world.json` |
| 1 | Game State | Context | User (data) | Current room name, description, present NPCs |
| 2 | NPC Cards | Character Description | User (data) | `<KnownNpcs>` condensed roster for all known NPCs; `<NpcsInRoom>` full cards for NPCs in the current room |
| 3 | Player | Persona Description | User (data) | `<PlayerCharacter>` persona sheet |
| 4 | World Info | World Info / Lorebook | User (data) | `<WorldLore>` world name + description |
| 5 | History | Chat History | User (data) | `<ConversationHistory>` full conversation history |
| 6 | User Input | User Message | User (data) | `<PlayerInput>` sanitized current player input |
| 7 | Guide | — | User (data) | Transient steering instruction on guided turns; omitted on plain turns |

Layer 0 is the only system-role layer; Layers 1–7 are user-role data.

### Post-History Splice (Between Layer 5 and Layer 6)

The `<writing_style>` and `<output_format>` sections are rendered into the user message after `<ConversationHistory>` and before `<PlayerInput>`. They are assembled as a separate string and spliced between the history and user-input layers. The splice is not a layer; it is a position in the rendered user message.

### Conditional Layers

Two steering surfaces alter the prompt conditionally, without changing the base sequence.

- **`<Guide>` (Layer 7)** — on a guided turn, this final layer is rendered after `<PlayerInput>`, carrying a transient steering instruction.
- **`<PlayerCharacter>` drop** — on an impersonated turn, the `<PlayerCharacter>` layer (Layer 3) is omitted. Persona data reaches the prompt through the impersonate preset's template macros instead.

Both are transient per-turn conditions; neither is persisted as a history entry. See the AI Steering reference for the steering behavior.

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

The system message shape is illustrated by the default system preset at `data/prompt_presets/system/default.json`. Empty sections are dropped. The `<role>` and `<instructions>` sections render the preset fields through the template engine (which substitutes `{{user}}` — see "Context Templates" below) before wrapping.

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

The mode is per-connection, so different backends can use different strategies within the same `AppSettings`.

## Prompt Injection Sanitization

User input enters the engine as `<PlayerInput>` content. The assembler passes it through `sanitize_for_prompt`, which replaces any `{{variable}}` pattern (double curly braces enclosing an identifier) with `[FILTERED]`. Legitimate text passes through unchanged; single braces and empty/unclosed brace pairs are preserved.

Sanitization runs at render time only. Substitution of `{{user}}` in author-controlled preset fields happens before user input reaches the assembler. Response sanitization (including the Gemma 4 thinking-channel suffix workaround) is handled downstream after the LLM call returns.

## Response Length Control

The user's selected response length (from `AppSettings.response_length`, persisted in the settings singleton) is appended inside the `<output_format>` section at assembly time, as a `Response Length:` heading followed by the configured guidance text. The default guidance is the scene-adaptive text shipped with the engine's default settings. The injection happens in the post-history splice, not in the system message.

## Context Templates

The template engine substitutes these variables in author-controlled preset fields at render time:

- **`{{user}}`** — the player persona's name.
- **`{{persona_description}}`** — the player persona's description.
- **`{{persona_personality}}`** — the player persona's personality.
- **`{{persona_background}}`** — the player persona's background / scenario.

`{{user}}` is available to every preset. The `{{persona_*}}` macros carry the persona sheet into presets that need it, such as the impersonate preset's voice apparatus. Unknown placeholders are left in place. Substitution in author-controlled preset fields happens before user input reaches the assembler; the `{{variable}}` pattern in user input is stripped during prompt sanitization (see "Prompt Injection Sanitization" above).

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

The engine also uses a quantifier prompt — a separate secondary LLM call that runs after narration to analyze the scene. It determines which NPCs are present and whether the player moved. It is **not** part of the layered narrative prompt stack.

## Prompt Presets

The four editable sections are stored on `PromptPreset` records. A preset has a `PresetType` — `System` (the narrator voice), `Quantifier` (the post-generation scene analysis), or `Impersonate` (the player-persona voice). The active preset id of each type is held on `AppSettings` (`active_system_prompt_preset_id` for the narrator, `active_impersonate_prompt_preset_id` for impersonation). At assembly time, the assembler reads the selected preset fresh from storage; `AppSettings` holds only the active-id references.

An impersonated turn selects the impersonate preset in place of the system preset; the quantifier preset runs a separate secondary call and is selected independently. Default presets ship under `data/prompt_presets/<type>/default.json` and are protected from edit or delete. The dashboard's Prompt Presets tab provides the create/copy/set-active surface for each type.

## Document References

- [`../../explanation/prompt_system_design.md`](../../explanation/prompt_system_design.md) — why the prompt system is shaped this way: system/user separation and two-tier NPC cards.
- [`./agent_system.md`](./agent_system.md) — the quantifier prompt as a separate secondary prompt, hosted by the `QuantifierAgent`.
- [`./narration_system.md`](./narration_system.md) — LLM transport, sanitization (response side + Gemma 4 workaround), forensics, and runtime tracing.
- [`./ai_steering.md`](./ai_steering.md) — the `<Guide>` layer and the `<PlayerCharacter>` drop as steering surfaces, and the impersonate preset replacing the system preset.
