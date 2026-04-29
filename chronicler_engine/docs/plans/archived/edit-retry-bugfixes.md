# Plan: Fix Edit/Retry UI Bugs

## Problem Statement

The current edit and retry implementation has three bugs:
1. Edit is cancelled when the 2-second HTMX poll replaces the story-log DOM
2. Quotes are removed when editing because HTML content is re-parsed as markdown
3. Retry button is in the header instead of on the last AI message

## Root Causes

### Bug 1: Edit Cancelled by Poll
- `hx-trigger="load, every 2s"` replaces entire story-log DOM every 2 seconds
- Inline textarea created by JavaScript is destroyed when HTMX refreshes

### Bug 2: Quotes Removed on Save
- `LogEntry.text` is stored as markdown, then converted to HTML for display
- On edit: HTML content is taken from `textContent` (not the original markdown)
- On save: HTML is re-parsed as markdown, corrupting the content

### Bug 3: Retry Button Location
- Retry button is in header area, not on the message itself
- SillyTavern has `.mes_continue` button on the last AI message

## Solution

### Fix Bug 1: Pause HTMX Polling During Edit

In `assets/index.html`:
- Add `isEditing` flag to track edit mode
- When `showEditForm()` is called, set `isEditing = true` and pause the HTMX polling
- When `submitEdit()` or `cancelEdit()` is called, set `isEditing = false` and resume polling
- Use `htmx.disable()` or CSS to prevent the poll from firing

```javascript
let isEditing = false;

function showEditForm(id) {
    isEditing = true;
    // Stop the 2s poll from firing
    const storyLog = document.getElementById('story-log');
    htmx.setAttr(storyLog, 'hx-trigger', 'none');
    // ... rest of edit logic
}

function cancelEdit() {
    // ... restore UI
    isEditing = false;
    htmx.setAttr(storyLog, 'hx-trigger', 'load, every 2s');
}
```

### Fix Bug 2: Store Raw Text for Editing

In `src/server/templates.rs`:
- Add `data-raw-text` attribute to each log entry with the raw text
- The raw text is the value of `entry.text` (before markdown conversion)

In `assets/index.html`:
- On edit, read from `entry.querySelector('.text').getAttribute('data-raw-text')` instead of `textContent`
- On save, store raw text directly without markdown re-parsing

### Fix Bug 3: Retry Button on Last AI Message

In `src/server/templates.rs`:
- Add retry button to only the last AI message (Narration or Dialogue log_type)
- Not shown on Input, System, or location headers

In `assets/index.html`:
- Add `submitRetry()` handler that calls `/retry` endpoint

## Files to Modify

| File | Changes |
|------|---------|
| `assets/index.html` | Pause polling during edit, read data-raw-text, retry button handler |
| `src/server/templates.rs` | Add data-raw-text attribute, add retry button to last AI message |
| `assets/styles.css` | Style retry button on message |

## Implementation Order

1. Fix Bug 2 first (data-raw-text storage)
2. Fix Bug 1 (polling pause)
3. Fix Bug 3 (retry button on message)
4. Test all three bugs are fixed

## Acceptance Criteria

1. Edit a message, wait 10 seconds - edit is NOT cancelled by poll
2. Edit a message with quotes, save, then edit again - quotes are preserved
3. Retry button appears on the last AI message (not in header)
4. All 239 tests pass