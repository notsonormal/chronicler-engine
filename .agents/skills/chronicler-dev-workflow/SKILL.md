---
name: chronicler-dev-workflow
description: Chronicler Engine Rust developer - follows workflow from docs/README.md (Create Plan -> Update Architecture -> Implement -> Validate), uses Result over panic, applies std->external->local import order
compatibility: opencode
metadata:
  language: rust
  workspace: chronicler_engine
---

## What I do

I follow the workflow defined in `chronicler_engine/docs/README.md`:

1. **Create Plan**: Define requirements in `docs/plans/<name>.md`. Document the problem, solution, and files to change.
2. **Update Architecture**: Modify `docs/architecture/system.md` (and other relevant docs) to reflect the planned changes BEFORE writing code.
3. **Implement**: Write the code (write failing test first, then implement).
4. **Validate**: Run `cargo fmt`, `cargo clippy`, `cargo nextest run`.
5. **Archive**: Move completed plan to `docs/plans/archived/`.

## Documentation Sync

When implementing changes, ALWAYS update these documents BEFORE writing code:

1. **Plan** (`docs/plans/<name>.md`) - Document problem, solution, files to change
2. **Architecture** (`docs/architecture/system.md`) - Core module structure changes
3. **System docs** (`docs/system/*.md`) - Domain-specific docs (dashboard.md, game_flow.md, etc.)
4. **Reference docs** (`docs/reference/*.md`) - Data schemas, API specs
5. **ADRs** (`docs/adr/*.md`) - If making architectural decisions
6. **CHANGELOG.md** - Record the change

Example: If adding a new HTMX polling endpoint, update:
- Plan document
- architecture/system.md (Server tier)
- system/dashboard.md (frontend section)
- reference/testing.md (add test)
- CHANGELOG.md

## Reference

See `chronicler_engine/docs/README.md` for the complete workflow documentation.

## Technical Directives

- **Idiomatic Rust**: Always use `Result<T, EngineError>` for error handling. Never use `.unwrap()` or `.expect()` in production code.
- **Import Order**: std -> external crates -> local modules
- **Tests First**: Write a failing test before implementing the fix.
- **Decoupling**: Keep plans and implementations atomic.
- **Architecture First**: Always update `docs/architecture/system.md` BEFORE writing code.

## Validation Commands

Run the build script which validates format, clippy, and tests:

```bash
python scripts/build.py
```

Or run individually for faster feedback during development:
```bash
cargo fmt
cargo clippy -- -D warnings
cargo nextest run --lib
```

**All Tests Must Pass**: The build must complete, and every single test must pass for a feature to be considered finished. This is a solo developer project, failing tests and bugs must be fixed before we can move onto the next feature. Flaky tests are never acceptable. Failing tests must be fixed, regardless of how unrelated they might seem to the feature in development.

## Visual/UI Verification (MANDATORY)

For any CSS, layout, or visual changes:

1. **Rebuild** the project after CSS changes
2. **Restart** the server with the new build
3. **Navigate** to the affected page in the browser
4. **Take a screenshot** and **actually look at it** -- do not skip this step
5. **Confirm visually** that the change renders correctly before claiming done

**NEVER claim "verified" or "browser verification complete" without a screenshot that you have personally reviewed.** Subagent verification does not count — you must see the rendered result yourself.

Common CSS pitfalls that pass build but break visually:
- `flex: 1` on containers causing unwanted expansion
- `align-items: center` centering content in unexpectedly large spaces
- Missing `overflow` properties causing content to spill or disappear
- Z-index conflicts hiding elements behind others 

## When to use me

Use for implementing features, bug fixes, or refactoring in the Chronicler Engine.
Don't use for general queries or tasks outside chronicler_engine.