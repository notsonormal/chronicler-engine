# 📖 Chronicler Engine

This is the core workspace for the Chronicler Engine, an interactive fiction and text adventure framework written in Rust.

## Architecture

The engine uses a data-driven model inspired by NetAF, ADRIFT, and SillyTavern character cards. The state is driven entirely by loading external JSON configurations into internal data structures.

- **World Card**: High-level rules and universe facts.
- **Map Definition**: `Overworld -> Region -> Room` locations and navigation.
- **Character Cards**: AI-ready NPC properties and player state.

## Documentation & Memory
The Chronicler Engine uses a tiered **Spec-Driven Development (SDD 2.0)** approach. Documentation is organized to provide the best possible context for both humans and AI agents:

- **[Architecture](docs/architecture/)**: System definition - single source of truth.
- **[System](docs/system/)**: Domain documentation - explains subsystems.
- **[Plans](docs/plans/)**: Implementation blueprints (active or archived).
- **[ADR](docs/adr/)**: Architecture Decision Records with context and rationale.
- **[Reference](docs/reference/)**: Data schemas and API specs.
- **[Learnings](../.ag-memory/CHRONICLER_LEARNINGS.md)**: Persistent memory of breakthroughs.

## Environment Variables

The engine requires a `.env` file or environment variables to be set for AI functionality:

- `OPENROUTER_API_KEY`: **(Required)** Your API key from OpenRouter.
- `LLM_MODEL`: The OpenRouter model ID to use. Defaults to `z-ai/glm-4.5-air:free` if unset.
