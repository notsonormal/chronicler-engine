---
trigger: when_working_in_chronicler_engine
---

# Chronicler Engine Developer Rules

When you (the AI) are tasked with building, debugging, or extending the `chronicler_engine`, you **MUST** follow the workflow defined in `docs/README.md`.

## Key Principles

1. **Architecture is single source of truth** - Update `chronicler_engine/docs/architecture/system.md` BEFORE implementing
2. **Plans update architecture first** - Before any code, update the architecture document
3. **Plan in `chronicler_engine/docs/plans/`** - Create implementation plans there
4. **Validate** - Run tests, format, clippy after implementation
5. **Test-First Context** - Use tests to understand existing behavior and triage bugs BEFORE deep-diving into code

## Workflow (from docs/README.md)

1. Create a **plan** in `chronicler_engine/docs/plans/` (or update existing)
2. Update **architecture** - Modify `chronicler_engine/architecture/system.md` to reflect changes
3. **Implement** - Write the code
4. **Validate** - Run tests, format, clippy
5. **Archive** - Move completed plans to `chronicler_engine/old-docs/archived-plans`

## Layer Boundary

The server tier (`src/server/`) must never touch `GameState` directly.

- **Reads:** Use `ApplicationService` narrow read methods (`get_story_log_entries`, `get_input_status`, `get_current_room_view`, `get_npc_headshots`, `get_debug_state_view`).
- **Writes:** Use `ApplicationService` command methods (`process_action`, `retry`, `reset`, etc.).
- **Tests:** Use `TestAppBuilder` in `test_support/` — never construct `GameState` in `tests/components/`.
- **Guardrails:** `tests/guardrails/layers.rs` enforces these boundaries at test time.

## Rust Idioms and Best Practices
- Ensure `cargo fmt`, `cargo clippy`, and `cargo nextest run` pass successfully.
- Prefer explicit error handling logic. Use `Result` heavily for parsing strings/data.