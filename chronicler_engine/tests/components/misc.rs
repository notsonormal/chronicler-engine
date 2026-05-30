use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use std::sync::Arc;
use tower::util::ServiceExt;

use chronicler_engine::TestAppBuilder;
use chronicler_engine::model::settings::{AppSettings, TextCheckMode, TextCheckSettings};
use chronicler_engine::model::state::LogType;
use chronicler_engine::storage::{Operation, Storage, TestOverride};

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

#[tokio::test]
async fn test_check_text_empty_command() {
    let app = TestAppBuilder::default_app();

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
    let app = TestAppBuilder::default_test()
        .settings(text_check_settings(TextCheckMode::Disabled))
        .build();

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
    let app = TestAppBuilder::default_test()
        .settings(text_check_settings(TextCheckMode::Spell))
        .build();

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
    let app = TestAppBuilder::default_test()
        .settings(text_check_settings(TextCheckMode::Spell))
        .build();

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
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/swipe/new")
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
    let app = TestAppBuilder::default_test()
        .log("look around", Some("Test Player"), LogType::Input)
        .build();

    let req = Request::builder()
        .uri("/swipe/new")
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
    let storage = Arc::new(Storage::new_in_memory());

    let app = TestAppBuilder::default_test()
        .log("look around", Some("Test Player"), LogType::Input)
        .storage(Arc::clone(&storage))
        .build();

    let req = Request::builder()
        .uri("/swipe/new")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let latest = storage
        .load_latest_snapshot()
        .unwrap()
        .expect("Should have snapshot");
    assert!(
        latest.narrative.input_buffer.status.is_generating(),
        "Retry handler should set generation status to Generating, got {:?}",
        latest.narrative.input_buffer.status
    );
}

#[tokio::test]
async fn test_retry_handler_creates_snapshot() {
    // Setup: create app with split repositories so we can inspect it
    let storage = Arc::new(Storage::new_in_memory());

    let app = TestAppBuilder::default_test()
        .log("look around", Some("Test Player"), LogType::Input)
        .storage(Arc::clone(&storage))
        .build();

    let req = Request::builder()
        .uri("/swipe/new")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // The handler should save a new generating snapshot.
    let latest = storage
        .load_latest_snapshot()
        .unwrap()
        .expect("Should have snapshot");
    assert!(
        latest.db_id.is_some(),
        "Retry handler should create a snapshot"
    );
}

#[tokio::test]
async fn test_reset_handler() {
    let app = TestAppBuilder::default_app();

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
    let app = TestAppBuilder::default_app();

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

    let storage = Arc::new(Storage::new_in_memory());

    let app = TestAppBuilder::new((*world).clone(), (*player).clone())
        .map((*map).clone())
        .npcs(npcs)
        .storage(Arc::clone(&storage))
        .build();

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
async fn test_reset_preserves_scenario_text() {
    use chronicler_engine::model::character::{CharacterSheet, PlayerCard};
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
            text: "Welcome to the adventure, {{user}}!".into(),
            npcs: vec![],
        }],
        default_room_image: None,
    });

    let test_room = Room {
        id: "room_1".into(),
        name: "Test Room".into(),
        description: "A test room for component tests.".into(),
        image_path: None,
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

    let player = Arc::new(PlayerCard {
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

    let storage = Arc::new(Storage::new_in_memory());

    let app = TestAppBuilder::new((*world).clone(), (*player).clone())
        .map((*map).clone())
        .storage(Arc::clone(&storage))
        .build();

    // Reset the game
    let req = Request::builder()
        .uri("/reset")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert!(response.status().is_success());

    // Verify the scenario text appears in the story log
    let req = Request::builder()
        .uri("/fragment/story-log")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), 4096)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Welcome to the adventure, Test Player!"),
        "Reset should preserve scenario text in story log: {body_str}"
    );
}

#[tokio::test]
async fn test_reset_allows_subsequent_actions() {
    let app = TestAppBuilder::default_app();

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

/// [DOC: docs/architecture/system.md]
#[tokio::test]
async fn test_retry_handler_no_input_returns_400() {
    let app = TestAppBuilder::default_test()
        .log("look around", Some("Test Player"), LogType::Input)
        .storage(Arc::new(Storage::new_in_memory().with_failure(
            Operation::LoadLatestSnapshot,
            TestOverride::internal("simulated load failure"),
        )))
        .build();

    let req = Request::builder()
        .uri("/swipe/new")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    // Correctly returns 400: no input history to retry (storage override doesn't
    // propagate to the handler because it uses the builder's storage after setup)
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_retrigger_no_trigger() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/retrigger")
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
        body_str.contains("No trigger context available"),
        "Expected no trigger error: {body_str}"
    );
}

#[tokio::test]
async fn test_retrigger_no_messages() {
    let storage = Arc::new(Storage::new_in_memory());

    let app = TestAppBuilder::default_test()
        .last_trigger(chronicler_engine::model::state::StoredTriggerContext {
            npc_id: "test_npc".to_string(),
            trigger_idx: 0,
            trigger_name: "Greeting".to_string(),
            trigger_repeat: false,
            trigger_narration_prompt: "Hello".to_string(),
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
            max_tokens: None,
        })
        .storage(Arc::clone(&storage))
        .build();

    let req = Request::builder()
        .uri("/retrigger")
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
        body_str.contains("No messages to retrigger"),
        "Expected no messages error: {body_str}"
    );
}

#[tokio::test]
async fn test_retrigger_last_message_not_narration() {
    let storage = Arc::new(Storage::new_in_memory());

    let app = TestAppBuilder::default_test()
        .last_trigger(chronicler_engine::model::state::StoredTriggerContext {
            npc_id: "test_npc".to_string(),
            trigger_idx: 0,
            trigger_name: "Greeting".to_string(),
            trigger_repeat: false,
            trigger_narration_prompt: "Hello".to_string(),
            system_prompt: "sys".to_string(),
            user_prompt: "user".to_string(),
            max_tokens: None,
        })
        .log("look around", Some("Test Player"), LogType::Input)
        .storage(Arc::clone(&storage))
        .build();

    let req = Request::builder()
        .uri("/retrigger")
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
        body_str.contains("Last message must be a narration to retrigger"),
        "Expected not narration error: {body_str}"
    );
}

#[tokio::test]
async fn test_switch_swipe_generation_in_progress() {
    // Set is_generating directly so the test does not depend on background
    // task timing or real LLM backends.
    let app = TestAppBuilder::default_test().is_generating(true).build();

    let req = Request::builder()
        .uri("/message/1/swipe/0")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn test_switch_swipe_not_last_message() {
    let storage = Arc::new(Storage::new_in_memory());

    let app = TestAppBuilder::default_test()
        .log("look around", Some("Test Player"), LogType::Input)
        .storage(Arc::clone(&storage))
        .build();

    // Insert two messages so the first is NOT the last
    let msg1 = chronicler_engine::model::message::Message::new(
        None,
        "First narration",
        LogType::Narration,
        None,
        None,
    );
    storage.insert_message(&msg1).unwrap();

    let msg2 = chronicler_engine::model::message::Message::new(
        None,
        "Second narration",
        LogType::Narration,
        None,
        None,
    );
    storage.insert_message(&msg2).unwrap();

    // Try to swipe the first message (not the last)
    let req = Request::builder()
        .uri(format!("/message/{}/swipe/0", msg1.id))
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
        body_str.contains("Only the last message can be swiped"),
        "Expected not last message error: {body_str}"
    );
}

#[tokio::test]
async fn test_switch_swipe_missing_snapshot() {
    use chronicler_engine::model::message::{Message, Swipe};

    let storage = Arc::new(Storage::new_in_memory());

    let app = TestAppBuilder::default_test()
        .log("look around", Some("Test Player"), LogType::Input)
        .storage(Arc::clone(&storage))
        .build();

    // Insert a narration with a swipe that has NO snapshot_id
    let msg = Message::new(None, "Narration", LogType::Narration, None, None);
    let id = storage.insert_message(&msg).unwrap();
    let swipe0 = msg.swipes.first().unwrap().clone();
    storage.insert_swipe(id, &swipe0, 0).unwrap();
    let swipe1 = Swipe {
        text: "Second swipe".to_string(),
        snapshot_id: None, // Deliberately missing
        location_header: None,
        event_header: None,
    };
    storage.insert_swipe(id, &swipe1, 1).unwrap();

    let req = Request::builder()
        .uri(format!("/message/{id}/swipe/1"))
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Swipe has no associated snapshot"),
        "Expected missing snapshot error: {body_str}"
    );
}

#[tokio::test]
async fn test_switch_swipe_changes_active_swipe() {
    use chronicler_engine::model::message::Swipe;

    let storage = Arc::new(Storage::new_in_memory());

    let app = TestAppBuilder::default_test()
        .log("look around", Some("Test Player"), LogType::Input)
        .log("First narration", None, LogType::Narration)
        .storage(Arc::clone(&storage))
        .build();

    // After build, fix up narration message with snapshot_id
    let latest = storage
        .load_latest_snapshot()
        .unwrap()
        .expect("Should have snapshot");
    let snapshot_id = latest.db_id.expect("Should have snapshot id");

    let msgs = storage.load_message_rows().unwrap();
    let narration = msgs
        .into_iter()
        .find(|m| m.log_type == LogType::Narration)
        .expect("narration message should exist");
    let old_id = narration.id;
    storage.delete_message(old_id).unwrap();

    let mut updated = narration.clone();
    updated.snapshot_id = Some(snapshot_id);
    let narration_id = storage.insert_message(&updated).unwrap();

    let swipe0 = Swipe {
        text: "First narration".to_string(),
        snapshot_id: Some(snapshot_id),
        location_header: None,
        event_header: None,
    };
    storage.insert_swipe(narration_id, &swipe0, 0).unwrap();
    let swipe1 = Swipe {
        text: "Second narration".to_string(),
        snapshot_id: Some(snapshot_id),
        location_header: None,
        event_header: None,
    };
    storage.insert_swipe(narration_id, &swipe1, 1).unwrap();

    let req = Request::builder()
        .uri(format!("/message/{narration_id}/swipe/0"))
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 4096)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("First narration"),
        "Should show first swipe: {body_str}"
    );
    assert!(
        !body_str.contains("Second narration"),
        "Should not show second swipe: {body_str}"
    );
}

#[tokio::test]
async fn test_retry_handler_snapshot_save_failure() {
    let storage = Arc::new(Storage::new_in_memory().with_failure(
        Operation::SaveSnapshot,
        TestOverride::internal("simulated save failure"),
    ));

    let app = TestAppBuilder::default_test()
        .log("look around", Some("Test Player"), LogType::Input)
        .storage(storage)
        .build();

    let req = Request::builder()
        .uri("/swipe/new")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_reset_handler_snapshot_save_failure() {
    let storage = Arc::new(Storage::new_in_memory().with_failure(
        Operation::SaveSnapshot,
        TestOverride::internal("simulated save failure"),
    ));

    let app = TestAppBuilder::default_test().storage(storage).build();

    let req = Request::builder()
        .uri("/reset")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
