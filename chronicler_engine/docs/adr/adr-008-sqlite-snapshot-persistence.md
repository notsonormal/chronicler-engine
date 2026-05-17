# ADR-008: SQLite Snapshot Persistence

**Date:** 2026-05-09

---

## Context

The engine previously held all mutable game state in a single `Arc<Mutex<GameState>>`. This central mutex was accessed by HTTP handlers, debug endpoints, and tests. While simple, it had critical limitations:

- **No per-turn history**: State mutations overwrote previous turns in-place
- **No reset support**: Players could not restart without killing the server process
- **No regeneration safety**: Retry logic had no stable anchor point to revert to
- **Test flakiness**: Concurrent tests shared mutable state through the same mutex

The Phase 1.7 migration (see `docs/plans/multi-agent-phase1-snapshots-reset-20260509.md`) removed the mutex in favor of SQLite-backed snapshots, but introduced an application-level read-modify-write race condition because multiple concurrent requests could load the same snapshot, modify independently, and overwrite each other.

---

## Decision

**Persist mutable game state as per-turn snapshots in SQLite, with a fast generation gate for concurrency control.**

### Snapshot Design

Only mutable sub-state is persisted. Immutable world data (`WorldCard`, `MapDef`, `PlayerCard`, `NpcCard` list) is cached in `AppState` as `Arc` references and re-attached on load.

```rust
pub struct GameStateSnapshot {
    pub movement: MovementState,
    pub narrative: NarrativeState,
    pub scene: SceneState,
    pub character_state: CharacterState,
}
```

### Storage Layer

- **`rusqlite` with `bundled` feature** — zero system dependencies
- **Migrations are code** — `run_migrations(conn)` applies `CREATE TABLE IF NOT EXISTS` at startup
- **Reset is hard delete** — drops all snapshot rows and rebuilds initial state from `GameState::new`

### Concurrency Control

An `AtomicBool` in `AppState` acts as a domain-level generation gate:

- `compare_exchange(false, true)` before accepting async actions
- Rejects concurrent actions with `"Still thinking..."`
- `GenerationGuard` (RAII) ensures the flag is cleared on task exit, even on panic
- Client-side: HTMX `hx-sync` + button disable prevent most double-submits

---

## Consequences

### Positive
- **Per-turn anchoring**: Pre-generation snapshots enable safe retry and regeneration
- **Reset without restart**: Clear SQLite and reload world JSON
- **Test isolation**: Each test can use an in-memory or temp-file database
- **Retry tracking**: Retry saves with incremented retry count, preserving original snapshot

### Negative
- **Disk I/O**: Every turn writes to SQLite (mitigated by WAL mode)
- **Schema evolution**: Future state changes require migration logic
- **Serialization cost**: `GameState` → `GameStateSnapshot` → JSON on every turn

### Trade-offs
- Chose SQLite over in-memory-only or JSON files for ACID guarantees and queryability
- Chose hard delete for reset over soft-delete for simplicity
- Chose generation gate over mutex for domain-semantic correctness (one turn at a time)

---

## Related ADRs

- [ADR-009: Agent Trait and Registry Architecture](./adr-009-agent-trait-registry.md) — Snapshots enable agent pipeline state anchoring

---

## History

- **2026-05-09**: Phase 1 implementation — SQLite snapshots + reset endpoint
- **2026-05-10**: Generation gate added to fix race condition introduced by mutex removal
