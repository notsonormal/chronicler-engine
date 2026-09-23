//! Test-app builders for the http test binary: narrator-wired AppState + router bundles.

use std::sync::Arc;

use chronicler_engine::adapters::driven::llm::providers::MockBackend;
use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::adapters::driving::http::AppState;
use chronicler_engine::application::agents::registry::AgentRegistry;
use chronicler_engine::application::ports::llm_provider::LlmProvider;
use chronicler_engine::domain::model::settings::AppSettings;
use chronicler_engine::test_support::{
    make_test_pipeline_with_backends, make_test_recorder_with_storage, TestAppBuilder,
};

/// Build an app with a custom narrator backend (default quantifier).
pub fn app_with_narrator(narrator: Arc<MockBackend>) -> (axum::Router, AppState) {
    app_with_narrator_and_settings(narrator, AppSettings::default())
}

/// Core narrator-wired test app: fresh in-memory storage, a storage-backed
/// narrator recorder, the given agent registry and settings. Returns
/// `(router, state, storage)` so tests can assert on recorded prompts and
/// stored state.
pub fn app_with_narrator_and_registry(
    narrator: Arc<MockBackend>,
    registry: AgentRegistry,
    settings: AppSettings,
) -> (axum::Router, AppState, Arc<Storage>) {
    let storage = Arc::new(Storage::new_in_memory());
    let recorder =
        make_test_recorder_with_storage(narrator as Arc<dyn LlmProvider>, Arc::clone(&storage));
    let pipeline = make_test_pipeline_with_backends(Arc::clone(&storage), recorder, registry);
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .pipeline(pipeline)
        .settings(settings)
        .build_with_state();
    (app, state, storage)
}

/// Build an app with a custom narrator backend and explicit settings (default quantifier).
pub fn app_with_narrator_and_settings(
    narrator: Arc<MockBackend>,
    settings: AppSettings,
) -> (axum::Router, AppState) {
    let (app, state, _storage) =
        app_with_narrator_and_registry(narrator, AgentRegistry::default(), settings);
    (app, state)
}
