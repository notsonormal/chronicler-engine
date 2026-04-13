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
- **Default Loading**: On startup, the engine strictly loads `data/personas/julian.json`.
- **Portability**: Personas are standalone. The "Julian Redmist" persona should be generalized enough to work in other settings, with the `WorldCard` providing setting-specific context.

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
- This spec does not cover a "Persona Switcher" UI *inside* the game yet. Selection happens at boot time.
- Character stats (strength, agility) are deferred to a future RPG Mechanics spec.
