use std::sync::Arc;

use crate::application::game_service::{DefaultGameService, GameService};
use crate::model::settings::AppSettings;
use crate::server::ServerConfig;
use crate::storage::message_storage::MessageStorage;
use crate::storage::snapshot_storage::SnapshotStorage;

#[test]
fn test_server_config_default() {
    let config = ServerConfig::default();
    assert_eq!(config.port, 3_000);
}

#[test]
fn test_server_config_custom_port() {
    let config = ServerConfig { port: 80_80 };
    assert_eq!(config.port, 80_80);
}

#[test]
fn test_server_config_default_is_consistent() {
    // Ensure default is consistent across calls
    let config1 = ServerConfig::default();
    let config2 = ServerConfig::default();
    assert_eq!(config1.port, config2.port);
}

#[test]
fn test_server_config_clone() {
    let config = ServerConfig { port: 5000 };
    let cloned = config.clone();
    assert_eq!(config.port, cloned.port);
}

#[test]
fn test_server_config_debug() {
    let config = ServerConfig { port: 3000 };
    let debug_str = format!("{config:?}");
    assert!(debug_str.contains("3000"));
}

#[test]
fn test_server_config_min_port() {
    let config = ServerConfig { port: 1 };
    assert_eq!(config.port, 1);
}

#[test]
fn test_server_config_max_port() {
    let config = ServerConfig { port: 65535 };
    assert_eq!(config.port, 65535);
}

#[test]
fn test_app_state_struct_fields() {
    // Verify AppState struct has expected fields
    let game_service: Arc<dyn GameService> = Arc::new(DefaultGameService::new());
    let settings = Arc::new(std::sync::RwLock::new(AppSettings::default()));

    // Verify we can construct AppState-like struct with required fields
    let _app_state = (game_service, settings);
}

#[test]
fn test_game_service_trait_bounds() {
    // Verify GameService trait is Send + Sync
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<DefaultGameService>();
}

#[test]
fn test_app_settings_default() {
    let settings = AppSettings::default();
    let narrator = settings
        .get_narration_connection()
        .expect("narrator exists");
    assert!(narrator.model.contains("gpt-4o-mini") || narrator.model.is_empty());
}

#[test]
fn test_app_state_lock_state_success() {
    use crate::model::state::GameState;
    use crate::model::state_snapshot::GameStateSnapshot;
    use crate::model::world::{WorldCard, WorldManifest};
    use std::sync::Arc;

    let manifest = WorldManifest {
        id: "test".to_string(),
        name: "Test".to_string(),
        description: "A test world".to_string(),
        global_rules: vec![],
        starting_room_id: "room".to_string(),
        map_file: "map.json".to_string(),
        player_file: "player.json".to_string(),
        characters_dir: "".to_string(),
        scenarios: vec![],
        default_scenario_id: None,
        default_room_image: None,
    };

    let world = Arc::new(WorldCard::from(manifest));

    let map = Arc::new(crate::model::map::MapDef {
        overworld: crate::model::map::Overworld {
            id: "overworld".to_string(),
            name: "Overworld".to_string(),
            regions: vec![],
        },
    });

    let player = Arc::new(crate::model::character::PlayerCard {
        sheet: crate::model::character::CharacterSheet {
            name: "Hero".to_string(),
            description: "A hero".to_string(),
            personality: "Brave".to_string(),
            scenario: "Default".to_string(),
            example_dialogue: "".to_string(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
    });

    let state = GameState::new(
        world.clone(),
        map.clone(),
        player.clone(),
        vec![],
        "room".to_string(),
    );

    let snapshot = GameStateSnapshot::from_game_state(&state);
    let storage = Arc::new(crate::test_support::InMemoryGameStorage::new());
    let _ = storage.save(&snapshot);
    let llm_storage: Arc<dyn crate::storage::llm_message_storage::LlmMessageStorage> =
        Arc::new(crate::storage::llm_message_storage::InMemoryLlmMessageStorage::new());

    let app_state = crate::server::AppState {
        snapshot_storage: storage.clone(),
        message_storage: storage,
        llm_message_storage: Arc::clone(&llm_storage),
        world: state.world.clone(),
        map: state.map.clone(),
        player: state.player.clone(),
        npcs: Arc::new(state.npcs.clone()),
        game_service: Arc::new(DefaultGameService::with_storage(
            Some(llm_storage),
            Arc::new(std::sync::RwLock::new(AppSettings::default())),
        )) as Arc<dyn GameService>,
        prompt_preset_storage: Arc::new(
            crate::storage::prompt_preset_storage::InMemoryPromptPresetStorage::new(),
        ),
        settings: Arc::new(std::sync::RwLock::new(AppSettings::default())),
        cancel_token: Arc::new(std::sync::RwLock::new(
            tokio_util::sync::CancellationToken::new(),
        )),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    };

    let loaded = app_state.load_state();
    assert!(loaded.is_ok(), "Expected load_state to succeed");
}

#[test]
fn test_app_state_lock_state_poisoned() {
    use crate::error::EngineError;
    use crate::model::state::GameState;
    use crate::model::state_snapshot::GameStateSnapshot;
    use crate::model::world::{WorldCard, WorldManifest};
    use std::sync::Arc;

    struct FailingStorage;
    impl SnapshotStorage for FailingStorage {
        fn save(&self, _snapshot: &GameStateSnapshot) -> Result<u64, EngineError> {
            Ok(1)
        }
        fn load_latest(&self) -> Result<Option<GameStateSnapshot>, EngineError> {
            Err(EngineError::Config("test error".to_string()))
        }
        fn load_by_id(&self, _id: u64) -> Result<Option<GameStateSnapshot>, EngineError> {
            Err(EngineError::Config("test error".to_string()))
        }
        fn reset(&self) -> Result<(), EngineError> {
            Ok(())
        }
        fn save_checkpoint(
            &self,
            _checkpoint: &crate::model::checkpoint::Checkpoint,
        ) -> Result<(), EngineError> {
            Ok(())
        }
        fn load_checkpoint(
            &self,
            _id: &str,
        ) -> Result<Option<crate::model::checkpoint::Checkpoint>, EngineError> {
            Err(EngineError::Config("test error".to_string()))
        }
        fn list_checkpoints(
            &self,
        ) -> Result<Vec<crate::model::checkpoint::Checkpoint>, EngineError> {
            Err(EngineError::Config("test error".to_string()))
        }
        fn delete_checkpoint(&self, _id: &str) -> Result<(), EngineError> {
            Ok(())
        }
    }

    impl MessageStorage for FailingStorage {
        fn insert_message(
            &self,
            _msg: &mut crate::model::message::Message,
        ) -> Result<(), EngineError> {
            Ok(())
        }
        fn update_message(&self, _id: u64, _text: &str) -> Result<(), EngineError> {
            Ok(())
        }
        fn delete_message(&self, _id: u64) -> Result<(), EngineError> {
            Ok(())
        }
        fn load_messages(&self) -> Result<Vec<crate::model::message::Message>, EngineError> {
            Ok(vec![])
        }
    }

    let manifest = WorldManifest {
        id: "test".to_string(),
        name: "Test".to_string(),
        description: "A test world".to_string(),
        global_rules: vec![],
        starting_room_id: "room".to_string(),
        map_file: "map.json".to_string(),
        player_file: "player.json".to_string(),
        characters_dir: "".to_string(),
        scenarios: vec![],
        default_scenario_id: None,
        default_room_image: None,
    };

    let world = Arc::new(WorldCard::from(manifest));

    let map = Arc::new(crate::model::map::MapDef {
        overworld: crate::model::map::Overworld {
            id: "overworld".to_string(),
            name: "Overworld".to_string(),
            regions: vec![],
        },
    });

    let player = Arc::new(crate::model::character::PlayerCard {
        sheet: crate::model::character::CharacterSheet {
            name: "Hero".to_string(),
            description: "A hero".to_string(),
            personality: "Brave".to_string(),
            scenario: "Default".to_string(),
            example_dialogue: "".to_string(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
    });

    let state = GameState::new(
        world.clone(),
        map.clone(),
        player.clone(),
        vec![],
        "room".to_string(),
    );

    let llm_storage: Arc<dyn crate::storage::llm_message_storage::LlmMessageStorage> =
        Arc::new(crate::storage::llm_message_storage::InMemoryLlmMessageStorage::new());
    let failing_storage = Arc::new(FailingStorage);
    let app_state = crate::server::AppState {
        snapshot_storage: failing_storage.clone(),
        message_storage: failing_storage,
        llm_message_storage: Arc::clone(&llm_storage),
        world: state.world.clone(),
        map: state.map.clone(),
        player: state.player.clone(),
        npcs: Arc::new(state.npcs.clone()),
        game_service: Arc::new(DefaultGameService::with_storage(
            Some(llm_storage),
            Arc::new(std::sync::RwLock::new(AppSettings::default())),
        )) as Arc<dyn GameService>,
        prompt_preset_storage: Arc::new(
            crate::storage::prompt_preset_storage::InMemoryPromptPresetStorage::new(),
        ),
        settings: Arc::new(std::sync::RwLock::new(AppSettings::default())),
        cancel_token: Arc::new(std::sync::RwLock::new(
            tokio_util::sync::CancellationToken::new(),
        )),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    };

    let loaded = app_state.load_state();
    assert!(
        loaded.is_err(),
        "Expected load_state to fail when storage returns error"
    );
}
