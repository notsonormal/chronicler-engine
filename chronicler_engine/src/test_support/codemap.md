# chronicler_engine/src/test_support/

## Responsibility
Test fixture builders and helper functions for unit and integration tests. Provides convenient constructors for `GameState`, `NpcCard`, `PlayerCard`, and other domain objects.

## Design Patterns
- **Builder Pattern**: `TestGameState`, `TestNpc`, `TestPlayer` provide fluent APIs for test setup.
- **Test Data Factory**: Pre-configured NPCs with triggers, rooms with exits, etc.

## Integration Points
- **Consumed by**: All `*_tests.rs` modules in `src/` and integration tests in `tests/`

## Files
| File | Purpose |
|------|---------|
| `fixtures.rs` | `TestGameState`, `TestNpc`, `TestPlayer`, `TestWorld`, quantifier result builders |
| `mod.rs` | Re-exports all fixtures |
