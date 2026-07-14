# Specification: Dashboard UI

## Overview

The Chronicler Engine presents a web-based HTMX dashboard for player interaction. The UI provides narrative immersion, visual grounding, and user input in a modern chat-app aesthetic inspired by SillyTavern.

## The Dashboard Layout

### 1. Header Bar (48px height)

Displays system-level context.

- **Content**: Game title (left), current game name (center-left), connection status (right)
- **Note**: Location is displayed in the story log, not the header. Game management (create, switch, delete, reset) lives in the "Games" tab.

### 2. Tab Bar

Navigation between Game, LLM Messages, and Settings views.

- **Tabs**: Game | Settings | Prompt Presets | Worlds | Games | LLM Messages
- **Active tab**: Green text with green bottom border
- **Inactive tab**: Muted gray text

### 3. Game Tab (Default)

#### Main Body (Flex: 1)

Horizontal split into story context and visual context:

- **Story Log (80%)**: Scrollable history of narration with chat-bubble styling
  - **Styles**:
    - **Location headers**: Inline "Room Name - HH:MM", green token, bold
    - **Event headers**: Inline "Event Name - HH:MM", cyan token (`.event-header`), bold
    - User input (right-aligned, darker gray background)
    - AI/Narration (left-aligned, dark cyan background)
    - AI/Dialogue (left-aligned, orange-tinted background, italic text)
    - System messages (center-aligned, yellow)
    - Character name prominent above message (bold, larger)
    - Subtle timestamp (HH:MM format, small gray)
    - Fade-in animation for new messages
    - Action buttons at top-right of every message:
      - Edit button (✎) on all entries
      - Delete button (🗑) on last entry only (hidden when only one entry exists)
      - Check button (✓) on input entries (spellcheck)
      - Retrigger button (♻) on the last narration or dialogue message when the previous turn had a trigger AND the message is not itself an event continuation
    - Swipe controls on the last narration or dialogue message when `swipe_count > 1`:
      - Left arrow (◀) — previous swipe (hidden on first swipe)
      - Counter — `active + 1 / swipe_count` (e.g., "2 / 3")
      - Right arrow (▶) — next swipe if not on latest; triggers new generation if on latest swipe
- **Visual Sidebar (20%)**:
  - Location Image (top): Full-width location image
  - NPC Portraits (bottom): Horizontal scrollable row of 80×80 square images

#### Action Area

Interactive zone for player input.

- **Content**: Text input field + send button + status indicator
- **Button States**:
  - Ready: Green button labeled "Send"
  - Thinking: Green button labeled "Stop", input disabled
- **Status States**:
  - "Ready" — awaiting input
  - "Thinking..." — LLM generating response (includes narrative continuation on empty input)
  - "Still thinking..." — concurrent generation in progress
- **Text Check Preview**: When spell/grammar issues are detected, the action area temporarily shows:
  - Original vs corrected text comparison
  - Issue tags (spell = orange, grammar = pink)
  - **Send** — submits corrected text to `/action`
  - **Send Original** — submits original text to `/action`
  - **Cancel** — restores normal action area

**Empty Input Behavior**: Pressing Send with an empty text box dispatches `process_action(String::new())`, which routes to `continue_narration` (same as SillyTavern's "Continue" button). The submit button immediately changes to "Stop"; the next polled status reads "Thinking...".

### 4. Settings Tab

Configuration panel for LLM connections.

- **Connections List**: Cards showing each configured connection
  - Name, provider, model
  - Badges: "Narrator" or "Quantifier" (if assigned)
  - Actions: Edit, Delete, Set as Narrator, Set as Quantifier
- **Add Connection Form**:
  - Name, Provider (OpenRouter/DeepSeek/Ollama), Model
  - API Key (optional), Base URL (optional)
  - Single User Message checkbox (for models that ignore system prompts)
- **Text Check Card**:
  - Mode dropdown (Disabled / Spell / Grammar / Spell + Grammar)
  - "Check before sending to LLM" checkbox

## Real-Time Updates

The dashboard uses HTMX polling for live updates:

- Story-log polls `/fragment/story-log` every 2 seconds for new content
- Visual sidebar polls `/fragment/visual-sidebar` every 5 seconds
- Status-display polls `/status/generating` every 5 seconds to update button state
- LLM Messages polls `/fragment/llm-messages` every 4 seconds

## Data Model

### MessageEntry

```rust
pub struct MessageEntry {
    pub id: u64,                           // Unique auto-incrementing ID
    pub sender: Option<String>,
    pub text: String,                       // Active swipe text (markdown source)
    pub message_type: MessageType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub swipe_count: usize,                 // Number of swipes for this message
    pub active_swipe_index: usize,          // Currently displayed swipe index
    pub location_header: Option<String>,    // Active swipe location header
    pub event_header: Option<String>,       // Active swipe event header
}
```

### MessageEntryView (Rendered)

```rust
pub struct MessageEntryView {
    pub id: u64,
    pub timestamp: String,
    pub sender: String,
    pub text: SafeHtml,                     // Rendered HTML (markdown converted)
    pub raw_text: String,                   // Original markdown (for editing)
    pub log_type: String,                  // Stringified MessageType for CSS class
    pub location_header: Option<String>,    // Room name header
    pub event_header: Option<String>,       // Event name header
    pub swipe_count: usize,
    pub active_swipe_index: usize,
    pub prev_swipe_index: Option<usize>,
    pub next_swipe_index: Option<usize>,
    pub show_retrigger: bool,
}
```

HTML template renders each entry with: timestamp, sender, text body, optional location/event header, and per-entry action buttons (edit always; delete/check/retrigger conditionally per spec above). Swipe controls render on the last entry when `swipe_count > 1`.

## Edit Flow (SillyTavern Pattern)

1. Click edit button → enter edit mode (textarea replaces text span); polling pauses
2. Save → raw text submitted to server
3. Cancel → exit edit mode; polling resumes

## Delete Flow

1. Click delete button → browser confirmation dialog
2. On confirm → server deletes last message and saves new snapshot
3. Story log refreshes via HTMX polling

## Button State Transitions

1. Generating state: submit button shows "Stop" and is disabled
2. Ready state: submit button shows "Send" and is enabled

### Game Management

Multiple independent games across all worlds, each with isolated snapshots and messages. The Games panel sections: Active Game, New Game, and Saved Games.

- **Create game**: New game section shows world + persona dropdowns and a "Start New Game" button. Game name auto-generated as `{WorldName} {YYYY-MM-DD} {N}` (format `%Y-%m-%d`, spaces between segments). Submit disabled when persona list is empty.
- **Active Game**: Shows current game name, world badge, persona badge, "Current" badge, and reset button.
- **Switch game**: Loads the selected game (cross-world switching allowed).
- **Delete game**: Removes the game and all its data.
- **Reset**: Deletes the current game and creates a new one with a fresh auto-generated name.

## Worlds Management Tab

Dedicated tab for multi-world orchestration with CRUD operations.

### LLM Messages Tab

Forensics panel showing the last 50 LLM calls with full request/response visibility.

- **Content**: Compact list of LLM calls with agent name, backend, model, and parsed response
- **Expandable rows**: Click to reveal raw request JSON and raw response JSON
- **Order**: Oldest-first (newest at bottom), matching chronological narrative flow
- **Empty state**: "No LLM messages yet" when no calls have been logged
- **Auto-pruning**: Both backends (SQLite + InMemory) cap `llm_messages` at 50 rows globally; oldest evicted on insert. The `llm_messages` table has no `game_id` column, so the cap is global, not per-game.
- **HTMX Polling**: `hx-get="/fragment/llm-messages" hx-trigger="load, every 4s"`

## Document References

- [ADR-001: HTMX Web Dashboard Architecture](../adr/adr-001-htmx-web-dashboard.md) — htmx + server-rendered HTML architecture
- [ADR-002: HTTP Polling for Real-Time Updates](../adr/adr-002-http-polling.md) — polling-based real-time update mechanism
- [ADR-003: Askama Template Engine for HTML Rendering](../adr/adr-003-askama-templates.md) — Askama template choice + compile-time validation
- [system/worlds.md](./worlds.md) — world model + data flow + panel interactions
