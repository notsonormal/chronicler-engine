---
name: chronicler-ui-investigator
description: "Investigation workflow for Chronicler Engine UI - used for testing, debugging, and post-plan verification. Triggers on: /chronicler-ui, /ui-investigate, investigate chronicler ui, test chronicler ui, debug ui"
argument-hint: "<action> [port] [world]"
---

# Chronicler UI Investigator

Investigation workflow for Chronicler Engine UI - used for testing, debugging, and post-plan verification.

## Overview

Provides browser automation to:
1. Launch Chronicler server (or connect to running instance)
2. Navigate to UI
3. Capture state (accessibility tree, screenshots, console errors)
4. Return findings for caller to interpret

**Does NOT include specific expectations** - caller provides what to look for.

## Prerequisites

- Requires a browser automation MCP (e.g. chrome-devtools) configured in the host.
- Chronicler Engine project at `chronicler_engine/`

## Usage Patterns

### For Testing
```
/chronicler-ui test [port]
```
- Starts fresh server
- Captures baseline UI state
- Returns accessibility tree for verification

### For Debugging
```
/chronicler-ui debug <port> <world>
```
- Connects to running server
- Focus on console errors and DOM state
- Useful when issue is already reproduced

### For Post-Plan Verification
```
/chronicler-ui verify [port]
```
- After plan implementation
- Captures full state for comparison
- Returns complete snapshot

---

## Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| `port` | 3000 | Server port |
| `world` | redmist_estate | World to load |
| `action` | test | test, debug, or verify |

---

## Workflow Commands

### Step 1: Ensure Server Running

**Option A: Start new server**
```javascript
// In PTY terminal
cargo run -- --world <world> --port <port>
```

**Option B: Use existing**
```javascript
// Skip if already running at http://127.0.0.1:<port>
```

### Step 2: Navigate

```javascript
browser_navigate(url="http://127.0.0.1:3000")
browser_wait_for(text="Chronicler")  // Or wait for specific element
```

### Step 3: Capture State

```javascript
// Console errors (critical for debugging)
browser_console_messages(level="error", all=true)

// Accessibility tree (for structural analysis)
browser_snapshot(depth=3)  // depth adjustable

// Screenshot (for visual analysis)
browser_take_screenshot(filename="chronicler-ui.png")
```

---

## Return Format

The skill returns raw data - caller interprets:

| Data | Use Case |
|------|----------|
| `browser_console_messages` | Detect JS errors, 404s |
| `browser_snapshot` | Verify elements present, layout structure |
| `browser_take_screenshot` | Visual regression, color checking |

---

## Mandatory Screenshot Verification

**Every UI investigation MUST end with a screenshot.** This is non-negotiable.

After making any changes and before claiming verification:
1. Navigate to the page
2. Take a screenshot: `browser_take_screenshot()`
3. **Look at the screenshot** — visually confirm the layout is correct
4. Report what you see and whether it matches expectations

**Do NOT claim "verified" based on:**
- Accessibility tree alone (doesn't show visual layout)
- Console logs alone (no errors ≠ correct rendering)
- Subagent reports alone (you must see it yourself)
- Test passes alone (CSS bugs don't fail tests)

**Only claim verified when you have:**
- A screenshot showing the actual rendered page
- Personally confirmed the visual result matches expectations

---

## Customization Guide

### Testing - Check element presence
```javascript
browser_snapshot(depth=2)
// Caller verifies specific elements exist
```

### Debugging - Find what's broken
```javascript
browser_console_messages(level="error", all=true)
browser_snapshot(depth=5)  // Deeper for detail
```

### Post-Plan - Full capture
```javascript
browser_console_messages(level="error", all=true)
browser_console_messages(level="warning", all=true)
browser_snapshot(depth=4)
browser_take_screenshot(filename="post-plan-ui.png", fullPage=true)
```

---

## Error Handling

| Issue | Check |
|-------|-------|
| Server won't start | `cargo run` manually to see errors |
| Page not loading | Verify server started, check port |
| Elements missing | Check world loaded correctly |
| Console errors | Analyze returned errors |

---

## Integration Points

- **In tests**: Capture baseline, compare post-change
- **In debugging**: Get state when issue occurs
- **In verification**: After plan implementation completes

---

## Notes

- No expected values hardcoded - caller provides assertions
- Screenshot saved to current working directory
- Use `browser_wait_for(text="...")` to wait for dynamic content
- For WebSocket state, check if "Connected" status appears