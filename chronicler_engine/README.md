# 📖 Chronicler Engine

This is the core workspace for the Chronicler Engine, an interactive fiction and text adventure framework written in Rust.

## Architecture

The engine uses a data-driven model inspired by NetAF, ADRIFT, and SillyTavern character cards. The state is driven entirely by loading external JSON configurations into internal data structures.

- **World Card**: High-level rules and universe facts.
- **Map Definition**: `Overworld -> Region -> Room` locations and navigation.
- **Character Cards**: AI-ready NPC properties and player state.

## Documentation & Memory
The Chronicler Engine uses a tiered **Spec-Driven Development (SDD 2.0)** approach. Documentation is organized to provide the best possible context for both humans and AI agents:

- **[Contracts](file:///workspaces/mrn-general/chronicler_engine/docs/specs/contract/)**: Data schemas, traits, and API boundaries.
- **[Logic](file:///workspaces/mrn-general/chronicler_engine/docs/specs/logic/)**: Behavioral rules, narration logic, and LLM processing.
- **[UI](file:///workspaces/mrn-general/chronicler_engine/docs/specs/ui/)**: Blueprints for the TUI dashboard and visual semantics.
- **[Blueprints](file:///workspaces/mrn-general/chronicler_engine/docs/specs/blueprints/)**: Archival records of completed major migrations.
- **[Learnings](file:///workspaces/mrn-general/.ag-memory/CHRONICLER_LEARNINGS.md)**: Persistent memory of breakthroughs and repeating mistakes.

## Environment Variables

The engine requires a `.env` file or environment variables to be set for AI functionality:

- `OPENROUTER_API_KEY`: **(Required)** Your API key from OpenRouter.
- `LLM_MODEL`: The OpenRouter model ID to use. Defaults to `z-ai/glm-4.5-air:free` if unset.
