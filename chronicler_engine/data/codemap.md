# chronicler_engine/data/

## Responsibility
Runtime game data directory. Contains JSON configuration files for worlds, characters, player personas, validation schemas, and runtime settings.

## Structure
| Subdirectory | Purpose |
|--------------|---------|
| `worlds/` | Game world definitions (map, manifest, scenarios) |
| `characters/` | NPC character cards (JSON with triggers, personality) |
| `personas/` | Player character definitions |
| `schemas/` | JSON schemas for data validation |
| `settings.json` | Runtime engine configuration (LLM connections, text check) |

## Validation
`scripts/validate_data.py` validates all JSON files against schemas in `data/schemas/`.
