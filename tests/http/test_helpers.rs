//! Shared test helpers for HTTP tests

use std::sync::Arc;

use axum::body::Body;
use axum::http::{self, Method, Request};
use tower::util::ServiceExt;

use chronicler_engine::adapters::driving::http::AppState;
use chronicler_engine::adapters::driven::llm::providers::MockBackend;
use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::application::ports::llm_provider::LlmProvider;
use chronicler_engine::application::agents::registry::AgentRegistry;
use chronicler_engine::domain::model::map::MapDef;
use chronicler_engine::domain::model::settings::{
    AppSettings, NarrativePerspective, NarrativeTense, NarratorMode,
};
use chronicler_engine::domain::model::world::WorldCard;
use chronicler_engine::test_support::{
    make_test_pipeline_with_backends, make_test_recorder_with_storage, TestAppBuilder, TestMap,
    TestPersona, TestWorld,
};

use crate::test_utils::wait_for_condition_async;

/// Seed an in-memory storage with a world, map, and persona, then create an
/// initial game and set it current. Returns `(storage, world_key, persona_key,
/// initial_game_id)`. Shared by the `games_create` / `games_switch` /
/// `games_delete` HTTP E2E tests to dedupe their setup shape.
pub fn seeded_storage_with_initial_game() -> (Arc<Storage>, String, String, i64) {
    let storage = Arc::new(Storage::new_in_memory());

    let world = TestWorld::minimal();
    let map = TestMap::single_room("start");
    storage.seed_world(&world, &map).unwrap();
    let player = TestPersona::standard();
    storage.seed_persona(&player.key, &player).unwrap();

    let initial_game_id = storage
        .create_game(
            &world.name,
            &world.key,
            &player.key,
            &player.sheet.name,
            "Initial Game",
        )
        .unwrap();
    storage.set_game_id(initial_game_id);

    (
        storage,
        world.key,
        player.key,
        initial_game_id.try_into().unwrap(),
    )
}

/// Fetch the response body as a String from the given URI.
/// Panics if the request fails or returns non-success status.
pub async fn fetch_body(app: &axum::Router, uri: &str) -> String {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert!(response.status().is_success(), "Expected success for {uri}");
    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    String::from_utf8_lossy(&body).to_string()
}

/// POST a url-encoded `command=...` body to `/action`.
pub async fn post_action(app: &axum::Router, command: &str) -> axum::response::Response<Body> {
    let body = format!("command={}", command.replace(' ', "+"));
    let req = Request::builder()
        .uri("/action")
        .method(Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from(body))
        .unwrap();
    app.clone().oneshot(req).await.unwrap()
}

/// POST a url-encoded `command=...` body to `/action/check`.
pub async fn post_action_check(
    app: &axum::Router,
    command: &str,
) -> axum::response::Response<Body> {
    let body = format!("command={}", command.replace(' ', "+"));
    let req = Request::builder()
        .uri("/action/check")
        .method(Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from(body))
        .unwrap();
    app.clone().oneshot(req).await.unwrap()
}

/// POST to a no-body endpoint (`/swipe/new`, `/history/delete`, `/reset`).
pub async fn post_empty(app: &axum::Router, uri: &str) -> axum::response::Response<Body> {
    let req = Request::builder()
        .uri(uri)
        .method(Method::POST)
        .body(Body::empty())
        .unwrap();
    app.clone().oneshot(req).await.unwrap()
}

/// Poll `AppState` until generation is idle (timeout `timeout_ms`, 15ms interval).
pub async fn wait_idle(state: &AppState, timeout_ms: u64) -> bool {
    wait_for_condition_async(
        std::time::Duration::from_millis(timeout_ms),
        std::time::Duration::from_millis(15),
        || async {
            !state
                .message_service
                .load_or_fresh()
                .narrative
                .input_buffer
                .status
                .is_generating()
        },
    )
    .await
}

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

/// POST a url-encoded body to an arbitrary URI.
pub async fn post_form(
    app: &axum::Router,
    uri: &str,
    body: &str,
) -> axum::response::Response<Body> {
    let req = Request::builder()
        .uri(uri)
        .method(Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from(body.to_string()))
        .unwrap();
    app.clone().oneshot(req).await.unwrap()
}

/// An Interactive Fiction world with second-person present-tense posture.
pub fn if_world() -> WorldCard {
    WorldCard {
        key: "if_world".to_string(),
        name: "IF World".to_string(),
        description: "An Interactive Fiction test world.".to_string(),
        narrator_mode: NarratorMode::InteractiveFiction,
        narrative_perspective: NarrativePerspective::Second,
        narrative_tense: NarrativeTense::Present,
        ..Default::default()
    }
}

/// Build a urlencoded `WorldForm` body for the worlds update endpoint.
/// Posture fields and the options toggle are appended by callers, so each
/// test controls exactly which fields the form carries.
pub fn world_form_body(world: &WorldCard, map: &MapDef) -> String {
    let map_json = serde_json::to_string(map).expect("map serializes");
    let scenarios_json = serde_json::to_string(&world.scenarios).expect("scenarios serialize");
    let pairs = vec![
        ("key", world.key.clone()),
        ("name", world.name.clone()),
        ("description", world.description.clone()),
        ("global_rules", world.global_rules.join("\n")),
        ("map_json", map_json),
        ("scenarios_json", scenarios_json),
    ];
    serde_urlencoded::to_string(pairs).expect("world form body serializes")
}
