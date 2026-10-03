//! HTTP module unit tests.

use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;

use crate::domain::model::settings::AppSettings;
use crate::adapters::driving::http::ServerConfig;
use crate::application::pipeline::ActionPipeline;

#[test]
fn test_server_config_default() {
    let config = ServerConfig::default();
    assert_eq!(config.host, IpAddr::V4(Ipv4Addr::UNSPECIFIED));
    assert_eq!(config.port, 3_000);
}

#[test]
fn test_server_config_custom_port() {
    let config = ServerConfig {
        host: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port: 80_80,
    };
    assert_eq!(config.port, 80_80);
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
        host: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port: 5000,
    };
    let cloned = config.clone();
    assert_eq!(config.port, cloned.port);
}

#[test]
fn test_server_config_debug() {
    let config = ServerConfig {
        host: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port: 3000,
    };
    let debug_str = format!("{config:?}");
    assert!(debug_str.contains("3000"));
}

#[test]
fn test_server_config_min_port() {
    let config = ServerConfig {
        host: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port: 1,
    };
    assert_eq!(config.port, 1);
}

#[test]
fn test_server_config_max_port() {
    let config = ServerConfig {
        host: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port: 65535,
    };
    assert_eq!(config.port, 65535);
}

#[test]
fn test_app_state_struct_fields() {
    let wired = crate::bootstrap::wiring::build_app_graph_for_tests(
        Arc::new(crate::adapters::driven::storage::Storage::new_in_memory()),
        None,
    )
    .expect("build_app_graph_for_tests should succeed");

    let _app_state = wired.pipeline;
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
