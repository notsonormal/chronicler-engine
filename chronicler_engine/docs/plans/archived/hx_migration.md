# Specification: Chronicler Engine System Definition

## Overview
The Chronicler Engine is an interactive narrative game engine that generates dynamic storylines using LLMs. Players interact through a web-based HTMX dashboard with real-time WebSocket updates.

## Architecture

### Components

1. **HTTP Server (axum)**
   - Port: 3000
   - Serves HTML frontend and WebSocket connections
   - Provides REST endpoints for HTML fragment updates

2. **WebSocket Hub**
   - Real-time broadcast to connected clients
   - Uses tokio broadcast channel
   - Protocol: JSON messages for fragment updates

3. **HTMX Frontend**
   - Single-page application via HTML fragment swaps
   - WebSocket connection for live updates (`ws-connect`)
   - CSS styling matching terminal aesthetic

4. **Game State**
   - Managed via `Arc<std::sync::Mutex<GameState>>`
   - Narration history with color-coded log types
   - TUI state tracking (input, generation status)

### Module Structure

```
crate::model::        # Data models (world, map, character, state)
crate::engine::       # Game logic (parser, action, logic)
crate::narrative::    # LLM integration (llm backend)
crate::server::       # HTTP + WebSocket (mod, hub, fragments)
assets/               # Static HTML (index.html)
```

## UI Layout

### 1. Header
- Fixed height: 48px
- Contains: Game title + current location name
- Style: Location in green bold (#00ff00)

### 2. Main Body
- Flex layout: 70% story log / 30% visual sidebar
- **Story Log**: Scrollable, displays narration history
  - Narration: Cyan (#00ffff)
  - Dialogue: White (#ffffff)  
  - System: Yellow (#ffff00)
  - Input: Gray (#888888)
- **Visual Sidebar**:
  - Location image: 40% height
  - NPC portraits: Remaining height

### 3. Action Area
- Fixed height: 60px
- Contains: Command input + status indicator
- Status states: "Ready" (green) / "Thinking..." (yellow pulse)

## HTTP Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/` | Main HTML page |
| GET | `/ws` | WebSocket upgrade |
| GET | `/fragment/header` | Header fragment |
| GET | `/fragment/story-log` | Story log fragment |
| GET | `/fragment/visual-sidebar` | Visual sidebar fragment |
| GET | `/fragment/action-area` | Action area fragment |
| POST | `/action` | Submit player command |

## WebSocket Protocol

### Client → Server
```json
{ "type": "subscribe" }
```

### Server → Client
```json
{ "type": "update", "fragment": "story-log", "html": "..." }
```

## Dependencies

```toml
axum = { version = "0.7", features = ["ws"] }
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = "0.21"
futures-util = "0.3"
tungstenite = "0.21"
```

## Supported Actions

| Action | Description |
|--------|-------------|
| `Look` | Display room description |
| `WalkTo <dir>` | Move in cardinal direction |
| `Talk <npc> <msg>` | Interact with NPC |
| `Inventory` | Show player inventory |
| `<free text>` | Free action triggers LLM narration |

## Security

- All user-generated content must be HTML-escaped before rendering
- No eval or dynamic script injection
- CSRF protection TODO for production

## Change Log

- **2025-04-12**: Migrated from Ratatui TUI to HTMX web dashboard
  - Added HTTP server + WebSocket for real-time updates
  - Created fragment-based UI with live refresh
  - Fixed XSS vulnerability in fragment rendering (html_escape applied)