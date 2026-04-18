# Specification: Game Flow

## Overview

This document defines the core game loop - the play-by-play experience from starting the game to receiving LLM responses. This is the fundamental user experience that must work reliably.

## The LLM Context Pipeline

When the engine needs LLM narration, it builds a comprehensive prompt using the **SillyTavern-style 8-layer system** (see `reference/sillytavern_prompt_system.md`):

```text
┌─────────────────────────────────────────────────────────────────────┐
│                     PHASE 4: LLM GENERATION                          │
│                  (If narrative action)                               │
│                                                                      │
│  1. Build 8-layer prompt (SillyTavern-style):                        │
│     - Layer 0: System prompt (game rules, narrator persona)         │
│     - Layer 1: Game state (room, inventory, NPCs)                   │
│     - Layer 2: NPC cards (in-room NPCs only)                         │
│     - Layer 3: Player persona                                        │
│     - Layer 4: World info (keyword-triggered lore)                  │
│     - Layer 5: Full narration history (up to 1000 entries)         │
│     - Layer 6: User input (current action)                          │
│     - Layer 7: Post-History Instructions                            │
│  2. Token budget check (8192 max, truncate if overflow)             │
│  3. Send to LLM (OpenRouter/DeepSeek)                               │
│  4. Receive narration response                                      │
│  5. Add to narration history as "Narration"                        │
│  6. Set status back to "Ready"                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## The Game Flow

```text
┌─────────────────────────────────────────────────────────────────────┐
│                        START GAME                                    │
│                  (Server starts, loads world)                       │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     PHASE 1: INITIALIZE                              │
│  1. Load world data (rooms, NPCs, items)                            │
│  2. Set player in starting room                                     │
│  3. Render initial UI fragments                                     │
│     - Header: location name                                         │
│     - Story Log: initial narration (room description)               │
│     - Visual Sidebar: location image + NPCs                         │
│  4. Establish HTMX polling (every 5s)                              │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     PHASE 2: AWAIT INPUT                              │
│                   (Status: "Ready")                                  │
│                                                                      │
│  User types command → submits form                                  │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     PHASE 3: PROCESS ACTION                          │
│  1. Parse command (look, move north, talk to npc, etc.)            │
│  2. Execute game logic                                              │
│     - Update player position if moving                              │
│     - Add command to narration history as "Input"                  │
│  3. Set status to "Thinking..."                                     │
│  4. If LLM needed: spawn async thread for generation               │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     PHASE 4: LLM GENERATION                          │
│                  (If narrative action)                               │
│                                                                      │
│  1. Build context: current room, NPCs present, player, action     │
│  2. Send to LLM (OpenRouter)                                        │
│  3. Receive narration response                                      │
│  4. Add to narration history as "Narration"                        │
│  5. Set status back to "Ready"                                      │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     PHASE 5: POLLING UPDATE                          │
│                                                                      │
│  1. Client polls /fragment/story-log every 5 seconds              │
│  2. Server returns updated log entries (innerHTML)                 │
│  3. HTMX swaps content without page reload                          │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │
                                  ▼
                           ┌───────────────┐
                           │  BACK TO 2    │
                           └───────────────┘
```

## Test Scenarios

### Scenario 1: Initial Load
```gherkin
Given the server is running with "test" world
When the user opens http://127.0.0.1:3000
Then the header shows "Chronicler Engine | <starting_room>"
And the story-log shows a minimal header with the room name
And after LLM generates: the story-log shows LLM-generated arrival narration
And the status shows "Ready"
```

### Scenario 2: Look Command
```gherkin
Given the game is loaded
When the user enters "look" and submits
Then the status shows "Thinking..."
And after LLM generates response, the story-log shows the LLM description
And the status shows "Ready"
```

### Scenario 3: Move to New Location
```gherkin
Given the game is loaded at starting room
When the user enters "go north" and submits
Then the status shows "Thinking..."
And the story-log shows a minimal header with the new room name
And after LLM generates response:
  And the story-log shows the LLM narration for arrival
  And the visual-sidebar shows the new room's image and NPCs
And the status shows "Ready"
```

### Scenario 4: Free Action (LLM Narration)
```gherkin
Given the game is loaded
When the user enters "examine the mysterious orb" and submits
Then the status shows "Thinking..."
And after LLM generates response:
  And the story-log shows the LLM's description of the orb
And the status shows "Ready"
```

## Error Handling

### LLM Timeout
- If LLM takes >30 seconds, show error in story-log
- Status returns to "Ready"

### Invalid Command
- Show helpful error in story-log
- Status returns to "Ready"

### Polling-based Updates
- HTMX automatically polls every 5 seconds for story-log updates
- Status-display polls `/status/generating` for button state
- No manual reconnection needed

## Reference Implementation

- **Server**: `src/server/fragments.rs` - `action_handler`, `process_action`
- **HTMX Polling**: `assets/index.html` - `hx-trigger="load, every 5s"`
- **LLM**: `src/narrative/llm.rs` - `narrate_action`, `narrate_arrival`
- **Prompt Builder**: `src/narrative/prompt.rs` - 8-layer prompt construction
- **LLM Tests**: `tests/flow_llm_tests.rs` - Real LLM integration tests
