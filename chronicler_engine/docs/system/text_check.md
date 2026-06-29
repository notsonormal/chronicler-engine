# Specification: Text Check System

> **Related Decisions**: [ADR-011](../adr/adr-011-text-check-integration.md)

## Overview

The Text Check system provides spell-checking and grammar-checking for player input before it reaches the LLM. It uses [harper-core](https://github.com/Automattic/harper) (v0.25.0) — a pure-Rust linter from Automattic — with a built-in FST dictionary (~8MB stripped) and configurable lint rules.

## Goals

1. **Automatic pre-flight check**: Before player input reaches the LLM, the engine checks it and — if issues are found — shows a preview UI where the player can choose the corrected or original text.
2. **Manual "Check Text" button**: A reusable UI button that can run the checker on-demand against any text.

## Architecture

```mermaid
flowchart TD
    A[Player Input] --> B[/action/check]
    B -- Issues found --> C[Preview Fragment]
    B -- No issues --> D[/action]
    C -- Send Corrected --> E[/action/confirm]
    C -- Send Original --> E
    C -- Cancel --> F[Restore Action Area]
```

## Module Structure

| File | Purpose |
|------|---------|
| `src/adapters/driven/text_check/mod.rs` | Module root — re-exports public API |
| `src/adapters/driven/text_check/check.rs` | Facade: `check_player_input()` entry point |
| `src/adapters/driven/text_check/harper_backend.rs` | `HarperBackend` — wraps harper-core linting |
| `src/adapters/driven/text_check/types.rs` | `CheckResult`, `CheckIssue`, `IssueKind` |

## Types

### `CheckResult`

```rust
pub struct CheckResult {
    pub original: String,
    pub corrected: String,
    pub issues: Vec<CheckIssue>,
}
```

### `CheckIssue`

```rust
pub struct CheckIssue {
    pub span: Range<usize>,      // Byte span in original text
    pub message: String,         // Human-readable description
    pub suggestion: Option<String>, // Replacement text, if any
    pub kind: IssueKind,         // Classification
}
```

### `IssueKind`

```rust
pub enum IssueKind {
    Spelling,
    Grammar,
    Capitalization,
    Formatting,
    Style,
    Other,
}
```

## Check Modes

`TextCheckMode` controls which lint rules are active:

| Mode | SpellCheck | Grammar Rules |
|------|-----------|---------------|
| `Disabled` | Off | Off |
| `Spell` | On | Off |
| `Grammar` | Off | On |
| `SpellGrammar` | On | On |

## Dictionary Strategy

The `HarperBackend` merges two dictionaries:

1. **`FstDictionary::curated()`** — harper-core's built-in English dictionary (~130K words)
2. **`MutableDictionary`** — user-provided ignored words from `TextCheckSettings.ignored_words`

This allows fantasy names, place names, and game-specific terms to be added to a personal dictionary.

## Server Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/action/check` | Pre-flight check. Returns preview if issues found, otherwise forwards to `/action`. |
| `POST` | `/action/confirm` | Accepts corrected or original text from the preview fragment and processes the action. |
| `POST` | `/check-text` | Manual check. Returns preview fragment for any text. |
| `POST` | `/settings/text-check` | Saves text check mode and auto-check preference. |

## Settings

`TextCheckSettings` is stored in `settings.json` alongside existing `AppSettings`:

```rust
pub struct TextCheckSettings {
    pub mode: TextCheckMode,
    pub enable_auto_check: bool,
    pub ignored_words: Vec<String>,
}
```

| Field | Default | Description |
|-------|---------|-------------|
| `mode` | `Disabled` | Which lint rules are active |
| `enable_auto_check` | `true` | Whether `/action/check` runs automatically before sending |
| `ignored_words` | `[]` | Words to treat as valid (fantasy names, etc.) |

## UI Integration

### Preview Fragment

When issues are found, the action area is replaced with a preview showing:
- Original text (muted)
- Corrected text (green accent)
- Issue tags (spell = orange, grammar = pink)
- **Send** — submits the corrected text to `/action/confirm`
- **Send Original** — submits the original text to `/action/confirm`
- **Cancel** — restores the normal action area

After submission, the action area restores to its normal form (`#command-form`) via `hx-on::after-request`.

### Settings Card

A "Text Check" card appears in the Settings tab below Connections:
- Mode dropdown (Disabled / Spell / Grammar / Spell + Grammar)
- "Check before sending to LLM" checkbox

## Error Handling

- If harper-core fails to lint, the error is logged and the original text is forwarded to the action handler (fail-open).
- If the preview template fails to render, a 500 error fragment is returned.
- If settings cannot be loaded, defaults are used (`Disabled` mode).

## Performance

- Linting is synchronous and in-memory (no I/O).
- `HarperBackend` is instantiated per-check (dictionary merge is cheap for small ignore lists).
- Typical check latency: <10ms for a single sentence.

## Testing

 **Integration tests**: `tests/http/endpoints/text_check_tests.rs` — misspelling detection, clean text, disabled mode, ignored words
- **Integration tests**: Preview endpoint returns fragment when issues exist; forwards when disabled

## Boundaries

- **Always**: Preserve prompt structure (XML tags, markdown). Never break `Action` parsing.
- **Never**: Auto-correct player input silently. Show a preview where the user can choose corrected, original, or cancel.
- **Never**: Commit dictionary secrets.
