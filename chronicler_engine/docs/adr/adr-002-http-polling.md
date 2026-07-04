# ADR-002: HTTP Polling for Real-Time Updates

**Date:** 2026-04-19
**Status:** Accepted

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

**Replace WebSocket with HTTP polling.**

```
Client ──HTTP POST──> Server (/action endpoint)
Client ──HTTP GET──> Server (/fragment/* endpoints, polled every 2-5s)
```

### Why Polling Over WebSocket/SSE

| Factor | WebSocket | SSE | HTTP Polling |
|--------|----------|-----|--------------|
| HTMX support | Requires `hx-ext="ws"` (flaky) | Native `hx-ext="sse"` | Native `hx-trigger` |
| Direction | Bidirectional | Server→client only | Request→response |
| Reconnection | Manual | Automatic | Automatic (each request) |
| Protocol | Binary/frame | Plain HTTP | Plain HTTP |
| Firewall-friendly | No | Yes | Yes |
| Implementation | Complex endpoint | Requires SSE infrastructure | Simple fragment endpoints |

### Architecture

The frontend polls dedicated fragment endpoints at staggered intervals:
- `/fragment/story-log` — every 2 seconds
- `/status/generating` — every 5 seconds
- `/fragment/visual-sidebar` — every 5 seconds
- `/fragment/llm-messages` — every 4 seconds

No persistent connection is maintained. Each poll is an independent HTTP request.

---

## Consequences

### Positive
- More reliable than WebSocket with HTMX
- No custom JavaScript (`ws.js` removed)
- Native HTMX support via `hx-trigger`
- Simpler protocol (plain HTTP)
- No SSE infrastructure or broadcast channels needed

### Negative
- Slightly higher latency than push (max 2-5s depending on endpoint)
- More frequent HTTP requests than SSE
- No true "push" capability

### Trade-offs
- Chose polling reliability over WebSocket/SSE push semantics
- We don't need sub-second updates; 2-5s latency is acceptable for narrative text

---

## Related ADRs

- [ADR-001: HTMX Web Dashboard Architecture](./adr-001-htmx-web-dashboard.md) - Foundation

---

## History

- **2025-04-12**: WebSocket implemented in HTMX migration
- **2026-04-19**: Replaced WebSocket with HTTP polling

---

## Historical Note

This decision was made after experiencing WebSocket reliability issues with HTMX in production. HTTP polling provided a simpler, more stable solution that requires no persistent connections or broadcast infrastructure.
