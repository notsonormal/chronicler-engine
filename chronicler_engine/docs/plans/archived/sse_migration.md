# SSE Migration Plan: WebSocket to Server-Sent Events

## Overview

### What We're Doing
Replace the existing WebSocket implementation with Server-Sent Events (SSE) for real-time LLM response updates in the Chronicler Engine HTMX dashboard.

### Why We're Doing It
1. **Reliability**: The HTMX WebSocket extension (`hx-ext="ws"`) is flaky and unreliable in production
2. **Simplicity**: SSE is unidirectional (server→client), perfect for our use case - we don't need client→server WebSocket
3. **Proven Pattern**: SillyTavern uses SSE for exactly this scenario (HTTP for client→server, SSE for server→client)
4. **Native HTMX Support**: HTMX has built-in SSE support via `hx-ext="sse"` with zero custom code
5. **No Binary Protocol**: SSE uses plain HTTP, making it firewall/proxy friendly

### Current Architecture (to be replaced)
```
Client ──HTTP POST──> Server (/action endpoint)
Client <──WebSocket── Server (/ws endpoint) ← Unreliable
```

### Target Architecture
```
Client ──HTTP POST──> Server (/action endpoint)
Client <────SSE────── Server (/sse endpoint) ← Reliable
```

---

## Files to Modify

### 1. `src/server/mod.rs`
- **Change**: Replace WebSocket handler with SSE handler
- **Current**: `ws_handler()` function with WebSocket upgrade
- **New**: `sse_handler()` function using `axum::response::sse::Sse`
- **Route change**: `/ws` → `/sse`

### 2. `src/server/hub.rs`
- **No changes needed**: Broadcast channel works identically for both WebSocket and SSE
- **Optional**: Add helper method for SSE-specific message formatting (if needed)

### 3. `src/server/fragments.rs`
- **No changes needed**: Fragment rendering and broadcast logic unchanged
- **Verify**: `broadcast_state()` function continues to work

### 4. `assets/index.html`
- **Change**: Replace `hx-ext="ws" ws-connect="/ws"` with `hx-ext="sse" sse-connect="/sse"`
- **Change**: Update event listeners from `htmx:ws*` to `htmx:sse*`
- **Remove**: `<script src="ws.js"></script>` reference (no longer needed)

### 5. `assets/ws.js`
- **Action**: Delete this file entirely (HTMX's native SSE doesn't need it)

---

## Implementation Steps

### Step 1: Add SSE Support to Server (`src/server/mod.rs`)

Add the SSE response type and replace the WebSocket handler:

```rust
// Add to imports
use axum::response::sse::{Event, Sse};
use std::convert::Infallible;
use std::convert::TryFrom;
use std::time::Duration;
use tokio_stream::StreamExt;

// Add new route in router
.route("/sse", get(sse_handler))

// Replace ws_handler with sse_handler
async fn sse_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    eprintln!("[SSE] Client connected");
    
    // Subscribe to hub broadcasts
    let mut rx = state.hub.subscribe();
    
    // Create stream that yields SSE events
    let stream = async_stream::stream! {
        // Send initial connection event
        yield Event::default()
            .data("{\"type\":\"connected\"}")
            .into();
        
        // Send initial story-log fragment
        if let Ok(html) = fragments::render_story_log(&state) {
            yield Event::default()
                .data(serde_json::json!({
                    "type": "update",
                    "fragment": "story-log",
                    "html": html
                }).to_string())
                .into();
        }
        
        // Stream messages from hub
        while let Ok(msg) = rx.recv().await {
            yield Event::default()
                .data(msg)
                .into();
        }
    };
    
    Sse::new(stream)
        .keep_alive(axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("{\"type\":\"ping\"}"))
}
```

### Step 2: Update HTML Frontend (`assets/index.html`)

Replace the WebSocket-specific attributes with SSE:

```html
<!-- Change from -->
<body hx-ext="ws" ws-connect="/ws">

<!-- To -->
<body hx-ext="sse" sse-connect="/sse" sse-swarp="innerHTML">
```

Remove the custom WebSocket script reference:

```html
<!-- Remove this line -->
<script src="ws.js"></script>
```

Update event listeners from WebSocket to SSE:

```javascript
// Replace these handlers:
document.addEventListener('htmx:wsOpen', function(evt) { ... });
document.addEventListener('htmx:wsClose', function(evt) { ... });
document.addEventListener('htmx:wsConnecting', function(evt) { ... });
document.addEventListener('htmx:wsError', function(evt) { ... });
document.addEventListener('htmx:wsAfterMessage', function(evt) { ... });

// With SSE handlers:
document.addEventListener('htmx:sseOpen', function(evt) { ... });
document.addEventListener('htmx:sseClose', function(evt) { ... });
document.addEventListener('htmx:sseError', function(evt) { ... });
document.addEventListener('htmx:sseMessage', function(evt) { ... });
```

### Step 3: Delete WebSocket Extension (`assets/ws.js`)

Delete the entire file - HTMX's native SSE support doesn't require it.

### Step 4: Verify Fragment Broadcasting (`src/server/fragments.rs`)

Ensure the `broadcast_state()` function continues to work. The current implementation broadcasts JSON messages that the frontend parses. This should work identically with SSE.

**Verification checklist**:
- [ ] `broadcast_state()` sends JSON with `type`, `fragment`, and `html` fields
- [ ] Frontend's `htmx:sseMessage` handler parses this JSON
- [ ] Auto-scroll to bottom works for story-log
- [ ] Status reset to "Ready" after story-log update
- [ ] Button state management works

### Step 5: Update Architecture Document (`docs/architecture/system.md`)

Update line 62 from:
```
Real-time updates via WebSocket.
```
To:
```
Real-time updates via Server-Sent Events (SSE).
```

---

## Testing Strategy

### Manual Testing Checklist

1. **Initial Load**
   - [ ] Dashboard loads without errors
   - [ ] SSE connection established (check browser console for `htmx:sseOpen`)
   - [ ] Initial story-log renders

2. **Command Submission**
   - [ ] Enter a command and submit
   - [ ] Button changes to "Stop" (disabled state)
   - [ ] Status shows "Thinking..."
   - [ ] LLM response appears in real-time via SSE

3. **Real-time Updates**
   - [ ] New messages appear at bottom of story-log
   - [ ] Story-log auto-scrolls to newest message
   - [ ] Location updates in header when moving rooms
   - [ ] Visual sidebar updates with new room images

4. **Status Reset**
   - [ ] After LLM response completes, status shows "Ready"
   - [ ] Button changes back to "Send" (enabled state)

5. **Reconnection**
   - [ ] Browser reconnects automatically after network hiccup
   - [ ] No duplicate messages on reconnect

### Automated Test Scenarios

Create a test script that:
1. Starts the server
2. Opens SSE connection via curl
3. Sends command via HTTP POST
4. Verifies SSE stream receives story-log update

```bash
# Test SSE endpoint
curl -N -H "Accept: text/event-stream" http://127.0.0.1:3000/sse
```

---

## Rollback Plan

If SSE implementation fails:

1. **Revert `src/server/mod.rs`**
   - Change `/sse` route back to `/ws`
   - Restore `ws_handler()` and `handle_socket()` functions

2. **Revert `assets/index.html`**
   - Restore `hx-ext="ws" ws-connect="/ws"`
   - Restore WebSocket event listeners
   - Re-add `<script src="ws.js"></script>`

3. **Keep `assets/ws.js`**
   - No deletion needed (it's already in git)

4. **Revert architecture doc**
   - Restore "Real-time updates via WebSocket"

---

## Success Criteria

### Functional Requirements
- [ ] User commands are sent via HTTP POST (unchanged)
- [ ] LLM responses appear in real-time via SSE
- [ ] New messages appear at bottom of story-log (chronological order)
- [ ] Button shows "Stop" while thinking, "Send" while ready
- [ ] Chat bubble styling maintained (narration=cyan, dialogue=orange, etc.)

### Technical Requirements
- [ ] `cargo build` succeeds without warnings
- [ ] `cargo clippy` passes
- [ ] `cargo test` passes
- [ ] SSE connection establishes within 1 second
- [ ] No WebSocket-related errors in browser console
- [ ] SSE reconnect works after network interruption

### Visual Checkpoints
1. **Initial State**: Dashboard shows "Ready" with green "Send" button
2. **During Generation**: Button disabled, shows "Stop", status shows "Thinking..." in yellow
3. **After Response**: Cyan narration bubble appears at bottom, status shows "Ready", button enabled
4. **Error State**: If LLM fails, error message in yellow system bubble

---

## Reference: SillyTavern Architecture Pattern

SillyTavern (production LLM chat application) uses this exact pattern:

```
┌─────────────────────────────────────────────────────────┐
│                    Client Browser                        │
├─────────────────────────────────────────────────────────┤
│  HTMX + SSE Extension                                   │
│  - HTTP POST for user messages                          │
│  - SSE stream for AI responses                          │
└───────────────────────┬─────────────────────────────────┘
                        │
                        │ HTTP + SSE
                        ▼
┌─────────────────────────────────────────────────────────┐
│                     Server (Axum)                       │
├─────────────────────────────────────────────────────────┤
│  - /action (POST) → Process user input                  │
│  - /sse (GET) → Stream responses to client              │
│  - Hub broadcast for state changes                      │
└─────────────────────────────────────────────────────────┘
```

This architecture is proven in production at scale with SillyTavern's thousands of concurrent users.

---

## Implementation Notes

### Why Keep HTTP for Client→Server
- Simpler than WebSocket for request/response
- Works with HTMX's existing form handling
- Easier to debug (plain HTTP traffic)
- Browser back button works naturally
- No connection management needed

### Why Use SSE for Server→Client
- Built into HTMX (`hx-ext="sse"`)
- Automatic reconnection
- Works over HTTP/1.1
- No custom JavaScript needed
- Proven reliability in production

### Hub Broadcasting (unchanged)
The `Hub` uses tokio's broadcast channel. This works equally well for:
- WebSocket messages (current)
- SSE events (new)

Both consumers receive the same string messages - only the transport layer changes.

---

## Related Documentation

- HTMX SSE Extension: https://htmx.org/extensions/sse/
- Axum SSE: https://docs.rs/axum/latest/axum/response/sse/index.html
- SillyTavern SSE Implementation: Internal reference (production proven)
