# Specification: OpenRouter LLM Dialogue Integration

**Status:** Completed

## Objective
Enhance the Chronicler Engine by plugging the text adventure NPC interactions into a live LLM (OpenRouter), shifting away from static JSON example dialogues.

## Architecture Decisions

1. **HTTP Client**: Use `reqwest::blocking` instead of async `tokio`. Since the engine is a local REPL that halts standard input while waiting for NPC generation, forcing the main thread to block on the HTTP call makes intuitive sense and isolates runtime complexity.
2. **Model Selection**: Defaults to `sao10k/l3.3-euryale-70b` 
3. **Environment**: Authentication relies on an `OPENROUTER_API_KEY` environmental variable, managed by the `dotenv` crate pulling from standard `mrn-general/.env`.

## Execution Mechanics
When the user executes `talk <npc> "<message>"`, the engine will construct a contextual prompt combining:
- The Global Rules from the WorldCard.
- The Room Description.
- The NPC personality and scenario.
- The player's explicit message to the NPC (or a generic 'The player approaches you.' if no message was provided).

This ensures the LLM's roleplay output is strictly anchored to the loaded constraints of the current game state.

## Testing Criteria
TDD dictates that:
1. `parser.rs` must correctly separate the target NPC from the quoted message, validated by unit tests.
2. The core library (`lib.rs`) handles the prompt generation cleanly.
