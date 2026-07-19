---
diataxis: reference
title: Startup and Bootstrap
---

> **Diátaxis mode:** Reference. This document describes engine startup as it is: data-path resolution, the two-phase bootstrap boundary, seeding dependencies, game-state initialization, server startup, settings loading, the JSON seed-file contracts, and the runtime invariants the seed schemas do not encode. The problem it solves for the reader is *look-up*: which boundary and invariant governs each startup concern.

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

The seeding contract, including startup blocking and corrupt-file handling, is enforced by the bootstrap layer.

## Game-State Initialization

The loader calls `default_scenario()` for the active world and reads that scenario's `starting_room_id` for the initial room. When no scenarios exist, the initial room id falls back to `"start"`.

## Server Startup

The HTTP server does not start until bootstrap has completed, including seeding and initial game-state initialization.

## Settings

Settings are loaded once during bootstrap and reload on restart.

## Schema Files

The engine reads seed data from JSON files under `data/` during bootstrap. The file shapes are defined by the JSON Schema files under `data/schemas/`.

| Schema file | Defines |
|---|---|
| `data/schemas/character.schema.json` | Shared `CharacterSheet` shape that both `PersonaCard` and `NpcCard` flatten. |
| `data/schemas/world.schema.json` | `WorldManifest` on-disk world shape. |
| `data/schemas/map.schema.json` | `MapDef` world map shape. |
| `data/schemas/settings.schema.json` | `AppSettings` engine settings file shape. |

The schema files are authoritative for seed-file fields and validation. Runtime behavior associated with those fields is covered by the invariants below and the linked Reference docs.

## Invariants Outside the Schemas

### Character identity and file locations

- `PersonaCard.key` is the JSON filename stem. The loader sets it from the persona filename; it is not part of the persona seed-file shape.
- `NpcCard.id` is a field in the NPC seed file.
- NPC files live at `data/characters/<group>/*.json`, where `<group>` is the world's `characters_dir` value. When that value is empty, the loader uses the world `id`.

### Seed-only world pointers

`WorldManifest.map_file` and `WorldManifest.characters_dir` are file pointers used during seeding. Conversion to `WorldCard` strips those pointers, so the runtime world card carries no filesystem locations.

### Persona and NPC roles

`PersonaCard` carries the shared character data. `triggers` and `relationships` belong to `NpcCard` and are NPC-only concerns.

### Trigger requirements

`Trigger.requirement` carries an `operator` and a `threshold`. The operator vocabulary is `Eq`, `Lt`, and `Gte`; runtime evaluation occurs in the trigger evaluation step.

### NPC event confidence

`NpcEvent.confidence` uses the `QuantifierConfidence` variants `High`, `Medium`, and `Low`. The events-diff path emits `Medium` when at least one transition occurred and `Low` when the previous and current NPC sets matched.

### Swipe snapshot reference

`Swipe.snapshot_id` references `GameStateSnapshot`. The SQL layer declares this relationship as non-FK; the relationship and its persistence semantics are described under Relationships and Messages.

## Document References

- [Storage design](../explanation/storage_design.md) — current-understanding rationale for the bootstrap flow, seed ordering, and seed files as templates.
- [Storage](./storage.md) — seeding contract, storage-layer invariants, eleven-table schema, and entity persistence (worlds, personas, messages).
- [Game Flow](./game_flow.md) — runtime trigger evaluation, phase pipeline, and event handling.
- [Agent System](./narrative/agent_system.md) — quantifier-agent processing.
- [ADR-017: Message Swipes](../../docs/adr/adr-017-message-swipes.md) — historical record of swipe snapshot semantics.
- [ADR-024: Migrate Game Data to SQLite with Seed Pattern](../../docs/adr/adr-024-game-data-migration-to-sqlite.md) — historical record of the seed pattern.
- [ADR-025: Multi-World Data Foundation](../../docs/adr/adr-025-multi-world-data-foundation.md) — historical record of the multi-world data foundation.
- [ADR-026: Relocate Persona Binding from World to Game](../../docs/adr/adr-026-persona-relocation-to-game.md) — historical record of per-game persona binding.
