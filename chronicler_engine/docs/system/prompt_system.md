# Chronicler Engine Prompt System

> **Related Decisions**: [ADR-004](../adr/adr-004-xml-prompt-format.md), [ADR-005](../adr/adr-005-layered-prompts.md)

## Overview

The Chronicler Engine uses a layered prompt construction system inspired by SillyTavern's Prompt Manager. The system builds comprehensive context for game narration by combining game state, character information, world lore, and conversation history into a structured prompt sent to the LLM.

For background on SillyTavern's original system, see [`reference/sillytavern_prompt_system.md`](../reference/sillytavern_prompt_system.md).

## Prompt Architecture: XML-Sectioned Instructions + XML Data

The engine follows a **Marinara-Engine-inspired pattern**:

- **Instructions are XML-sectioned** — The system prompt contains behavioral instructions (`<role>`, `<instructions>`, `<global_rules>`). Prose and structural constraints (`<writing_style>`, `<output_format>`) are appended after conversation history in the user message to maximize recency bias. Two dynamic sections (`<global_rules>`, response length) are injected at assembly time.
- **Data is XML-wrapped** — External context (`<GameState>`, `<KnownNpcs>`, `<ConversationHistory>`, etc.) uses XML tags because it is *data*, not instructions.
- **Why sections?** Labeled content containers let users edit individual prompt aspects (role, rules, style, format) without rewriting the entire prompt. The imperative text inside each section remains plain.
- **Why not self-referential tags?** Tags like `<SystemPrompt>` or `<Role>` can trigger reasoning models (e.g., Gemma 4) to enter meta-analysis mode. The section tags (`<role>`, `<instructions>`) are content labels, not objects of analysis. See ADR-004 v4 for the full evolution.

## Source

- **SillyTavern Docs**: https://docs.sillytavern.app/usage/prompts/prompt-manager/
- **SillyTavern GitHub**: https://github.com/SillyTavern/SillyTavern

## The 7-Layer Prompt System

The Chronicler Engine implements a 7-layer prompt structure mapped from SillyTavern's Prompt Manager:

| Layer | Name | SillyTavern Equivalent | Purpose |
|-------|------|----------------------|---------|
| 0 | System | Main Prompt | XML-wrapped sections: role, instructions, global_rules |
| 1 | Game State | Context | Current room, present NPCs |
| 2 | NPC Cards | Character Description | In-room NPC character sheets |
| 3 | Player | Persona Description | Player persona and description |
| 4 | World Info | World Info / Lorebook | World lore triggered by keywords |
| 5 | History | Chat History | Full conversation history |
| 6 | Post-History | (n/a) | writing_style + output_format — placed after history for recency bias |
| 7 | User Input | User Message | Current player input |

## Detailed Layer Descriptions

### Layer 0: System Prompt (Main Prompt)
- **Role**: System
- **Position**: Absolute (top)
- **Content**: Assembled XML sections from the active preset — role, instructions, global_rules (from `world.json`). Writing style and output format are split out and placed after conversation history (see Post-History layer).
- **Renders**: `build_system_prompt()` in `assembler.rs`
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
- **See also**: [`reference/system_prompt.md`](../reference/system_prompt.md) for the full prompt text and section definitions

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
  - `<NpcsInRoom>`: Full character sheets for NPCs **currently present** (name, description, personality, scenario, goals, relationships)
- **Relationships**: For each in-room NPC, if they have `relationships` with other NPCs also present in the room, a `Relationships:` subsection is appended. Uses the `dynamic` text if non-empty, otherwise falls back to `static_text`.
- **Why two-tier**: The LLM needs awareness of off-screen characters to reference them or write introduction scenes, but full cards for every NPC would bloat the prompt. Condensed cards (~40-60 words) preserve identity and motivation without the bulk.

### Layer 3: Player Persona
- **Role**: User (data)
- **Content**: Player's character sheet
- **Format**: XML-wrapped (`<PlayerCharacter>... </PlayerCharacter>`)
- **Includes**: name, description, personality, scenario

### Layer 4: World Info (Lorebook)
- **Role**: User (data)
- **Trigger**: Keyword matching in conversation
- **Content**: World lore, setting facts, background information
- **Format**: XML-wrapped (`<WorldLore>... </WorldLore>`)
- **Implementation**: Renders `world.name` and `world.description` only. `global_rules` were previously duplicated here but have been moved exclusively to Layer 0 (System Prompt) to reduce token waste.

### Layer 5: Chat History
- **Role**: User (data)
- **Content**: Full conversation history (up to token limit)
- **Format**: XML-wrapped (`<ConversationHistory>... </ConversationHistory>`)
- **Note**: No summarization — all conversation retained and sent. Oldest entries are trimmed first if the context window is exceeded.

### Layer 6: Post-History Instructions
- **Role**: User (instructions)
- **Position**: After `<ConversationHistory>`, before `<PlayerInput>`
- **Content**: `<writing_style>` and `<output_format>` sections from the active preset
- **Why here?** LLMs exhibit strong recency bias. Placing prose constraints and structural rules at the end of the context window — after all story data but immediately before the generation point — makes them significantly more effective than burying them at the top of the prompt in the system message. This matches Marinara Engine's proven prompt architecture.
- **Assembly**: `build_post_history_prompt()` in `assembler.rs` assembles these sections directly from the preset — no string splitting or delimiter transport is required.

### Layer 7: User Input
- **Role**: User (data)
- **Content**: The player's current message/action
- **Format**: XML-wrapped (`<PlayerInput>... </PlayerInput>`)

## System / User Separation

The `LayeredPromptAssembler` separates instructions from data to maximize compatibility with OpenAI-compatible APIs:

- **System half**: XML-sectioned instruction sections (Layer 0)
- **User half**: XML-wrapped data (Layers 1–5) + post-history instructions (Layer 6) + player input (Layer 7)

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

Response length guidance is appended inside the `<output_format>` section by `build_post_history_prompt()` at assembly time:

```
Response Length:
flexible, based on the current scene. During a conversation, keep it concise
(under 150 words) to allow back-and-forth. For scene transitions, travel, or
plot developments, build content (above 150 words), but allow the player to react.
```

- **Source**: `AppSettings.response_length` (persisted in `settings.json`)
- **Default**: Flexible scene-adaptive guidance
- **Injection point**: Appended inside `<output_format>` section content by `build_post_history_prompt()`

## Context Templates

The engine uses a variable system similar to SillyTavern's Handlebars-style templates:
- Variables populated from `GameState` at render time
- Used in prompt construction within `LayeredPromptAssembler`

## World Info / Knowledge Base

- **Trigger**: Keywords appear in player input or history
- **Content**: World name and description. World `global_rules` appear in Layer 0 (System Prompt), not here.
- **Method**: Simple string matching (no RAG or vector DB)

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

- Stored in `NpcCard` and `PlayerCard` structures
- Stored as JSON in `data/characters/<world>/`

## Other Prompt Systems

### Quantifier Prompt (Separate)

The engine also uses a **quantifier prompt** — a separate secondary LLM call that runs *after* narration to analyze the scene. It determines which NPCs are present and whether the player moved. This is **not** part of the 7-layer narrative prompt stack.

- See [`reference/quantifier_prompt.md`](../reference/quantifier_prompt.md) for the full prompt text
- Rendered by: `QuantifierPromptBuilder` in `src/narrative/agents/quantifier/prompt.rs`
- Uses a separate model connection from the main narration LLM
- The quantifier also follows the XML-sectioned instructions + XML-wrapped data pattern

## Implementation

### Key Files
- `src/narrative/prompt/assembler.rs` — `PromptAssembler` trait, `LayeredPromptAssembler` with 7-layer construction, context fitting, and budget management
- `src/narrative/prompt/types.rs` — `PromptContext`, `PromptLayer`
- `src/narrative/llm/mod.rs` — LLM backend module (pure transport, no prompt assembly)
- `src/model/state.rs` — `GameState` provides context data
- `src/model/character.rs` — `NpcCard`, `PlayerCard` structures

### Code Example
```rust
let assembled = assembler.assemble(&context, &preset, &global_rules, Some(response_length))?;
// assembled.system_prompt — Layer 0 (sent as system message)
// assembled.user_prompt   — Layers 1-7 (sent as user message)
// assembled.max_tokens    — dynamically capped to fit context window
```

## Differences from SillyTavern

| Feature | SillyTavern | Chronicler Engine |
|---------|-------------|-------------------|
| API | Chat Completion | OpenRouter/DeepSeek/Ollama |
| Context | Characters + Users | Game State |
| History | Full chat | narrative history (messages) |
| Memory | Vector RAG | Keyword triggers only |
| UI | Web GUI | None (server) |
| Prompt style | XML-wrapped instructions | XML-sectioned instructions + XML data |
| Context fitting | Manual | Automatic per connection |

## References

- SillyTavern Prompt Manager: https://docs.sillytavern.app/usage/prompts/prompt-manager/
- SillyTavern Prompt Building: https://docs.sillytavern.app/usage/prompts/prompt-building/
- Prompt Assembly Pipeline: https://deepwiki.com/SillyTavern/SillyTavern/3.3-prompt-assembly-pipeline
