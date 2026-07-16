---
diataxis: reference
title: Persona System
---

> **Diátaxis mode:** Reference. This document describes the player persona as it is: its world-independent loading, per-game binding, empty-database creation behavior, and cross-world portability. The problem it solves for the reader is *look-up*: how a persona enters the database and attaches to a game. The card shape is defined by `data/schemas/character.schema.json` and described in `./data_schemas.md`.

## Overview

The player persona is a `PersonaCard`, a structured character sheet used by the Game Master. Inspired by SillyTavern, it replaces the hardcoded `Hero` placeholder with a world-independent JSON card, allowing the same player identity to carry across worlds while giving the protagonist the same narrative detail as an NPC. The persona's binding is per game, while NPC data is seeded per world.

## Card Shape

The field shape is defined by `data/schemas/character.schema.json`; the loader and runtime invariants are catalogued in `./data_schemas.md`.

## Default Loading

During bootstrap, the seeding flow scans `data/personas/*.json` and creates the corresponding persona rows. This pass is world-independent and idempotent; each persona is keyed by its JSON filename stem.

## Per-Game Binding

A game binds one persona at creation time through `games.persona_key`. The cross-cluster logical reference (non-FK) is documented in `./data_layer.md`. Game loading uses the bound key to hydrate the player's prompt context.

## Auto-Create on Empty DB

When the database is empty, bootstrap auto-creates a game using the `--world` and `--persona` CLI flags. The requested persona key must already be present in the seeded persona rows. A missing persona key is a hard startup error.

## Portability

A persona is world-independent, so the same persona can be bound to games in different worlds without changing the persona data.

## Document References

- [Storage design](../explanation/storage_design.md) — current-understanding rationale for storage, seeding, and persona data as templates.
- [ADR-026: Relocate Persona Binding from World to Game](../../docs/adr/adr-026-persona-relocation-to-game.md) — historical record of the persona-binding decision.
- [Data Schemas](./data_schemas.md) — character-card field shape and seed-file invariants.
- [Data Layer](./data_layer.md) — `personas` table and `games.persona_key` logical reference.
- [Startup and Bootstrap](./startup.md) — bootstrap boundary and persona seeding pass.
- [Storage](./storage.md) — seeding contract and game-scoped storage.
