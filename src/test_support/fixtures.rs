//! Test fixtures shared between unit and integration tests.
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::model::character::{CharacterSheet, NpcCard, PersonaCard};
use crate::domain::model::map::{Direction, MapDef, Overworld, Region, Room};
use crate::domain::model::message::{Message, Swipe};
use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::message_types::MessageType;
use crate::domain::model::state::trigger_context::StoredTriggerContext;
use crate::domain::model::trigger::{ComparisonOperator, Trigger, TriggerNarration, TriggerRequirement};
use crate::domain::model::scenario::StartingScenario;
use crate::domain::model::world::{WorldCard, WorldManifest};
use crate::adapters::driven::storage::Storage;
use crate::adapters::driven::storage::db::DbPool;
use crate::application::llm_message::SaveLlmMessageFn;
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
        GameState::new(room_id)
    }
}

pub struct TestStoredTriggerContext;

impl TestStoredTriggerContext {
    pub fn standard() -> StoredTriggerContext {
        StoredTriggerContext {
            npc_id: "npc_1".to_string(),
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

/// Builds a `GameState` with only a starting room.
pub fn create_minimal_test_state() -> GameState {
    GameState::new("room1".to_string())
}

/// Builds a minimal `WorldCard` for tests.
pub fn create_test_world() -> WorldCard {
    WorldCard {
        key: "test".to_string(),
        name: "Test Realm".to_string(),
        description: "A small testing kingdom".to_string(),
        ..Default::default()
    }
}

/// Builds a standard `PersonaCard` for tests.
pub fn create_test_player() -> PersonaCard {
    PersonaCard {
        key: "test_player".to_string(),
        sheet: CharacterSheet {
            name: "Test Player".to_string(),
            description: "A brave adventurer".to_string(),
            personality: "Brave and curious".to_string(),
            scenario: "Exploring the test realm".to_string(),
            example_dialogue: "Hello, world!".to_string(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
    }
}

/// Builds a three-room `MapDef` with deterministic exits for tests.
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

/// Builds a single test NPC.
pub fn create_test_npcs() -> Vec<NpcCard> {
    vec![NpcCard {
        id: "test_npc".to_string(),
        sheet: CharacterSheet {
            name: "Innkeeper".to_string(),
            description: "A friendly innkeeper".to_string(),
            personality: "Helpful and cheerful".to_string(),
            scenario: "Runs the local tavern".to_string(),
            example_dialogue: "Welcome, traveler!".to_string(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
        triggers: vec![],
        relationships: vec![],
    }]
}

/// Builds a `GameState` with selected NPCs in the starting area.
pub fn create_test_state_with_npcs(room_npcs: Vec<String>, npcs: Vec<NpcCard>) -> GameState {
    let _world = Arc::new(WorldCard {
        key: "test".into(),
        name: "Test World".into(),
        description: "A test world".into(),
        scenarios: vec![StartingScenario {
            id: "default".into(),
            name: "Default".into(),
            description: "Test scenario".into(),
            starting_room_id: "room1".into(),
            text: String::new(),
            npcs: vec![],
        }],
        default_scenario_id: Some("default".into()),
        ..Default::default()
    });

    let room1 = Room {
        id: "room1".into(),
        name: "Test Tavern".into(),
        description: "A cozy tavern with wooden beams and warm fire.".into(),
        exits: HashMap::new(),
        items: vec![],
        image_path: None,
        navigation_description: None,
    };

    let region = Region {
        id: "test_region".into(),
        name: "Test Region".into(),
        rooms: vec![room1],
    };

    let _map = Arc::new(MapDef {
        overworld: Overworld {
            id: "test_overworld".into(),
            name: "Test World".into(),
            regions: vec![region],
        },
    });

    let _player = Arc::new(PersonaCard {
        key: "test_player".to_string(),
        sheet: CharacterSheet {
            name: "Test Player".into(),
            description: "A test player".into(),
            personality: "Brave".into(),
            scenario: "Test scenario".into(),
            example_dialogue: "Hello!".into(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
    });

    let mut state = GameState::new("room1");
    for npc in npcs.iter().filter(|n| room_npcs.contains(&n.id)) {
        state.scene.npcs_in_area.push(npc.clone());
    }
    state
}

/// Builds a `GameState` seeded with the default test NPC.
pub fn create_test_state() -> GameState {
    create_test_state_with_npcs(vec!["test_npc".to_string()], create_test_npcs())
}

/// Builds a `WorldCard` with a default scenario for tests.
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

/// Builds the shopkeeper NPC used by mutation-order tests.
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

/// Converts a vector of NPCs into an id-keyed map.
pub fn npc_map(npcs: Vec<NpcCard>) -> HashMap<String, NpcCard> {
    npcs.into_iter().map(|npc| (npc.id.clone(), npc)).collect()
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
        replay: None,
    }
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

/// The swipe inherits the message's text, snapshot_id, location_header, and
/// event_header so the row matches what production code writes.
pub fn insert_message_with_swipe(
    storage: &Storage,
    msg: &Message,
) -> Result<(), crate::error::EngineError> {
    let id = storage.insert_message(msg)?;
    if let Some(swipe) = msg.swipes.first() {
        let mut swipe = swipe.clone();
        swipe.text = msg.text().to_string();
        swipe.snapshot_id = msg.snapshot_id();
        swipe.location_header = msg.location_header().map(|s| s.to_string());
        swipe.event_header = msg.event_header().map(|s| s.to_string());
        let _ = storage.insert_swipe(id, &swipe, 0);
    }
    Ok(())
}

/// Event retry flow: Input → Main narration (with `last_trigger`) →
/// Event narration (with `event_header`). Each message gets its own snapshot.
pub fn seed_event_flow(
    state: &crate::adapters::driving::http::AppState,
    storage: &Storage,
) -> Result<(), crate::error::EngineError> {
    use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;

    let input_snap_id = {
        let mut gs = state.message_service.load_or_fresh();
        gs.add_message(
            "look".to_string(),
            Some("Player".to_string()),
            MessageType::Input,
        );
        let snap = GameStateSnapshot::from_game_state(&gs);
        let id = storage.save_snapshot(&snap)?;
        if let Some(last) = gs.narrative.history.last_mut() {
            last.set_snapshot_id(Some(id));
            insert_message_with_swipe(storage, last)?;
        }
        id
    };
    let _ = input_snap_id;

    let _pre_event_id = {
        let mut gs = state.message_service.load_or_fresh();
        gs.narrative.last_trigger = Some(TestStoredTriggerContext::standard());
        gs.add_message("Main narration".to_string(), None, MessageType::Narration);
        let snap = GameStateSnapshot::from_game_state(&gs);
        let id = storage.save_snapshot(&snap)?;
        if let Some(last) = gs.narrative.history.last_mut() {
            last.set_snapshot_id(Some(id));
            insert_message_with_swipe(storage, last)?;
        }
        id
    };

    let _final_id = {
        let mut gs = state.message_service.load_or_fresh();
        gs.narrative.pending_event = Some("Event".to_string());
        gs.add_message("Event narration".to_string(), None, MessageType::Narration);
        if let Some(last) = gs.narrative.history.last_mut() {
            last.set_event_header(Some("Event".to_string()));
        }
        let snap = GameStateSnapshot::from_game_state(&gs);
        let id = storage.save_snapshot(&snap)?;
        if let Some(last) = gs.narrative.history.last_mut() {
            last.set_snapshot_id(Some(id));
            insert_message_with_swipe(storage, last)?;
        }
        id
    };

    // Persist last_trigger on the latest snapshot so retry's `from_snapshot` sees it.
    let mut gs = state.message_service.load_or_fresh();
    gs.narrative.last_trigger = Some(TestStoredTriggerContext::standard());
    let _ = state.message_service.save_state(&gs);
    Ok(())
}
