# 📖 Chronicler Engine

This is the core workspace for the Chronicler Engine, an interactive fiction and text adventure framework written in Rust.

## Architecture

The engine uses a data-driven model inspired by NetAF, ADRIFT, and SillyTavern character cards. The state is driven entirely by loading external JSON configurations into internal data structures.

- **World Card**: High-level rules and universe facts.
- **Map Definition**: `Overworld -> Region -> Room` locations and navigation.
- **Character Cards**: AI-ready NPC properties, images, and player state.

The engine runs as an HTTP/WebSocket server with an HTMX-based dashboard for the web UI.

## Data Structure

```
data/
├── worlds/              # Game data (per world)
│   ├── redmist_estate/  # Default world
│   │   ├── world.json
│   │   └── map.json
│   └── test/            # Test world
├── characters/          # NPC definitions (shared across worlds)
│   ├── redmist_estate/
│   └── test/
├── personas/            # Player definitions (shared across worlds)
│   ├── julian.json
│   └── test_player.json
└── images/              # Character sprites and assets
```

## LLM Integration

The engine uses a trait-based `LlmBackend` design for flexible LLM integration:

- **Trait**: `LlmBackend` in `src/narrative/llm.rs`
- **Implementations**: 
  - `OpenRouterClient` - Real API calls to OpenRouter
  - `MockLlmBackend` - For testing without API calls
- **Configuration**: Configure connections in `data/settings.json` — see `docs/adr/adr-007-settings-system.md`

## Environment Variables

The engine requires a `.env` file or environment variables to be set for AI functionality:

- `OPENROUTER_API_KEY`: **(Required for real LLM)** Your API key from OpenRouter.
- Configure models and backends in `data/settings.json` — see `docs/adr/adr-007-settings-system.md`.
- **Note**: Settings are loaded from SQLite DB at runtime. For tests, use `--settings-path <file>` CLI flag to import mock settings.

## Quick Start

```bash
# Full build and test (recommended) — fast suite, LLM tests excluded
python build.py

# Or manual commands
cargo build
cargo run -- --world redmist_estate --port 3000

# Run tests
cargo nextest run --test flow_mock_tests    # Fast - no API key needed
python build.py --llm-only           # LLM integration tests only
python build.py --include-llm        # Full suite including LLM tests
```