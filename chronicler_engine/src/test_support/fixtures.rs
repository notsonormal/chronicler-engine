//! [DOC: docs/reference/test_support.md — section "Fixtures"]
//!
//! Test fixtures shared between unit and integration tests.
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::model::character::{CharacterSheet, NpcCard, PlayerCard};
use crate::domain::model::map::{MapDef, Overworld, Region, Room};
use crate::domain::model::message::{Message, Swipe};
use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::domain::model::state::game_state::{GameState, GameStateBuilder};
use crate::domain::model::state::message_types::MessageType;
use crate::domain::model::state::trigger_context::StoredTriggerContext;
use crate::domain::model::trigger::{ComparisonOperator, Trigger, TriggerNarration, TriggerRequirement};
use crate::domain::model::world::{WorldCard, WorldManifest};
use crate::adapters::driven::storage::Storage;
use crate::adapters::driven::storage::db::DbPool;

pub struct TestWorld;

impl TestWorld {
    pub fn minimal() -> WorldCard {
        WorldCard {
            key: "test".to_string(),
            name: "Test World".to_string(),
            description: "A test world.".to_string(),
            ..Default::default()
        }
    }

    pub fn with_rule(rule: &str) -> WorldCard {
        WorldCard {
            global_rules: vec![rule.to_string()],
            ..Self::minimal()
        }
    }
}

pub struct TestPlayer;

impl TestPlayer {
    pub fn named(name: &str) -> PlayerCard {
        PlayerCard {
            key: name.to_lowercase().replace(' ', "-"),
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

    pub fn standard() -> PlayerCard {
        Self::named("Hero")
    }
}

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

    pub fn named(id: &str, name: &str) -> NpcCard {
        NpcCard {
            id: id.to_string(),
            sheet: Self::sheet(name),
            inventory: vec![],
            triggers: vec![],
            relationships: vec![],
        }
    }

    pub fn with_times_met_trigger(id: &str, name: &str, op: ComparisonOperator, n: u32) -> NpcCard {
        NpcCard {
            id: id.to_string(),
            sheet: Self::sheet(name),
            inventory: vec![],
            triggers: vec![Trigger {
                requirement: TriggerRequirement {
                    operator: op,
                    threshold: n,
                },
                narration: TriggerNarration {
                    name: format!("{name} Introduction"),
                    narration_prompt: format!("{name} introduces themselves."),
                },
                repeat: false,
                room_id: None,
            }],
            relationships: vec![],
        }
    }

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
                requirement: TriggerRequirement {
                    operator: op,
                    threshold: n,
                },
                narration: TriggerNarration {
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

pub struct TestMap;

impl TestMap {
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

    pub fn two_rooms(room_a_id: &str, room_b_id: &str) -> MapDef {
        use crate::domain::model::map::Direction;
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

pub struct TestGameState;

impl TestGameState {
    pub fn in_room(room_id: &str) -> GameState {
        GameState::new(
            Arc::new(TestWorld::minimal()),
            Arc::new(TestMap::single_room(room_id)),
            Arc::new(TestPlayer::standard()),
            vec![],
            room_id.to_string(),
        )
    }

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

    pub fn with_npc_raw(room_id: &str, npc: NpcCard) -> GameState {
        GameStateBuilder::new(
            Arc::new(TestWorld::minimal()),
            Arc::new(TestMap::single_room(room_id)),
            Arc::new(TestPlayer::standard()),
            room_id,
        )
        .with_npcs(vec![npc])
        .build()
    }

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
        GameStateBuilder::new(
            Arc::new(TestWorld::minimal()),
            Arc::new(map),
            Arc::new(TestPlayer::standard()),
            room_id,
        )
        .with_npcs(vec![npc])
        .build()
    }
}

pub struct TestStoredTriggerContext;

impl TestStoredTriggerContext {
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

pub struct TestPromptPreset;

impl TestPromptPreset {
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

    pub fn system_default_with_instructions(
        id: &str,
        name: &str,
        instructions: &str,
    ) -> PromptPreset {
        let mut p = Self::system_default(id, name);
        p.instructions = Some(instructions.to_string());
        p
    }
}

pub struct TestWorldManifest;

impl TestWorldManifest {
    pub fn minimal() -> WorldManifest {
        WorldManifest {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "A test world".to_string(),
            global_rules: vec![],
            map_file: "map.json".to_string(),
            characters_dir: "".to_string(),
            scenarios: vec![],
            default_scenario_id: None,
            default_room_image: None,
        }
    }
}

pub struct TestCharacterSheet;

impl TestCharacterSheet {
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

pub fn seed_default_game_row(
    pool: &crate::adapters::driven::storage::db::DbPool,
    game_id: u64,
) -> Result<(), crate::error::EngineError> {
    let conn = pool.conn();
    conn.execute(
        "INSERT INTO games (id, world_name, world_key, persona_key, persona_name, name, created_at, updated_at)
         VALUES (?1, 'test', 'test', 'test_player', 'Test Player', 'Test Game', ?2, ?2)",
        rusqlite::params![game_id as i64, chrono::Utc::now().to_rfc3339()],
    )
    .map_err(|e| crate::error::EngineError::Config(format!("seed_default_game_row: {e}")))?;
    Ok(())
}

pub fn sqlite_storage() -> Result<Storage, crate::error::EngineError> {
    let pool = DbPool::new(":memory:")?;
    seed_default_game_row(&pool, 1)?;
    Ok(Storage::new_sqlite(pool, 1))
}

pub fn dummy_message(text: &str) -> Message {
    Message::new(
        Some("Player".to_string()),
        text,
        MessageType::Input,
        None,
        None,
    )
}

pub fn dummy_swipe(text: &str) -> Swipe {
    Swipe {
        text: text.to_string(),
        snapshot_id: None,
        location_header: None,
        event_header: None,
    }
}
