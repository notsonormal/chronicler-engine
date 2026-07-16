---
diataxis: reference
title: Startup and Bootstrap
---

> **Diátaxis mode:** Reference. This document describes engine startup as it is: data-path resolution, the two-phase bootstrap boundary, seeding dependencies, game-state initialization, server startup, and settings loading. The problem it solves for the reader is *look-up*: which boundary and invariant governs each startup concern. Seeding guarantees live in `./storage.md`; the eleven-table relationships live in `./data_layer.md`.

## Overview

Bootstrap resolves the seed-data location, completes file seeding, initializes the first game state from database-backed world data, and only then starts the HTTP server. The file-to-database boundary is the central startup contract: file reads belong to seeding, and runtime reads use the database.

## Data Path Resolution

The engine resolves the `data/` directory in this priority order:

1. A `data/` directory alongside the executable.
2. `./data` under the current working directory.

The first available directory is the seed source for the run.

## Two-Phase Bootstrap

Bootstrap has two phases with a strict boundary:

- **File seeding** reads JSON seed files and creates any missing database rows. Seeding is idempotent.
- **DB-first runtime** uses database rows for all subsequent reads.

### Seeding Order

The dependency invariant for seed rows is:

- **Worlds and maps precede characters.** Character rows depend on the world being present.
- **Personas are world-independent.** Their seed pass does not depend on a world row.
- **Prompt presets are independent.** Their seed pass does not depend on world, map, character, or persona rows.

The seeding contract, including startup blocking and corrupt-file handling, is documented in `./storage.md`. Seed-file shapes are defined by the schemas listed in `./data_schemas.md`.

## Game-State Initialization

The loader calls `default_scenario()` for the active world and reads that scenario's `starting_room_id` for the initial room. When no scenarios exist, the initial room id falls back to `"start"`.

## Server Startup

The HTTP server does not start until bootstrap has completed, including seeding and initial game-state initialization.

## Settings

Settings are loaded once during bootstrap and reload on restart. The detailed reload rules are in `./architecture_system.md`.

## Document References

- [Storage design](../explanation/storage_design.md) — current-understanding rationale for the bootstrap flow and seed ordering.
- [ADR-024: Migrate Game Data to SQLite with Seed Pattern](../../docs/adr/adr-024-game-data-migration-to-sqlite.md) — historical record of the seed pattern.
- [ADR-026: Relocate Persona Binding from World to Game](../../docs/adr/adr-026-persona-relocation-to-game.md) — historical record of per-game persona binding.
- [Storage](./storage.md) — seeding contract and storage-layer invariants.
- [Data Layer](./data_layer.md) — eleven-table schema and foreign-key relationships.
- [Data Schemas](./data_schemas.md) — JSON seed-file shapes.
