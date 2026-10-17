//! Test-app builders for the http test binary: scripted-model AppState + router bundles.

use std::sync::Arc;

use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::adapters::driving::http::AppState;
use chronicler_engine::adapters::driving::http::builders::router::build_router;
use chronicler_engine::application::agents::AgentRunner;
use chronicler_engine::application::llm_recorder::LlmCallRecorder;
use chronicler_engine::bootstrap::wiring::build_app_graph;
use chronicler_engine::domain::model::llm_backend::LlmBackendType;
use chronicler_engine::domain::model::settings::AppSettings;
use chronicler_engine::test_support::{
    configured_runner, make_test_pipeline_with_backends, make_test_recorder_with_storage,
    ScriptedModel, TestAppBuilder,
};

pub fn app_with_model(model: Arc<ScriptedModel>) -> (axum::Router, AppState) {
    app_with_model_and_settings(model, AppSettings::default())
}

pub fn app_with_model_and_settings(
    model: Arc<ScriptedModel>,
    settings: AppSettings,
) -> (axum::Router, AppState) {
    let (app, state, _storage) = app_with_script_and_agents(model, settings, configured_runner);
    (app, state)
}

pub fn app_with_script(
    model: impl Into<Arc<ScriptedModel>>,
) -> (axum::Router, AppState, Arc<Storage>) {
    app_with_script_and_agents(model, AppSettings::default(), configured_runner)
}

/// The same app whose Agents `agents` builds from the model's recorder, for the
/// tests that leave an Agent out or replace it.
pub fn app_with_script_and_agents(
    model: impl Into<Arc<ScriptedModel>>,
    settings: AppSettings,
    agents: impl FnOnce(Arc<LlmCallRecorder>) -> AgentRunner,
) -> (axum::Router, AppState, Arc<Storage>) {
    app_with_model_and_storage(
        Arc::new(Storage::new_in_memory()),
        model.into(),
        settings,
        agents,
    )
}

/// The returned storage is the instance the app holds.
pub fn app_with_model_and_storage(
    storage: Arc<Storage>,
    model: Arc<ScriptedModel>,
    settings: AppSettings,
    agents: impl FnOnce(Arc<LlmCallRecorder>) -> AgentRunner,
) -> (axum::Router, AppState, Arc<Storage>) {
    let recorder = make_test_recorder_with_storage(model, Arc::clone(&storage));
    let runner = agents(Arc::clone(&recorder));
    let pipeline = make_test_pipeline_with_backends(Arc::clone(&storage), recorder, runner);
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .pipeline(pipeline)
        .settings(settings)
        .build_with_state();
    (app, state, storage)
}

/// The production graph, whose provider comes from the settings.
pub fn app_with_production_graph(settings: AppSettings) -> (axum::Router, AppState, Arc<Storage>) {
    assert!(
        settings
            .connections
            .iter()
            .all(|c| c.provider == LlmBackendType::Mock),
        "app_with_production_graph requires Mock connections: the production graph \
         builds the provider the settings name"
    );

    let storage = Arc::new(Storage::new_in_memory());
    storage.save_settings(&settings).expect("seed settings");
    let wired = build_app_graph(Arc::clone(&storage), None).expect("production graph should build");
    let state = AppState::from_wired(wired);
    (build_router(state.clone()), state, storage)
}
