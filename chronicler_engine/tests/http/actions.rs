use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use std::sync::Arc;
use tower::ServiceExt;

use chronicler_engine::TestAppBuilder;
use chronicler_engine::storage::{Storage, TestOverride};

#[tokio::test]
async fn test_action_handler_load_state_failure_graceful_degradation() {
    let storage = Arc::new(Storage::new_in_memory().with_failure(
        "load_latest_snapshot",
        TestOverride::internal("simulated load failure"),
    ));

    use chronicler_engine::test_support::{TestWorld, TestMap, TestPlayer};
    let world = TestWorld::minimal();
    let map = TestMap::single_room("start");
    storage.seed_world(&world, &map).unwrap();
    let player = TestPlayer::standard();
    storage.seed_persona(&player.key, &player).unwrap();
    let game_id = storage
        .create_game(
            &world.name,
            &world.key,
            &player.key,
            &player.sheet.name,
            "Test Game",
        )
        .unwrap();
    storage.set_game_id(game_id);

    let app = TestAppBuilder::default_test().storage(storage).build();

    let req = Request::builder()
        .uri("/action")
        .method(Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from("command=look"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_action_handler_snapshot_save_failure() {
    let storage = Arc::new(Storage::new_in_memory().with_failure(
        "save_snapshot",
        TestOverride::internal("simulated save failure"),
    ));

    let app = TestAppBuilder::default_test().storage(storage).build();

    let req = Request::builder()
        .uri("/action")
        .method(Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from("command=look"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_action_confirm_handler_load_state_failure_graceful_degradation() {
    let storage = Arc::new(Storage::new_in_memory().with_failure(
        "load_latest_snapshot",
        TestOverride::internal("simulated load failure"),
    ));

    use chronicler_engine::test_support::{TestWorld, TestMap, TestPlayer};
    let world = TestWorld::minimal();
    let map = TestMap::single_room("start");
    storage.seed_world(&world, &map).unwrap();
    let player = TestPlayer::standard();
    storage.seed_persona(&player.key, &player).unwrap();
    let game_id = storage
        .create_game(
            &world.name,
            &world.key,
            &player.key,
            &player.sheet.name,
            "Test Game",
        )
        .unwrap();
    storage.set_game_id(game_id);

    let app = TestAppBuilder::default_test().storage(storage).build();

    let req = Request::builder()
        .uri("/action/confirm")
        .method(Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from("command=look"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_action_handler_message_insert_failure() {
    let storage = Arc::new(Storage::new_in_memory().with_failure(
        "insert_message",
        TestOverride::internal("simulated insert failure"),
    ));

    use chronicler_engine::test_support::{TestWorld, TestMap, TestPlayer};
    let world = TestWorld::minimal();
    let map = TestMap::single_room("start");
    storage.seed_world(&world, &map).unwrap();
    let player = TestPlayer::standard();
    storage.seed_persona(&player.key, &player).unwrap();
    let game_id = storage
        .create_game(
            &world.name,
            &world.key,
            &player.key,
            &player.sheet.name,
            "Test Game",
        )
        .unwrap();
    storage.set_game_id(game_id);

    let app = TestAppBuilder::default_test().storage(storage).build();

    let req = Request::builder()
        .uri("/action")
        .method(Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from("command=look"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_action_handler_load_messages_failure() {
    let storage = Arc::new(Storage::new_in_memory().with_failure(
        "load_message_rows",
        TestOverride::internal("simulated load messages failure"),
    ));

    use chronicler_engine::test_support::{TestWorld, TestMap, TestPlayer};
    let world = TestWorld::minimal();
    let map = TestMap::single_room("start");
    storage.seed_world(&world, &map).unwrap();
    let player = TestPlayer::standard();
    storage.seed_persona(&player.key, &player).unwrap();
    let game_id = storage
        .create_game(
            &world.name,
            &world.key,
            &player.key,
            &player.sheet.name,
            "Test Game",
        )
        .unwrap();
    storage.set_game_id(game_id);

    let app = TestAppBuilder::default_test().storage(storage).build();

    let req = Request::builder()
        .uri("/action")
        .method(Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from("command=look"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_action_check_handler_empty_command() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/action/check")
        .method(Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from("command="))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert!(
        response.status().is_client_error() || response.status() == StatusCode::OK,
        "Empty command check should return error or OK with validation message"
    );
}

/// Test action handler with special characters in command.
#[tokio::test]
async fn test_action_handler_special_characters() {
    let app = TestAppBuilder::default_app();

    // URL-encoded: look at "the sign"
    let req = Request::builder()
        .uri("/action")
        .method(Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from("command=look+at+%22the+sign%22"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert!(
        response.status().is_success(),
        "Special characters in command should be handled: {:?}",
        response.status()
    );
}

#[tokio::test]
async fn test_action_confirm_snapshot_save_failure() {
    let storage = Arc::new(Storage::new_in_memory().with_failure(
        "save_snapshot",
        TestOverride::internal("simulated snapshot save failure"),
    ));

    let app = TestAppBuilder::default_test().storage(storage).build();

    let req = Request::builder()
        .uri("/action/confirm")
        .method(Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from("command=look"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
