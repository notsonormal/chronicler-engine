use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use std::sync::Arc;
use tower::util::ServiceExt;

use chronicler_engine::create_app_for_testing_with_settings;
use chronicler_engine::model::settings::{AppSettings, TextCheckMode, TextCheckSettings};
use chronicler_engine::model::state::{GameState, LogType};

use crate::create_test_state;

fn text_check_settings(mode: TextCheckMode) -> AppSettings {
    AppSettings {
        text_check: TextCheckSettings {
            mode,
            enable_auto_check: true,
            ignored_words: vec![],
        },
        ..Default::default()
    }
}

fn state_with_input() -> GameState {
    let mut state = create_test_state();
    state.add_log(
        "look around".to_string(),
        Some("Test Player".to_string()),
        LogType::Input,
    );
    state
}

#[tokio::test]
async fn test_check_text_empty_command() {
    let app = chronicler_engine::create_app_for_testing(create_test_state());

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

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Enter text to check"),
        "Expected error message: {body_str}"
    );
}

#[tokio::test]
async fn test_check_text_disabled_mode() {
    let app = create_app_for_testing_with_settings(
        create_test_state(),
        text_check_settings(TextCheckMode::Disabled),
    );

    let req = Request::builder()
        .uri("/check-text")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=hello"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Text check is disabled"),
        "Expected disabled message: {body_str}"
    );
}

#[tokio::test]
async fn test_check_text_finds_issues() {
    let app = create_app_for_testing_with_settings(
        create_test_state(),
        text_check_settings(TextCheckMode::Spell),
    );

    let req = Request::builder()
        .uri("/check-text")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=go to the casle"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 4096)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("text-check-preview"),
        "Expected preview HTML: {body_str}"
    );
}

#[tokio::test]
async fn test_check_text_no_issues() {
    let app = create_app_for_testing_with_settings(
        create_test_state(),
        text_check_settings(TextCheckMode::Spell),
    );

    let req = Request::builder()
        .uri("/check-text")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=go to the castle"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("No issues found"),
        "Expected no issues message: {body_str}"
    );
}

#[tokio::test]
async fn test_retry_no_input() {
    let app = chronicler_engine::create_app_for_testing(create_test_state());

    let req = Request::builder()
        .uri("/retry")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("No input to retry"),
        "Expected error: {body_str}"
    );
}

#[tokio::test]
async fn test_retry_success() {
    let app = chronicler_engine::create_app_for_testing(state_with_input());

    let req = Request::builder()
        .uri("/retry")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Retrying..."),
        "Expected retry message: {body_str}"
    );
}

#[tokio::test]
async fn test_retry_handler_sets_generating_status() {
    let app = chronicler_engine::create_app_for_testing(state_with_input());

    let req = Request::builder()
        .uri("/retry")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Retrying..."),
        "Expected retry message: {body_str}"
    );
}

#[tokio::test]
async fn test_retry_handler_preserves_swipe_index() {
    // Setup: create app with InMemorySnapshotStorage so we can inspect it
    let mut state = create_test_state();
    state.add_log(
        "look around".to_string(),
        Some("Test Player".to_string()),
        LogType::Input,
    );
    let snapshot = chronicler_engine::model::state_snapshot::GameStateSnapshot::from_game_state(
        &state,
        "test-turn".to_string(),
        0,
    );
    let storage = Arc::new(chronicler_engine::test_support::InMemorySnapshotStorage::new())
        as Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage>;
    let _ = storage.save(&snapshot);

    let llm_storage =
        Arc::new(chronicler_engine::storage::llm_message_storage::InMemoryLlmMessageStorage::new())
            as Arc<dyn chronicler_engine::storage::llm_message_storage::LlmMessageStorage>;

    let app = chronicler_engine::server::create_app_with_storage(
        state,
        Arc::clone(&storage),
        llm_storage,
        AppSettings::default(),
    );

    let req = Request::builder()
        .uri("/retry")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // The handler should save the generating snapshot with swipe_index = 0,
    // NOT prematurely increment it to 1.
    let latest = storage
        .load_latest(None)
        .unwrap()
        .expect("Should have snapshot");
    assert_eq!(
        latest.swipe_index, 0,
        "Retry handler should preserve current swipe_index, not increment it"
    );
}

#[tokio::test]
async fn test_reset_handler() {
    let app = chronicler_engine::create_app_for_testing(create_test_state());

    let req = Request::builder()
        .uri("/reset")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    assert_eq!(
        response.headers().get("HX-Refresh"),
        Some(&http::HeaderValue::from_static("true")),
        "Expected HX-Refresh: true header"
    );
}

#[tokio::test]
async fn test_reset_button_clears_state() {
    let mut state = create_test_state();
    state.add_log(
        "hello".to_string(),
        Some("Player".to_string()),
        LogType::Input,
    );
    state.add_log("Welcome!".to_string(), None, LogType::Narration);

    let app = chronicler_engine::create_app_for_testing(state);

    let req = Request::builder()
        .uri("/reset")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();

    assert!(response.status().is_success());

    // Check via a second request that state was reset
    let req = Request::builder()
        .uri("/fragment/story-log")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), 4096)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    // After reset, story log should not contain the old entries
    assert!(
        !body_str.contains("hello"),
        "Reset should clear story log: {body_str}"
    );
    assert!(
        !body_str.contains("Welcome!"),
        "Reset should clear story log: {body_str}"
    );
}

#[tokio::test]
async fn test_reset_preserves_scenario_npcs() {
    use chronicler_engine::model::character::{CharacterSheet, NpcCard};
    use chronicler_engine::model::map::{MapDef, Overworld, Region, Room};
    use chronicler_engine::model::scenario::StartingScenario;
    use chronicler_engine::model::world::WorldCard;
    use std::collections::HashMap;

    let world = Arc::new(WorldCard {
        name: "Test World".into(),
        description: "A test world".into(),
        global_rules: vec![],
        starting_room_id: "room_1".into(),
        scenarios: vec![StartingScenario {
            id: "test".into(),
            name: "Test".into(),
            description: "Test scenario".into(),
            starting_room_id: "room_1".into(),
            text: "".into(),
            npcs: vec!["npc_1".into()],
        }],
        default_room_image: None,
    });

    let test_room = Room {
        id: "room_1".into(),
        name: "Test Room".into(),
        description: "A test room for component tests.".into(),
        image_path: Some("data/images/test_room.png".into()),
        exits: HashMap::new(),
        items: vec![],
        navigation_description: None,
    };

    let map = Arc::new(MapDef {
        overworld: Overworld {
            id: "test_overworld".into(),
            name: "Test Overworld".into(),
            regions: vec![Region {
                id: "region_1".into(),
                name: "Test Region".into(),
                rooms: vec![test_room],
            }],
        },
    });

    let player = Arc::new(chronicler_engine::model::character::PlayerCard {
        sheet: CharacterSheet {
            name: "Test Player".into(),
            description: "A test player".into(),
            personality: "Brave".into(),
            scenario: "Test scenario".into(),
            example_dialogue: "Hello!".into(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
    });

    let npcs = vec![NpcCard {
        id: "npc_1".into(),
        sheet: CharacterSheet {
            name: "Test NPC".into(),
            description: "A test NPC".into(),
            personality: "Friendly".into(),
            scenario: "Test scenario".into(),
            example_dialogue: "Hello there!".into(),
            summary: None,
            profile_image: Some("data/images/npc.png".into()),
            headshot_image: Some("data/images/npc_headshot.png".into()),
        },
        inventory: vec![],
        triggers: vec![],
        relationships: vec![],
    }];

    let state = chronicler_engine::model::state::GameState::new(
        world.clone(),
        map,
        player,
        npcs,
        world.starting_room_id.clone(),
    );

    let storage = Arc::new(chronicler_engine::test_support::InMemorySnapshotStorage::new())
        as Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage>;
    let snapshot = chronicler_engine::model::state_snapshot::GameStateSnapshot::from_game_state(
        &state,
        "initial".to_string(),
        0,
    );
    storage.save(&snapshot).unwrap();

    let app = chronicler_engine::server::create_app_with_storage(
        state,
        storage,
        Arc::new(chronicler_engine::storage::llm_message_storage::InMemoryLlmMessageStorage::new()),
        chronicler_engine::model::settings::AppSettings::default(),
    );

    // Reset the game
    let req = Request::builder()
        .uri("/reset")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert!(response.status().is_success());

    // Check that the scenario NPC appears in the visual sidebar
    let req = Request::builder()
        .uri("/fragment/visual-sidebar")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), 4096)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Test NPC"),
        "Reset should preserve scenario NPCs in sidebar: {body_str}"
    );
}

#[tokio::test]
async fn test_reset_allows_subsequent_actions() {
    let state = create_test_state();
    let app = chronicler_engine::create_app_for_testing(state);

    // Reset the game
    let req = Request::builder()
        .uri("/reset")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert!(response.status().is_success());

    // Post an async action — should be accepted, not rejected
    let req = Request::builder()
        .uri("/action")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=look+around"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Thinking"),
        "After reset, async actions should be accepted: {body_str}"
    );
    assert!(
        !body_str.contains("Server is shutting down"),
        "After reset, cancel_token should be fresh: {body_str}"
    );
}
