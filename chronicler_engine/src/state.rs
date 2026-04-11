use crate::character::{NpcCard, PlayerCard};
use crate::map::MapDef;
use crate::world::WorldCard;
use std::collections::HashMap;

#[derive(Debug)]
pub struct GameState {
    pub world: WorldCard,
    pub map: MapDef,
    pub player: PlayerCard,
    pub npcs: HashMap<String, NpcCard>,
    pub current_room_id: String,
}

impl GameState {
    pub fn new(
        world: WorldCard,
        map: MapDef,
        player: PlayerCard,
        npcs: Vec<NpcCard>,
        starting_room: String,
    ) -> Self {
        let mut npcs_map = HashMap::new();
        for npc in npcs {
            npcs_map.insert(npc.id.clone(), npc);
        }
        Self {
            world,
            map,
            player,
            npcs: npcs_map,
            current_room_id: starting_room,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_state_initialization() {
        let world = WorldCard {
            name: "W".into(),
            description: "D".into(),
            global_rules: vec![],
        };
        let map = MapDef {
            overworld: crate::map::Overworld {
                id: "ow".into(),
                name: "ow".into(),
                regions: vec![],
            },
        };
        let player = PlayerCard {
            name: "P".into(),
            description: "P".into(),
            inventory: vec![],
        };
        let npc = NpcCard {
            id: "npc_1".into(),
            name: "N".into(),
            description: "D".into(),
            personality: "P".into(),
            scenario: "S".into(),
            example_dialogue: "E".into(),
            inventory: vec![],
        };

        let state = GameState::new(world, map, player, vec![npc], "room_1".to_string());

        assert_eq!(state.current_room_id, "room_1");
        assert_eq!(state.npcs.len(), 1);
        assert!(state.npcs.contains_key("npc_1"));
    }
}
