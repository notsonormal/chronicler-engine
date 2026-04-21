# chronicler_engine/src/server/

## Responsibility

HTTP server for the Chronicler Engine. Serves the game interface via HTTP/WebSocket with HTMX-based dashboard, handles player input, streams narrative responses, and manages game state.

## Design

**Key modules:**
- `mod.rs` — Axum router setup, route definitions, WebSocket handler, state management
- `templates.rs` — HTMX template rendering for the game dashboard
- `fragments.rs` — HTMX partial response fragments for incremental UI updates

**Patterns:**
- Axum web framework for routing and request handling
- HTMX for client-side interactivity without heavy JavaScript
- WebSocket for real-time narrative streaming
- Template-based rendering for HTML responses

## Flow

1. Server starts → Axum router configured with routes
2. Client connects → serves initial dashboard HTML
3. Player submits input → server parses → dispatches to engine logic
4. Engine processes → narrative generated via LLM → streamed back via WebSocket
5. HTMX fragments update UI incrementally without full page reload

## Integration

- **Consumes**: `engine/` (action parsing, logic), `model/` (game state), `narrative/` (LLM generation)
- **Produces**: HTML responses, WebSocket streams, HTMX fragments
- **Consumed by**: Web browser client (HTMX dashboard)
