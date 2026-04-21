# System: Auto-Trigger & Reactive Encounters

The Auto-Trigger system allows the game world to react dynamically to the player's presence based on NPC-specific conditions.

## Overview
When a player enters a room or performs an action, the engine evaluates a set of rules (Triggers) associated with the NPCs present in that area. If conditions are met, a reactive event (Action) is fired.

## Core Flow
1. **Detection**: Player moves or performs an action.
2. **Evaluation**: `crate::engine::trigger_eval::evaluate_triggers` scans all NPCs in `state.npcs_in_area`.
3. **Condition Check**: Each trigger is checked against the current `CharacterState`.
4. **Execution**:
   - If repeatable: Trigger fires and can fire again in the future.
   - If non-repeatable: Trigger is marked as "fired" in `CharacterState` and won't re-fire.
5. **Narration**: Trigger actions typically generate "Continuation Narration" which is appended to the main game log.

## Trigger Conditions
| Condition | Description |
| :--- | :--- |
| `TimesMet` | Evaluates the `times_met` counter for the NPC using `Eq`, `Lt`, or `Gte`. |
| `HasItem` | (Planned) Checks player inventory for a specific item ID. |

## Trigger Actions
| Action | Description |
| :--- | :--- |
| `Narrate` | Appends a custom LLM prompt to the arrival/action narration. |

## Character State Tracking
The `CharacterState` struct is the source of truth for all reactive logic:
- `times_met`: Incremented whenever a significant encounter occurs.
- `fired_triggers`: A bitmask or list of indices for non-repeatable triggers.
