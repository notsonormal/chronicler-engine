//! HTTP E2E tests for game creation (POST /games).

use std::sync::Arc;

use axum::{body::Body, http::Request, http::StatusCode};
use tower::util::ServiceExt;

use chronicler_engine::TestAppBuilder;
use chronicler_engine::domain::model::game::GameId;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::test_support::{
    active_game_of, TestMap, TestPersona, TestWorld, empty_sqlite_storage,
};

use crate::support::http_fixtures::story_of;

// [docs/specs/games.md] SCENARIO: 17.1
#[tokio::test]
async fn test_create_game_handler() {
    // games_create needs a world with a scenario; the shared helper seeds a
    // minimal world without one, so build storage directly here.
    let storage = Arc::new(empty_sqlite_storage());

    let mut world = TestWorld::minimal();
    world.scenarios = vec![
        chronicler_engine::domain::model::scenario::StartingScenario {
            id: "test_intro".to_string(),
            name: "Test Intro".to_string(),
            description: "Test scenario".to_string(),
            starting_room_id: "start".to_string(),
            text: "Welcome to the test world!".to_string(),
            npcs: vec![],
        },
    ];
    let mut map = TestMap::single_room("start");
    map.overworld.regions[0].rooms[0].id = "start".to_string();
    storage.seed_world(&world, &map).unwrap();
    let player = TestPersona::standard();
    storage.seed_persona(&player.key, &player).unwrap();

    let initial_game_id = chronicler_engine::test_support::seed_game_row(
        &storage,
        &chronicler_engine::test_support::test_new_game(
            &world.name,
            &world.key,
            &player.key,
            &player.sheet.name,
            "Initial Game",
        ),
    )
    .0;

    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .active_game(GameId(initial_game_id))
        .build_with_state();

    let old_id = active_game_of(&state);

    let req = Request::builder()
        .uri("/games")
        .method(http::Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(format!(
            "world_key={}&persona_key={}",
            world.key, player.key
        )))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("HX-Refresh").unwrap(),
        "true",
        "Should return HX-Refresh header"
    );

    assert_ne!(
        active_game_of(&state),
        old_id,
        "Should have switched to the new game"
    );

    let messages = story_of(&state);
    assert!(
        !messages.is_empty(),
        "New game should have at least one message (scenario introduction)"
    );
    let scenario_msg = &messages[0];
    assert_eq!(
        scenario_msg.message_type,
        MessageType::Narration,
        "First message should be Narration type"
    );
    assert!(
        !scenario_msg.swipes.is_empty(),
        "the scenario message must carry its first swipe"
    );
    // The create unit writes the opening Message and the Swipe that carries the
    // state the Game starts from; the Swipe's own state is not a tier-1 read
    // surface, so this test reads the records the story reads return.
}

// [docs/specs/games.md] SCENARIO: 17.2
#[tokio::test]
async fn test_create_game_handler_unknown_world_key() {
    let app = TestAppBuilder::default_app();

    let form_data = "world_key=no_such_world&persona_key=test_player";
    let req = Request::builder()
        .uri("/games")
        .method(http::Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Expected 400 for unknown world_key: {:?}",
        response.status()
    );

    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("World not found"),
        "Expected 'World not found' in error body: {body_str}"
    );
}

// [docs/specs/games.md] SCENARIO: 17.3
#[tokio::test]
async fn test_create_game_handler_unknown_persona_key() {
    let app = TestAppBuilder::default_app();

    let form_data = "world_key=test&persona_key=no_such_persona";
    let req = Request::builder()
        .uri("/games")
        .method(http::Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Expected 400 for unknown persona_key: {:?}",
        response.status()
    );

    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Persona not found"),
        "Expected 'Persona not found' in error body: {body_str}"
    );
}
