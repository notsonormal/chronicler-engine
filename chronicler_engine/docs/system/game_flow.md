# Specification: Game Flow

## Overview

This document defines the core game loop - the play-by-play experience from starting the game to receiving LLM responses. This is the fundamental user experience that must work reliably.

## The Game Flow

```
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

## Real-Time Updates: Polling vs WebSocket

The engine uses **HTMX polling** for real-time updates instead of WebSockets.

### Why Polling?
- **Reliability**: WebSocket connections are unreliable in headless browser testing (Playwright)
- **Simplicity**: No additional server infrastructure needed
- **Fallback**: Works even if JavaScript fails

### Implementation
- **Polling Interval**: 5 seconds (`hx-trigger="load, every 5s"`)
- **Endpoint**: `/fragment/story-log` returns log entries
- **Swap**: `hx-swap="innerHTML"` replaces content without wrapper

### Alternative Considered: WebSocket
WebSocket was attempted but had issues in test environments. The polling approach provides 100% reliable updates at the cost of a 5-second maximum delay.

> **Note**: WebSocket is still connected for status indicator but polling handles main content updates.

## Phase Details

### Phase 1: Initialize

**Steps:**
1. Server loads world from `worlds/<world_name>/world.yaml`
2. Creates `GameState` with:
   - Current room from world manifest
   - Empty narration history
   - Player character
3. Renders initial HTML fragments
4. WebSocket connects, receives initial state

**Expected UI State:**
- Header shows: "Chronicler Engine | <starting_room_name>"
- Story Log shows: Room description as first narration entry
- Visual Sidebar shows: Room image + NPCs in room
- Status: "Ready"

### Phase 2: Await Input

**Steps:**
1. User types command in input field
2. User clicks "Send" or presses Enter
3. Form submits via HTMX POST to `/action`
4. Input field clears

**Expected UI State:**
- Form input is cleared (after request completes)
- Status transitions to "Thinking..." (from response)

### Phase 3: Process Action

**Steps:**
1. Parse command into `Action` enum
2. Execute action:
   - **Look**: Get current room description
   - **WalkTo**: Update player position, check exits
   - **Talk**: Record dialogue intent
   - **Inventory**: Show player items
   - **FreeAction**: Pass to LLM for narration
3. Add user input to narration history as `Input` type
4. If LLM needed, spawn async thread
5. Return "Thinking..." status to client immediately

**Expected UI State:**
- Status shows "Thinking..."
- Input entry appears in story-log (gray text)

### Phase 4: LLM Generation

**Context sent to LLM:**
- Current room (name, description, atmosphere)
- NPCs present in room
- Player character summary
- The action being taken

**Response:**
- Natural language narration
- Added to history as `Narration` type (cyan text)

### Phase 5: Broadcast

**Steps:**
1. After action completes (sync or async), render all fragments
2. Send JSON messages via WebSocket:
   ```json
   {"type": "update", "fragment": "story-log", "html": "..."}
   {"type": "update", "fragment": "header", "html": "..."}
   {"type": "update", "fragment": "visual-sidebar", "html": "..."}
   ```
3. Client JavaScript receives and updates DOM

**Expected UI State:**
- New narration appears in story-log (no reload)
- Location updates in header (if moved)
- Visual sidebar updates (if NPCs changed)
- Status returns to "Ready"

## Test Scenarios

### Scenario 1: Initial Load
```gherkin
Given the server is running with "test" world
When the user opens http://127.0.0.1:3000
Then the header shows "Chronicler Engine | <starting_room>"
And the story-log contains the starting room description
And the status shows "Ready"
```

### Scenario 2: Look Command
```gherkin
Given the game is loaded
When the user enters "look" and submits
Then the status shows "Thinking..."
And after processing, the story-log shows the room description again
And the status shows "Ready"
```

### Scenario 3: Move to New Location
```gherkin
Given the game is loaded at starting room
When the user enters "go north" and submits
Then the status shows "Thinking..."
And after LLM generates response:
  And the header shows the new room name
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

### WebSocket Disconnect
- Show "Disconnected" in connection status
- Attempt reconnection automatically
- UI continues to work via HTMX fallback

## Reference Implementation

- **Server**: `src/server/fragments.rs` - `action_handler`, `process_action`, `broadcast_state`
- **WebSocket**: `src/server/mod.rs` - `ws_handler`, `handle_socket`
- **Client**: `assets/index.html` - WebSocket message handling in `<script>`
- **LLM**: `src/narrative/llm.rs` - `narrate_action`, `narrate_arrival`
