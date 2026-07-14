# Specification: Player Persona System

## Objective

Decouple the player character from the hardcoded "Hero" placeholder. Implement a persona system inspired by SillyTavern, allowing players to load a detailed JSON character card that defines their identity, personality, and background across different game worlds.

## Core Concepts

### 1. The Persona Structural Model

The player character requires the same depth as an NPC for the LLM Game Master to narrate accurately. The current `PersonaCard` is too simple.

**Persona fields:**

- `name`: The character's name.
- `description`: Physical appearance and basic traits.
- `personality`: Behavioral traits (e.g., "Arrogant, brave, tech-savvy").
- `scenario`: The character's personal background or current motivation (e.g., "The long-lost heir to a fortune").
- `inventory`: A list of item IDs.

### 2. Unified Character Logic

To avoid code duplication, `PersonaCard` and `NpcCard` share a common data structure — see the unified `CharacterSheet` schema.

### 3. Persona Management

The engine loads persona files from `data/personas/`.

- **Default Loading**: On server startup, `bootstrap/load.rs::seed_game_data` scans `data/personas/*.json` and seeds each persona into the `personas` table. Personas are world-independent; the world manifest does not reference a persona file.
- **Per-Game Binding**: When a game is created via the Games tab, a persona is chosen explicitly and stored on the `games` row as `persona_key`/`persona_name`.
- **Auto-Create via CLI**: On first boot with an empty DB, `resolve_game_id` auto-creates a game for the `--world` (default `redmist_estate`) using the `--persona` CLI flag (default `julian`). If the persona key is not found in the DB, boot hard-errors with `EngineError::PersonaNotFound(key)` — no silent fallback.
- **Portability**: Personas are standalone. The same persona can be used across different worlds.

## Document References

- [ADR-026: Relocate Persona Binding from World to Game](../adr/adr-026-persona-relocation-to-game.md) — persona is world-independent; per-game binding via `games.persona_key`
- [reference/data_schemas.md](../reference/data_schemas.md) — unified `CharacterSheet` schema shared by `PersonaCard` and `NpcCard`


