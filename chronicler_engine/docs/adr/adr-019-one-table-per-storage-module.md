# ADR-019: One Table Per Storage Module

## Status

**Superseded** by unified `Storage` enum (booster-gold-damage-domino).

## Context

Chronicler Engine's storage layer originally accumulated multi-table repositories that bundled logically-related but physically-separate SQLite tables into single Rust modules. Two cases were active:

1. **`message_storage.rs`** managed both `messages` and `message_swipes`.
2. **`snapshot_storage.rs`** managed both `games` and `game_state_snapshots`.

This coupling encouraged shortcuts: when two tables lived in the same repository, developers naturally treated them as a single unit. The result was domain logic that recreated aggregates instead of updating them, and storage methods that grew into cross-table transactions.

## Decision (Historical)

**Every physical SQLite table gets its own `xxx_storage.rs` module containing exactly one trait and one SQLite repository.** No storage module may touch more than one table.

A guardrail (`guardrails_one_table_per_storage` in `tests/guardrails.rs`) enforced this by failing the build if any `src/storage/*_storage.rs` file referenced more than one table.

### Specific Splits

| Table | Module | Trait |
|-------|--------|-------|
| `messages` | `message_storage.rs` | `MessageStorage` |
| `message_swipes` | `message_swipe_storage.rs` | `MessageSwipeStorage` |
| `games` | `game_storage.rs` | `GameStorage` |
| `game_state_snapshots` | `snapshot_storage.rs` | `SnapshotStorage` |
| `llm_messages` | `llm_message_storage.rs` | `LlmMessageStorage` |
| `prompt_presets` | `prompt_preset_storage.rs` | `PromptPresetStorage` |

## Supersession

The six individual traits and their implementations were consolidated into a single `Storage` struct with a `Backend` enum (`Sqlite`, `InMemory`, `Test`). The guardrail was removed because there are no longer any `*_storage.rs` files to enforce it on.

The **underlying principle** — that each storage operation should be table-scoped, with cross-table coordination living in `GameServiceContext` helpers — remains valid. The `Storage` enum preserves this: every method on `Storage` touches exactly one table, and callers compose operations explicitly via `GameServiceContext::save_message_and_snapshot`, `load_messages_with_swipes`, etc.

## Consequences

### Positive

- **Prevents bundling shortcuts.** Separating concerns at the method level makes it impossible to "accidentally" create a new message just to get a new swipe.
- **Testability.** The unified `Storage` struct supports dynamic failure injection via `TestOverride`, eliminating the need for custom mock structs per trait.
- **Simpler dependency graph.** Callers pass a single `Arc<Storage>` instead of five separate `Arc<dyn Trait>` fields.

### Negative (Historical)

- **More modules.** Six tables → six storage modules (mitigated by unification into one struct).
- **Non-atomic cross-storage operations.** A crash between `insert_message` and `insert_swipe` could leave a message with no swipes. The window is tiny (two sequential SQLite statements), but it exists.

## References

- `src/storage/storage.rs` — Unified `Storage` enum
- `src/application/context.rs` — `GameServiceContext` and cross-storage helpers
- ADR-008, ADR-017 — Previous storage architecture decisions
