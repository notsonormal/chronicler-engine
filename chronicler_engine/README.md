# 📖 Chronicler Engine

This is the core workspace for the Chronicler Engine, an interactive fiction and text adventure framework written in Rust.

## Architecture

The engine uses a data-driven model inspired by NetAF, ADRIFT, and SillyTavern character cards. The state is driven entirely by loading external JSON configurations into internal data structures.

- **World Card**: High-level rules and universe facts.
- **Map Definition**: `Overworld -> Region -> Room` locations and navigation.
- **Character Cards**: AI-ready NPC properties and player state.

## AI-First Development

This project leverages strict Spec-Driven and Test-Driven Development (TDD).
If you are an AI working on this repository, you **must** adhere to the rules defined in `../.agents/rules/chronicler_engine.md`.

Specifications for new features are stored in `docs/specs/`.
