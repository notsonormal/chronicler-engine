# SillyTavern Prompt System Reference

## Overview

SillyTavern is a popular open-source frontend for LLMs, famous for its sophisticated prompt management system. The Chronicler Engine borrows from SillyTavern's Prompt Manager to build comprehensive context for game narration.

This document explains SillyTavern's system as a reference for understanding the Chronicler Engine's implementation.

## Source

- **Official Docs**: https://docs.sillytavern.app/usage/prompts/prompt-manager/
- **GitHub**: https://github.com/SillyTavern/SillyTavern
- **Source Code**: `public/scripts/PromptManager.js`

## The 8-Layer Prompt System

SillyTavern uses a layered prompt construction system where each layer serves a specific purpose. The Chronicler Engine implements this as:

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
- **Example**:
  ```
  You are a text adventure game master. Narrate outcomes in a literary fiction style.
  Never speak on behalf of the player. Keep descriptions immersive and concise.
  ```

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
- **Content**: Character sheets for NPCs in the current room only
- **Includes**: name, description, personality, scenario, example_dialogue
- **Why current room only**: Reduces token count, maintains relevance

### Layer 3: Player Persona
- **Role**: System
- **Content**: Player's character sheet
- **Includes**: name, description, personality, scenario

### Layer 4: World Info (Lorebook)
- **Role**: System
- **Trigger**: Keyword matching in conversation
- **Content**: World lore, setting facts, background information
- **SillyTavern Feature**: Entries have:
  - `name`: Entry identifier
  - `content`: The lore text
  - `keywords`: Array of trigger words
  - `priority`: Order if multiple match

### Layer 5: Chat History
- **Role**: User/Assistant alternating
- **Content**: Full conversation history (up to token limit)
- **SillyTavern Feature**: 
  - Injected at configurable "depth" relative to end
  - Can use "Continue" nudges to extend conversations
  - Supports "squashing" consecutive system messages

### Layer 6: User Input
- **Role**: User
- **Content**: The player's current message/action

### Layer 7: Post-History Instructions (PHI)
- **Role**: System (injected as user)
- **Position**: After history, before response
- **Content**: Final behavioral instructions
- **Example**:
  ```
  Describe the outcome of the player's action. If NPCs react, include their dialogue.
  Keep responses under 2 paragraphs.
  ```

## Token Budget Management

### SillyTavern's Approach
- **maxContext**: Maximum tokens in context window (e.g., 8192)
- **maxResponse**: Tokens reserved for LLM response (e.g., 1024)
- **availableForPrompt**: maxContext - maxResponse
- **Strategies**:
  1. Truncation (remove oldest messages)
  2. Summarization (compress history) - **Not used in Chronicler**
  3. Hierarchical memory - **Not used in Chronicler**

### Chronicler Engine Implementation
- Uses character-based token estimation (simple and fast)
- Hard truncation: Removes oldest history entries to fit budget
- **No summarization** - maintains accuracy over compression

## Context Templates

SillyTavern uses template variables (Handlebars-style):
- `{{char}}` - Character name
- `{{user}}` - User name
- `{{description}}` - Character description
- `{{scenario}}` - Character scenario
- `{{personality}}` - Character personality

### Chronicler Adaptation
- Uses similar variable system in prompt construction
- Variables populated from GameState at render time

## World Info / Knowledge Base

### SillyTavern Features
- **Keyword Triggers**: Match words in conversation to inject lore
- **Regex Support**: Advanced pattern matching
- **Depth Priority**: Order of insertion when multiple match
- **Format Templates**: Wrap content in specific tags

### Chronicler Implementation
- **Simple Keywords**: Basic string matching
- **Content**: World global_rules used as lore
- **Trigger**: Keywords appear in player input or history

## Prompt Manager UI Concepts

SillyTavern provides drag-and-drop prompt ordering with:
- **Position**: Absolute (top/bottom) or Relative (depth in history)
- **Depth**: How far from end of history to inject
- **Order**: Numeric ordering within same depth/role
- **Triggers**: Conditional inclusion based on context

## Character Card Format

SillyTavern character cards (Jailbreak format):
```json
{
  "name": "Character Name",
  "description": "Physical appearance, personality",
  "personality": "Behavior traits",
  "scenario": "Setting context",
  "example_dialogue": "Sample conversations"
}
```

### Chronicler Adaptation
- Uses same structure in `NpcCard` and `PlayerCard`
- Stored as JSON in `data/characters/<world>/`

## Implementation in Chronicler Engine

### Key Files
- `src/narrative/prompt.rs` - PromptBuilder with 8-layer construction
- `src/narrative/llm.rs` - LLM calls using PromptBuilder
- `src/model/state.rs` - GameState provides context data
- `src/model/character.rs` - NpcCard, PlayerCard structures

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
| API | Chat Completion | OpenRouter/DeepSeek |
| Context | Characters + Users | Game State |
| History | Full chat | narration_history |
| Memory | Vector RAG | Keyword triggers only |
| UI | Web GUI | None (server) |

## References

- SillyTavern Prompt Manager: https://docs.sillytavern.app/usage/prompts/prompt-manager/
- SillyTavern Prompt Building: https://docs.sillytavern.app/usage/prompts/prompt-building/
- Prompt Assembly Pipeline: https://deepwiki.com/SillyTavern/SillyTavern/3.3-prompt-assembly-pipeline
