//! Test-app builders for the http test binary: scripted-model AppState + router bundles.

use std::sync::Arc;

use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::adapters::driving::http::AppState;
use chronicler_engine::adapters::driving::http::builders::router::build_router;
use chronicler_engine::application::agents::AgentRunner;
use chronicler_engine::application::llm_recorder::LlmCallRecorder;
use chronicler_engine::bootstrap::wiring::build_app_graph;
use chronicler_engine::domain::model::agent::Role;
use chronicler_engine::domain::model::character::CharacterCard;
use chronicler_engine::domain::model::game::GameId;
use chronicler_engine::domain::model::llm_backend::LlmBackendType;
use chronicler_engine::domain::model::map::MapDef;
use chronicler_engine::domain::model::settings::{AppSettings, LlmConnection};
use chronicler_engine::domain::model::trigger::ComparisonOperator;
use chronicler_engine::test_support::{
    active_game_of, analysis_report, configured_runner, empty_sqlite_storage, make_test_job_deps,
    test_connection, make_test_recorder_with_storage, seed_game_row, test_new_game, ScriptedModel,
    TestAppBuilder, TestData, TestDataBuilder, TestMap, TestCharacter, TestPersona,
};

use crate::support::http_fixtures::{append_narrations, seeded_storage_with_initial_game};
use crate::support::http_requests::{post_action, wait_settled};

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
    app_over_data(
        TestDataBuilder::default_test().build(),
        storage,
        model,
        settings,
        agents,
    )
}

/// The app of `app_with_model_and_storage`, over the World, Persona and Characters of `data`.
fn app_over_data(
    data: TestData,
    storage: Arc<Storage>,
    model: Arc<ScriptedModel>,
    settings: AppSettings,
    agents: impl FnOnce(Arc<LlmCallRecorder>) -> AgentRunner,
) -> (axum::Router, AppState, Arc<Storage>) {
    let recorder = make_test_recorder_with_storage(model, Arc::clone(&storage));
    let runner = agents(Arc::clone(&recorder));
    let job_deps = make_test_job_deps(Arc::clone(&storage), recorder, runner);
    let (app, state) = TestAppBuilder::with_data(data)
        .storage(Arc::clone(&storage))
        .job_deps(job_deps)
        .settings(settings)
        .build_with_state();
    (app, state, storage)
}

/// The production graph, whose provider comes from the settings.
pub fn app_with_production_graph(settings: AppSettings) -> (axum::Router, AppState, Arc<Storage>) {
    app_with_production_graph_over(Arc::new(empty_sqlite_storage()), settings)
}

/// The production graph over `storage`: the World, the Persona and the presets
/// come from it too, so a test can create a Game through the route.
pub fn app_with_production_graph_over(
    storage: Arc<Storage>,
    settings: AppSettings,
) -> (axum::Router, AppState, Arc<Storage>) {
    assert!(
        settings
            .connections
            .iter()
            .all(|c| c.provider == LlmBackendType::Mock),
        "app_with_production_graph_over requires Mock connections: the production graph \
         builds the provider the settings name"
    );

    storage.save_settings(&settings).expect("seed settings");
    let wired = build_app_graph(Arc::clone(&storage), None).expect("production graph should build");
    let state = AppState::from_wired(wired);
    (build_router(state.clone()), state, storage)
}

/// A Game over `map` whose starting room holds `character`, so the Quantifier of the Job
/// meets it and its Trigger fires.
pub fn app_with_a_triggered_character(
    map: MapDef,
    character: CharacterCard,
    model: ScriptedModel,
) -> (axum::Router, AppState) {
    let data = TestDataBuilder::default_test()
        .map(map)
        .characters(vec![character])
        .build();
    TestAppBuilder::default_test()
        .data(data)
        .model(Arc::new(model))
        .build_with_state()
}

/// A Game whose `/impersonate` fired a Trigger and whose event Narration failed, so
/// the Impersonate Input holds the event record. The model holds no reply for a
/// Retrigger.
pub async fn app_after_an_impersonate_with_a_failed_event() -> (axum::Router, AppState) {
    impersonated_app_with_a_failed_event(false).await
}

/// The Game of `app_after_an_impersonate_with_a_failed_event`, with the replies of
/// one Retrigger scripted.
pub async fn app_ready_to_retrigger_after_a_failed_event() -> (axum::Router, AppState) {
    impersonated_app_with_a_failed_event(true).await
}

async fn impersonated_app_with_a_failed_event(
    scripts_a_retrigger: bool,
) -> (axum::Router, AppState) {
    let character = TestCharacter::with_times_met_trigger(
        "character_1",
        "Test Character",
        ComparisonOperator::Eq,
        0,
    );
    let model = ScriptedModel::new()
        .narration("I say hello.")
        .after_narration(analysis_report(&["character_1"], None))
        .failed(Role::Trigger);
    let model = if scripts_a_retrigger {
        model
            .event_narration("The event fires again.")
            .after_narration(analysis_report(&["character_1"], None))
    } else {
        model
    };
    let (app, state) =
        app_with_a_triggered_character(TestMap::single_room("room_1"), character, model);

    assert!(
        post_action(&app, "/impersonate hello")
            .await
            .status()
            .is_success()
    );
    assert!(
        wait_settled(&state, 2000).await,
        "the impersonate turn should settle after its failed continuation"
    );
    (app, state)
}

/// One memory entry from the first Message of the story, the oldest Message of the
/// first cut.
pub const TALE_OF_THE_INN_ENTRIES: &str = r#"{"entries": [{"from": 1, "to": 1, "time": "Day 1, evening", "place": "The inn", "took_part": [], "others": [], "what_happened": "The tale of the inn was told at length."}]}"#;

pub fn long_story_app(window: Option<u32>) -> (axum::Router, AppState, Arc<Storage>, GameId) {
    // A narrow window cuts the history at one of the two Actions, and the cut
    // calls the story memory writer.
    let model = Arc::new(
        ScriptedModel::new()
            .turn("First narration.")
            .turn("Second narration.")
            .story_memory(TALE_OF_THE_INN_ENTRIES)
            .optional(),
    );
    padded_story_app(model, window, 150)
}

/// The one connection serves the narration and the analysis.
pub fn padded_story_app(
    model: Arc<ScriptedModel>,
    window: Option<u32>,
    padding: usize,
) -> (axum::Router, AppState, Arc<Storage>, GameId) {
    padded_story_app_over(
        model,
        story_connection(window),
        TestDataBuilder::default_test().build(),
        padding,
    )
}

pub fn story_connection(window: Option<u32>) -> LlmConnection {
    LlmConnection {
        model: "mock-model".into(),
        max_context_tokens: window,
        ..test_connection("narration-conn", LlmBackendType::Mock)
    }
}

/// `connection` serves the narration and the analysis, and `data` is the Game's World.
pub fn padded_story_app_over(
    model: Arc<ScriptedModel>,
    connection: LlmConnection,
    data: TestData,
    padding: usize,
) -> (axum::Router, AppState, Arc<Storage>, GameId) {
    let settings = AppSettings {
        connections: vec![connection],
        narration_connection_id: "narration-conn".into(),
        quantifier_connection_id: "narration-conn".into(),
        ..AppSettings::default()
    };
    let (app, state, storage) = app_over_data(
        data,
        Arc::new(empty_sqlite_storage()),
        model,
        settings,
        configured_runner,
    );
    let game_id = active_game_of(&state);
    pad_story(&storage, game_id, 0, padding);
    (app, state, storage, game_id)
}

/// Each padding Narration starts from the newest Swipe state, so it keeps that
/// state's story memory.
pub fn pad_story(storage: &Storage, game: GameId, from: usize, count: usize) {
    append_narrations(storage, game, count, |index| {
        format!(
            "Story entry {}: the long tale of the inn, told again at length \
             for everyone who came to listen that evening.",
            from + index
        )
    });
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
