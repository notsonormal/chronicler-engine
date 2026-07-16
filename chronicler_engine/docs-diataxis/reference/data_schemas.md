---
diataxis: reference
title: Data Schemas
---

> **Diátaxis mode:** Reference. This document describes the JSON seed-file contracts as they are: the authoritative schema files and the runtime invariants they do not encode. The problem it solves for the reader is *look-up*: where each seed shape is defined and which loader or persistence rules accompany it. Storage and bootstrap contracts live in `./storage.md` and `./startup.md`.

## Overview

The engine reads seed data from JSON files under `data/` during bootstrap. The file shapes are defined by the JSON Schema files under `data/schemas/`; this document names those schemas, records the load-bearing invariants outside the schemas, and points to the storage and bootstrap references that describe their use.

## Schema Files

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

`Trigger.requirement` carries an `operator` and a `threshold`. The operator vocabulary is `Eq`, `Lt`, and `Gte`; runtime evaluation occurs in the trigger evaluation step described in `./triggers.md`.

### NPC event confidence

`NpcEvent.confidence` uses the `QuantifierConfidence` variants `High`, `Medium`, and `Low`. The events-diff path emits `Medium` when at least one transition occurred and `Low` when the previous and current NPC sets matched. The quantifier step is described in `./triggers.md` and `./agent_system.md`.

### Swipe snapshot reference

`Swipe.snapshot_id` references `GameStateSnapshot`. The SQL layer declares this relationship as non-FK; the relationship and its persistence semantics are described in `./data_layer.md` and `./message_model.md`.

## Document References

- [Storage design](../explanation/storage_design.md) — current-understanding rationale for seed files as templates and the database as runtime source of truth.
- [ADR-024: Migrate Game Data to SQLite with Seed Pattern](../../docs/adr/adr-024-game-data-migration-to-sqlite.md) — historical record of the seed pattern.
- [ADR-025: Multi-World Data Foundation](../../docs/adr/adr-025-multi-world-data-foundation.md) — historical record of the multi-world data foundation.
- [ADR-026: Relocate Persona Binding from World to Game](../../docs/adr/adr-026-persona-relocation-to-game.md) — historical record of per-game persona binding.
- [ADR-017: Message Swipes](../../docs/adr/adr-017-message-swipes.md) — historical record of swipe snapshot semantics.
- [Storage](./storage.md) — storage and seeding contracts.
- [Startup and Bootstrap](./startup.md) — bootstrap boundary and seeding order.
- [Data Layer](./data_layer.md) — persistence relationships and the non-FK snapshot reference.
- [Message Model](./message_model.md) — message and swipe persistence semantics.
- [Triggers](./triggers.md) — runtime trigger evaluation and event handling.
- [Agent System](./agent_system.md) — quantifier-agent processing.
