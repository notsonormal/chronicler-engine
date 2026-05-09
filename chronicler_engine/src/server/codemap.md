# chronicler_engine/src/server/

## Responsibility
The HTTP/WebSocket server tier. Axum-based web server serving an HTMX dashboard. Handles all HTTP routes, template rendering, fragment responses for HTMX partial updates, and debug endpoints.

## Design Patterns
- **Axum Router Pattern**: Central `build_router()` defines all routes with typed handlers.
- **Template Engine**: Askama templates (`templates.rs`) for HTML generation.
- **Fragment Architecture**: HTMX swaps partial HTML fragments (`fragments.rs`) instead of full page reloads.
- **State Extraction**: `AppState` wraps `Arc<Mutex<GameState>>` shared across handlers.

## Data & Control Flow
```
HTTP Request → Axum Router → Handler
  → lock_state() → read GameState
    → render template → Html<String> response
      → HTMX frontend swaps fragment

Action POST → action_handler()
  → parse_command() → GameService.execute_action()
    → spawn async generation
      → Client polls /fragment/story-log every 5s
```

## Integration Points
- **Consumes**: `engine/` (GameService, parser, logic), `model/` (state, settings)
- **Consumed by**: External browser/HTTP clients

## Files
| File | Purpose |
|------|---------|
| `mod.rs` | `build_router()`, `AppState`, `ServerConfig`, route definitions |
| `fragments.rs` | HTMX fragment handlers — story log, header, sidebar, action area, character headshots |
| `templates.rs` | Askama template structs for all HTML fragments |
| `settings_fragment.rs` | Settings panel UI — connections, configuration |
| `debug.rs` | Debug endpoints for inspecting game state |
