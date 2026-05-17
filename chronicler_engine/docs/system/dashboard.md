# Specification: Dashboard UI

> **Related Decisions**: [ADR-001](../adr/adr-001-htmx-web-dashboard.md), [ADR-002](../adr/adr-002-sse-realtime-updates.md), [ADR-003](../adr/adr-003-askama-templates.md)

## Overview
The Chronicler Engine presents a web-based HTMX dashboard for player interaction. The UI provides narrative immersion, visual grounding, and user input in a modern chat-app aesthetic inspired by SillyTavern.

## The Dashboard Layout

### 1. Header Bar (48px height)
Displays system-level context.
- **Content**: Game title (left), reset button (center-right), connection status (right)
- **Reset Button**: "Reset Game" button styled with danger/red tokens, uses `hx-post="/reset"` with `hx-confirm` dialog
- **Note**: Location is displayed in the story log, not the header

### 2. Tab Bar
Navigation between Game, LLM Messages, and Settings views.
- **Tabs**: Game | LLM Messages | Settings
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
      - Retry button (↻) on last AI message only (hidden when only one entry exists)
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
  - "Thinking..." - Yellow (#ffff00) with pulse animation, LLM generating response
- **Text Check Preview**: When spell/grammar issues are detected, the action area temporarily shows:
  - Original vs corrected text comparison
  - Issue tags (spell = orange, grammar = pink)
  - **Send Corrected** — submits corrected text to `/action`
  - **Send Original** — submits original text to `/action`
  - **Cancel** — restores normal action area

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
- **HTMX Polling**: `hx-trigger="load, every 5s"` for real-time updates
- **Styling**: Modern chat-app aesthetic with chat bubbles, fade animations
- **Templates**: Uses `askama` for compile-time validated HTML fragments (pilot)

## Data Model

### LogEntry
```rust
pub struct LogEntry {
    pub id: u64,                           // Unique auto-incrementing ID
    pub sender: Option<String>,
    pub text: String,                       // Raw text (markdown source)
    pub log_type: LogType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
```

### LogEntryView (Rendered)
```rust
pub struct LogEntryView {
    pub id: u64,
    pub text: SafeHtml,                     // Rendered HTML (markdown converted)
    pub raw_text: String,                   // Original markdown (for editing)
    pub log_type: String,
    pub is_location: bool,
    pub is_event: bool,
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
            {% if last_ai_message %}
            <button class="action-btn retry-btn" title="Retry">↻</button>
            {% endif %}
        </div>
    </div>
    <span class="text">{{ entry.text }}</span>
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

## Checkpoints
Bookmark system for saving/restoring specific snapshots:
- **Save checkpoint**: Button triggers `POST /checkpoint` at current snapshot
- **Checkpoint list**: `GET /fragment/checkpoints` renders saved checkpoints with restore/delete buttons
- **Restore**: `POST /checkpoint/:id/restore` loads the snapshot
- **Delete**: `POST /checkpoint/:id/delete` removes the bookmark

## CSS Classes
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
- `.log-entry .retry-btn` - Retry refresh icon, last AI message only
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