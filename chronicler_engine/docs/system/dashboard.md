# Specification: Dashboard UI

> **Related Decisions**: [ADR-001](../adr/adr-001-htmx-web-dashboard.md), [ADR-002](../adr/adr-002-http-polling.md), [ADR-003](../adr/adr-003-askama-templates.md)

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
    - **Location headers**: Inline "Room Name - HH:MM", green color (#4ade80), bold
    - **Event headers**: Inline "Event Name - HH:MM", blue/cyan color (#38bdf8), bold
    - User input (right-aligned, darker gray background #2a2a2a)
    - AI/Narration (left-aligned, dark cyan background #1a3a3a)
    - AI/Dialogue (left-aligned, orange-tinted background #3a2a1a, italic text)
    - System messages (center-aligned, yellow #ffff00)
    - Character name prominent above message (bold, larger)
    - Subtle timestamp (HH:MM format, small gray)
    - Fade-in animation for new messages
    - Action buttons at top-right of every message:
      - Edit button (✎) on all entries
      - Delete button (🗑) on last entry only (hidden when only one entry exists)
      - Check button (✓) on input entries (spellcheck)
      - Retrigger button (♻) on last narration when trigger context is available
    - Swipe controls on the last message when swipe count > 1:
      - Left arrow (◀) — previous swipe (hidden on first swipe)
      - Counter — `active + 1 / swipe_count` (e.g., "2 / 3")
      - Right arrow (▶) — next swipe if not on latest; triggers new generation if on latest swipe
- **Visual Sidebar (20%)**:
  - Location Image (top): Full-width location image, max-height 200px, object-fit contain
  - NPC Portraits (bottom): Horizontal scrollable row, 80×80px square images, object-fit cover

#### Action Area (64px height)
Interactive zone for player input.
- **Content**: Text input field + send button + action hints + status indicator
- **Button States**:
  - Ready: Green button with "Send" text and play icon (▶)
  - Thinking: Green button with "Stop" text and square icon (■), disabled input
- **Status States**:
  - "Ready" - Green (#00ff00), awaiting input
  - "Thinking..." - Yellow (#ffff00) with pulse animation, LLM generating response (includes narrative continuation on empty input)
  - "Still thinking..." - Yellow (#ffff00), concurrent generation in progress
- **Text Check Preview**: When spell/grammar issues are detected, the action area temporarily shows:
  - Original vs corrected text comparison
  - Issue tags (spell = orange, grammar = pink)
  - **Send** — submits corrected text to `/action`
  - **Send Original** — submits original text to `/action`
  - **Cancel** — restores normal action area

**Empty Input Behavior**: Pressing Send with an empty text box triggers narrative continuation via `continue_narration()` → `process_action(CONTINUE_SENTINEL)`. The LLM generates the next scene without player input, same as SillyTavern's "Continue" button. Status shows "Thinking..." (unified with normal generation). No HTML5 validation blocks empty submit.

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
- Action hints poll `/hints` every 5 seconds
- LLM Messages polls `/fragment/llm-messages` every 4 seconds
- New narration appears automatically with fade-in effect
- Button changes state during LLM processing
- No manual refresh required

## Frontend Implementation
- **HTMX**: Handles partial page updates via `hx-post` and `hx-target`
- **HTMX Polling**: `hx-trigger="load, every 2s"` for story-log; `every 5s` for status; `every 4-5s` for sidebar and LLM messages
- **Styling**: Modern chat-app aesthetic with chat bubbles, fade animations
- **Templates**: Uses `askama` for compile-time validated HTML fragments (pilot)

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

HTML template renders with `data-raw-text` attribute for inline editing:

```html
<div class="log-entry" data-id="{{ entry.id }}" data-raw-text="{{ entry.raw_text }}">
    <div class="message-header">
        <div class="message-info">
            <span class="timestamp">HH:MM</span>
            <span class="sender">Sender:</span>
        </div>
        <div class="message-actions">
            <button class="action-btn edit-btn" title="Edit">✎</button>
            <button class="action-btn delete-btn" title="Delete">🗑</button>
            {% if input_entry %}
            <button class="action-btn check-btn" title="Check">✓</button>
            {% endif %}
            {% if show_retrigger %}
            <button class="action-btn retrigger-btn" title="Retrigger Event">♻</button>
            {% endif %}
        </div>
    </div>
    <span class="text">{{ entry.text }}</span>
    {% if loop.last && swipe_count > 1 %}
    <div class="swipe-controls">
        <button class="action-btn swipe-btn" disabled>◀</button>
        <span class="swipe-counter">1 / 3</span>
        <button class="action-btn swipe-btn" onclick="submitNewSwipe()">▶</button>
    </div>
    {% endif %}
</div>
```

## Edit Flow (SillyTavern Pattern)

1. Click edit button → textarea replaces text span, polling pauses
2. Textarea height matches original rendered height (+ padding/border compensation)
3. Textarea auto-resizes on input if content grows taller
4. Edit text in textarea (uses `data-raw-text`, not HTML textContent)
5. Click save → textarea value sent to server, stored as raw text
6. Click cancel → restore original text, resume polling

## Delete Flow

1. Click delete button → browser confirmation dialog
2. On confirm → POST to `/history/delete`
3. Server calls `delete_last_log()`, removing the last message
4. A new snapshot is saved reflecting the shortened history
5. Client refreshes story log via HTMX polling or manual trigger

## Button Logic (JavaScript)
1. Monitor status element changes via MutationObserver or HTMX events
2. When status contains "Thinking...":
   - Disable the submit button
   - Change button text from "▶ Send" to "■ Stop"
   - Keep button green (no red)
3. When status returns to "Ready":
   - Re-enable the submit button
   - Change button text back to "▶ Send"

### Game Management
Multiple independent games across all worlds, each with isolated snapshots and messages:
- **List games**: `GET /fragment/games` renders the Games panel with three sections: Active Game, New Game, and Saved Games
- **Active Game**: Shows current game name, world badge, "Current" badge, and a small reset button on the card
- **Create game**: `POST /games` accepts form data with `world_key` parameter; creates a game under the chosen world with auto-generated name (`{WorldName}_{Date}_N`). The New Game section shows an always-visible world dropdown + "Start New Game" button
- **Switch game**: `POST /games/:id/switch` loads the selected game (cross-world switching allowed) and refreshes the page
- **Delete game**: `POST /games/:id/delete` removes the game and all its data, then removes the item from the list via `hx-swap`
- **Reset**: `POST /reset` deletes the current game and creates a new one with a fresh auto-generated name. Triggered by the reset button on the Active Game card with `hx-confirm` dialog

## Worlds Management Tab
Dedicated tab for multi-world orchestration with CRUD operations:

### Worlds Panel
- **Endpoint**: `GET /fragment/worlds`
- **Content**: List of all worlds with game count indicators
- **Actions per world**:
  - Edit button — replaces worlds list with edit form inline (HTMX `hx-get` + `hx-target=".worlds-panel" hx-swap="outerHTML"`)
  - Delete button — blocked if games reference the world (validation error)
- **Create New World button** — replaces worlds list with empty form inline (no modal)

### World Form (Inline HTMX Swap)
Create/Edit uses inline HTMX swaps — no modal overlay:
- **Create flow**: Button `hx-get="/fragment/worlds/new" hx-target=".worlds-panel" hx-swap="outerHTML"` — replaces panel with empty form
- **Edit flow**: Button `hx-get="/worlds/:key/edit" hx-target=".worlds-panel" hx-swap="outerHTML"` — replaces panel with pre-populated form
- **Submit**: Form posts to:
  - Create: `POST /worlds` with full `WorldForm` data
  - Update: `POST /worlds/:key` with updated world data
- **Cancel**: Button `hx-get="/fragment/worlds" hx-target=".worlds-panel" hx-swap="outerHTML"` — returns to worlds list
- **Fields**:
  - **Key** — unique identifier (readonly in edit mode)
  - **Name** — display name
  - **Description** — world lore/description
  - **Global Rules** — one rule per line
  - **Starting Room ID** — initial room for new games
  - **Player Persona** — dropdown of available personas (by key)
  - **Default Room Image** — optional default image path
  - **Map JSON** — room/region structure as JSON
  - **Scenarios JSON** — starting scenarios as JSON array
- **Refresh**: On success, handler returns re-rendered worlds panel HTML (inline HTMX swap replaces `.worlds-panel`); no full page reload

### Backend Implementation
- **Storage layer**: `Storage::get_world(key)` returns `Option<WorldWithMap>` with `world_id` for updates
- **Service layer**: `ApplicationService::get_world()`, `update_world(id, world_card, map)`
- **Validation**: Delete blocked if `games` table has rows with matching `world_key`
- **HTMX handlers**:
  - `new_world_form_handler` — renders create form with persona dropdown
  - `edit_world_form_handler` — renders edit form with pre-populated data
  - `create_world_handler` — creates world, returns re-rendered worlds panel HTML
  - `update_world_handler` — updates world by ID, returns re-rendered worlds panel HTML
  - `delete_world_handler` — validates no games reference world, deletes if safe
  - `list_personas_fragment` — returns persona `<option>` tags for dropdown

## CSS Classes
- `.games-panel` — container for games tab content (in `assets/games.css`)
- `.games-section` — section container within games panel (Active Game, New Game, Saved Games)
- `.worlds-panel` — container for worlds list with game counts (in `assets/worlds.css`)
- `.world-form-container` — inline form container styling (in `assets/worlds.css`)
- `.btn-primary` — shared green button utility class (gradient, green border/text, padding 8px 20px, font-size base)
- `.btn-cyan` — shared cyan button utility class (gradient, cyan border/text)
- `.btn-danger` — shared red button utility class (gradient, red border/text)
- `.btn-new-world` — "Create New World" button (uses `.btn-primary` + layout override for `hx-get` inline swap)
- `.btn-reset-small` — small reset icon button on Active Game card (border-only red, ↺ glyph)
- `.active-game-info` — flex wrapper for game name + badges on the Active Game card
- `.new-game-form` — New Game form container (flex column)
- `.new-game-form .form-row` — side-by-side select + submit button layout
- `.form-group` — label + input wrapper
- `.json-editor` — monospace textarea for JSON fields
- `.location-header` - Room name in location entry, inline, green bold (#4ade80)
- `.location-timestamp` - Timestamp for location entry, inline after room name
- `.event-header` - Event name in trigger event entry, inline, blue/cyan bold (#38bdf8)
- `.event-timestamp` - Timestamp for event entry, inline after event name
- `.log-entry.narration` - AI narration, left-aligned, cyan-tinted bubble
- `.log-entry.dialogue` - Character dialogue, left-aligned, orange-tinted
- `.log-entry.system` - System messages, centered, yellow
- `.log-entry.input` - User input, right-aligned, darker gray bubble
- `.log-entry.event` - Trigger event header, inline header with no edit/retry buttons
- `.log-entry .timestamp` - Small gray timestamp above message
- `.log-entry .sender` - Bold name above message content
- `.log-entry .text` - Message content
- `.log-entry .edit-btn` - Edit pencil icon, always visible (opacity: 1)
- `.log-entry .retrigger-btn` - Retrigger event icon, last narration when trigger available
- `.swipe-controls` - Container for swipe navigation arrows and counter
- `.swipe-btn` - Swipe arrow buttons (left/right)
- `.swipe-counter` - Swipe index counter (e.g., "1 / 3")
- `.log-entry .delete-btn` - Delete trash icon, always visible
- `.log-entry .check-btn` - Check icon, input entries only
- `.message-header` - Flex container for message info + actions
- `.message-actions` - Flex container for action buttons (top-right)
- `.action-btn` - Base style for all message action buttons
- `.edit-textarea` - Inline edit textarea, full width, no resize
- `.save-btn` - Save/confirm button (green on hover)
- `.cancel-btn` - Cancel button (red on hover)
- `@keyframes fadeIn` - Opacity 0 to 1 for new messages

### 5. LLM Messages Tab
Forensics panel showing the last 50 LLM calls with full request/response visibility.
- **Content**: Compact list of LLM calls with agent name, backend, model, and parsed response
- **Expandable rows**: Click to reveal raw request JSON and raw response JSON
- **Order**: Oldest-first (newest at bottom), matching chronological narrative flow
- **Empty state**: "No LLM messages yet" when no calls have been logged
- **Auto-pruning**: SQLite storage caps at 50 rows globally; oldest evicted on insert
- **HTMX Polling**: `hx-get="/fragment/llm-messages" hx-trigger="load, every 4s"`

#### LLM Message CSS Classes
- `.llm-message-list` - Container for the message list
- `.llm-message-item` - Individual message card
- `.llm-message-header` - Top row with agent, backend, model, timestamp
- `.llm-message-summary` - Collapsed view showing parsed response
- `.llm-message-detail` - Expanded view with raw JSON (hidden by default)
- `.llm-message-toggle` - Click target to expand/collapse details