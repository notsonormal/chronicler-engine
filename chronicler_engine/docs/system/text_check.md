# Specification: Text Check System

## Goals

1. **Automatic pre-flight check**: Before player input reaches the LLM, the engine checks it and — if issues are found — shows a preview UI where the player can choose the corrected or original text.
2. **Manual "Check Text" button**: A reusable UI button that can run the checker on-demand against any text.

## Dictionary Strategy

User-provided ignored words are merged with the checker's built-in dictionary at service construction; the snapshot does not refresh per-check.

## Settings

Settings are stored as a row in the SQLite `settings` table. Mode defaults to `Disabled`; ignored words default to empty. See `src/domain/model/settings.rs` for variant definitions of `TextCheckMode`.

## Issue Classification

`IssueKind` classifies a check issue (`Spelling`, `Grammar`, `Capitalization`, `Formatting`, `Style`, `Other`). See `src/application/ports/text_checker.rs` for variant definitions.

## Error Handling

- If the checker fails to lint, the error is logged and the original text is forwarded (fail-open).
- If settings cannot be loaded, defaults are used (`Disabled` mode).

## Boundaries

- **Always**: Preserve prompt structure (XML tags, markdown). Never break `Action` parsing.
- **Never**: Auto-correct player input silently. Show a preview where the user can choose corrected, original, or cancel.
- **Never**: Commit dictionary secrets.

## Document References

- [ADR-011: Text Check Integration](../adr/adr-011-text-check-integration.md) — harper-core choice, pre-flight + manual UI flow
- [Harper Core](https://github.com/Automattic/harper) (v0.25.0)