use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::util::ServiceExt;

use chronicler_engine::TestAppBuilder;
use chronicler_engine::domain::model::message::{Message, Swipe};
use chronicler_engine::domain::model::settings::{AppSettings, TextCheckMode, TextCheckSettings};
use chronicler_engine::domain::model::state::generation_status::GenerationPhase;
use chronicler_engine::domain::model::state::generation_status::GenerationStatus;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::adapters::driven::storage::{Storage, TestOverride};
use chronicler_engine::test_support::TestPlayer;

use super::test_helpers::fetch_body;
use crate::TEST_PERSONA;

#[tokio::test]
async fn test_basic_fragments_return_html() {
    let app = TestAppBuilder::default_app();
    let fragments = [
        ("/fragment/header", "class=\"header\""),
        ("/fragment/story-log", "id=\"story-log\""),
        ("/fragment/visual-sidebar", "id=\"visual-sidebar\""),
        ("/fragment/action-area", "id=\"action-area\""),
    ];
    for (uri, expected) in fragments {
        let body = fetch_body(app.clone(), uri).await;
        assert!(
            body.contains(expected),
            "Fragment {uri} should contain '{expected}'"
        );
    }
}

#[tokio::test]
async fn test_visual_sidebar_renders_room_image() {
    let body = fetch_body(TestAppBuilder::default_app(), "/fragment/visual-sidebar").await;
    assert!(
        body.contains("data/images/test_room.png"),
        "Expected room image in sidebar: {body}"
    );
    assert!(
        !body.contains("No Location Image"),
        "Should not show placeholder when image exists: {body}"
    );
}

#[tokio::test]
async fn test_action_area_fragment_returns_html() {
    let body = fetch_body(TestAppBuilder::default_app(), "/fragment/action-area").await;
    assert!(
        body.contains("id=\"action-area\""),
        "Expected action-area id: {body}"
    );
}

#[tokio::test]
async fn test_action_handler_accepts_command() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/action")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=look"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("status"));
}

#[tokio::test]
async fn test_action_handler_empty_command() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/action")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command="))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Thinking"),
        "Expected empty command to trigger continuation: {body_str}"
    );
}

#[tokio::test]
async fn test_status_ready_handler() {
    let body = fetch_body(TestAppBuilder::default_app(), "/status/ready").await;
    assert!(body.contains("Ready"));
}

#[tokio::test]
async fn test_character_headshots_fragment() {
    let body = fetch_body(
        TestAppBuilder::default_app(),
        "/fragment/character-headshots",
    )
    .await;

    assert!(
        body.contains("headshot"),
        "Expected headshot in fragment: {body}"
    );
}

#[tokio::test]
async fn test_generating_status_variants() {
    let body = fetch_body(TestAppBuilder::default_app(), "/status/generating").await;
    assert!(body.contains("idle"), "Default status should be idle");

    let body = fetch_body(
        TestAppBuilder::default_test()
            .generation_status(GenerationStatus::Generating, GenerationPhase::Narrating)
            .build(),
        "/status/generating",
    )
    .await;
    assert!(
        body.contains("narrating"),
        "Narrating status should contain 'narrating'"
    );

    let body = fetch_body(
        TestAppBuilder::default_test()
            .generation_status(GenerationStatus::Generating, GenerationPhase::Quantifying)
            .build(),
        "/status/generating",
    )
    .await;
    assert!(
        body.contains("quantifying"),
        "Quantifying status should contain 'quantifying'"
    );
}

#[tokio::test]
async fn test_reset_generating_handler() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/status/reset-generating")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("reset"));
}

#[tokio::test]
async fn test_edit_history_handler_success() {
    let storage = Arc::new(Storage::new_in_memory());

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

    let app = TestAppBuilder::default_test()
        .log("Original text", Some("Test"), MessageType::Narration)
        .storage(Arc::clone(&storage))
        .build();

    let entry_id = storage.load_message_rows().unwrap().last().unwrap().id;

    let req = Request::builder()
        .uri(format!("/history/{entry_id}"))
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("text=Edited+text"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Edited"),
        "Expected success message: {body_str}"
    );
}

#[tokio::test]
async fn test_edit_history_handler_not_found() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/history/999")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("text=Edited text"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_delete_history_handler_success() {
    let app = TestAppBuilder::default_test()
        .log("Test message", Some("Test"), MessageType::Narration)
        .build();

    let req = Request::builder()
        .uri("/history/delete")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = fetch_body(app, "/fragment/story-log").await;
    assert!(
        !body.contains("Test message"),
        "Log entry should be deleted from rendered story log"
    );
}

#[tokio::test]
async fn test_delete_history_handler_empty() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/history/delete")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_action_confirm_empty_command() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/action/confirm")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command="))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("thinking") || body_str.contains("Generating"),
        "Expected empty command to trigger continuation: {body_str}"
    );
}

#[tokio::test]
async fn test_action_concurrent_rejection() {
    let app = TestAppBuilder::default_test().is_generating(true).build();

    let req = Request::builder()
        .uri("/action")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=go south"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Still thinking..."),
        "Expected concurrent rejection: {body_str}"
    );
}

#[tokio::test]
async fn test_action_async_inventory() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/action")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=inventory"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let hx_trigger = response.headers().get("HX-Trigger");
    assert!(hx_trigger.is_none(), "Inventory should be async, not sync");
}

#[tokio::test]
async fn test_list_games_fragment_populated() {
    let app = TestAppBuilder::default_app();
    let world_key = "test"; // Default test app world key

    let req = Request::builder()
        .uri("/games")
        .method(http::Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(format!(
            "world_key={world_key}&persona_key={TEST_PERSONA}"
        )))
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let req = Request::builder()
        .uri("/games")
        .method(http::Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(format!(
            "world_key={world_key}&persona_key={TEST_PERSONA}"
        )))
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let req = Request::builder()
        .uri("/fragment/games")
        .method(http::Method::GET)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    let html = String::from_utf8_lossy(&body);
    assert!(
        html.contains("game-item"),
        "Should render game item: {html}"
    );
    assert!(html.contains("Switch"), "Should have switch button: {html}");
    assert!(html.contains("Delete"), "Should have delete button: {html}");
}

#[tokio::test]
async fn test_list_games_fragment_escapes_html() {
    let storage = Arc::new(Storage::new_in_memory());

    let _ = storage
        .create_game(
            "Test World",
            "Test World",
            "test_player",
            "Test Player",
            "<script>alert('xss')</script>",
        )
        .unwrap();

    let app = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build();

    let req = Request::builder()
        .uri("/fragment/games")
        .method(http::Method::GET)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    let html = String::from_utf8_lossy(&body);
    assert!(
        !html.contains("<script>"),
        "Should escape HTML in game name: {html}"
    );

    assert!(
        html.contains("&#60;script&#62;") || html.contains("&lt;script&gt;"),
        "Should contain escaped script tag: {html}"
    );
}

#[tokio::test]
async fn test_create_game_handler() {
    let storage = Arc::new(Storage::new_in_memory());

    let mut world = chronicler_engine::test_support::TestWorld::minimal();
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
    let mut map = chronicler_engine::test_support::TestMap::single_room("start");
    map.overworld.regions[0].rooms[0].id = "start".to_string();
    storage.seed_world(&world, &map).unwrap();
    let player = chronicler_engine::test_support::TestPlayer::standard();
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

    let app = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build();

    let old_id = storage.current_game_id();

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

    let new_id = storage.current_game_id();
    assert_ne!(new_id, old_id, "Should have switched to the new game");

    let latest = storage.load_latest_snapshot().unwrap();
    assert!(latest.is_some(), "New game should have an initial snapshot");
    let messages = storage.load_message_rows().unwrap();
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
    let swipe_count = storage.count_swipes_for_message(scenario_msg.id).unwrap();
    assert!(
        swipe_count > 0,
        "Scenario message should have at least one swipe (text content)"
    );
}
#[tokio::test]
async fn test_switch_game_handler_success() {
    let storage = Arc::new(Storage::new_in_memory());

    use chronicler_engine::test_support::{TestWorld, TestMap, TestPlayer};
    let world = TestWorld::minimal();
    let map = TestMap::single_room("start");
    storage.seed_world(&world, &map).unwrap();
    let player = TestPlayer::standard();
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

    let app = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build();

    let other_id = storage
        .create_game(
            "Test World",
            "Test World",
            &player.key,
            &player.sheet.name,
            "Test World_2026-01-01_1",
        )
        .unwrap();
    assert_ne!(other_id, storage.current_game_id());

    let req = Request::builder()
        .uri(format!("/games/{other_id}/switch"))
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("HX-Refresh").unwrap(),
        "true",
        "Should return HX-Refresh header"
    );
    assert_eq!(storage.current_game_id(), other_id);
}

#[tokio::test]
async fn test_switch_game_handler_not_found() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/games/9999/switch")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_switch_game_handler_cross_world_allowed() {
    let storage = Arc::new(Storage::new_in_memory());

    let world_a = chronicler_engine::test_support::TestWorld::minimal();
    let map_a = chronicler_engine::test_support::TestMap::single_room("start");
    storage.seed_world(&world_a, &map_a).unwrap();

    let mut world_b = chronicler_engine::test_support::TestWorld::minimal();
    world_b.key = "world_b".to_string();
    world_b.name = "World B".to_string();
    let map_b = chronicler_engine::test_support::TestMap::single_room("room_b");
    storage.seed_world(&world_b, &map_b).unwrap();

    let player = TestPlayer::standard();
    storage.seed_persona(&player.key, &player).unwrap();
    let game_a_id = storage
        .create_game(
            &world_a.name,
            &world_a.key,
            &player.key,
            &player.sheet.name,
            "Game A",
        )
        .unwrap();
    let game_b_id = storage
        .create_game(
            &world_b.name,
            &world_b.key,
            &player.key,
            &player.sheet.name,
            "Game B",
        )
        .unwrap();

    storage.set_game_id(game_a_id);

    let app = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build();

    let req = Request::builder()
        .uri(format!("/games/{game_b_id}/switch"))
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Cross-world switch should succeed"
    );
    assert_eq!(
        storage.current_game_id(),
        game_b_id,
        "Should switch to game B"
    );
}

#[tokio::test]
async fn test_delete_game_handler_success() {
    let storage = Arc::new(Storage::new_in_memory());

    use chronicler_engine::test_support::{TestWorld, TestMap, TestPlayer};
    let world = TestWorld::minimal();
    let map = TestMap::single_room("start");
    storage.seed_world(&world, &map).unwrap();
    let player = TestPlayer::standard();
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

    let app = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build();

    let other_id = storage
        .create_game(
            "Test World",
            "Test World",
            &player.key,
            &player.sheet.name,
            "Test World_2026-01-01_1",
        )
        .unwrap();
    assert_ne!(other_id, storage.current_game_id());

    let req = Request::builder()
        .uri(format!("/games/{other_id}/delete"))
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(storage.get_game(other_id).unwrap().is_none());
}

#[tokio::test]
async fn test_delete_game_handler_active_game() {
    let storage = Arc::new(Storage::new_in_memory());

    let world = chronicler_engine::test_support::TestWorld::minimal();
    let map = chronicler_engine::test_support::TestMap::single_room("start");
    storage.seed_world(&world, &map).unwrap();
    let player = chronicler_engine::test_support::TestPlayer::standard();
    storage.seed_persona(&player.key, &player).unwrap();

    let game_id = storage
        .create_game(
            &world.name,
            &world.key,
            &player.key,
            &player.sheet.name,
            "Active Game",
        )
        .unwrap();
    storage.set_game_id(game_id);

    let app = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build();

    let active_game_id = storage.current_game_id();

    let req = Request::builder()
        .uri(format!("/games/{active_game_id}/delete"))
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Cannot delete active game"
    );
}

#[tokio::test]
async fn test_delete_game_handler_generating() {
    let app = TestAppBuilder::default_test().is_generating(true).build();

    let req = Request::builder()
        .uri("/games/9999/delete")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn test_list_games_fragment_storage_error() {
    let storage = Arc::new(
        Storage::new_in_memory()
            .with_failure("list_games", TestOverride::config("list_games failed")),
    );

    let app = TestAppBuilder::default_test().storage(storage).build();

    let req = Request::builder()
        .uri("/fragment/games")
        .method(http::Method::GET)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_list_games_fragment_shows_world_badge() {
    let storage = Arc::new(Storage::new_in_memory());
    let world_a = chronicler_engine::test_support::TestWorld::minimal();
    let map_a = chronicler_engine::test_support::TestMap::single_room("start");
    storage.seed_world(&world_a, &map_a).unwrap();
    let player = chronicler_engine::test_support::TestPlayer::standard();
    storage.seed_persona(&player.key, &player).unwrap();

    let mut world_b = chronicler_engine::test_support::TestWorld::minimal();
    world_b.key = "world_b_test".to_string();
    world_b.name = "World B Test".to_string();
    let map_b = chronicler_engine::test_support::TestMap::single_room("start");
    storage.seed_world(&world_b, &map_b).unwrap();
    storage.seed_persona(&player.key, &player).unwrap();

    let _game_a = storage
        .create_game(
            &world_a.name,
            &world_a.key,
            &player.key,
            &player.sheet.name,
            "Game A",
        )
        .unwrap();
    let _game_b = storage
        .create_game(
            &world_b.name,
            &world_b.key,
            &player.key,
            &player.sheet.name,
            "Game B",
        )
        .unwrap();

    let app = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build();

    let req = Request::builder()
        .uri("/fragment/games")
        .method(http::Method::GET)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    let html = String::from_utf8_lossy(&body);
    assert!(
        html.contains("world-badge"),
        "Should render world-badge class: {html}"
    );
    assert!(
        html.contains(&world_a.name),
        "Should show world A name: {html}"
    );
    assert!(
        html.contains(&world_b.name),
        "Should show world B name: {html}"
    );
}

#[tokio::test]
async fn test_list_games_fragment_shows_new_game_form() {
    let storage = Arc::new(Storage::new_in_memory());
    let world_a = chronicler_engine::test_support::TestWorld::minimal();
    let map_a = chronicler_engine::test_support::TestMap::single_room("start");
    storage.seed_world(&world_a, &map_a).unwrap();
    let player = chronicler_engine::test_support::TestPlayer::standard();
    storage.seed_persona(&player.key, &player).unwrap();

    let app = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build();

    let req = Request::builder()
        .uri("/fragment/games")
        .method(http::Method::GET)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    let html = String::from_utf8_lossy(&body);
    assert!(
        html.contains("<select") && html.contains("world_key"),
        "Should render world picker select element: {html}"
    );
}

#[tokio::test]
async fn test_create_game_with_world_key() {
    let storage = Arc::new(Storage::new_in_memory());
    let world_a = chronicler_engine::test_support::TestWorld::minimal();
    let map_a = chronicler_engine::test_support::TestMap::single_room("start");
    storage.seed_world(&world_a, &map_a).unwrap();
    let player = chronicler_engine::test_support::TestPlayer::standard();
    storage.seed_persona(&player.key, &player).unwrap();

    let app = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build();

    let req = Request::builder()
        .uri("/games")
        .method(http::Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(format!(
            "world_key={}&persona_key={}",
            world_a.key, player.key
        )))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("HX-Refresh").unwrap(),
        "true",
        "Should return HX-Refresh header"
    );

    let games = storage.list_games().unwrap();

    for game in &games {
        assert_eq!(
            game.world_key, world_a.key,
            "All games should be in world '{}'",
            world_a.key
        );
    }
    assert!(!games.is_empty(), "Should have created at least one game");
}

#[tokio::test]
async fn test_create_game_with_invalid_world_key() {
    let storage = Arc::new(Storage::new_in_memory());
    let world_a = chronicler_engine::test_support::TestWorld::minimal();
    let map_a = chronicler_engine::test_support::TestMap::single_room("start");
    storage.seed_world(&world_a, &map_a).unwrap();
    let player = chronicler_engine::test_support::TestPlayer::standard();
    storage.seed_persona(&player.key, &player).unwrap();

    let app = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build();

    let req = Request::builder()
        .uri("/games")
        .method(http::Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(format!(
            "world_key=nonexistent_world&persona_key={TEST_PERSONA}",
        )))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Should return bad_request for non-existent world"
    );
}

// ─── Misc Fragment Handler Tests ───

#[tokio::test]
async fn test_check_text_handler_disabled() {
    let app = TestAppBuilder::default_test()
        .settings(AppSettings {
            text_check: TextCheckSettings {
                mode: TextCheckMode::Disabled,
                enable_auto_check: true,
                ignored_words: vec![],
            },
            ..Default::default()
        })
        .build();

    let req = Request::builder()
        .uri("/check-text")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=look"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 4096)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("disabled"),
        "Expected disabled message: {body_str}"
    );
}

#[tokio::test]
async fn test_check_text_handler_finds_issues() {
    let app = TestAppBuilder::default_test()
        .settings(AppSettings {
            text_check: TextCheckSettings {
                mode: TextCheckMode::Spell,
                enable_auto_check: true,
                ignored_words: vec![],
            },
            ..Default::default()
        })
        .build();

    let req = Request::builder()
        .uri("/check-text")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=go+to+the+casle"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 4096)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("text-check-preview"),
        "Expected preview fragment: {body_str}"
    );
}

#[tokio::test]
async fn test_retry_handler_requires_context() {
    let storage = Arc::new(Storage::new_in_memory());
    let app = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build();

    let game_id = storage.current_game_id();
    storage.delete_game(game_id).unwrap();

    let req = Request::builder()
        .uri("/swipe/new")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "Should fail when game context is missing"
    );
}

#[tokio::test]
async fn test_retrigger_handler_requires_context() {
    let storage = Arc::new(Storage::new_in_memory());
    let app = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build();

    let game_id = storage.current_game_id();
    storage.delete_game(game_id).unwrap();

    let req = Request::builder()
        .uri("/retrigger")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "Should fail when game context is missing"
    );
}

#[tokio::test]
async fn test_switch_swipe_handler_success() {
    let storage = Arc::new(Storage::new_in_memory());
    let app = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build();

    let latest_snap = storage.load_latest_snapshot().unwrap().unwrap();
    let snap_id = latest_snap.db_id.unwrap();

    let msg = Message::new(
        Some("Player".to_string()),
        "original",
        MessageType::Narration,
        None,
        None,
    );
    let id = storage.insert_message(&msg).unwrap();

    storage
        .insert_swipe(
            id,
            &Swipe {
                text: "first swipe".to_string(),
                snapshot_id: Some(snap_id),
                location_header: None,
                event_header: None,
            },
            0,
        )
        .unwrap();
    storage
        .insert_swipe(
            id,
            &Swipe {
                text: "second swipe".to_string(),
                snapshot_id: Some(snap_id),
                location_header: None,
                event_header: None,
            },
            1,
        )
        .unwrap();

    let req = Request::builder()
        .uri(format!("/message/{id}/swipe/1"))
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_reset_handler_success() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/reset")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let hx_refresh = response.headers().get("HX-Refresh");
    assert_eq!(
        hx_refresh,
        Some(&axum::http::HeaderValue::from_static("true")),
        "Reset should trigger HX-Refresh"
    );
}

#[tokio::test]
async fn test_check_text_handler_empty() {
    let app = TestAppBuilder::default_test()
        .settings(AppSettings {
            text_check: TextCheckSettings {
                mode: TextCheckMode::Spell,
                enable_auto_check: true,
                ignored_words: vec![],
            },
            ..Default::default()
        })
        .build();

    let req = Request::builder()
        .uri("/check-text")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command="))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Empty command should return bad request"
    );
}

#[tokio::test]
async fn test_check_text_handler_no_issues() {
    let app = TestAppBuilder::default_test()
        .settings(AppSettings {
            text_check: TextCheckSettings {
                mode: TextCheckMode::Spell,
                enable_auto_check: true,
                ignored_words: vec![],
            },
            ..Default::default()
        })
        .build();

    let req = Request::builder()
        .uri("/check-text")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=go+to+the+castle"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 4096)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("No issues found"),
        "Expected no-issues response: {body_str}"
    );
}

#[tokio::test]
async fn test_retry_handler_valid_context_error() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/swipe/new")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Retry with no input should return bad request"
    );
}

#[tokio::test]
async fn test_retrigger_handler_valid_context_error() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/retrigger")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Retrigger with no trigger context should return bad request"
    );
}

#[tokio::test]
async fn test_switch_swipe_handler_not_last() {
    let storage = Arc::new(Storage::new_in_memory());
    let app = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build();

    let latest_snap = storage.load_latest_snapshot().unwrap().unwrap();
    let snap_id = latest_snap.db_id.unwrap();

    let msg1 = Message::new(
        Some("Player".to_string()),
        "first",
        MessageType::Narration,
        None,
        None,
    );
    let id1 = storage.insert_message(&msg1).unwrap();
    storage
        .insert_swipe(
            id1,
            &Swipe {
                text: "swipe".to_string(),
                snapshot_id: Some(snap_id),
                location_header: None,
                event_header: None,
            },
            0,
        )
        .unwrap();

    let msg2 = Message::new(
        Some("Player".to_string()),
        "second",
        MessageType::Narration,
        None,
        None,
    );
    let _id2 = storage.insert_message(&msg2).unwrap();

    let req = Request::builder()
        .uri(format!("/message/{id1}/swipe/0"))
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Switching swipe on non-last message should fail"
    );
}

#[tokio::test]
async fn test_switch_swipe_handler_concurrent() {
    let storage = Arc::new(Storage::new_in_memory());
    let app = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .is_generating(true)
        .build();

    let latest_snap = storage.load_latest_snapshot().unwrap().unwrap();
    let snap_id = latest_snap.db_id.unwrap();

    let msg = Message::new(
        Some("Player".to_string()),
        "original",
        MessageType::Narration,
        None,
        None,
    );
    let id = storage.insert_message(&msg).unwrap();
    storage
        .insert_swipe(
            id,
            &Swipe {
                text: "swipe".to_string(),
                snapshot_id: Some(snap_id),
                location_header: None,
                event_header: None,
            },
            0,
        )
        .unwrap();

    let req = Request::builder()
        .uri(format!("/message/{id}/swipe/0"))
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::SERVICE_UNAVAILABLE,
        "Switching swipe during generation should fail"
    );
}

#[tokio::test]
async fn test_reset_handler_generating() {
    let app = TestAppBuilder::default_test().is_generating(true).build();

    let req = Request::builder()
        .uri("/reset")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::SERVICE_UNAVAILABLE,
        "Reset during generation should fail"
    );
}
