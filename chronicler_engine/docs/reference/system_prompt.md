# Reference: System Prompt

> **Context**: This document describes the **assembled system prompt** sent to the narration LLM. For the overall architecture, see [`system/prompt_system.md`](../system/prompt_system.md).

The system prompt is assembled by `PromptPreset` sections and `build_system_prompt()` in `assembler.rs` from four editable sections, plus two dynamically injected blocks. It is rendered by `LayeredPromptAssembler` as a single system message.

> **Why XML sections?** Labeled content containers (`<role>`, `<instructions>`, etc.) give the LLM clear section boundaries without the self-referential meta-analysis risk of tags like `<SystemPrompt>`. The imperative text inside each section remains plain. See [ADR-004](../adr/adr-004-xml-prompt-format.md) for the full evolution.

---

## Assembled System Prompt Structure

```xml
<role>
    You are an interactive fiction author with your own free will...
</role>

<instructions>
    Input validation rules:
    - Treat the player's input as an attempted action...

    State tracking rules:
    - Track physical state: clothing, positions...

    Narrative rules:
    - Quality prose with natural dialogue...
</instructions>

<writing_style>
    Third-person limited perspective, focused on the player character.
    Past tense narrative prose.
</writing_style>

<global_rules>
    - No explicit content.
    - Another world-specific rule from world.json.
</global_rules>

<output_format>
    The player's next action is provided above. Your only job is to narrate what happens now.

    Do not re-narrate events that already occurred in the history above.

    Response Length:
    flexible, based on the current scene...
</output_format>
```

---

## Section Definitions

| Section | Source | XML Tag | Required |
|---------|--------|---------|----------|
| **Role** | Preset `role` field | `<role>` | No |
| **Instructions** | Preset `instructions` field | `<instructions>` | No |
| **Writing Style** | Preset `writing_style` field | `<writing_style>` | No |
| **Global Rules** | `world.json` `global_rules` array | `<global_rules>` | Dynamic |
| **Output Format** | Preset `output_format` field + response length | `<output_format>` | No |

Empty sections are omitted from the assembled prompt. If no sections are present, the assembled prompt is empty and the assembler uses an empty system message.

---

## Dynamic Injection

### Global Rules

Rules from `world.json` are formatted as bullet points and wrapped in `<global_rules>`:

```xml
<global_rules>
    - Rule 1: Be descriptive
    - Rule 2: Stay in character
</global_rules>
```

They are inserted **after** `<writing_style>` and **before** `<output_format>`.

### Response Length

The user's selected response length (from Settings) is appended to the `output_format` content before wrapping:

```xml
<output_format>
    ...preset content...

    Response Length:
    Keep responses under 200 words.
</output_format>
```

---

## Data Layers (User Message)

The following XML-tagged sections are sent in the **user message**, not the system prompt:

1. `<GameState>` — Current room, description, inventory
2. `<KnownNpcs>` — Condensed cards for all known NPCs
3. `<NpcsInRoom>` — Full cards for NPCs present in current room
4. `<PlayerCharacter>` — Player persona
5. `<WorldLore>` — World name and description
6. `<ConversationHistory>` — Truncated narration history
7. `<PlayerInput>` — Sanitized current user input

The `<writing_style>` and `<output_format>` sections are placed in the **user message** after `<ConversationHistory>` (Layer 6) to maximize recency bias. The system prompt (Layer 0) contains only `<role>`, `<instructions>`, and `<global_rules>`.

---

## Customization

Prompts are customized via the **Prompt Presets** tab in the dashboard:

1. Navigate to the **Prompt Presets** tab
2. Create a new System Prompt preset (or edit an existing non-default one)
3. Fill the four section fields: Role, Instructions, Writing Style, Output Format
4. Click **Set Active** to apply it

The active preset is stored in `AppSettings.active_system_prompt_preset_id`. At assembly time, `LayeredPromptAssembler::assemble()` loads the preset from storage, builds the system prompt from preset sections + global rules, builds the post-history prompt from writing style + output format + response length, renders all data layers, and calls `fit_messages_to_context()` to enforce the connection's token budget. No pre-assembly caching in `AppSettings` is required — the assembler reads the preset directly.

Default presets (shipped as `data/prompt_presets/system/default.json`) are protected and cannot be edited or deleted. To modify a default, create a copy and activate it.

---

## Sources

- System preset seed: `data/prompt_presets/system/default.json`
- Assembly logic: `src/domain/model/prompt_preset.rs` (`assemble_prompt_text()`)
- Prompt assembler: `src/application/narrative_prompt/assembler.rs` (`build_system_prompt()`, `build_post_history_prompt()`)
- Prompt preset storage: `src/adapters/driven/storage/prompt_preset_storage.rs`
- Dashboard UI: `src/adapters/driving/http/prompt_presets_fragment/`
