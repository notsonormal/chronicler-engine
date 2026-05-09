# chronicler_engine/data/characters/

## Responsibility
NPC character card storage. Each subdirectory is a character group (world) containing JSON files that deserialize into `NpcCard` structs.

## JSON Schema
```json
{
  "id": "npc_id",
  "name": "Display Name",
  "description": "...",
  "personality": "...",
  "scenario": "...",
  "triggers": [...]
}
```

## Subdirectories
| Directory | Purpose |
|-----------|---------|
| `redmist_estate/` | Characters for the Redmist Estate world |
| `test/` | Test-world characters for integration tests |
