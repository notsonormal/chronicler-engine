# chronicler_engine/src/narrative/text_check/

## Responsibility
Pre-flight spell and grammar checking for player input via harper-core. Integrates with the action pipeline to optionally warn players about typos before submitting commands.

## Design Patterns
- **Adapter Pattern**: `HarperBackend` wraps `harper-core` library in the engine's `CheckResult`/`CheckIssue` types.
- **Strategy Pattern**: `TextCheckMode` enum (`Disabled`, `Spell`, `SpellGrammar`) controls checking behavior.

## Data & Control Flow
```
Player input → check_player_input(text, mode, ignored_words)
  → HarperBackend::new(ignored_words)
    → backend.check(text, mode)
      → harper-core analysis → Vec<CheckIssue>
        → CheckResult { original, corrected, issues }
```

## Integration Points
- **Consumed by**: `server/fragments.rs` (`action_check_handler`, `check_text_handler`)
- **Depends on**: `model/settings.rs` (`TextCheckMode`)

## Files
| File | Purpose |
|------|---------|
| `types.rs` | `CheckResult`, `CheckIssue`, `IssueKind` — result types |
| `check.rs` | `check_player_input()` — entry point with mode dispatch |
| `harper_backend.rs` | `HarperBackend` — harper-core adapter |
| `mod.rs` | Module exports |
