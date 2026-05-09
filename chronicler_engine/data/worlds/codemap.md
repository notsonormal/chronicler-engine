# chronicler_engine/data/worlds/

## Responsibility
Game world definitions. Each subdirectory is a self-contained world with a manifest, map, and scenario data.

## Subdirectories
| Directory | Purpose |
|-----------|---------|
| `redmist_estate/` | Production world — mystery/horror themed estate |
| `test/` | Minimal test world for integration tests |

## File Structure (per world)
```
world.json      → WorldManifest
map.json        → MapDef (regions, rooms, exits)
scenarios/      → StartingScenario definitions
```
