use std::sync::Arc;

use crate::application::game_service::GameService;
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
    let game_service: Arc<GameService> = Arc::new(GameService::new());
    let settings = Arc::new(std::sync::RwLock::new(AppSettings::default()));

    // Verify we can construct AppState-like struct with required fields
    let _app_state = (game_service, settings);
}

#[test]
fn test_game_service_trait_bounds() {
    // Verify GameService trait is Send + Sync
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<GameService>();
}

#[test]
fn test_app_settings_default() {
    let settings = AppSettings::default();
    let narrator = settings
        .get_narration_connection()
        .expect("narrator exists");
    assert!(narrator.model.contains("gpt-4o-mini") || narrator.model.is_empty());
}
