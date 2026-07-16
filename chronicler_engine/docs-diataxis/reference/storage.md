---
diataxis: reference
title: Storage
---

> **Diátaxis mode:** Reference. This document describes the storage layer as it is: the `Storage` boundary, backend decorator, game scoping, read contracts, snapshots, seeding guarantees, and testing seam. The problem it solves for the reader is *look-up*: which storage contract applies to a given operation. Schema relationships live in `./data_layer.md`; authoritative DDL lives in `src/adapters/driven/storage/db.rs`.

## Overview

`Storage` is the engine's persistence boundary for game sessions, narrative content, settings, and LLM call forensics. `Backend` identifies the real storage implementations (`Sqlite` and `InMemory`), while `BackendKind` selects direct access or the `Test` decorator. Storage methods operate on one table; multi-table coordination lives in `DefaultApplicationService`. The eleven-table schema is documented in `./data_layer.md`.

## Backend and BackendKind

- **`Backend`** identifies the real storage implementations: `Sqlite` and `InMemory`.
- **`BackendKind`** selects `Direct` access or `Test` failure injection around one `Backend`.
- **Decorator invariant.** `Test` wraps a single `Backend`, never another `BackendKind`. The storage seam therefore permits at most one `Test` layer at a time.

## Game Scoping

A `Storage` instance is bound to its game id when it is constructed. Game-scoped operations use that binding for the `games`, `game_state_snapshots`, `messages`, and `message_swipes` tables; callers do not supply a game id per call.

## Seeding Contract

The bootstrap seeding flow reads JSON templates under `data/` and writes the corresponding rows before runtime database reads begin. Its guarantees are:

- **Idempotent.** Re-running the flow does not create duplicate rows.
- **Database authority.** The database is the sole source of truth at runtime; seed files are templates used during bootstrap.
- **Startup-blocking.** Seeding completes before the HTTP server starts.
- **Corrupt-file tolerance.** A malformed seed file is skipped and seeding continues with the remaining files.

The bootstrap boundary and dependency order are described in `./startup.md`; the seed-file shapes are catalogued in `./data_schemas.md`.

## Settings Persistence

Settings occupy a singleton row in the `settings` table. Bootstrap loads the engine's settings once; the load-once and reload-on-restart rules are documented in `./architecture_system.md`.

## Read Contract: get_* and require_*

`Storage` exposes two absence policies for entity reads:

- **`get_*`** returns an optional result when absence is a valid runtime state.
- **`require_*`** returns a required result and turns absence into a typed error.

`GameNotFound` and `MessageNotFound` are the typed absence errors for their corresponding required reads. `Character` has no `require_*` helper; character access uses the listing contract. Backend read failures propagate as storage errors.

## GameStateSnapshot

`GameStateSnapshot` is a domain-owned DTO persisted by the storage layer. The storage layer serializes it, while application code uses the domain type. Snapshot JSON contains the serializable game-state subset; message history is persisted in `messages` and hydrated separately.

## Testing Strategy

Tests use an in-memory `Storage`. Failure-injection tests apply the `Test` decorator around a `Backend`, rather than around another decorator. This preserves the single-test-layer invariant while exercising storage failures through the same seam as production backends.

## Document References

- [Storage design](../explanation/storage_design.md) — current-understanding rationale for the storage layer, bootstrap flow, seed-as-template pattern, database authority, backend decorator, and single-test-layer invariant.
- [ADR-020: Unified Storage Struct](../../docs/adr/adr-020-storage-consolidation.md) — historical record of the unified storage decision.
- [ADR-024: Migrate Game Data to SQLite with Seed Pattern](../../docs/adr/adr-024-game-data-migration-to-sqlite.md) — historical record of the JSON-to-SQLite seed pattern.
- [ADR-026: Relocate Persona Binding from World to Game](../../docs/adr/adr-026-persona-relocation-to-game.md) — historical record of per-game persona binding.
- [Data Layer](./data_layer.md) — eleven-table schema and relationships.
- [Startup and Bootstrap](./startup.md) — bootstrap boundary, seeding order, and data-path resolution.
- [Data Schemas](./data_schemas.md) — JSON seed-file shapes.
