# Chronicler Engine Prompt System

## Overview

The Chronicler Engine uses a layered prompt construction system inspired by SillyTavern's Prompt Manager. The system builds comprehensive context for game narration by combining game state, character information, world lore, and conversation history into a structured prompt sent to the LLM.

## Prompt Architecture: XML-Sectioned Instructions + XML Data

The engine follows a **Marinara-Engine-inspired pattern**:

- **Instructions are XML-sectioned** — The system prompt contains behavioral instructions (`<role>`, `<instructions>`, `<global_rules>`). Prose and structural constraints (`<writing_style>`, `<output_format>`) are appended after conversation history in the user message, where LLMs weight them most heavily due to recency bias. Two dynamic sections (`<global_rules>`, response length) are injected at assembly time.
- **Data is XML-wrapped** — External context (`<GameState>`, `<KnownNpcs>`, `<ConversationHistory>`, etc.) uses XML tags because it is *data*, not instructions.
- **Why sections?** Labeled content containers let users edit individual prompt aspects (role, rules, style, format) without rewriting the entire prompt. The imperative text inside each section remains plain.
- **Why not self-referential tags?** Tags like `<SystemPrompt>` or `<Role>` can trigger reasoning models (e.g., Gemma 4) to enter meta-analysis mode. The section tags (`<role>`, `<instructions>`) are content labels, not objects of analysis.

## Source

- **SillyTavern Docs**: <https://docs.sillytavern.app/usage/prompts/prompt-manager/>
- **SillyTavern GitHub**: <https://github.com/SillyTavern/SillyTavern>

## Layered Prompt System (with Post-History Splice)

The Chronicler Engine builds a layered prompt mapped from SillyTavern's Prompt Manager. A **post-history splice** adds `<writing_style>` and `<output_format>` between chat history and user input (see Post-History Splice Is Not a Layer below).

| Layer | Name | SillyTavern Equivalent | Purpose |
|-------|------|----------------------|---------|
| 0 | System | Main Prompt | XML-wrapped sections: role, instructions, global_rules |
| 1 | Game State | Context | Current room, present NPCs |
| 2 | NPC Cards | Character Description | In-room NPC character sheets |
| 3 | Player | Persona Description | Player persona and description |
| 4 | World Info | World Info / Lorebook | World name + description |
| 5 | History | Chat History | Full conversation history |
| 6 | User Input | User Message | Current player input |

### Post-History Splice Is Not a Layer

`<writing_style>` and `<output_format>` are rendered **after** `<ConversationHistory>` and **before** `<PlayerInput>` in the assembled user half. They are assembled as a separate string and spliced between the history and user-input layers; they are not a layer variant.

## Detailed Layer Descriptions

### Layer 0: System Prompt (Main Prompt)

- **Role**: System
- **Position**: Absolute (top)
- **Content**: Assembled XML sections from the active preset — role, instructions, global_rules (from `world.json`). Writing style and output format are split out and placed after conversation history (see Post-History Splice below).
- **Format**: XML-wrapped sections (see example below)
- **Example**:

  ```xml
  <role>
      You are an interactive fiction author...
  </role>

  <instructions>
      Input validation rules:
      - ...
  </instructions>

  <global_rules>
      - Rule 1: Be descriptive
  </global_rules>
  ```



### Layer 1: Game State

- **Role**: User (data)
- **Content**: Current room name, description, NPCs in the current room
- **Format**: XML-wrapped (`<GameState>... </GameState>`)
- **Example**:

  ```xml
  <GameState>
  Current Location: Grand Foyer

  A cavernous entrance hall with marble floors.
  </GameState>
  ```

### Layer 2: NPC Cards (Character Description)

- **Role**: User (data)
- **Content**: Two sections:
  - `<KnownNpcs>`: Condensed roster of **all** NPCs the player has met (name, location, 3-line summary)
  - `<NpcsInRoom>`: Full character sheets for NPCs **currently present** (name, description, personality, scenario, relationships)
- **Relationships**: For each in-room NPC, if they have `relationships` with other NPCs also present in the room, a `Relationships:` subsection is appended. Uses the `dynamic` text if non-empty, otherwise falls back to `static_text`.
- **Why two-tier**: The LLM needs awareness of off-screen characters to reference them or write introduction scenes, but full cards for every NPC would bloat the prompt. Condensed cards (~40-60 words) preserve identity and motivation without the bulk.

### Layer 3: Player Persona

- **Role**: User (data)
- **Content**: Player's character sheet
- **Format**: XML-wrapped (`<PlayerCharacter>... </PlayerCharacter>`)
- **Includes**: name, description, personality, scenario (rendered as `Background:`)

### Layer 4: World Info (Lorebook)

- **Role**: User (data)
- **Content**: World name and description
- **Format**: XML-wrapped (`<WorldLore>... </WorldLore>`)
- **Implementation**: Renders `world.name` and `world.description` only. `global_rules` live in Layer 0 (System Prompt) to reduce token waste.
- **Note**: No keyword-matching trigger exists. The doc previously described a keyword-driven lorebook; that mechanism is not implemented.

### Layer 5: Chat History

- **Role**: User (data)
- **Content**: Full conversation history (up to token limit)
- **Format**: XML-wrapped (`<ConversationHistory>... </ConversationHistory>`)
- **Note**: No summarization — all conversation retained and sent. Oldest entries are trimmed first if the context window is exceeded.

### Post-History Splice

- **Role**: User (instructions)
- **Position**: After `<ConversationHistory>`, before `<PlayerInput>`
- **Content**: `<writing_style>` and `<output_format>` sections from the active preset
- **Why here?** LLMs exhibit strong recency bias. Placing prose constraints and structural rules at the end of the context window — after all story data but immediately before the generation point — makes them significantly more effective than burying them at the top of the prompt in the system message. This matches Marinara Engine's proven prompt architecture.
- **Assembly**: Assembled directly from the active preset — no string splitting or delimiter transport is required.

### Layer 6: User Input

- **Role**: User (data)
- **Content**: The player's current message/action
- **Format**: XML-wrapped (`<PlayerInput>... </PlayerInput>`)

## System / User Separation

The `PromptAssembler` separates instructions from data to maximize compatibility with OpenAI-compatible APIs:

- **System half**: XML-sectioned instruction sections (Layer 0)
- **User half**: XML-wrapped data (Layers 1–5) + post-history splice + player input (Layer 6)

This separation ensures that reasoning models receive clear imperative instructions in the system role, while all external context stays in the user role.

## Token Budget Management

- **MAX_CONTEXT_TOKENS**: 32768 (fallback default; configurable per connection via `max_context_tokens`)
- **MAX_RESPONSE_TOKENS**: 2048 (fallback default)
- **MAX_HISTORY_TOKENS**: 16000
- **SAFETY_MARGIN_TOKENS**: 256 (reserved for token estimation error)
- **MIN_INPUT_BUDGET_TOKENS**: 512 (minimum space reserved for input)
- **Strategy**: Context-aware fitting — `fit_messages_to_context()` caps `max_tokens` dynamically and trims oldest history entries first to fit within the connection's configured context window.
- **No summarization** — maintains accuracy over compression
- **Estimation**: Character-based token estimation (simple and fast)

## Response Length Control

Response length guidance is appended inside the `<output_format>` section at assembly time:

```
Response Length:
flexible, based on the current scene. During a conversation, keep it concise
(under 150 words) to allow back-and-forth. For scene transitions, travel, or
plot developments, build content (above 150 words), but allow the player to react.
```

- **Source**: `AppSettings.response_length` (persisted in `settings.json`)
- **Default**: Flexible scene-adaptive guidance
- **Injection point**: Appended inside `<output_format>` section content at assembly time

## Context Templates

The engine supports a single template variable:

- `{{user}}` — substituted from the player persona's `name` at render time
- Used in prompt construction within `PromptAssembler`

No other `GameState`-derived variables are supported by the template engine today.

## Prompt Context Structure

The `PromptContext` struct bundles all data needed for prompt assembly. To reduce parameter count and improve clarity, NPC-related data is bundled into `NpcContext<'a>`:

```rust
pub struct NpcContext<'a> {
    pub all_npcs: &'a [NpcCard],      // Full NPC roster (known characters)
    pub npcs_in_area: &'a [NpcCard],  // NPCs currently present in the room
}

pub struct PromptContext<'a> {
    pub world: &'a WorldCard,
    pub room: &'a Room,
    pub npcs: NpcContext<'a>,         // Bundled NPC context
    pub persona: &'a PersonaCard,
    pub user_message: &'a str,
    pub history: &'a [MessageEntry],
    pub template_vars: TemplateVars,
}
```

The `make_prompt_context` helper function constructs `PromptContext` from individual parameters, taking 6 arguments (down from 7 after bundling NPC slices).

## World Info / Knowledge Base

- **Content**: World name and description. World `global_rules` appear in Layer 0 (System Prompt), not here.
- **Method**: Static rendering of `world.name` and `world.description`; no keyword matching or RAG.

## Character Card Format

Uses the same structure as SillyTavern character cards (Jailbreak format):

```json
{
  "name": "Character Name",
  "description": "Physical appearance, personality",
  "personality": "Behavior traits",
  "scenario": "Setting context",
  "example_dialogue": "Sample conversations"
}
```

- Stored in `NpcCard` and `PersonaCard` structures
- Stored as JSON in `data/characters/<world>/`

## Other Prompt Systems

### Quantifier Prompt (Separate)

The engine also uses a **quantifier prompt** — a separate secondary LLM call that runs *after* narration to analyze the scene. It determines which NPCs are present and whether the player moved. This is **not** part of the layered narrative prompt stack.

## Document References

- [ADR-004: XML-Structured LLM Prompts](../adr/adr-004-xml-prompt-format.md) — XML-sectioned instructions + XML-wrapped data; tags not objects of analysis
- [ADR-005: SillyTavern-Style Layered Prompt System](../adr/adr-005-layered-prompts.md) — layered prompt architecture
- [reference/system_prompt.md](../reference/system_prompt.md) — full system prompt text + section definitions
- [reference/quantifier_prompt.md](../reference/quantifier_prompt.md) — full quantifier prompt text

## References

- SillyTavern Prompt Manager: <https://docs.sillytavern.app/usage/prompts/prompt-manager/>
- SillyTavern Prompt Building: <https://docs.sillytavern.app/usage/prompts/prompt-building/>
- Prompt Assembly Pipeline: <https://deepwiki.com/SillyTavern/SillyTavern/3.3-prompt-assembly-pipeline>
