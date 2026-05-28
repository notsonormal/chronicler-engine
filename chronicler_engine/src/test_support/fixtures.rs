use std::collections::HashMap;
use std::sync::Arc;

use crate::model::character::{CharacterSheet, NpcCard, PlayerCard};
use crate::model::map::{MapDef, Overworld, Region, Room};
use crate::model::prompt_preset::{PresetType, PromptPreset};
use crate::model::state::{GameState, StoredTriggerContext};
use crate::model::trigger::{ComparisonOperator, Trigger, TriggerCondition, TriggerEffect};
use crate::model::world::{WorldCard, WorldManifest};

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
                effect: TriggerEffect {
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
                effect: TriggerEffect {
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
    /// `npc_encounter_log` set to default (no starting-room encounter tracking).
    pub fn with_npc_raw(room_id: &str, npc: NpcCard) -> GameState {
        crate::model::state::GameStateBuilder::new(
            Arc::new(TestWorld::minimal()),
            Arc::new(TestMap::single_room(room_id)),
            Arc::new(TestPlayer::standard()),
            room_id,
        )
        .with_npcs(vec![npc])
        .build()
    }

    /// Like `with_npc_raw` but with a custom room display name.
    pub fn with_npc_in_named_room_raw(room_id: &str, room_name: &str, npc: NpcCard) -> GameState {
        let room = TestMap::room_named(room_id, room_name);
        let map = MapDef {
            overworld: Overworld {
                id: "test_overworld".to_string(),
                name: "Test Overworld".to_string(),
                regions: vec![Region {
                    id: "test_region".to_string(),
                    name: "Test Region".to_string(),
                    rooms: vec![room],
                }],
            },
        };
        crate::model::state::GameStateBuilder::new(
            Arc::new(TestWorld::minimal()),
            Arc::new(map),
            Arc::new(TestPlayer::standard()),
            room_id,
        )
        .with_npcs(vec![npc])
        .build()
    }
}

// ─── StoredTriggerContext ────────────────────────────────────────────────────

pub struct TestStoredTriggerContext;

impl TestStoredTriggerContext {
    /// Standard test trigger context used across pipeline and retry tests.
    pub fn standard() -> StoredTriggerContext {
        StoredTriggerContext {
            npc_id: "npc1".to_string(),
            trigger_idx: 0,
            trigger_name: "Test".to_string(),
            trigger_repeat: false,
            trigger_narration_prompt: "Test prompt".to_string(),
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
            max_tokens: None,
        }
    }

    /// A trigger context for a specific NPC with custom narration prompt.
    pub fn for_npc(
        npc_id: &str,
        trigger_name: &str,
        narration_prompt: &str,
    ) -> StoredTriggerContext {
        StoredTriggerContext {
            npc_id: npc_id.to_string(),
            trigger_idx: 0,
            trigger_name: trigger_name.to_string(),
            trigger_repeat: false,
            trigger_narration_prompt: narration_prompt.to_string(),
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
            max_tokens: None,
        }
    }

    /// A trigger context with a specific name and NPC ID.
    pub fn named(trigger_name: &str, npc_id: &str) -> StoredTriggerContext {
        StoredTriggerContext {
            npc_id: npc_id.to_string(),
            trigger_idx: 0,
            trigger_name: trigger_name.to_string(),
            trigger_repeat: false,
            trigger_narration_prompt: format!("{trigger_name} fires."),
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
            max_tokens: None,
        }
    }

    /// A trigger context with max_tokens set.
    pub fn with_max_tokens(
        npc_id: &str,
        trigger_name: &str,
        narration_prompt: &str,
        max_tokens: u32,
    ) -> StoredTriggerContext {
        StoredTriggerContext {
            npc_id: npc_id.to_string(),
            trigger_idx: 0,
            trigger_name: trigger_name.to_string(),
            trigger_repeat: false,
            trigger_narration_prompt: narration_prompt.to_string(),
            system_prompt: "system prompt text".to_string(),
            user_prompt: "user prompt text".to_string(),
            max_tokens: Some(max_tokens),
        }
    }
}

// ─── PromptPreset ────────────────────────────────────────────────────────────

pub struct TestPromptPreset;

impl TestPromptPreset {
    /// A system preset with the given id and name.
    pub fn system(id: &str, name: &str) -> PromptPreset {
        PromptPreset {
            id: id.to_string(),
            name: name.to_string(),
            role: None,
            instructions: Some(format!("{name}.")),
            writing_style: None,
            output_format: None,
            is_default: false,
            preset_type: PresetType::System,
        }
    }

    /// A system preset marked as default.
    pub fn system_default(id: &str, name: &str) -> PromptPreset {
        PromptPreset {
            id: id.to_string(),
            name: name.to_string(),
            role: None,
            instructions: Some(format!("{name}.")),
            writing_style: None,
            output_format: None,
            is_default: true,
            preset_type: PresetType::System,
        }
    }
}

// ─── WorldManifest ───────────────────────────────────────────────────────────

pub struct TestWorldManifest;

impl TestWorldManifest {
    /// A minimal world manifest for bootstrap validation tests.
    pub fn minimal() -> WorldManifest {
        WorldManifest {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "A test world".to_string(),
            global_rules: vec![],
            starting_room_id: "room_a".to_string(),
            map_file: "map.json".to_string(),
            player_file: "player.json".to_string(),
            characters_dir: "".to_string(),
            scenarios: vec![],
            default_scenario_id: None,
            default_room_image: None,
        }
    }
}

// ─── CharacterSheet ──────────────────────────────────────────────────────────

pub struct TestCharacterSheet;

impl TestCharacterSheet {
    /// A standard hero character sheet used in bootstrap tests.
    pub fn hero() -> CharacterSheet {
        CharacterSheet {
            name: "Hero".to_string(),
            description: "A hero".to_string(),
            personality: "Brave".to_string(),
            scenario: "Default".to_string(),
            example_dialogue: "".to_string(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        }
    }
}
