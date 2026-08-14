//! HTTP module unit tests.

use std::sync::Arc;

use crate::domain::model::settings::AppSettings;
use crate::adapters::driving::http::ServerConfig;
use crate::application::pipeline::ActionPipeline;

#[test]
fn test_server_config_default() {
    let config = ServerConfig::default();
    assert_eq!(config.port, 3_000);
    assert_eq!(config.bind_attempts, None);
}

#[test]
fn test_server_config_custom_port() {
    let config = ServerConfig {
        port: 80_80,
        bind_attempts: Some(3),
    };
    assert_eq!(config.port, 80_80);
    assert_eq!(config.bind_attempts, Some(3));
}

#[test]
fn test_server_config_default_is_consistent() {
    let config1 = ServerConfig::default();
    let config2 = ServerConfig::default();
    assert_eq!(config1.port, config2.port);
}

#[test]
fn test_server_config_clone() {
    let config = ServerConfig {
        port: 5000,
        bind_attempts: Some(2),
    };
    let cloned = config.clone();
    assert_eq!(config.port, cloned.port);
    assert_eq!(config.bind_attempts, cloned.bind_attempts);
}

#[test]
fn test_server_config_debug() {
    let config = ServerConfig {
        port: 3000,
        bind_attempts: None,
    };
    let debug_str = format!("{config:?}");
    assert!(debug_str.contains("3000"));
}

#[test]
fn test_server_config_min_port() {
    let config = ServerConfig {
        port: 1,
        bind_attempts: Some(1),
    };
    assert_eq!(config.port, 1);
}

#[test]
fn test_server_config_max_port() {
    let config = ServerConfig {
        port: 65535,
        bind_attempts: None,
    };
    assert_eq!(config.port, 65535);
}

#[test]
fn test_app_state_struct_fields() {
    let settings = Arc::new(std::sync::RwLock::new(AppSettings::default()));
    let wired = crate::bootstrap::wiring::build_app_graph_for_tests(
        Arc::clone(&settings),
        Arc::new(crate::adapters::driven::storage::Storage::new_in_memory()),
        None,
    )
    .expect("build_app_graph_for_tests should succeed");

    let _app_state = (wired.pipeline, settings);
}

#[test]
fn test_pipeline_trait_bounds() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ActionPipeline>();
}

#[test]
fn test_app_settings_default() {
    let settings = AppSettings::default();
    let narrator = settings
        .get_narration_connection()
        .expect("narrator exists");
    assert!(narrator.model.contains("gpt-4o-mini") || narrator.model.is_empty());
}
