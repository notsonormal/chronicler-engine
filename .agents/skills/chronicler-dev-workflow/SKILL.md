---
name: chronicler-dev-workflow
description: Chronicler Engine Rust developer - follows workflow from chronicler_engine/docs/README.md (Create Plan -> Update Architecture -> Implement -> Validate), uses Result over panic, applies std->external->local import order
compatibility: opencode
metadata:
  language: rust
  workspace: chronicler_engine
---

## What I do

I follow the workflow defined in `chronicler_engine/docs/README.md`:

1. **Create Plan**: Define requirements in `chronicler_engine/docs/plans/<name>.md`. Document the problem, solution, and files to change.
2. **Update Architecture**: Modify `chronicler_engine/docs/architecture/system.md` (and other relevant docs) to reflect the planned changes BEFORE writing code.
3. **Implement**: Write the code (write failing test first, then implement).
4. **Validate**: Run `cargo fmt`, `cargo clippy`, `cargo nextest run`.
5. **Archive**: Move completed plan to `chronicler_engine/old-docs/archived-plans`.

_See `.agents/skills/_shared/chronicler-shared.md` for documentation sync and visual verification steps._

## Reference

See `chronicler_engine/docs/README.md` for the complete workflow documentation.

## Technical Directives

- **Idiomatic Rust**: Always use `Result<T, EngineError>` for error handling. Never use `.unwrap()` or `.expect()` in production code.
- **Import Order**: std -> external crates -> local modules
- **Tests First**: Write a failing test before implementing the fix.
- **Decoupling**: Keep plans and implementations atomic.
- **Architecture First**: Always update `chronicler_engine/docs/architecture/system.md` BEFORE writing code.

## Validation Commands

Run the build script which validates format, clippy, and tests:

```bash
python chronicler_engine/scripts/build.py
```

Or run individually for faster feedback during development:
```bash
cargo fmt
cargo clippy -- -D warnings
cargo nextest run --lib
```

**All Tests Must Pass**: The build must complete, and every single test must pass for a feature to be considered finished. This is a solo developer project, failing tests and bugs must be fixed before we can move onto the next feature. Flaky tests are never acceptable. Failing tests must be fixed, regardless of how unrelated they might seem to the feature in development.

_See `.agents/skills/_shared/chronicler-shared.md` for documentation sync and visual verification steps._

## When to use me

Use for implementing features, bug fixes, or refactoring in the Chronicler Engine.
Don't use for general queries or tasks outside chronicler_engine.