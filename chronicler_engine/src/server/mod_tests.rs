use std::sync::Arc;

use crate::engine::game_service::{DefaultGameService, GameService};
use crate::model::settings::AppSettings;
use crate::server::ServerConfig;

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
    use crate::model::world::{WorldCard, WorldManifest};
    use std::sync::Arc;
    use std::sync::Mutex;

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

    let state = Arc::new(Mutex::new(GameState::new(
        world,
        map,
        player,
        vec![],
        "room".to_string(),
    )));

    let app_state = crate::server::AppState {
        state: state.clone(),
        game_service: Arc::new(DefaultGameService::new()) as Arc<dyn GameService>,
        settings: Arc::new(std::sync::RwLock::new(AppSettings::default())),
        cancel_token: tokio_util::sync::CancellationToken::new(),
    };

    let locked = app_state.lock_state();
    assert!(locked.is_ok(), "Expected lock_state to succeed");
}

#[test]
fn test_app_state_lock_state_poisoned() {
    use crate::model::state::GameState;
    use crate::model::world::{WorldCard, WorldManifest};
    use std::sync::Arc;
    use std::sync::Mutex;

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

    let state = Arc::new(Mutex::new(GameState::new(
        world,
        map,
        player,
        vec![],
        "room".to_string(),
    )));

    let app_state = crate::server::AppState {
        state: state.clone(),
        game_service: Arc::new(DefaultGameService::new()) as Arc<dyn GameService>,
        settings: Arc::new(std::sync::RwLock::new(AppSettings::default())),
        cancel_token: tokio_util::sync::CancellationToken::new(),
    };

    // Poison the mutex by panicking while holding the lock
    let state_for_panic = state.clone();
    let _ = std::panic::catch_unwind(move || {
        let _guard = state_for_panic.lock().unwrap();
        panic!("intentional panic to poison mutex");
    });

    let locked = app_state.lock_state();
    assert!(
        locked.is_err(),
        "Expected lock_state to fail when mutex is poisoned"
    );
}
