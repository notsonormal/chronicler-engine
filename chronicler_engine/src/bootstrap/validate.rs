//! [DOC: docs/system/startup.md]
//! Data validation utilities

use crate::model::character::{NpcCard, PlayerCard};
use crate::model::map::MapDef;
use crate::model::world::WorldCard;

pub fn validate_loaded_data(
    world: &WorldCard,
    map: &MapDef,
    _player: &PlayerCard,
    npcs: &[NpcCard],
) -> Result<(), String> {
    let mut errors = Vec::new();

    let mut valid_room_ids = std::collections::HashSet::new();
    for region in &map.overworld.regions {
        for room in &region.rooms {
            valid_room_ids.insert(room.id.clone());
        }
    }

    let resolved_starting_room = world.starting_room_id();
    if !valid_room_ids.contains(&resolved_starting_room) {
        errors.push(format!(
            "starting room '{resolved_starting_room}' not found in map"
        ));
    }

    for npc in npcs {
        for (i, trigger) in npc.triggers.iter().enumerate() {
            if let Some(room_id) = &trigger.room_id {
                if !valid_room_ids.contains(room_id) {
                    errors.push(format!(
                        "NPC '{}' Trigger[{}] references non-existent room_id: '{}'",
                        npc.id, i, room_id
                    ));
                }
            }
        }
    }

    let loaded_npc_ids: std::collections::HashSet<_> = npcs.iter().map(|n| n.id.clone()).collect();
    for scenario in &world.scenarios {
        for npc_id in &scenario.npcs {
            if !loaded_npc_ids.contains(npc_id) {
                errors.push(format!(
                    "Scenario '{}' references missing NPC '{}'",
                    scenario.id, npc_id
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}
