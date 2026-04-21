# chronicler_engine/

## Responsibility

Rust game engine for interactive fiction/text adventures. Provides HTTP/WebSocket server with HTMX dashboard, LLM-powered narrative generation, and data-driven game state from JSON configs.

## Design

- **Edition**: Rust 2024 (requires 1.85+)
- **Structure**: Single crate (binary + library)
- **Key modules**: engine/, model/, narrative/, server/, ui/
- **Data-driven**: World, map, character, triggers defined in JSON

## Flow

1. Load JSON world data → `model/` types
2. Parse player actions → `engine/` actions
3. Evaluate triggers → execute logic
4. Generate narrative via `narrative/` (LLM)
5. Serve via `server/` (HTTP/WebSocket)

## Integration

- **Parent**: mrn-general workspace
- **Downstream**: Docker service (`docker-compose.yml`)
