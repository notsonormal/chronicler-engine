---
trigger: when_working_in_chronicler_engine
---

# Chronicler Engine Developer Rules

When you (the AI) are tasked with building, debugging, or extending the `chronicler_engine`, you **MUST** follow the workflow defined in `docs/README.md`.

## Key Principles

1. **Architecture is single source of truth** - Update `docs/architecture/system.md` BEFORE implementing
2. **Plans update architecture first** - Before any code, update the architecture document
3. **Plan in `docs/plans/`** - Create implementation plans there
4. **Validate** - Run tests, format, clippy after implementation

## Workflow (from docs/README.md)

1. Create a **plan** in `docs/plans/` (or update existing)
2. Update **architecture** - Modify `architecture/system.md` to reflect changes
3. **Implement** - Write the code
4. **Validate** - Run tests, format, clippy
5. **Archive** - Move completed plans to `plans/archived/`

## Rust Idioms and Best Practices
- Ensure `cargo fmt`, `cargo clippy`, and `cargo test` pass successfully.
- Prefer explicit error handling logic. Use `Result` heavily for parsing strings/data.
