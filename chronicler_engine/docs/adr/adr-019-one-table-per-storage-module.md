# ADR-019: One Table Per Storage Module

## Status

**Accepted**

## Context

Chronicler Engine's storage layer has accumulated multi-table repositories that bundle logically-related but physically-separate SQLite tables into single Rust modules. Two cases are active today:

1. **`message_storage.rs`** manages both `messages` and `message_swipes`. The `MessageStorage::insert_message` method accepts a `&Message` (which contains `Vec<Swipe>`) and inserts into both tables. This bundling made it "easiest" for domain logic to create an entirely new `Message` row whenever a new swipe was needed, then migrate old swipes into it, rather than simply adding a new swipe to the existing message.

2. **`snapshot_storage.rs`** manages both `games` and `game_state_snapshots`. It also performs cross-table cleanup in `delete_game()`, manually deleting from snapshots and messages before deleting the game row.

This coupling encourages shortcuts: when two tables live in the same repository, developers naturally treat them as a single unit. The result is domain logic that recreates aggregates instead of updating them, and storage methods that grow into cross-table transactions that are hard to test and reason about.

### Related Decisions

- **ADR-008**: Introduced SQLite snapshot persistence with bundled `SnapshotStorage` trait.
- **ADR-017**: Introduced `message_swipes` as a separate table, but kept it inside `MessageStorage` for convenience.

## Decision

**Every physical SQLite table gets its own `xxx_storage.rs` module containing exactly one trait and one SQLite repository.** No storage module may touch more than one table.

> **Enforcement**: Guardrail `guardrails_one_table_per_storage` in `tests/guardrails.rs` fails the build if any `src/storage/*_storage.rs` file references more than one table.

### Specific Splits

| Table | New / Updated Module | Trait |
|-------|----------------------|-------|
| `messages` | `message_storage.rs` (refactored) | `MessageStorage` |
| `message_swipes` | `message_swipe_storage.rs` (new) | `MessageSwipeStorage` |
| `games` | `game_storage.rs` (new) | `GameStorage` |
| `game_state_snapshots` | `snapshot_storage.rs` (refactored) | `SnapshotStorage` |
| `llm_messages` | `llm_message_storage.rs` (unchanged) | `LlmMessageStorage` |
| `prompt_presets` | `prompt_preset_storage.rs` (unchanged) | `PromptPresetStorage` |

### Trait API Changes

#### `MessageStorage`
- **`insert_message`** no longer accepts a `&Message` with swipes. It persists only message metadata (sender, log_type, timestamp, active_swipe_index, is_deleted). Callers must separately call `MessageSwipeStorage::insert_swipe` for every swipe, including the first.
- **`update_message` removed.** There is no `text` column on `messages`; updating message text means updating the active swipe. A `GameServiceContext::update_message_text()` helper coordinates the two storage calls.
- **`load_messages` removed.** Loading full `Message` objects requires joining swipes. A `GameServiceContext::load_messages()` helper loads from both storages and assembles.
- **`migrate_swipes` removed.** This cross-table transaction is temporarily provided as a `GameServiceContext` helper until the retry pipeline is refactored to use `insert_swipe` + `update_active_swipe` instead of message recreation.
- **`insert_swipe` and `shift_swipe_indices` moved** to `MessageSwipeStorage`.

#### `SnapshotStorage`
- **`list_games`, `create_game`, `delete_game`, `get_game` moved** to `GameStorage`.
- **`set_game_id` moved** to `GameStorage`. Updating `games.updated_at` when switching games is a game-metadata concern, not a snapshot concern.
- **`delete_game` simplified.** Schema migration adds `ON DELETE CASCADE` FKs from `game_state_snapshots` and `messages` to `games`, so deleting a game row automatically cleans up children.

### Cross-Table Coordination

Because `DbPool` is a single `Arc<Mutex<Connection>>`, callers cannot acquire a connection, start a `BEGIN`, and pass the connection into storage methods — Rust's `std::sync::Mutex` is not recursive and would deadlock. Therefore:

- **No caller-level SQLite transactions.** Each storage method is individually atomic (each is a single statement or an internal `BEGIN...COMMIT` block).
- **Cross-table operations are explicit.** `GameServiceContext` provides convenience methods (`load_messages`, `update_message_text`, `migrate_swipes`) that call multiple storage modules in sequence. The small gap between calls is accepted; it matches the existing behavior of `save_message_and_snapshot`, which has always been non-atomic across snapshot and message storage.

## Consequences

### Positive

- **Prevents bundling shortcuts.** Separating `message_swipes` into its own storage module makes it impossible to "accidentally" create a new message just to get a new swipe. The pipeline must explicitly call `insert_swipe`.
- **Single responsibility.** Each storage file maps 1:1 to a database table. Reasoning about what a method modifies is trivial.
- **Testability.** In-memory test implementations mirror the same 1:1 split, making mocks smaller and more focused.
- **Schema cleanup.** `ON DELETE CASCADE` removes the need for manual multi-table `DELETE` transactions in `delete_game`.

### Negative

- **More modules.** Six tables → six storage modules (four existing, two new).
- **Non-atomic cross-storage operations.** A crash between `insert_message` and `insert_swipe` could leave a message with no swipes. The window is tiny (two sequential SQLite statements), but it exists.
- **`GameServiceContext` grows.** It accumulates convenience methods for cross-storage assembly and coordination, blurring the line between "context" and "service." A dedicated thin service layer may be warranted in the future.

## Alternatives Considered

1. **Keep multi-table repositories, add internal sub-modules.** Rejected because it doesn't solve the psychological bundling problem. Developers still see `insert_message` taking a `Message` with swipes and treat them as a unit.
2. **Introduce a UnitOfWork / TransactionManager.** Rejected for now because `DbPool`'s single `Mutex<Connection>` makes transaction-passing architecturally difficult without significant refactor. Revisit if cross-storage atomicity becomes a concrete bug.
3. **Use `rusqlite` savepoints or nested transactions.** Rejected because `Connection::transaction()` requires a mutable reference held across storage calls, which conflicts with the `Arc<dyn Trait>` pattern used throughout the codebase.

## Migration Path

1. **Schema**: Add `ON DELETE CASCADE` FKs (migration v9).
2. **New traits**: Create `MessageSwipeStorage` and `GameStorage` without breaking existing code.
3. **Helpers**: Add `GameServiceContext` convenience methods that delegate to old trait methods.
4. **Caller migration**: Move all callers to helpers.
5. **Trait refactor**: Break `MessageStorage` and `SnapshotStorage` traits, update implementations.
6. **Pipeline refactor**: Rewrite retry logic to use `insert_swipe` + `update_active_swipe`, remove `migrate_swipes` helper.

## References

- `src/storage/message_storage.rs` — Original bundled message + swipe repository
- `src/storage/snapshot_storage.rs` — Original bundled game + snapshot repository
- `src/application/context.rs` — `GameServiceContext` and cross-storage helpers
- `src/application/action_pipeline/retry.rs` — Retry logic that recreates messages instead of adding swipes
- ADR-008, ADR-017 — Previous storage architecture decisions
