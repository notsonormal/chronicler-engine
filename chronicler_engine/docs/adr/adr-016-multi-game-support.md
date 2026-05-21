# ADR-016: Multi-Game Support

**Date:** 2026-05-21

---

## Context

The engine previously persisted all game state into a single default game (`id=1`) in the `games` table. There was no way to have multiple parallel playthroughs, and the checkpoint system was intended to provide named save points within that single game.

However, checkpoints had a critical UX problem: restoring a checkpoint rolled back the game state but did **not** roll back the message history. This meant the UI showed the full message history alongside older state, making checkpoints feel non-functional to users.

## Decision

**Replace checkpoints with multi-game support.**

Rather than fixing checkpoints (which would require truncating message history and complex UI state management), we leverage the existing `games` table to create truly independent playthroughs. Each game has:
- Its own snapshots
- Its own message history
- An auto-generated display name
- Runtime switching via `set_game_id`

### Game Naming

Auto-generated names follow `{WorldName}_{Date}_N` where `N` is `max(existing N) + 1`:
- `Redmist_2026-05-21_1`
- `Redmist_2026-05-21_2` (if `_1` exists)
- `Redmist_2026-05-21_4` (if `_1` and `_3` exist)

### Storage Traits

`SnapshotStorage` and `MessageStorage` gain `set_game_id(&self, game_id: u64)` so the active game can be switched without recreating `AppState`. The `SqliteGameStorage` struct wraps `game_id` in `AtomicU64` for interior mutability.

### Schema Changes

- `games` table: added `name TEXT NOT NULL DEFAULT 'Unnamed'`
- `checkpoints` table: **dropped entirely**

## Consequences

### Positive
- **True isolation**: Each game is completely independent — no shared state, no confusing history mismatches.
- **Simpler mental model**: "Games" is a more familiar concept than "checkpoints + snapshots".
- **No message truncation needed**: Switching games loads the correct history automatically.
- **Pre-release flexibility**: Breaking schema changes are acceptable per existing migration policy.

### Negative
- **Data loss for existing users**: Migration v5 drops the `checkpoints` table. Existing checkpoint data is lost.
- **More database growth**: Multiple games mean multiple independent snapshot/message histories.
- **UI complexity**: Need a dedicated Save / Load tab with game listing, creation, switching, and deletion flows.

## Related ADRs

- [ADR-008: SQLite Snapshot Persistence](./adr-008-sqlite-snapshot-persistence.md) — The snapshot system that multi-game support builds on.

## UI Architecture

Game management lives in a dedicated **Save / Load** tab, separate from the main Game tab:

- **Active Game** section — highlights the current game with visual distinction (green accent + "Current" badge)
- **Saved Games** list — all other games for the current world, each with Switch and Delete actions
- **Actions** row — "New Game" (creates and switches to a fresh game) and "Reset Current Game" (deletes active game, creates fresh replacement)

The header bar is intentionally minimal — engine title, active game name, and connection status only. Game CRUD does not belong in the header because it is not part of every-session gameplay; it is a session-management concern that deserves its own space.

## State Initialization

Every newly created game must start from the same baseline as a bootstrapped world. This means the creation flow must:

1. Create the game record
2. Switch the storage layer to the new game ID
3. Build fresh initial state (scenario logs, NPCs, scene)
4. Persist the initial snapshot
5. Persist the starting scenario message

This ensures new games are immediately playable — they do not start empty and require a separate initialization step.

## State Loading

When the engine loads state from storage, it reconstructs `GameState` from the latest snapshot first, then overlays message history from the messages table. Message history is only applied when present; an empty messages table does not override the snapshot-derived state. This separation keeps snapshots (structural state) and messages (display history) independently recoverable.

## History

- **2026-05-21**: Initial implementation — multi-game CRUD, auto-naming, checkpoint removal.
- **2026-05-21**: UI redesigned as dedicated Save / Load tab (previously header widget). New-game initialization unified with bootstrap path. Defensive loading added to prevent empty message storage from clearing snapshot history.
