//! Test fixtures shared between unit and integration tests.
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::model::character::{CharacterSheet, NpcCard, PersonaCard};
use crate::domain::model::map::{Direction, MapDef, Overworld, Region, Room};
use crate::domain::model::game::GameId;
use crate::domain::model::message::{Maker, Message, Swipe};
use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::domain::model::settings::{NarrativePerspective, NarrativeTense, NarratorMode};
use crate::domain::model::utils::settings_defaults;
use crate::domain::model::state::swipe_state::SwipeState;
use crate::domain::model::state::message_types::{MessageKind, MessageType};
use crate::adapters::driven::storage::TxToken;
use crate::domain::model::trigger::{ComparisonOperator, Trigger, TriggerNarration, TriggerRequirement};
use crate::domain::model::scenario::StartingScenario;
use crate::domain::model::world::{WorldCard, WorldManifest};
use crate::adapters::driven::storage::{Backend, Storage};
use crate::adapters::driven::storage::db::DbPool;
use crate::adapters::driven::storage::transaction::Tx;
use crate::application::llm_recorder::SaveLlmMessageFn;
use crate::domain::model::llm_message::LlmMessage;
use crate::application::llm_recorder::LlmCallRecorder;
use crate::application::ports::llm_provider::LlmProvider;

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

pub struct TestPersona;

impl TestPersona {
    pub fn named(name: &str) -> PersonaCard {
        PersonaCard {
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

    pub fn standard() -> PersonaCard {
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

pub struct TestSwipeState;

impl TestSwipeState {
    pub fn in_room(room_id: &str) -> SwipeState {
        SwipeState::starting(None, &HashMap::new(), room_id)
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
            allowed_modes: settings_defaults::default_allowed_modes(),
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
            allowed_modes: vec![NarratorMode::Novel],
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

    pub fn options(id: &str, name: &str) -> PromptPreset {
        PromptPreset {
            id: id.to_string(),
            name: name.to_string(),
            role: None,
            instructions: Some(format!("{name}.")),
            writing_style: None,
            output_format: None,
            allowed_modes: settings_defaults::default_allowed_modes(),
            is_default: false,
            preset_type: PresetType::Options,
        }
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
            narrator_mode: crate::domain::model::settings::NarratorMode::Novel,
            narrative_perspective: crate::domain::model::settings::NarrativePerspective::Third,
            narrative_tense: crate::domain::model::settings::NarrativeTense::Past,
            options_always_on: false,
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

pub fn create_test_world() -> WorldCard {
    WorldCard {
        key: "test".to_string(),
        name: "Test Realm".to_string(),
        description: "A small testing kingdom".to_string(),
        ..Default::default()
    }
}

pub fn create_test_map() -> MapDef {
    let mut room1_exits = HashMap::new();
    room1_exits.insert(Direction::North, "room2".to_string());

    let mut room2_exits = HashMap::new();
    room2_exits.insert(Direction::South, "room1".to_string());
    room2_exits.insert(Direction::East, "room3".to_string());

    let room3_exits = HashMap::new();

    let room1 = Room {
        id: "room1".to_string(),
        name: "Test Tavern".to_string(),
        description: "A cozy tavern with wooden beams and warm fire.".to_string(),
        exits: room1_exits,
        items: vec![],
        image_path: None,
        navigation_description: None,
    };

    let room2 = Room {
        id: "room2".to_string(),
        name: "Village Square".to_string(),
        description: "A bustling village square with a fountain.".to_string(),
        exits: room2_exits,
        items: vec![],
        image_path: None,
        navigation_description: None,
    };

    let room3 = Room {
        id: "room3".to_string(),
        name: "Forest Path".to_string(),
        description: "A quiet path through the woods.".to_string(),
        exits: room3_exits,
        items: vec![],
        image_path: None,
        navigation_description: None,
    };

    let region = Region {
        id: "test_region".to_string(),
        name: "Test Region".to_string(),
        rooms: vec![room1, room2, room3],
    };

    let overworld = Overworld {
        id: "test_overworld".to_string(),
        name: "Test World".to_string(),
        regions: vec![region],
    };

    MapDef { overworld }
}

pub fn create_test_world_with_scenario() -> WorldCard {
    WorldCard {
        key: "test".to_string(),
        name: "Test Realm".to_string(),
        description: "A small testing kingdom".to_string(),
        scenarios: vec![StartingScenario {
            id: "test_scenario".to_string(),
            name: "Test Scenario".to_string(),
            description: "A test".to_string(),
            starting_room_id: "room1".to_string(),
            text: "You wake up in a cozy room.".to_string(),
            npcs: vec![],
        }],
        ..Default::default()
    }
}

pub fn shopkeeper_npc() -> NpcCard {
    NpcCard {
        id: "shopkeeper".into(),
        sheet: CharacterSheet {
            name: "Shopkeeper Sarah".into(),
            description: "A shrewd shopkeeper".into(),
            personality: "Business-minded".into(),
            scenario: "Runs the shop".into(),
            example_dialogue: "Welcome!".into(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
        triggers: vec![Trigger {
            requirement: TriggerRequirement {
                operator: ComparisonOperator::Eq,
                threshold: 0,
            },
            narration: TriggerNarration {
                name: "Greeting".into(),
                narration_prompt: "The shopkeeper greets you.".into(),
            },
            repeat: false,
            room_id: None,
        }],
        relationships: vec![],
    }
}

pub fn seed_default_game_row(
    pool: &crate::adapters::driven::storage::db::DbPool,
    game_id: u64,
) -> Result<(), crate::error::EngineError> {
    let conn = pool.conn();
    conn.execute(
        "INSERT INTO games (id, world_name, world_key, persona_key, persona_name, name, display_name, created_at, updated_at)
         VALUES (?1, 'test', 'test', 'test_player', 'Test Player', 'Test Game', 'Test Game', ?2, ?2)",
        rusqlite::params![game_id as i64, chrono::Utc::now().to_rfc3339()],
    )
    .map_err(|e| crate::error::EngineError::Config(format!("seed_default_game_row: {e}")))?;
    Ok(())
}

pub fn sqlite_storage() -> Result<Storage, crate::error::EngineError> {
    let pool = DbPool::new(":memory:")?;
    seed_default_game_row(&pool, 1)?;
    Ok(Storage::new_sqlite(pool))
}

/// The request a hand-made test Game row is written from: the defaults the
/// storage layer's own create gave a Game with no World to inherit from.
pub fn test_new_game(
    world_name: &str,
    world_key: &str,
    persona_key: &str,
    persona_name: &str,
    name: &str,
) -> crate::domain::model::game::NewGame {
    crate::domain::model::game::NewGame {
        world_name: world_name.to_string(),
        world_key: world_key.to_string(),
        persona_key: persona_key.to_string(),
        persona_name: persona_name.to_string(),
        name: name.to_string(),
        narrator_mode: NarratorMode::Novel,
        narrative_perspective: NarrativePerspective::Third,
        narrative_tense: NarrativeTense::Past,
        system_prompt_preset_id: "system_default".to_string(),
        quantifier_prompt_preset_id: "quantifier_default".to_string(),
        impersonate_prompt_preset_id: "impersonate_default".to_string(),
        options_prompt_preset_id: "options_default".to_string(),
        options_always_on: false,
    }
}

/// A Game row a test only needs to write into. The row lands whole and off any
/// unit, as `seed_default_game_row` does for the shared SQLite fixture: the
/// create unit's clauses are the seam contract's, and a seeded Game holds no
/// opening Narration.
pub fn seed_game_row(storage: &Storage, request: &crate::domain::model::game::NewGame) -> GameId {
    storage
        .with_backend_mut("seed_game_row", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                Ok(Tx::write_game_row(&conn, request)?)
            }
            Backend::InMemory(data) => {
                let id = data.next_game_id;
                data.next_game_id += 1;
                let now = chrono::Utc::now();
                data.games.push(crate::domain::model::game::Game {
                    id,
                    world_name: request.world_name.clone(),
                    world_key: request.world_key.clone(),
                    persona_key: request.persona_key.clone(),
                    persona_name: request.persona_name.clone(),
                    name: request.name.clone(),
                    display_name: request.display_name(),
                    created_at: now,
                    updated_at: now,
                    narrator_mode: request.narrator_mode,
                    narrative_perspective: request.narrative_perspective,
                    narrative_tense: request.narrative_tense,
                    active_system_prompt_preset_id: request.system_prompt_preset_id.clone(),
                    active_quantifier_prompt_preset_id: request.quantifier_prompt_preset_id.clone(),
                    active_impersonate_prompt_preset_id: request
                        .impersonate_prompt_preset_id
                        .clone(),
                    active_options_prompt_preset_id: request.options_prompt_preset_id.clone(),
                    options_always_on: request.options_always_on,
                    failure: None,
                });
                Ok(GameId(id))
            }
        })
        // arch-lint: allow(no-unwrap-expect) reason="test fixture panics when the seed write fails"
        .expect("test setup: the Game row must land")
}

pub fn empty_sqlite_storage() -> Storage {
    let pool = DbPool::new(":memory:")
        // arch-lint: allow(no-unwrap-expect) reason="test setup fixture panics on a storage failure"
        .expect("test setup: in-memory SQLite must open");
    Storage::new_sqlite(pool)
}

/// The token a storage test's unit needs: `TxToken`'s constructor is crate-only,
/// and the storage seam contract is a test crate.
pub fn tx_token() -> TxToken {
    TxToken::new()
}

pub fn dummy_message(text: &str) -> Message {
    Message::new(
        MessageType::Input,
        MessageKind::TypedInput,
        Swipe::new(text, Maker::Player, TestSwipeState::in_room("room1")),
    )
}

pub fn llm_message(
    agent: &str,
    model: &str,
    error: Option<&str>,
    created_offset_secs: i64,
) -> LlmMessage {
    LlmMessage {
        id: 0,
        agent_name: agent.to_string(),
        backend_name: "Mock".to_string(),
        model_name: model.to_string(),
        system_prompt: String::new(),
        user_prompt: String::new(),
        raw_request_json: String::new(),
        raw_response_json: String::new(),
        parsed_response: String::new(),
        error_message: error.map(str::to_string),
        created_at: chrono::Utc::now() + chrono::Duration::seconds(created_offset_secs),
    }
}

pub fn dummy_swipe(text: &str) -> Swipe {
    Swipe::new(text, Maker::Narrator, TestSwipeState::in_room("room1"))
}

pub fn make_noop_save_fn() -> SaveLlmMessageFn {
    Arc::new(|_message: &LlmMessage| Ok(()))
}

pub fn make_test_recorder(provider: Arc<dyn LlmProvider>) -> Arc<LlmCallRecorder> {
    Arc::new(LlmCallRecorder::new(provider, make_noop_save_fn()))
}

pub fn make_test_recorder_with_storage(
    provider: Arc<dyn LlmProvider>,
    storage: Arc<Storage>,
) -> Arc<LlmCallRecorder> {
    let save_fn: SaveLlmMessageFn =
        Arc::new(move |message: &LlmMessage| storage.save_llm_message(message));
    Arc::new(LlmCallRecorder::new(provider, save_fn))
}

/// Seeding a Game always points the app at it, so a missing pointer is a broken
/// fixture.
pub fn active_game_of(state: &crate::adapters::driving::http::AppState) -> GameId {
    state
        .game_catalogue
        .active_game_id()
        // arch-lint: allow(no-unwrap-expect) reason="test fixture panics on a broken fixture"
        .expect("test fixture: an active game")
}
