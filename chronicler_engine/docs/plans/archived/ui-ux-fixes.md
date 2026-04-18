# UI/UX Bug Fixes Spec

**Created:** 2026-04-13
**Status:** Draft
**Issues:** 4

## Issue 1: LLM Response Text Formatting

### Problem
LLM responses contain paragraphs separated by newlines (`\n`). These are not rendered as HTML, showing as literal `\n` characters.

### Current Behavior
```rust
// fragments.rs:90 - text is just escaped, newlines preserved as literals
html_escape(&entry.text)
```

### Expected Behavior
- Single newlines → `<br>` tags (inline breaks)
- Double newlines → `<p>` tags (paragraphs)
- Preserves the narrative structure from LLM

### Solution
Create a helper function that converts newlines to HTML:
- `\n\n` → `</p><p>`
- `\n` → `<br>`
- Wrap in `<p>...</p>` for proper paragraph styling

---

## Issue 2: Logging Not Writing to File

### Problem
The app uses `log` crate but never initializes a logger. No log messages appear anywhere when running `cargo run`.

### Current Behavior
- `log::info!()`, `log::error!()` calls throughout code
- No `env_logger::init()` or similar in `main.rs`
- Logs silently dropped

### Expected Behavior
- Logs written to a file: `logs/chronicler_YYYYMMDD.log`
- Logs also written to stdout for development
- Configurable log level via environment variable

### Solution
1. Add `env_logger` crate to Cargo.toml (or use existing `log` with file target)
2. Initialize logger in `main.rs` with file output
3. Use `RUST_LOG` env var for level control (debug/info/warn/error)

---

## Issue 3: NPC Images Too Small

### Problem
NPC portrait images in the right sidebar are too small (48% width, max 180px height).

### Current Behavior
```css
// index.html:117-122
.npc-portrait { 
    width: 48%; 
    max-width: 48%;
}
.image-container img { 
    max-height: 180px; 
}
```

### Expected Behavior
- Larger images that fill the sidebar better
- Responsive sizing based on number of NPCs

### Solution
- Increase width to ~80-90% for single NPC
- Increase max-height to 300px or higher
- Use `flex` layout to give each NPC more space

---

## Issue 4: NPC Images Don't Update on Room Change

### Problem
The visual sidebar (with NPC portraits) only loads on page load (`hx-trigger="load"`). When moving to a new room, the images don't refresh.

### Current Behavior
```html
<!-- index.html:216 -->
<div class="visual-sidebar" hx-get="/fragment/visual-sidebar" hx-trigger="load"></div>
```

### Expected Behavior
- Visual sidebar should poll/update when room changes
- Same as story-log which uses `hx-trigger="load, every 5s"`

### Solution
Add polling trigger to visual-sidebar:
```html
hx-trigger="load, every 5s"
```
This ensures it refreshes every 5 seconds, catching room changes.

---

## Files to Modify

1. `src/server/fragments.rs` - Add text formatting helper
2. `src/main.rs` - Initialize logger with file output
3. `Cargo.toml` - (check if env_logger needed)
4. `assets/index.html` - Fix CSS for images, add polling trigger

---

## Acceptance Criteria

1. LLM responses with paragraphs render as proper HTML paragraphs
2. Logs appear in `logs/` directory with timestamps
3. NPC portraits are visibly larger
4. NPC images update when player moves to new room (within 5s)
5. All existing tests pass