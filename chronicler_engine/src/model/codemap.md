# chronicler_engine/src/model/

## Responsibility
The domain model tier. Defines all data structures for the game world, characters, map, state, triggers, and settings. Pure data containers with serde serialization — no business logic.

## Design Patterns
- **Data Transfer Objects (DTOs)**: All structs are plain data with `pub` fields, derived `Serialize`/`Deserialize`.
- **Flatting**: `PlayerCard` and `NpcCard` use `#[serde(flatten)]` to embed `CharacterSheet`.
- **Newtype Pattern**: `WorldCard` is a slimmed view of `WorldManifest` for runtime use.
- **State Pattern**: `GenerationStatus` and `GenerationPhase` enums track engine lifecycle state.

## Data & Control Flow
```
Bootstrap (JSON files)
  → WorldManifest, MapDef, PlayerCard, Vec<NpcCard>
    → GameState (aggregates all + runtime mutable state)
      → Server handlers read/write GameState via Arc<Mutex<_>>
        → Engine logic reads GameState immutably
          → Trigger eval reads CharacterState
```

## Integration Points
- **Consumed by**: `engine/` (all logic), `server/` (rendering), `narrative/` (prompt building)
- **Layer enforcement**: `arch-lint` forbids `model/` from importing `server/`, `narrative/`, or `engine/`

## Files
| File | Purpose |
|------|---------|
| `world.rs` | `WorldCard`, `WorldManifest` — world metadata and defaults |
| `map.rs` | `MapDef`, `Overworld`, `Region`, `Room`, `Direction` — spatial data |
| `character.rs` | `CharacterSheet`, `PlayerCard`, `NpcCard` — character definitions |
| `state.rs` | `GameState`, `LogEntry`, `LogType`, `GenerationState`, `GenerationStatus`, `GenerationPhase` |
| `trigger.rs` | `Trigger`, `TriggerCondition`, `ComparisonOperator`, `CharacterState`, `NpcEncounterState` |
| `scenario.rs` | `StartingScenario` — initial game conditions |
| `settings.rs` | `AppSettings`, `Connection`, `TextCheckMode` — runtime configuration |
| `llm_backend.rs` | `LlmConnection` — backend connection configuration |
| `mod.rs` | Module exports and test module declarations |
