# Chronicler Engine Prompt System

## Overview

The Chronicler Engine uses a layered prompt construction system inspired by SillyTavern's Prompt Manager. The system builds comprehensive context for game narration by combining game state, character information, world lore, and conversation history into a structured prompt sent to the LLM.

For background on SillyTavern's original system, see [`reference/sillytavern_prompt_system.md`](../reference/sillytavern_prompt_system.md).

## Prompt Architecture: Plain-Text Instructions + XML Data

The engine follows a **Marinara-Engine-inspired pattern** designed for compatibility with reasoning models:

- **Instructions are plain text** — No XML tags wrapping the system prompt or PHI layer. Imperative voice only ("You are...", "Your job is...", "Never...").
- **Data is XML-wrapped** — External context (`<GameState>`, `<KnownNpcs>`, `<ConversationHistory>`, etc.) uses XML tags because it is *data*, not instructions.
- **Why?** Self-referential XML (`<SystemPrompt>`, `<Role>`, `<AuxiliaryInstructions>`) can trigger reasoning models (e.g., Gemma 4) to enter meta-analysis mode, consuming all tokens in `reasoning` fields and producing empty `content`. Plain-text instructions avoid this trap.

## Source

- **SillyTavern Docs**: https://docs.sillytavern.app/usage/prompts/prompt-manager/
- **SillyTavern GitHub**: https://github.com/SillyTavern/SillyTavern

## The 8-Layer Prompt System

The Chronicler Engine implements an 8-layer prompt structure mapped from SillyTavern's Prompt Manager:

| Layer | Name | SillyTavern Equivalent | Purpose |
|-------|------|----------------------|---------|
| 0 | System | Main Prompt | Game rules, role instructions, narrative style |
| 1 | Game State | Context | Current room, inventory, present NPCs |
| 2 | NPC Cards | Character Description | In-room NPC character sheets |
| 3 | Player | Persona Description | Player persona and description |
| 4 | World Info | World Info / Lorebook | World lore triggered by keywords |
| 5 | History | Chat History | Full conversation history |
| 6 | User Input | User Message | Current player input |
| 7 | PHI | Post-History Instructions | Behavioral guidance after history |

## Detailed Layer Descriptions

### Layer 0: System Prompt (Main Prompt)
- **Role**: System
- **Position**: Absolute (top)
- **Content**: Global game rules, narrator persona, narrative style guidelines
- **Renders**: `PromptBuilder::render_system_layer()`
- **Format**: Plain text (no XML wrapper)
- **Example**:
  ```
  You are an interactive fiction author with your own free will, intellect, and emotional intelligence...
  ```
- **See also**: [`reference/system_prompt.md`](../reference/system_prompt.md) for the full prompt text

### Layer 1: Game State
- **Role**: User (data)
- **Content**: Current room name, description, player inventory, NPCs in the current room
- **Format**: XML-wrapped (`<GameState>... </GameState>`)
- **Example**:
  ```xml
  <GameState>
  Current Location: Grand Foyer

  A cavernous entrance hall with marble floors.

  --- Inventory ---
  - Rusty Key
  - Candle
  </GameState>
  ```

### Layer 2: NPC Cards (Character Description)
- **Role**: User (data)
- **Content**: Two sections:
  - `<KnownNpcs>`: Condensed roster of **all** NPCs the player has met (name, location, 3-line summary)
  - `<NpcsInRoom>`: Full character sheets for NPCs **currently present** (name, description, personality, scenario, goals)
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

### Layer 6: User Input
- **Role**: User (data)
- **Content**: The player's current message/action
- **Format**: XML-wrapped (`<PlayerInput>... </PlayerInput>`)

### Layer 7: Post-History Instructions (PHI)
- **Role**: User (instruction appended to user message)
- **Position**: After history, before response
- **Content**: Final behavioral instructions
- **Format**: Plain text (no XML wrapper)
- **Content**: Universal behavioral instructions (immersive prose, don't ask questions, end descriptively)
- **Split behavior**: In `build_split()`, PHI is appended to the **user message** (not the system prompt) so it sits closest to the generation point, matching the ordering in `build()` where `PlayerInput` precedes the PHI instructions.
- **Example**:
  ```
  Narrate the outcome of the player's action in immersive prose.
  Do NOT conclude with any form of player direction, question, or prompt.
  ```

## `build_split()` Separation

`build_split()` separates instructions from data to maximize compatibility with OpenAI-compatible APIs:

- **System half**: Plain-text instructions only (Layer 0)
- **User half**: XML-wrapped data (Layers 1–6) + plain-text PHI (Layer 7)

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

The `PromptBuilder` supports an optional `response_length` field (set via `.with_response_length()`) that appends scene-adaptive length guidance to the system prompt:

```
Response Length:
flexible, based on the current scene. During a conversation, keep it concise
(under 150 words) to allow back-and-forth. For scene transitions, travel, or
plot developments, build content (above 150 words), but allow the player to react.
```

- **Source**: `AppSettings.response_length` (persisted in `settings.json`)
- **Default**: Flexible scene-adaptive guidance
- **Injection point**: Appended after `Global Rules` in `render_system_layer()`

## Context Templates

The engine uses a variable system similar to SillyTavern's Handlebars-style templates:
- Variables populated from `GameState` at render time
- Used in prompt construction within `PromptBuilder`

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

The engine also uses a **quantifier prompt** — a separate secondary LLM call that runs *after* narration to analyze the scene. It determines which NPCs are present and whether the player moved. This is **not** part of the 8-layer narrative prompt stack.

- See [`reference/quantifier_prompt.md`](../reference/quantifier_prompt.md) for the full prompt text
- Rendered by: `QuantifierPromptBuilder` in `src/narrative/quantifier.rs`
- Uses a separate model connection from the main narration LLM
- The quantifier also follows the plain-text instructions + XML-wrapped data pattern

## Implementation

### Key Files
- `src/narrative/prompt.rs` — `PromptBuilder` with 8-layer construction, context fitting, and budget management
- `src/narrative/llm.rs` — LLM backend implementations that configure `PromptBuilder` with connection-specific context windows
- `src/model/state.rs` — `GameState` provides context data
- `src/model/character.rs` — `NpcCard`, `PlayerCard` structures

### Code Example
```rust
let (system, user, max_tokens) = PromptBuilder::from_context(&ctx)
    .with_max_context_tokens(8192)
    .with_max_tokens(2048)
    .with_response_length(&settings.response_length)
    .build_split()?;
```

## Differences from SillyTavern

| Feature | SillyTavern | Chronicler Engine |
|---------|-------------|-------------------|
| API | Chat Completion | OpenRouter/DeepSeek/Ollama |
| Context | Characters + Users | Game State |
| History | Full chat | narration_history |
| Memory | Vector RAG | Keyword triggers only |
| UI | Web GUI | None (server) |
| Prompt style | XML-wrapped instructions | Plain-text instructions + XML data |
| Context fitting | Manual | Automatic per connection |

## References

- SillyTavern Prompt Manager: https://docs.sillytavern.app/usage/prompts/prompt-manager/
- SillyTavern Prompt Building: https://docs.sillytavern.app/usage/prompts/prompt-building/
- Prompt Assembly Pipeline: https://deepwiki.com/SillyTavern/SillyTavern/3.3-prompt-assembly-pipeline
