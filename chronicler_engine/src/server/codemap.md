# chronicler_engine/src/server/

## Responsibility

HTTP server for the Chronicler Engine. Serves the game interface via HTTP/WebSocket with HTMX-based dashboard, handles player input, streams narrative responses, and manages game state. Provides character portrait display, story log with edit/retry capabilities, and visual sidebar for location/NPC images.

## Design

**Key modules:**
- `mod.rs` — Axum router setup, route definitions, AppState, ServerConfig, port binding with retry
- `templates.rs` — HTMX Askama templates with SafeHtml wrapper, markdown-to-HTML conversion, LogEntryView
- `fragments.rs` — HTMX fragment handlers, action processing, history edit/retry handlers

**Patterns:**
- Axum web framework for routing and request handling
- HTMX for client-side interactivity without heavy JavaScript
- Askama templates with compile-time validation
- Template-based rendering with SafeHtml wrapper for unescaped content
- Mutex-wrapped GameState for thread-safe access
- Sync vs async action handling (Look/Inventory/Quit sync, others async via thread spawn)

## Key Types

**mod.rs:**
- `ServerConfig` — port configuration (default: 3000)
- `AppState` — holds Arc<Mutex<GameState>> and Arc<dyn GameService>
- `run_server()` / `run_server_with_config()` — server startup with port binding retry

**templates.rs:**
- `SafeHtml` — wrapper for unescaped HTML in templates
- `LogEntryView` — LogEntry transformed for template rendering
- `HeaderTemplate` — room header with connection status
- `StoryLogTemplate` — story log with edit/retry buttons
- `VisualSidebarTemplate` — location image + NPC portraits
- `CharacterHeadshotsTemplate` — clickable NPC headshots
- `ActionAreaTemplate` — command input with hints/status
- `markdown_to_html()` — converts markdown with smart quotes

**fragments.rs:**
- `render_header()` / `render_story_log()` / `render_visual_sidebar()` / `render_action_area()` / `render_character_headshots()` — fragment rendering
- Fragment handlers — header_fragment, story_log_fragment, visual_sidebar_fragment, action_area_fragment, character_headshots_fragment, hints_handler, status handlers
- `action_handler()` — main POST handler for player commands (sync/async split)
- `edit_history_handler()` — PATCH /history/:id for log entry editing
- `retry_handler()` — POST /retry for regenerating last AI response
- `process_sync_action()` — handles Look/Inventory/Quit immediately

## Flow

1. Server starts → Axum router configured with routes + state
2. Client connects → serves initial dashboard HTML from `/assets/index.html`
3. Player submits command → `action_handler()` processes
   - Sync actions (Look/Inventory/Quit) → processed immediately, triggers HTMX refresh
   - Async actions → spawned to thread, returns "Thinking..."
4. Engine processes → narrative generated via LLM
5. HTMX fragments update UI: header, story-log, visual-sidebar, action-area

## Routes

| Route | Method | Handler |
|-------|--------|---------|
| `/` | GET | index_handler (static HTML) |
| `/fragment/header` | GET | header_fragment |
| `/fragment/story-log` | GET | story_log_fragment |
| `/fragment/visual-sidebar` | GET | visual_sidebar_fragment |
| `/fragment/action-area` | GET | action_area_fragment |
| `/fragment/character-headshots` | GET | character_headshots_fragment |
| `/action` | POST | action_handler |
| `/hints` | GET | hints_handler |
| `/status/ready` | GET | status_ready_handler |
| `/status/generating` | GET | generating_status_handler |
| `/status/reset-generating` | POST | reset_generating_handler |
| `/history/:id` | POST | edit_history_handler |
| `/retry` | POST | retry_handler |
| `/assets/*` | GET | ServeDir (static assets) |
| `/data/*` | GET | ServeDir (game data) |

## Integration

- **Consumes**: `engine/` (action parsing, logic), `model/` (game state), `narrative/` (LLM generation)
- **Produces**: HTML responses, HTMX fragments
- **Consumed by**: Web browser client (HTMX dashboard)
