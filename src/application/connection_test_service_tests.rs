//! Tests for `ConnectionTestService`.

use std::sync::Arc;

use crate::adapters::driven::llm::providers::MockBackend;
use crate::adapters::driven::storage::Storage;
use crate::application::connection_test_service::{
    ConnectionTestService, ConnectionTestResult, ProviderFactory,
};
use crate::application::errors::ApplicationError;
use crate::application::ports::llm_provider::LlmProvider;
use crate::application::settings_service::SettingsService;
use crate::domain::model::llm_backend::LlmBackendType;
use crate::domain::model::settings::{AppSettings, LlmProviderConfig};

fn mock_connection(id: &str, model: &str) -> LlmProviderConfig {
    LlmProviderConfig {
        id: id.to_string(),
        name: id.to_string(),
        provider: LlmBackendType::Mock,
        model: model.to_string(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    }
}

fn passing_factory() -> ProviderFactory {
    Arc::new(|_config: &LlmProviderConfig| Arc::new(MockBackend::new()) as Arc<dyn LlmProvider>)
}

fn failing_factory() -> ProviderFactory {
    Arc::new(|_config: &LlmProviderConfig| {
        Arc::new(MockBackend::new().with_fail()) as Arc<dyn LlmProvider>
    })
}

fn make_service(factory: ProviderFactory) -> (ConnectionTestService, Arc<Storage>) {
    let storage = Arc::new(Storage::new_in_memory());
    let settings = AppSettings {
        connections: vec![mock_connection("conn-alpha", "mock-model-a")],
        narration_connection_id: "conn-alpha".to_string(),
        quantifier_connection_id: "conn-alpha".to_string(),
        ..AppSettings::default()
    };
    storage
        .save_settings(&settings)
        .expect("save_settings should succeed");
    (
        ConnectionTestService::new(SettingsService::new(Arc::clone(&storage)), factory),
        storage,
    )
}

fn run_saved(service: &ConnectionTestService) -> Result<ConnectionTestResult, ApplicationError> {
    service.test_saved_connection("conn-alpha")
}

#[test]
fn test_saved_connection_reports_the_provider_result() {
    let (service, storage) = make_service(passing_factory());

    let result = run_saved(&service).expect("a mock connection must pass");

    assert_eq!(result.backend_name, "Mock");
    assert_eq!(result.model_name, "mock");
    assert!(
        storage
            .list_latest_llm_messages(10)
            .expect("llm message read should succeed")
            .is_empty(),
        "a connection test must not write a forensic row"
    );
}

#[test]
fn test_saved_connection_loads_the_provider_from_its_config() {
    let (service, _storage) = make_service(Arc::new(|config: &LlmProviderConfig| {
        assert_eq!(config.model, "mock-model-a");
        Arc::new(MockBackend::new().with_model("built-from-config")) as Arc<dyn LlmProvider>
    }));

    let result = run_saved(&service).expect("the factory-built provider must be used");

    assert_eq!(result.model_name, "built-from-config");
}

#[test]
fn test_unknown_connection_is_refused() {
    let (service, _storage) = make_service(passing_factory());

    let error = service
        .test_saved_connection("missing")
        .expect_err("an unknown connection must be refused");

    assert!(
        matches!(error, ApplicationError::Validation(_)),
        "got {error:?}"
    );
}

#[test]
fn test_provider_failure_propagates_and_records_nothing() {
    let (service, storage) = make_service(failing_factory());

    let error = run_saved(&service).expect_err("a failing provider must surface its error");

    assert!(
        matches!(error, ApplicationError::Engine(_)),
        "got {error:?}"
    );
    assert!(
        storage
            .list_latest_llm_messages(10)
            .expect("llm message read should succeed")
            .is_empty(),
        "a failed connection test must not write a forensic row"
    );
}
