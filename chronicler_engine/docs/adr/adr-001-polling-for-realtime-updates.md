# ADR-001: HTMX Polling over WebSocket for Real-Time Updates

## Date
2026-04-13

## Status
Accepted

## Context
The Chronicler Engine needs real-time UI updates when:
- User submits a command
- LLM generates narration
- Player moves to new room

We initially implemented WebSocket connections via HTMX SSE extension for real-time updates.

## Decision
We will use HTMX polling (every 5 seconds) instead of WebSocket for content updates.

## Reasons

### 1. Test Reliability
- WebSocket connections via HTMX SSE are unreliable in Playwright headless Chromium
- Tests failed intermittently with `ProtocolError("net::ERR_CONNECTION_REFUSED")` and `net::ERR_CONNECTION_RESET`
- Polling provides 100% reliable updates in all test environments

### 2. Simplicity
- No additional server infrastructure (no `tokio-tungstenite` for WS handling)
- Standard HTMX patterns work out of the box
- Fallback works even if JavaScript fails

### 3. Trade-offs Accepted
- **Delay**: Max 5-second delay between LLM completion and UI update
- **Efficiency**: Slightly more HTTP requests than WebSocket
- **Trade-off**: Reliability + simplicity > instant updates for our use case

## Implementation

### Server
- `/fragment/story-log` endpoint returns just log entries (no wrapper div)
- `/status/generating` endpoint returns "generating" or "idle" for test synchronization

### Client (index.html)
```html
<div id="story-log" 
     hx-get="/fragment/story-log" 
     hx-trigger="load, every 5s" 
     hx-swap="innerHTML">
```

### Tests
- `wait_for_llm_idle()` helper polls `/status/generating` until LLM completes
- Tests verify updates appear after polling catches the change

## Consequences

### Positive
- All UI tests pass reliably (80+ tests)
- Simpler server code
- No WS reconnection logic needed

### Negative
- 5-second maximum delay for updates
- Slightly more server load (polling requests)

## Related
- Replaces initial WebSocket implementation in `src/server/hub.rs`
- See `docs/plans/polling_migration.md` for migration notes