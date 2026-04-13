# Specification: Dashboard UI

## Overview
The Chronicler Engine presents a web-based HTMX dashboard for player interaction. The UI provides narrative immersion, visual grounding, and user input in a modern chat-app aesthetic inspired by SillyTavern.

## The Dashboard Layout

### 1. Header (48px height)
Displays system-level context.
- **Content**: Game title + current location name
- **Style**: Location name in green bold (#00ff00)

### 2. Main Body (Flex: 1)
Horizontal split into story context and visual context:

- **Story Log (80%)**: Scrollable history of narration with chat-bubble styling
  - **Styles**:
    - User input (right-aligned, darker gray background #2a2a2a)
    - AI/Narration (left-aligned, dark cyan background #1a3a3a)
    - System messages (center-aligned, yellow #ffff00)
    - Character name prominent above message (bold, larger)
    - Subtle timestamp (HH:MM format, small gray)
    - Fade-in animation for new messages
- **Visual Sidebar (20%)**: 
  - Location Image (40% height): Displays the current room's visual
  - NPC Portraits (60%): Vertical stack of present NPCs

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
- Story-log polls `/fragment/story-log` every 5 seconds for new content
- Status-display polls `/status/generating` every 5 seconds to update button state
- New narration appears automatically with fade-in effect
- Button changes state during LLM processing
- No manual refresh required

## Frontend Implementation
- **HTMX**: Handles partial page updates via `hx-post` and `hx-target`
- **HTMX Polling**: `hx-trigger="load, every 5s"` for real-time updates
- **Styling**: Modern chat-app aesthetic with chat bubbles, fade animations

## Data Model

### LogEntry
```rust
pub struct LogEntry {
    pub sender: Option<String>,
    pub text: String,
    pub log_type: LogType,
    pub timestamp: chrono::DateTime<chrono::Utc>,  // Added for timestamps
}
```

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
- `.log-entry.narration` - AI narration, left-aligned, cyan-tinted bubble
- `.log-entry.dialogue` - Character dialogue, left-aligned, orange-tinted
- `.log-entry.system` - System messages, centered, yellow
- `.log-entry.input` - User input, right-aligned, darker gray bubble
- `.log-entry .timestamp` - Small gray timestamp above message
- `.log-entry .sender` - Bold name above message content
- `.log-entry .text` - Message content
- `@keyframes fadeIn` - Opacity 0 to 1 for new messages