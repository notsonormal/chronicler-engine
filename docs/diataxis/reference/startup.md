---
diataxis: reference
title: Startup and Bootstrap
---

## Overview

Bootstrap resolves the seed-data location, completes file seeding, initializes the first game state from database-backed world data, and only then starts the HTTP server. The file-to-database boundary is the central startup contract: file reads belong to seeding, and runtime reads use the database.

## Two-Phase Bootstrap

Bootstrap has two phases with a strict boundary:

- **File seeding** reads JSON seed files and creates any missing database rows. Seeding is idempotent.
- **DB-first runtime** uses database rows for all subsequent reads.

### Seeding Order

The dependency invariant for seed rows is:

- **Worlds and maps precede characters.** Character rows depend on the world being present.
- **Personas are world-independent.** Their seed pass does not depend on a world row.
- **Prompt presets are independent.** Their seed pass does not depend on world, map, character, or persona rows.
- **Settings are independent.** Their seed pass inserts the singleton row when it is absent. A malformed `data/settings.json` halts bootstrap — the one fail-loud exception to skip-and-continue.

Seeding completes before the HTTP server starts. A malformed world, persona, or preset file is skipped without halting bootstrap.

## Game-State Initialization

Bootstrap picks the newest Game of the selected World through `GameCatalogue`.
A World with no Game gets one through
`GameCatalogue::create_game`, the path the Games page uses. That path writes the Game, its dock record and its opening Narration in one transaction.

Every Game is idle after a start. The holders of the busy state live in memory.

## Schema Files

The JSON Schema files under `data/schemas/` are authoritative for seed-file fields and validation.

## Invariants Outside the Schemas

### Character identity and file locations

- `PersonaCard.key` is the JSON filename stem. The loader sets it from the persona filename. It is not part of the persona seed-file shape.
- `CharacterCard.id` is a field in the Character seed file.
- Character files live at `data/characters/<group>/*.json`, where `<group>` is the world's `characters_dir` value. When that value is empty, the loader uses the world `id`.

### Seed-only world pointers

`WorldManifest.map_file` and `WorldManifest.characters_dir` are file pointers used during seeding. Conversion to `WorldCard` strips those pointers, so the runtime world card carries no filesystem locations.

## Document References

- [Storage design](../explanation/storage_design.md) — current-understanding rationale for the bootstrap flow, seed ordering, and seed files as templates.
- [Storage](./storage.md) — seeding contract, storage-layer invariants, schema, and entity persistence (worlds, personas, messages).
- [Game Flow](./game_flow.md) — the Job, runtime trigger evaluation, and event handling.
- [Agent System](./narrative/agent_system.md) — quantifier-agent processing.
