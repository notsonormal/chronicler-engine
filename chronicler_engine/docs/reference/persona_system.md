# Specification: Player Persona System

## Objective

Decouple the player character from the hardcoded "Hero" placeholder. Implement a persona system inspired by SillyTavern, allowing players to load a detailed JSON character card that defines their identity, personality, and background across different game worlds.

## Core Concepts

### 1. The Persona Structural Model

The player character requires the same depth as an NPC for the LLM Game Master to narrate accurately. The current `PlayerCard` is too simple.

**New Persona Fields:**

- `name`: The character's name.
- `description`: Physical appearance and basic traits.
- `personality`: Behavioral traits (e.g., "Arrogant, brave, tech-savvy").
- `scenario`: The character's personal background or current motivation (e.g., "The long-lost heir to a fortune").
- `inventory`: A list of item IDs.

### 2. Unified Character Logic

To avoid code duplication, `PlayerCard` and `NpcCard` should leverage a shared data structure (e.g., `CharacterSheet` - see `data_schemas.md` for proposed schema) for narrative fields. This ensures that the Game Master's narration logic can treat the player and NPCs with equal granular detail.

### 3. Persona Management

The engine will look for persona files in `data/personas/`.

- **Default Loading**: On server startup, `bootstrap/load.rs::seed_game_data` scans `data/personas/*.json` directly and seeds each persona into the `personas` table. Personas are world-independent; the world manifest no longer references a persona file (see ADR-026).
- **Per-Game Binding**: When a game is created via the Games tab, a persona is chosen explicitly and stored on the `games` row as `persona_key`/`persona_name` (denormalized for display).
- **Auto-Create via CLI**: On first boot with an empty DB, `resolve_game_id` auto-creates a game for the `--world` (default `redmist_estate`) using the `--persona` CLI flag (default `julian`). If the persona key is not found in the DB, boot hard-errors with `EngineError::Config` — no silent fallback. The CLI flag is a bootstrap parameter, not a runtime default; the Games-tab form remains the primary creation path for interactive use.
- **Portability**: Personas are standalone. The same persona can be used across different worlds.

## LLM Integration Changes

The Game Master system prompt must be updated to include the player's full persona context:

- The `personality` and `scenario` fields of the player's persona must be injected into the `narrate_action` prompt.
- This allows the GM to say: *"Since you are [Personality], you react with [Action]..."*.

## Role of "Julian Redmist"

By default, the engine uses "Julian Redmist":

- **Name**: Julian Redmist
- **Personality**: Curious, slightly overwhelmed, polite but firm.
- **Scenario**: Julian was raised by a single mother in a distant city, unaware of their lineage until Bernard Redmist's death. Julian has now arrived at the estate as the sole heir.

## Boundaries

- A "Persona Switcher" UI inside the game is not covered: persona selection happens at game-creation time (Games-tab "New Game" form), not within an active playthrough.
- Character stats (strength, agility) are deferred to a future RPG Mechanics spec.
