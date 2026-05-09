# chronicler_engine/

## Responsibility
Rust text adventure / interactive fiction engine. HTTP/WebSocket server with HTMX dashboard, LLM-powered narrative generation, and data-driven game state from JSON configs.

## Architecture
Three-tier architecture with a server layer:
1. **Engine Tier** (`src/engine/`) — Game logic, parsing, movement, triggers
2. **Model Tier** (`src/model/`) — Pure data structures (serde-serializable)
3. **Narrative Tier** (`src/narrative/`) — LLM integration, prompt building, quantification
4. **Server Tier** (`src/server/`) — Axum HTTP server, HTMX fragments

## Key Directories
| Directory | Responsibility | Map |
|-----------|----------------|-----|
| `src/` | Engine source code (lib + binary) | [View Map](src/codemap.md) |
| `tests/` | Integration tests (7 files) | — |
| `docs/` | Extensive documentation (52+ .md files, auto-indexed) | — |
| `data/` | Game data — worlds, characters, personas, schemas | [View Map](data/codemap.md) |
| `scripts/` | Python helpers for validation, docs, coverage | [View Map](scripts/codemap.md) |

## Entry Points
- `cargo run -- --world redmist_estate --port 3000`
- `python build.py` — validation, tests, coverage

## Key Config Files
| File | Purpose |
|------|---------|
| `Cargo.toml` | Rust crate config — dependencies, features, dev-dependencies |
| `data/settings.json` | Runtime LLM connections, narration settings, text check config |
| `arch-lint.toml` | Architecture guardrail rules (layer enforcement) |
