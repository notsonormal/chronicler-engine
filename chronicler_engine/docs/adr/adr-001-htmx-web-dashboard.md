# ADR-001: HTMX Web Dashboard Architecture

**Date:** 2025-04-12

---

## Context

The Chronicler Engine originally used a Terminal User Interface (TUI) with Ratatui for visual display. This presented challenges:
- Terminal graphics protocols (Sixel, Kitty) had limited compatibility
- Portrait rendering was complex to implement
- Deployment required terminal emulation support
- Testing required browser automation (Playwright)

The team sought a solution that provided:
- Rich visual displays (images, portraits)
- Cross-platform compatibility
- Easier testing
- Real-time updates

---

## Decision

**Adopt HTMX web dashboard with Server-Sent Events (SSE) for real-time updates.**

The Chronicler Engine now uses:
1. **Axum HTTP server** on port 3000
2. **HTMX** for partial page updates (fragment swapping)
3. **SSE** for server→client real-time push
4. **WebSocket** was initially used but replaced with SSE (see ADR-002)

### Architecture Components

```
Client Browser (HTMX + SSE)
        ↓ HTTP POST /action
Axum HTTP Server
        ↓
   ┌────┴────┐
   │         │
Game State │   LLM Backend
(Mutex)    │   (OpenRouter)
   │         │
   └────┬────┘
        ↓
   Broadcast Channel
        ↓
   SSE → Client
```

### UI Layout

- **Header** (48px): Game title + current location
- **Main Body** (flex): 70% story log / 30% visual sidebar
- **Action Area** (60px): Command input + status indicator

---

## Consequences

### Positive
- Cross-platform: Works in any modern browser
- Rich visuals: CSS styling, images, portraits
- Easier testing: HTTP endpoints can be tested directly
- Real-time: SSE provides push updates without WebSocket complexity
- HTMX simplicity: No custom JavaScript required

### Negative
- Requires web server (more complex than CLI)
- SSE has reconnection behavior to handle
- Browser dependency for players

### Trade-offs
- Chose SSE over WebSocket for reliability with HTMX
- Chose HTMX over SPA (React/Vue) for simplicity and server-side rendering

---

## Related ADRs

- [ADR-002: Server-Sent Events for Real-Time Updates](./adr-002-sse-realtime-updates.md) - Transport layer decision
- [ADR-003: Askama Template Engine](./adr-003-askama-templates.md) - Template rendering
- [ADR-006: Quantifier-Driven Game Systems](./adr-006-quantifier-systems.md) - Quantifier-powered features

---

## History

- **2025-04-12**: Initial HTMX migration (hx_migration.md plan)
- **2026-04-19**: SSE migration replaces WebSocket (see ADR-002)

---

## References

- Architecture: `docs/architecture/system.md`

## Historical Note

This decision replaced an earlier TUI approach using Ratatui. The HTMX migration was the pivotal moment the engine moved from terminal-based to web-based UI.
