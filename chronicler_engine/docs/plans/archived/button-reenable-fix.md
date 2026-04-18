# Plan: Send/Stop Button Re-enable Fix

## Status: Completed

## Problem

The Send/Stop button in the HTMX dashboard never gets re-enabled after submitting a command:
- Button shows "Stop" and stays disabled
- Status shows "Thinking..." and never changes back to "Ready"
- User cannot submit new commands without refreshing the page

## Root Cause

1. Form submission disables button via `hx-on::after-request="updateToThinking()"`
2. No mechanism re-enables the button when LLM finishes
3. Server has `/status/generating` endpoint returning "generating" or "idle"
4. Status display element (`#status-display`) had no polling to check status

## Solution

1. Add HTMX polling to `#status-display` to poll `/status/generating` every 5 seconds
2. Add `onStatusPoll()` JS handler to re-enable button when status returns "idle"

## Files Changed

| File | Change |
|------|-------|
| `assets/index.html` | Added polling to status-display, added onStatusPoll function |
| `tests/behavior_tests.rs` | Added test_button_reenabled_after_command |

## Implementation

### index.html Changes
```html
<div class="status ready" id="status-display" 
     hx-get="/status/generating" 
     hx-trigger="load, every 5s" 
     hx-swap="innerHTML"
     hx-on::after-swap="onStatusPoll(this)"><span class="status ready">Ready</span></div>
```

```javascript
function onStatusPoll(el) {
    const text = el.textContent.trim();
    if (text === 'idle') {
        el.innerHTML = '<span class="status ready">Ready</span>';
        setButtonState(false);  // Re-enable button
    } else if (text === 'generating') {
        el.innerHTML = '<span class="status thinking">Thinking...</span>';
        setButtonState(true);  // Keep disabled
    }
}
```

### test Changes

Test verifies:
1. Button is enabled initially
2. Button gets disabled after submit
3. Button is re-enabled after LLM responds (via wait_for_llm_idle polling)

## Validation
```bash
cargo fmt      # Pass
cargo clippy  # Pass
cargo test --test behavior_tests  # New test passes
```