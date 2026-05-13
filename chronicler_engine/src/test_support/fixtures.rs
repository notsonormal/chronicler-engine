use std::collections::HashMap;
use std::sync::Arc;

use crate::model::character::{CharacterSheet, NpcCard, PlayerCard};
use crate::model::map::{MapDef, Overworld, Region, Room};
use crate::model::state::GameState;
use crate::model::trigger::{ComparisonOperator, Trigger, TriggerAction, TriggerCondition};
use crate::model::world::WorldCard;

// ─── World ───────────────────────────────────────────────────────────────────

pub struct TestWorld;

impl TestWorld {
    /// A minimal `WorldCard` with no rules.
    pub fn minimal() -> WorldCard {
        WorldCard {
            name: "Test World".to_string(),
            description: "A test world.".to_string(),
            global_rules: vec![],
            starting_room_id: "start".to_string(),
            scenarios: vec![],
            default_room_image: None,
        }
    }

    /// A `WorldCard` with one global rule.
    pub fn with_rule(rule: &str) -> WorldCard {
        WorldCard {
            global_rules: vec![rule.to_string()],
            ..Self::minimal()
        }
    }
}

// ─── Player ───────────────────────────────────────────────────────────────────

pub struct TestPlayer;

impl TestPlayer {
    /// A `PlayerCard` with the given name and sensible defaults.
    pub fn named(name: &str) -> PlayerCard {
        PlayerCard {
            sheet: CharacterSheet {
                name: name.to_string(),
                description: format!("The protagonist named {name}."),
                personality: "Determined".to_string(),
                scenario: "Test scenario.".to_string(),
                example_dialogue: String::new(),
                summary: None,
                profile_image: None,
                headshot_image: None,
            },
            inventory: vec![],
        }
    }

    /// Default test player named "Hero".
    pub fn standard() -> PlayerCard {
        Self::named("Hero")
    }
}

// ─── NPC ─────────────────────────────────────────────────────────────────────

pub struct TestNpc;

impl TestNpc {
    fn sheet(name: &str) -> CharacterSheet {
        CharacterSheet {
            name: name.to_string(),
            description: format!("A character named {name}."),
            personality: "Neutral".to_string(),
            scenario: "Test scenario.".to_string(),
            example_dialogue: String::new(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        }
    }

    /// An `NpcCard` with the given ID and display name, and no triggers.
    pub fn named(id: &str, name: &str) -> NpcCard {
        NpcCard {
            id: id.to_string(),
            sheet: Self::sheet(name),
            inventory: vec![],
            triggers: vec![],
            relationships: vec![],
        }
    }

    /// An `NpcCard` with one `TimesMet` trigger.
    pub fn with_times_met_trigger(id: &str, name: &str, op: ComparisonOperator, n: u32) -> NpcCard {
        NpcCard {
            id: id.to_string(),
            sheet: Self::sheet(name),
            inventory: vec![],
            triggers: vec![Trigger {
                condition: TriggerCondition::TimesMet(op, n),
                action: TriggerAction {
                    name: format!("{name} Introduction"),
                    narration_prompt: format!("{name} introduces themselves."),
                },
                repeat: false,
                room_id: None,
            }],
            relationships: vec![],
        }
    }

    /// An `NpcCard` with a room-scoped `TimesMet` trigger.
    pub fn with_room_scoped_trigger(
        id: &str,
        name: &str,
        op: ComparisonOperator,
        n: u32,
        room_id: &str,
    ) -> NpcCard {
        NpcCard {
            id: id.to_string(),
            sheet: Self::sheet(name),
            inventory: vec![],
            triggers: vec![Trigger {
                condition: TriggerCondition::TimesMet(op, n),
                action: TriggerAction {
                    name: format!("{name} Encounter in {room_id}"),
                    narration_prompt: format!("{name} acknowledges you in this specific room."),
                },
                repeat: false,
                room_id: Some(room_id.to_string()),
            }],
            relationships: vec![],
        }
    }
}

// ─── Map ──────────────────────────────────────────────────────────────────────

pub struct TestMap;

impl TestMap {
    /// A `Room` with the given ID, no exits, and no NPCs.
    pub fn room(id: &str) -> Room {
        Room {
            id: id.to_string(),
            name: format!("Room {id}"),
            description: format!("A plain test room ({id})."),
            exits: HashMap::new(),
            items: vec![],
            image_path: None,
            navigation_description: None,
        }
    }

    /// A `Room` with the given ID and display name, no exits, and no NPCs.
    pub fn room_named(id: &str, name: &str) -> Room {
        Room {
            id: id.to_string(),
            name: name.to_string(),
            description: format!("A plain test room ({id})."),
            exits: HashMap::new(),
            items: vec![],
            image_path: None,
            navigation_description: None,
        }
    }

    /// A `MapDef` containing a single region with a single room.
    pub fn single_room(room_id: &str) -> MapDef {
        MapDef {
            overworld: Overworld {
                id: "test_overworld".to_string(),
                name: "Test Overworld".to_string(),
                regions: vec![Region {
                    id: "test_region".to_string(),
                    name: "Test Region".to_string(),
                    rooms: vec![Self::room(room_id)],
                }],
            },
        }
    }

    /// A `MapDef` with two connected rooms (north/south exits).
    pub fn two_rooms(room_a_id: &str, room_b_id: &str) -> MapDef {
        use crate::model::map::Direction;
        let mut room_a = Self::room(room_a_id);
        let mut room_b = Self::room(room_b_id);
        room_a.exits.insert(Direction::North, room_b_id.to_string());
        room_b.exits.insert(Direction::South, room_a_id.to_string());
        MapDef {
            overworld: Overworld {
                id: "test_overworld".to_string(),
                name: "Test Overworld".to_string(),
                regions: vec![Region {
                    id: "test_region".to_string(),
                    name: "Test Region".to_string(),
                    rooms: vec![room_a, room_b],
                }],
            },
        }
    }
}

// ─── GameState ────────────────────────────────────────────────────────────────

pub struct TestGameState;

impl TestGameState {
    /// A fully initialised `GameState` in the given room with no NPCs.
    pub fn in_room(room_id: &str) -> GameState {
        GameState::new(
            Arc::new(TestWorld::minimal()),
            Arc::new(TestMap::single_room(room_id)),
            Arc::new(TestPlayer::standard()),
            vec![],
            room_id.to_string(),
        )
    }

    /// A `GameState` in the given room with one NPC loaded (and listed in the room).
    pub fn with_npc(room_id: &str, npc: NpcCard) -> GameState {
        let _npc_id = npc.id.clone();
        GameState::new(
            Arc::new(TestWorld::minimal()),
            Arc::new(TestMap::single_room(room_id)),
            Arc::new(TestPlayer::standard()),
            vec![npc],
            room_id.to_string(),
        )
    }

    /// A `GameState` in the given room with multiple NPCs loaded.
    pub fn with_npcs(room_id: &str, npcs: Vec<NpcCard>) -> GameState {
        let _npc_ids: Vec<String> = npcs.iter().map(|n| n.id.clone()).collect();
        let map = MapDef {
            overworld: Overworld {
                id: "test_overworld".to_string(),
                name: "Test Overworld".to_string(),
                regions: vec![Region {
                    id: "test_region".to_string(),
                    name: "Test Region".to_string(),
                    rooms: vec![TestMap::room(room_id)],
                }],
            },
        };
        GameState::new(
            Arc::new(TestWorld::minimal()),
            Arc::new(map),
            Arc::new(TestPlayer::standard()),
            npcs,
            room_id.to_string(),
        )
    }

    /// A `GameState` in the given room with one NPC loaded, but with
    /// `character_state` set to default (no starting-room encounter tracking).
    pub fn with_npc_raw(room_id: &str, npc: NpcCard) -> GameState {
        let npc_id = npc.id.clone();
        let mut npcs_map = HashMap::new();
        npcs_map.insert(npc_id.clone(), npc);
        GameState {
            world: Arc::new(TestWorld::minimal()),
            map: Arc::new(TestMap::single_room(room_id)),
            player: Arc::new(TestPlayer::standard()),
            npcs: npcs_map,
            movement: crate::model::state::MovementState {
                current_room_id: room_id.to_string(),
                dynamic_rooms: HashMap::new(),
            },
            narrative: crate::model::state::NarrativeState {
                turns: Vec::new(),
                next_log_id: 1,
                generation: Default::default(),
                last_trigger: None,
                pending_location: None,
                pending_event: None,
            },
            scene: crate::model::state::SceneState {
                npcs_in_area: Vec::new(),
            },
            character_state: Default::default(),
        }
    }

    /// Like `with_npc_raw` but with a custom room display name.
    pub fn with_npc_in_named_room_raw(room_id: &str, room_name: &str, npc: NpcCard) -> GameState {
        let npc_id = npc.id.clone();
        let room = TestMap::room_named(room_id, room_name);
        let mut npcs_map = HashMap::new();
        npcs_map.insert(npc_id, npc);
        GameState {
            world: Arc::new(TestWorld::minimal()),
            map: Arc::new(MapDef {
                overworld: Overworld {
                    id: "test_overworld".to_string(),
                    name: "Test Overworld".to_string(),
                    regions: vec![Region {
                        id: "test_region".to_string(),
                        name: "Test Region".to_string(),
                        rooms: vec![room],
                    }],
                },
            }),
            player: Arc::new(TestPlayer::standard()),
            npcs: npcs_map,
            movement: crate::model::state::MovementState {
                current_room_id: room_id.to_string(),
                dynamic_rooms: HashMap::new(),
            },
            narrative: crate::model::state::NarrativeState {
                turns: Vec::new(),
                next_log_id: 1,
                generation: Default::default(),
                last_trigger: None,
                pending_location: None,
                pending_event: None,
            },
            scene: crate::model::state::SceneState {
                npcs_in_area: Vec::new(),
            },
            character_state: Default::default(),
        }
    }
}
