# Specification: Dashboard UI

## Overview
The Chronicler Engine presents a web-based HTMX dashboard for player interaction. The UI provides narrative immersion, visual grounding, and user input in a modern chat-app aesthetic inspired by SillyTavern.

## The Dashboard Layout

### 1. Header (48px height)
Displays system-level context.
- **Content**: Game title only (location displayed in story log)

### 2. Main Body (Flex: 1)
Horizontal split into story context and visual context:

- **Story Log (80%)**: Scrollable history of narration with chat-bubble styling
  - **Styles**:
    - **Location headers**: Inline "Room Name - HH:MM", green color (#4ade80), bold
    - User input (right-aligned, darker gray background #2a2a2a)
    - AI/Narration (left-aligned, dark cyan background #1a3a3a)
    - System messages (center-aligned, yellow #ffff00)
    - Character name prominent above message (bold, larger)
    - Subtle timestamp (HH:MM format, small gray)
    - Fade-in animation for new messages
- **Visual Sidebar (20%)**:
  - Location Image (top): Full-width location image, max-height 200px, object-fit contain (scales to fit without cropping)
  - NPC Portraits (bottom): Horizontal scrollable row, 80×80px square images, object-fit cover

### 3. Action Area (64px height)
Interactive zone for player input.
- **Content**: Text input field + send button + status indicator
- **Button States**:
  - Ready: Green button with "Send" text and play icon (▶)
  - Thinking: Green button with "Stop" text and square icon (■), disabled input
- **Status States**:
  - "Ready" - Green (#00ff00), awaiting input
  - "Thinking..." - Yellow (#ffff00) with pulse animation, LLM generating response

## Real-Time Updates
The dashboard uses HTMX polling for live updates:
- Story-log polls `/fragment/story-log` every 2 seconds for new content
- Visual sidebar polls `/fragment/visual-sidebar` every 5 seconds
- Status-display polls `/status/generating` every 5 seconds to update button state
- Action hints poll `/hints` every 5 seconds
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
}
```

HTML template renders with `data-raw-text` attribute for inline editing:

```html
<div class="log-entry" data-id="{{ entry.id }}" data-raw-text="{{ entry.raw_text }}">
    <span class="text">{{ entry.text }}</span>
    <button class="edit-btn">✏️</button>
    {% if last_ai_message %}
    <button class="retry-btn">🔄</button>
    {% endif %}
</div>
```

## Edit Flow (SillyTavern Pattern)

1. Click edit button → textarea replaces text span, polling pauses
2. Edit text in textarea (uses `data-raw-text`, not HTML textContent)
3. Click save → textarea value sent to server, stored as raw text
4. Click cancel → restore original text, resume polling

## Button Logic (JavaScript)
1. Monitor status element changes via MutationObserver or HTMX events
2. When status contains "Thinking...":
   - Disable the submit button
   - Change button text from "▶ Send" to "■ Stop"
   - Keep button green (no red)
3. When status returns to "Ready":
   - Re-enable the submit button
   - Change button text back to "▶ Send"

## CSS Classes
- `.location-header` - Room name in location entry, inline, green bold (#4ade80)
- `.location-timestamp` - Timestamp for location entry, inline after room name
- `.log-entry.narration` - AI narration, left-aligned, cyan-tinted bubble
- `.log-entry.dialogue` - Character dialogue, left-aligned, orange-tinted
- `.log-entry.system` - System messages, centered, yellow
- `.log-entry.input` - User input, right-aligned, darker gray bubble
- `.log-entry .timestamp` - Small gray timestamp above message
- `.log-entry .sender` - Bold name above message content
- `.log-entry .text` - Message content
- `.log-entry .edit-btn` - Edit pencil icon, always visible (opacity: 1)
- `.log-entry .retry-btn` - Retry refresh icon, last AI message only
- `.edit-textarea` - Inline edit textarea, full width, no resize
- `.save-btn` - Save/confirm button (green on hover)
- `.cancel-btn` - Cancel button (red on hover)
- `@keyframes fadeIn` - Opacity 0 to 1 for new messages