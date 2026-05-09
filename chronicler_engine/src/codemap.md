# chronicler_engine/src/

## Responsibility
Rust game engine source code. A text adventure/interactive fiction engine with LLM-powered narrative generation, HTMX dashboard, and data-driven game state from JSON configs.

## Architecture (3 Tiers)
1. **Engine Tier** (`engine/`) — Game logic, action parsing, movement, trigger evaluation
2. **Model Tier** (`model/`) — Pure data structures (serde-serializable)
3. **Narrative Tier** (`narrative/`) — LLM integration, prompt building, quantification
4. **Server Tier** (`server/`) — Axum HTTP server, HTMX fragments, WebSocket

## System Entry Points
- `main.rs` — Binary entry: `dotenv` → `init_logging` → `parse_args` → `run()`
- `lib.rs` — Library root: modules, guardrail lints, re-exports
- `bootstrap.rs` — World loading from JSON, data validation, server startup

## Cross-Cutting Concerns
- `error.rs` — `EngineError` enum with `thiserror`, `Result<T>` alias
- `settings.rs` — Runtime settings loading (`settings.json`)
- `cli.rs` — CLI argument parsing with `clap`
- `test_support/` — Test fixtures and builders

## Integration Flow
```
Browser → server/ (Axum)
  → engine/ (GameService)
    → narrative/ (LLM + Quantifier)
      → model/ (GameState mutations)
        → server/ (HTML fragments via Askama)
```

## Subdirectory Maps
| Directory | Responsibility | Map |
|-----------|----------------|-----|
| `engine/` | Game logic, action processing, trigger evaluation | [View Map](engine/codemap.md) |
| `model/` | Domain data structures, state, settings | [View Map](model/codemap.md) |
| `narrative/` | LLM backends, prompt building, quantification | [View Map](narrative/codemap.md) |
| `server/` | HTTP server, HTMX fragments, templates | [View Map](server/codemap.md) |
| `test_support/` | Test fixtures and builders | [View Map](test_support/codemap.md) |

## Key Files
| File | Purpose |
|------|---------|
| `lib.rs` | Library root, guardrail lint attributes, module declarations |
| `main.rs` | Binary entry point |
| `bootstrap.rs` | World JSON loading, validation, server initialization |
| `error.rs` | Central error type (`EngineError`) |
| `settings.rs` | Runtime configuration loading |
| `cli.rs` | Command-line argument parsing |
