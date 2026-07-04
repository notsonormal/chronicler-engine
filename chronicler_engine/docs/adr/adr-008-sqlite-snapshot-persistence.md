# ADR-008: SQLite Snapshot Persistence

**Date:** 2026-05-09
**Status:** Accepted

---

## Context

The engine previously held all mutable game state in a single `Arc<Mutex<GameState>>`. This central mutex was accessed by HTTP handlers, debug endpoints, and tests. While simple, it had critical limitations:

- **No granular history**: State mutations overwrote previous state in-place
- **No reset support**: Players could not restart without killing the server process
- **No regeneration safety**: Retry logic had no stable anchor point to revert to
- **Test flakiness**: Concurrent tests shared mutable state through the same mutex

The Phase 1.7 migration removed the mutex in favor of SQLite-backed snapshots, but introduced an application-level read-modify-write race condition because multiple concurrent requests could load the same snapshot, modify independently, and overwrite each other.

---

## Decision

**Persist mutable game state as message-aligned snapshots in SQLite, with a fast generation gate for concurrency control.**

### Snapshot Design

Only mutable sub-state is persisted. Immutable world data (`WorldCard`, `MapDef`, `PlayerCard`, `NpcCard` list) is cached in `AppState` as `Arc` references and re-attached on load.

```rust
pub struct GameStateSnapshot {
    pub movement: MovementState,
    pub narrative: NarrativeSnapshot,
    pub scene: SceneState,
    pub npc_encounter_log: NpcEncounterLog,
}
```

### Storage Layer

- **`rusqlite` with `bundled` feature** — zero system dependencies
- **Migrations are code** — `run_migrations(conn)` applies `CREATE TABLE IF NOT EXISTS` at startup
- **Reset is hard delete** — drops all snapshot rows and rebuilds initial state from `GameState::new`

Concurrency control (generation gate, AtomicBool, RAII guard) is documented in ADR-010.

---

## Consequences

### Positive
- **Message-level anchoring**: Pre-generation snapshots enable safe retry and regeneration
- **Reset without restart**: Clear SQLite and reload world JSON
- **Test isolation**: Each test can use an in-memory or temp-file database
- **Retry tracking**: Retry saves with incremented retry count, preserving original snapshot

### Negative
- **Disk I/O**: Every message generation writes to SQLite (mitigated by WAL mode)
- **Schema evolution**: Future state changes require migration logic
- **Serialization cost**: `GameState` → `GameStateSnapshot` → JSON on every action

### Trade-offs
- Chose SQLite over in-memory-only or JSON files for ACID guarantees and queryability
- Chose hard delete for reset over soft-delete for simplicity

---

## Related ADRs

- [ADR-010: Concurrency and Generation Gate Model](./adr-010-concurrency-generation-gate.md) — Generation gate that replaced the old mutex
- [ADR-009: Agent Trait and Registry Architecture](./adr-009-agent-trait-registry.md) — Snapshots enable agent pipeline state anchoring

---

## History

- **2026-05-09**: Phase 1 implementation — SQLite snapshots + reset endpoint
- **2026-05-10**: Generation gate added to fix race condition introduced by mutex removal
- **2026-05-20**: Immediate message persistence — removed `committed` flag, `commit()` method, and batch persistence. Each message is now persisted immediately alongside its snapshot via `save_message_and_snapshot`.
