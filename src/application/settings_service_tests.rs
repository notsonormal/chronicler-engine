//! Tests for `SettingsService`.

use std::sync::Arc;

use crate::adapters::driven::storage::Storage;
use crate::application::errors::ApplicationError;
use crate::application::settings_service::SettingsService;
use crate::domain::model::llm_backend::LlmBackendType;
use crate::domain::model::settings::{AppSettings, LlmProviderConfig};

fn mock_connection(id: &str, name: &str) -> LlmProviderConfig {
    LlmProviderConfig {
        id: id.to_string(),
        name: name.to_string(),
        provider: LlmBackendType::Mock,
        model: "mock-model".to_string(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    }
}

fn make_service() -> (SettingsService, Arc<Storage>) {
    let storage = Arc::new(Storage::new_in_memory());
    let settings = AppSettings {
        connections: vec![mock_connection("conn-alpha", "Alpha")],
        narration_connection_id: "conn-alpha".to_string(),
        quantifier_connection_id: "conn-alpha".to_string(),
        ..AppSettings::default()
    };
    storage
        .save_settings(&settings)
        .expect("save_settings should succeed");
    (SettingsService::new(Arc::clone(&storage)), storage)
}

#[test]
fn test_add_connection_refuses_duplicate_name() {
    let (service, _storage) = make_service();

    let error = service
        .add_connection(mock_connection("conn-beta", "Alpha"))
        .expect_err("a duplicate name must be refused");

    assert!(matches!(error, ApplicationError::Validation(_)));
}

#[test]
fn test_add_connection_refuses_case_and_space_variant() {
    let (service, _storage) = make_service();

    let error = service
        .add_connection(mock_connection("conn-beta", "  alpha  "))
        .expect_err("a name differing only in case and space must be refused");

    assert!(matches!(error, ApplicationError::Validation(_)));
}

#[test]
fn test_add_connection_allows_distinct_name() {
    let (service, storage) = make_service();

    let updated = service
        .add_connection(mock_connection("conn-beta", "Beta"))
        .expect("a distinct name must be accepted");

    assert_eq!(updated.connections.len(), 2);
    assert_eq!(storage.get_settings().unwrap().connections.len(), 2);
}

#[test]
fn test_update_connection_keeps_own_name() {
    let (service, _storage) = make_service();

    let (updated, is_narrator, is_quantifier) = service
        .update_connection("conn-alpha", |connection| {
            connection.name = "Alpha".to_string();
            connection.model = "changed-model".to_string();
        })
        .expect("keeping the connection's own name must succeed");

    assert_eq!(updated.name, "Alpha");
    assert_eq!(updated.model, "changed-model");
    assert!(is_narrator);
    assert!(is_quantifier);
}

#[test]
fn test_update_connection_refuses_another_connections_name() {
    let (service, storage) = make_service();
    service
        .add_connection(mock_connection("conn-beta", "Beta"))
        .expect("a distinct name must be accepted");

    let error = service
        .update_connection("conn-alpha", |connection| {
            connection.name = "Beta".to_string();
        })
        .expect_err("renaming onto another connection's name must be refused");

    assert!(matches!(error, ApplicationError::Validation(_)));
    let stored = storage.get_settings().unwrap();
    assert_eq!(stored.find_connection("conn-alpha").unwrap().name, "Alpha");
}
