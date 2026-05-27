use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use std::sync::Arc;
use tower::util::ServiceExt;

use chronicler_engine::TestAppBuilder;
use chronicler_engine::model::settings::{AppSettings, TextCheckMode, TextCheckSettings};
use chronicler_engine::model::state::LogType;

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
    let snapshot_storage: Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage> =
        Arc::new(chronicler_engine::test_support::InMemorySnapshotRepository::new());
    let message_storage: Arc<dyn chronicler_engine::storage::message_storage::MessageStorage> =
        Arc::new(chronicler_engine::test_support::InMemoryMessageRepository::new());

    let app = TestAppBuilder::default_test()
        .log("look around", Some("Test Player"), LogType::Input)
        .snapshot_storage(Arc::clone(&snapshot_storage))
        .message_storage(Arc::clone(&message_storage))
        .build();

    let req = Request::builder()
        .uri("/swipe/new")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let latest = snapshot_storage
        .load_latest()
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
    let snapshot_storage: Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage> =
        Arc::new(chronicler_engine::test_support::InMemorySnapshotRepository::new());
    let message_storage: Arc<dyn chronicler_engine::storage::message_storage::MessageStorage> =
        Arc::new(chronicler_engine::test_support::InMemoryMessageRepository::new());

    let app = TestAppBuilder::default_test()
        .log("look around", Some("Test Player"), LogType::Input)
        .snapshot_storage(Arc::clone(&snapshot_storage))
        .message_storage(Arc::clone(&message_storage))
        .build();

    let req = Request::builder()
        .uri("/swipe/new")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // The handler should save a new generating snapshot.
    let latest = snapshot_storage
        .load_latest()
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

    let snapshot_storage: Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage> =
        Arc::new(chronicler_engine::test_support::InMemorySnapshotRepository::new());
    let message_storage: Arc<dyn chronicler_engine::storage::message_storage::MessageStorage> =
        Arc::new(chronicler_engine::test_support::InMemoryMessageRepository::new());

    let app = TestAppBuilder::new((*world).clone(), (*player).clone())
        .map((*map).clone())
        .npcs(npcs)
        .snapshot_storage(Arc::clone(&snapshot_storage))
        .message_storage(Arc::clone(&message_storage))
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

    let snapshot_storage: Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage> =
        Arc::new(chronicler_engine::test_support::InMemorySnapshotRepository::new());
    let message_storage: Arc<dyn chronicler_engine::storage::message_storage::MessageStorage> =
        Arc::new(chronicler_engine::test_support::InMemoryMessageRepository::new());

    let app = TestAppBuilder::new((*world).clone(), (*player).clone())
        .map((*map).clone())
        .snapshot_storage(Arc::clone(&snapshot_storage))
        .message_storage(Arc::clone(&message_storage))
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

#[tokio::test]
async fn test_retry_handler_load_state_failure() {
    let snapshot_storage_inner: Arc<
        dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage,
    > = Arc::new(chronicler_engine::test_support::InMemorySnapshotRepository::new());
    let message_storage: Arc<dyn chronicler_engine::storage::message_storage::MessageStorage> =
        Arc::new(chronicler_engine::test_support::InMemoryMessageRepository::new());

    struct FailingLoadStorage {
        inner: Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage>,
    }

    impl chronicler_engine::storage::snapshot_storage::SnapshotStorage for FailingLoadStorage {
        fn save(
            &self,
            snapshot: &chronicler_engine::model::state_snapshot::GameStateSnapshot,
        ) -> Result<u64, chronicler_engine::error::EngineError> {
            self.inner.save(snapshot)
        }
        fn load_latest(
            &self,
        ) -> Result<
            Option<chronicler_engine::model::state_snapshot::GameStateSnapshot>,
            chronicler_engine::error::EngineError,
        > {
            Err(chronicler_engine::error::EngineError::Internal(
                chronicler_engine::error::internal_error("simulated load failure"),
            ))
        }
        fn load_by_id(
            &self,
            id: u64,
        ) -> Result<
            Option<chronicler_engine::model::state_snapshot::GameStateSnapshot>,
            chronicler_engine::error::EngineError,
        > {
            self.inner.load_by_id(id)
        }
        fn set_game_id(&self, game_id: u64) {
            self.inner.set_game_id(game_id);
        }

        fn current_game_id(&self) -> u64 {
            self.inner.current_game_id()
        }
    }

    let storage_dyn = Arc::clone(&snapshot_storage_inner);
    let snapshot_storage: Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage> =
        Arc::new(FailingLoadStorage { inner: storage_dyn });
    let llm_storage =
        Arc::new(chronicler_engine::storage::llm_message_storage::InMemoryLlmMessageStorage::new())
            as Arc<dyn chronicler_engine::storage::llm_message_storage::LlmMessageStorage>;

    let app = TestAppBuilder::default_test()
        .log("look around", Some("Test Player"), LogType::Input)
        .snapshot_storage(snapshot_storage)
        .message_storage(message_storage)
        .llm_storage(llm_storage)
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
    let snapshot_storage: Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage> =
        Arc::new(chronicler_engine::test_support::InMemorySnapshotRepository::new());
    let message_storage: Arc<dyn chronicler_engine::storage::message_storage::MessageStorage> =
        Arc::new(chronicler_engine::test_support::InMemoryMessageRepository::new());
    let llm_storage =
        Arc::new(chronicler_engine::storage::llm_message_storage::InMemoryLlmMessageStorage::new())
            as Arc<dyn chronicler_engine::storage::llm_message_storage::LlmMessageStorage>;

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
        .snapshot_storage(Arc::clone(&snapshot_storage))
        .message_storage(Arc::clone(&message_storage))
        .llm_storage(llm_storage)
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
    let snapshot_storage: Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage> =
        Arc::new(chronicler_engine::test_support::InMemorySnapshotRepository::new());
    let message_storage: Arc<dyn chronicler_engine::storage::message_storage::MessageStorage> =
        Arc::new(chronicler_engine::test_support::InMemoryMessageRepository::new());
    let llm_storage =
        Arc::new(chronicler_engine::storage::llm_message_storage::InMemoryLlmMessageStorage::new())
            as Arc<dyn chronicler_engine::storage::llm_message_storage::LlmMessageStorage>;

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
        .snapshot_storage(Arc::clone(&snapshot_storage))
        .message_storage(Arc::clone(&message_storage))
        .llm_storage(llm_storage)
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
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/action")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=go north"))
        .unwrap();
    let _response = app.clone().oneshot(req).await.unwrap();

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
    let snapshot_storage: Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage> =
        Arc::new(chronicler_engine::test_support::InMemorySnapshotRepository::new());
    let message_storage: Arc<dyn chronicler_engine::storage::message_storage::MessageStorage> =
        Arc::new(chronicler_engine::test_support::InMemoryMessageRepository::new());

    let app = TestAppBuilder::default_test()
        .log("look around", Some("Test Player"), LogType::Input)
        .snapshot_storage(Arc::clone(&snapshot_storage))
        .message_storage(Arc::clone(&message_storage))
        .build();

    // Insert two messages so the first is NOT the last
    let msg1 = chronicler_engine::model::message::Message::new(
        None,
        "First narration",
        LogType::Narration,
        None,
        None,
    );
    message_storage.insert_message(&msg1).unwrap();

    let msg2 = chronicler_engine::model::message::Message::new(
        None,
        "Second narration",
        LogType::Narration,
        None,
        None,
    );
    message_storage.insert_message(&msg2).unwrap();

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

    let snapshot_storage: Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage> =
        Arc::new(chronicler_engine::test_support::InMemorySnapshotRepository::new());
    let message_storage: Arc<dyn chronicler_engine::storage::message_storage::MessageStorage> =
        Arc::new(chronicler_engine::test_support::InMemoryMessageRepository::new());

    let message_swipe_storage: Arc<
        dyn chronicler_engine::storage::message_swipe_storage::MessageSwipeStorage,
    > = Arc::new(chronicler_engine::test_support::InMemoryMessageSwipeStorage::new());

    let app = TestAppBuilder::default_test()
        .log("look around", Some("Test Player"), LogType::Input)
        .snapshot_storage(Arc::clone(&snapshot_storage))
        .message_storage(Arc::clone(&message_storage))
        .message_swipe_storage(Arc::clone(&message_swipe_storage))
        .build();

    // Insert a narration with a swipe that has NO snapshot_id
    let msg = Message::new(None, "Narration", LogType::Narration, None, None);
    let id = message_storage.insert_message(&msg).unwrap();
    let swipe0 = msg.swipes.first().unwrap().clone();
    message_swipe_storage.insert_swipe(id, &swipe0, 0).unwrap();
    let swipe1 = Swipe {
        text: "Second swipe".to_string(),
        snapshot_id: None, // Deliberately missing
        location_header: None,
        event_header: None,
    };
    message_swipe_storage.insert_swipe(id, &swipe1, 1).unwrap();

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

    let snapshot_storage: Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage> =
        Arc::new(chronicler_engine::test_support::InMemorySnapshotRepository::new());
    let message_storage: Arc<dyn chronicler_engine::storage::message_storage::MessageStorage> =
        Arc::new(chronicler_engine::test_support::InMemoryMessageRepository::new());

    let message_swipe_storage: Arc<
        dyn chronicler_engine::storage::message_swipe_storage::MessageSwipeStorage,
    > = Arc::new(chronicler_engine::test_support::InMemoryMessageSwipeStorage::new());

    let app = TestAppBuilder::default_test()
        .log("look around", Some("Test Player"), LogType::Input)
        .log("First narration", None, LogType::Narration)
        .snapshot_storage(Arc::clone(&snapshot_storage))
        .message_storage(Arc::clone(&message_storage))
        .message_swipe_storage(Arc::clone(&message_swipe_storage))
        .build();

    // After build, fix up narration message with snapshot_id
    let latest = snapshot_storage
        .load_latest()
        .unwrap()
        .expect("Should have snapshot");
    let snapshot_id = latest.db_id.expect("Should have snapshot id");

    let msgs = message_storage.load_message_rows().unwrap();
    let narration = msgs
        .into_iter()
        .find(|m| m.log_type == LogType::Narration)
        .expect("narration message should exist");
    let old_id = narration.id;
    message_storage.delete_message(old_id).unwrap();

    let mut updated = narration.clone();
    updated.snapshot_id = Some(snapshot_id);
    let narration_id = message_storage.insert_message(&updated).unwrap();

    let swipe0 = Swipe {
        text: "First narration".to_string(),
        snapshot_id: Some(snapshot_id),
        location_header: None,
        event_header: None,
    };
    message_swipe_storage
        .insert_swipe(narration_id, &swipe0, 0)
        .unwrap();
    let swipe1 = Swipe {
        text: "Second narration".to_string(),
        snapshot_id: Some(snapshot_id),
        location_header: None,
        event_header: None,
    };
    message_swipe_storage
        .insert_swipe(narration_id, &swipe1, 1)
        .unwrap();

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
    let snapshot_storage_inner: Arc<
        dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage,
    > = Arc::new(chronicler_engine::test_support::InMemorySnapshotRepository::new());
    let message_storage: Arc<dyn chronicler_engine::storage::message_storage::MessageStorage> =
        Arc::new(chronicler_engine::test_support::InMemoryMessageRepository::new());

    struct FailingSaveStorage {
        inner: Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage>,
    }

    impl chronicler_engine::storage::snapshot_storage::SnapshotStorage for FailingSaveStorage {
        fn save(
            &self,
            _snapshot: &chronicler_engine::model::state_snapshot::GameStateSnapshot,
        ) -> Result<u64, chronicler_engine::error::EngineError> {
            Err(chronicler_engine::error::EngineError::Internal(
                chronicler_engine::error::internal_error("simulated save failure"),
            ))
        }
        fn load_latest(
            &self,
        ) -> Result<
            Option<chronicler_engine::model::state_snapshot::GameStateSnapshot>,
            chronicler_engine::error::EngineError,
        > {
            self.inner.load_latest()
        }
        fn load_by_id(
            &self,
            id: u64,
        ) -> Result<
            Option<chronicler_engine::model::state_snapshot::GameStateSnapshot>,
            chronicler_engine::error::EngineError,
        > {
            self.inner.load_by_id(id)
        }
        fn set_game_id(&self, game_id: u64) {
            self.inner.set_game_id(game_id);
        }

        fn current_game_id(&self) -> u64 {
            self.inner.current_game_id()
        }
    }

    let storage_dyn = Arc::clone(&snapshot_storage_inner);
    let snapshot_storage: Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage> =
        Arc::new(FailingSaveStorage { inner: storage_dyn });
    let llm_storage =
        Arc::new(chronicler_engine::storage::llm_message_storage::InMemoryLlmMessageStorage::new())
            as Arc<dyn chronicler_engine::storage::llm_message_storage::LlmMessageStorage>;

    let app = TestAppBuilder::default_test()
        .log("look around", Some("Test Player"), LogType::Input)
        .snapshot_storage(snapshot_storage)
        .message_storage(message_storage)
        .llm_storage(llm_storage)
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
    let snapshot_storage_inner: Arc<
        dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage,
    > = Arc::new(chronicler_engine::test_support::InMemorySnapshotRepository::new());
    let message_storage: Arc<dyn chronicler_engine::storage::message_storage::MessageStorage> =
        Arc::new(chronicler_engine::test_support::InMemoryMessageRepository::new());

    struct FailingSaveStorage {
        inner: Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage>,
    }

    impl chronicler_engine::storage::snapshot_storage::SnapshotStorage for FailingSaveStorage {
        fn save(
            &self,
            _snapshot: &chronicler_engine::model::state_snapshot::GameStateSnapshot,
        ) -> Result<u64, chronicler_engine::error::EngineError> {
            Err(chronicler_engine::error::EngineError::Internal(
                chronicler_engine::error::internal_error("simulated save failure"),
            ))
        }
        fn load_latest(
            &self,
        ) -> Result<
            Option<chronicler_engine::model::state_snapshot::GameStateSnapshot>,
            chronicler_engine::error::EngineError,
        > {
            self.inner.load_latest()
        }
        fn load_by_id(
            &self,
            id: u64,
        ) -> Result<
            Option<chronicler_engine::model::state_snapshot::GameStateSnapshot>,
            chronicler_engine::error::EngineError,
        > {
            self.inner.load_by_id(id)
        }
        fn set_game_id(&self, game_id: u64) {
            self.inner.set_game_id(game_id);
        }

        fn current_game_id(&self) -> u64 {
            self.inner.current_game_id()
        }
    }

    let storage_dyn = Arc::clone(&snapshot_storage_inner);
    let snapshot_storage: Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage> =
        Arc::new(FailingSaveStorage { inner: storage_dyn });
    let llm_storage =
        Arc::new(chronicler_engine::storage::llm_message_storage::InMemoryLlmMessageStorage::new())
            as Arc<dyn chronicler_engine::storage::llm_message_storage::LlmMessageStorage>;

    let app = TestAppBuilder::default_test()
        .snapshot_storage(snapshot_storage)
        .message_storage(message_storage)
        .llm_storage(llm_storage)
        .build();

    let req = Request::builder()
        .uri("/reset")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
