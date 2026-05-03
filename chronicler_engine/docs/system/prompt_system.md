# Chronicler Engine Prompt System

## Overview

The Chronicler Engine uses a layered prompt construction system inspired by SillyTavern's Prompt Manager. The system builds comprehensive context for game narration by combining game state, character information, world lore, and conversation history into a structured prompt sent to the LLM.

For background on SillyTavern's original system, see [`reference/sillytavern_prompt_system.md`](../reference/sillytavern_prompt_system.md).

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
- **Example**:
  ```
  You are a text adventure game master. Narrate outcomes in a literary fiction style.
  Never speak on behalf of the player. Keep descriptions immersive and concise.
  ```
- **See also**: [`reference/system_prompt.md`](../reference/system_prompt.md) for the full prompt text

### Layer 1: Game State
- **Role**: System
- **Content**: Current room name, description, player inventory, NPCs in the current room
- **Example**:
  ```
  ## Current Location
  Name: Grand Foyer
  Description: A cavernous entrance hall with marble floors.
  Inventory: Rusty Key, Candle

  NPCs Present: Butler, Maid
  ```

### Layer 2: NPC Cards (Character Description)
- **Role**: System
- **Content**: Two sections:
  - `<KnownNpcs>`: Condensed roster of **all** NPCs the player has met (name, location, 3-line summary)
  - `<NpcsInRoom>`: Full character sheets for NPCs **currently present** (name, description, personality, scenario, goals)
- **Why two-tier**: The LLM needs awareness of off-screen characters to reference them or write introduction scenes, but full cards for every NPC would bloat the prompt. Condensed cards (~40-60 words) preserve identity and motivation without the bulk.

### Layer 3: Player Persona
- **Role**: System
- **Content**: Player's character sheet
- **Includes**: name, description, personality, scenario

### Layer 4: World Info (Lorebook)
- **Role**: System
- **Trigger**: Keyword matching in conversation
- **Content**: World lore, setting facts, background information
- **Implementation**: Simple keyword matching from `world.json` `global_rules`

### Layer 5: Chat History
- **Role**: User/Assistant alternating
- **Content**: Full conversation history (up to token limit)
- **Note**: No summarization — all conversation retained and sent

### Layer 6: User Input
- **Role**: User
- **Content**: The player's current message/action

### Layer 7: Post-History Instructions (PHI)
- **Role**: System (injected as user)
- **Position**: After history, before response
- **Content**: Final behavioral instructions
- **Modes**: See [`reference/system_prompt.md`](../reference/system_prompt.md) for `PhiMode::Narration` and `PhiMode::Continuation`
- **Example**:
  ```
  Describe the outcome of the player's action. If NPCs react, include their dialogue.
  Keep responses under 2 paragraphs.
  ```

## Token Budget Management

- **MAX_CONTEXT_TOKENS**: 32000
- **MAX_RESPONSE_TOKENS**: 1024
- **MAX_HISTORY_TOKENS**: 16000
- **Strategy**: Hard truncation — removes oldest history entries to fit budget
- **No summarization** — maintains accuracy over compression
- **Estimation**: Character-based token estimation (simple and fast)

## Context Templates

The engine uses a variable system similar to SillyTavern's Handlebars-style templates:
- Variables populated from `GameState` at render time
- Used in prompt construction within `PromptBuilder`

## World Info / Knowledge Base

- **Trigger**: Keywords appear in player input or history
- **Content**: World `global_rules` used as lore
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

## Implementation

### Key Files
- `src/narrative/prompt.rs` — `PromptBuilder` with 8-layer construction
- `src/narrative/llm.rs` — LLM calls using `PromptBuilder`
- `src/model/state.rs` — `GameState` provides context data
- `src/model/character.rs` — `NpcCard`, `PlayerCard` structures

### Code Example
```rust
let prompt = PromptBuilder::new()
    .with_game_state(&state)
    .with_history(&state.narration_history)
    .with_user_input(input)
    .build()?;
```

## Differences from SillyTavern

| Feature | SillyTavern | Chronicler Engine |
|---------|-------------|-------------------|
| API | Chat Completion | OpenRouter/DeepSeek/Ollama |
| Context | Characters + Users | Game State |
| History | Full chat | narration_history |
| Memory | Vector RAG | Keyword triggers only |
| UI | Web GUI | None (server) |

## References

- SillyTavern Prompt Manager: https://docs.sillytavern.app/usage/prompts/prompt-manager/
- SillyTavern Prompt Building: https://docs.sillytavern.app/usage/prompts/prompt-building/
- Prompt Assembly Pipeline: https://deepwiki.com/SillyTavern/SillyTavern/3.3-prompt-assembly-pipeline
