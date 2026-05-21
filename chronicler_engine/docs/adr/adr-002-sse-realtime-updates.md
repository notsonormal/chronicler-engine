# ADR-002: Server-Sent Events for Real-Time Updates

**Date:** 2026-04-19

---

## Context

The HTMX migration (ADR-001) initially used WebSocket for real-time server→client updates:
```
Client ──HTTP POST──> Server (/action endpoint)
Client <──WebSocket── Server (/ws endpoint)
```

Issues identified:
1. **Reliability**: The HTMX WebSocket extension (`hx-ext="ws"`) was flaky in production
2. **Custom code**: Required `ws.js` for WebSocket handling
3. **Complexity**: WebSocket protocol handling vs HTTP simplicity

---

## Decision

**Replace WebSocket with Server-Sent Events (SSE).**

```
Client ──HTTP POST──> Server (/action endpoint)
Client <────SSE──── Server (/sse endpoint)
```

### Why SSE Over WebSocket

| Factor | WebSocket | SSE |
|--------|----------|-----|
| HTMX support | Requires `hx-ext="ws"` (flaky) | Native `hx-ext="sse"` |
| Direction | Bidirectional | Server→client only (our use case) |
| Reconnection | Manual | Automatic |
| Protocol | Binary/frame | Plain HTTP |
| Firewall-friendly | No | Yes |

### Architecture

The server exposes a dedicated SSE endpoint that the client connects to via HTMX's SSE extension. A keep-alive heartbeat prevents connection timeouts. The frontend subscribes to event streams that correspond to server-side broadcast channels.

---

## Consequences

### Positive
- More reliable than WebSocket with HTMX
- No custom JavaScript (`ws.js` removed)
- Native HTMX support
- Automatic reconnection
- Simpler protocol (HTTP)

### Negative
- Unidirectional only (but sufficient for our use case)
- SSE has max 6 connections limit per browser (not hit in practice)
- No binary data support (not needed)

### Trade-offs
- Chose SSE reliability over WebSocket bidirectionality
- We don't need client→server WebSocket (HTTP POST handles commands)

---

## Related ADRs

- [ADR-001: HTMX Web Dashboard Architecture](./adr-001-htmx-web-dashboard.md) - Foundation

---

## History

- **2025-04-12**: WebSocket implemented in HTMX migration
- **2026-04-19**: Replaced WebSocket with SSE

---

## Historical Note

This decision was made after experiencing WebSocket reliability issues with HTMX in production. SSE provided a more stable solution.