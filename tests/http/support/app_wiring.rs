//! Test-app builders for the http test binary: scripted-model AppState + router bundles.

use std::sync::Arc;

use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::adapters::driving::http::AppState;
use chronicler_engine::adapters::driving::http::builders::router::build_router;
use chronicler_engine::application::agents::AgentRunner;
use chronicler_engine::application::llm_recorder::LlmCallRecorder;
use chronicler_engine::bootstrap::wiring::build_app_graph;
use chronicler_engine::domain::model::character::NpcCard;
use chronicler_engine::domain::model::game::GameId;
use chronicler_engine::domain::model::llm_backend::LlmBackendType;
use chronicler_engine::domain::model::map::MapDef;
use chronicler_engine::domain::model::settings::{AppSettings, LlmProviderConfig};
use chronicler_engine::test_support::{
    active_game_of, configured_runner, empty_sqlite_storage, make_test_job_deps,
    make_test_recorder_with_storage, seed_game_row, test_new_game, ScriptedModel, TestAppBuilder,
    TestDataBuilder, TestPersona,
};

use crate::support::http_fixtures::{append_narrations, seeded_storage_with_initial_game};

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

/// Builds the Agent runner from `agents`, so a test can leave one out or replace
/// it.
pub fn app_with_script_and_agents(
    model: impl Into<Arc<ScriptedModel>>,
    settings: AppSettings,
    agents: impl FnOnce(Arc<LlmCallRecorder>) -> AgentRunner,
) -> (axum::Router, AppState, Arc<Storage>) {
    app_with_model_and_storage(
        Arc::new(empty_sqlite_storage()),
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
    let job_deps = make_test_job_deps(Arc::clone(&storage), recorder, runner);
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .job_deps(job_deps)
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

    let storage = Arc::new(empty_sqlite_storage());
    storage.save_settings(&settings).expect("seed settings");
    let wired = build_app_graph(Arc::clone(&storage), None).expect("production graph should build");
    let state = AppState::from_wired(wired);
    (build_router(state.clone()), state, storage)
}

/// A Game over `map` whose starting room holds `npc`, so the Turn's analysis
/// meets it and its Trigger fires.
pub fn app_with_a_triggered_npc(
    map: MapDef,
    npc: NpcCard,
    model: ScriptedModel,
) -> (axum::Router, AppState) {
    let data = TestDataBuilder::default_test()
        .map(map)
        .npcs(vec![npc])
        .build();
    TestAppBuilder::default_test()
        .data(data)
        .model(Arc::new(model))
        .build_with_state()
}

/// A Game whose story holds the opening Narration and 150 padding Narration
/// Messages, over one mock connection whose context window is `window`.
pub fn long_story_app(window: Option<u32>) -> (axum::Router, AppState, Arc<Storage>, GameId) {
    let model = Arc::new(
        ScriptedModel::new()
            .turn("First narration.")
            .turn("Second narration."),
    );
    let connection = LlmProviderConfig {
        model: "mock-model".into(),
        max_context_tokens: window,
        ..LlmProviderConfig::new("narration-conn", "narration-conn", LlmBackendType::Mock)
    };
    let settings = AppSettings {
        connections: vec![connection],
        narration_connection_id: "narration-conn".into(),
        quantifier_connection_id: "narration-conn".into(),
        ..AppSettings::default()
    };
    let (app, state, storage) = app_with_model_and_storage(
        Arc::new(empty_sqlite_storage()),
        model,
        settings,
        configured_runner,
    );
    let game_id = active_game_of(&state);

    append_narrations(&storage, game_id, 150, |index| {
        format!(
            "Story entry {index}: the long tale of the inn, told again at length \
             for everyone who came to listen that evening."
        )
    });
    (app, state, storage, game_id)
}

/// An app over the seeded storage, which holds the active Game plus a second
/// saved Game the caller acts on. The returned id is the second Game's.
pub fn two_game_app() -> (axum::Router, AppState, Arc<Storage>, u64) {
    let (storage, _world_key, persona_key, _initial_game_id) = seeded_storage_with_initial_game();
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();
    let other_id = seed_game_row(
        &storage,
        &test_new_game(
            "Test World",
            "Test World",
            &persona_key,
            &TestPersona::standard().sheet.name,
            "Test World_2026-01-01_1",
        ),
    )
    .0;
    assert_ne!(
        other_id,
        active_game_of(&state).0,
        "the second saved Game is not the active one"
    );
    (app, state, storage, other_id)
}
