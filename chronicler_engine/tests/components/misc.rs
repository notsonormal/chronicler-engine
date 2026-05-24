use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use std::sync::Arc;
use tower::util::ServiceExt;

use chronicler_engine::create_app_for_testing_with_settings;
use chronicler_engine::model::settings::{AppSettings, TextCheckMode, TextCheckSettings};
use chronicler_engine::model::state::{GameState, LogType};
use chronicler_engine::storage::message_storage::MessageStorage;
use chronicler_engine::storage::snapshot_storage::SnapshotStorage;

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
    let app = chronicler_engine::create_app_for_testing(state_with_input());

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
    let mut state = create_test_state();
    state.add_log(
        "look around".to_string(),
        Some("Test Player".to_string()),
        LogType::Input,
    );
    let snapshot =
        chronicler_engine::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    let storage = Arc::new(chronicler_engine::test_support::InMemoryGameStorage::new());
    let _ = storage.save(&snapshot);
    for mut msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        let _ = storage.insert_message(&mut msg);
    }

    let llm_storage =
        Arc::new(chronicler_engine::storage::llm_message_storage::InMemoryLlmMessageStorage::new())
            as Arc<dyn chronicler_engine::storage::llm_message_storage::LlmMessageStorage>;

    let app = chronicler_engine::server::create_app_with_storage(
        state,
        Arc::clone(&storage)
            as Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage>,
        Arc::clone(&storage)
            as Arc<dyn chronicler_engine::storage::message_storage::MessageStorage>,
        llm_storage,
        AppSettings::default(),
    );

    let req = Request::builder()
        .uri("/swipe/new")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let latest = storage
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
    // Setup: create app with InMemoryGameStorage so we can inspect it
    let mut state = create_test_state();
    state.add_log(
        "look around".to_string(),
        Some("Test Player".to_string()),
        LogType::Input,
    );
    let snapshot =
        chronicler_engine::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    let storage = Arc::new(chronicler_engine::test_support::InMemoryGameStorage::new());
    let _ = storage.save(&snapshot);
    for mut msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        let _ = storage.insert_message(&mut msg);
    }

    let llm_storage =
        Arc::new(chronicler_engine::storage::llm_message_storage::InMemoryLlmMessageStorage::new())
            as Arc<dyn chronicler_engine::storage::llm_message_storage::LlmMessageStorage>;

    let app = chronicler_engine::server::create_app_with_storage(
        state,
        Arc::clone(&storage)
            as Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage>,
        Arc::clone(&storage)
            as Arc<dyn chronicler_engine::storage::message_storage::MessageStorage>,
        llm_storage,
        AppSettings::default(),
    );

    let req = Request::builder()
        .uri("/swipe/new")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // The handler should save a new generating snapshot.
    let latest = storage
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

    let storage = Arc::new(chronicler_engine::test_support::InMemoryGameStorage::new());
    let snapshot =
        chronicler_engine::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    storage.save(&snapshot).unwrap();

    let app = chronicler_engine::server::create_app_with_storage(
        state,
        Arc::clone(&storage)
            as Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage>,
        Arc::clone(&storage)
            as Arc<dyn chronicler_engine::storage::message_storage::MessageStorage>,
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

    let state = chronicler_engine::model::state::GameState::new(
        world.clone(),
        map,
        player,
        vec![],
        world.starting_room_id.clone(),
    );

    let storage = Arc::new(chronicler_engine::test_support::InMemoryGameStorage::new());
    let snapshot =
        chronicler_engine::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    storage.save(&snapshot).unwrap();

    let app = chronicler_engine::server::create_app_with_storage(
        state,
        Arc::clone(&storage)
            as Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage>,
        Arc::clone(&storage)
            as Arc<dyn chronicler_engine::storage::message_storage::MessageStorage>,
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

#[tokio::test]
async fn test_retry_handler_load_state_failure() {
    let state = state_with_input();
    let storage = Arc::new(chronicler_engine::test_support::InMemoryGameStorage::new());

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

        fn list_games(
            &self,
        ) -> Result<Vec<chronicler_engine::model::game::Game>, chronicler_engine::error::EngineError>
        {
            self.inner.list_games()
        }
        fn create_game(
            &self,
            world_name: &str,
            name: &str,
        ) -> Result<u64, chronicler_engine::error::EngineError> {
            self.inner.create_game(world_name, name)
        }
        fn delete_game(&self, id: u64) -> Result<(), chronicler_engine::error::EngineError> {
            self.inner.delete_game(id)
        }
        fn get_game(
            &self,
            id: u64,
        ) -> Result<
            Option<chronicler_engine::model::game::Game>,
            chronicler_engine::error::EngineError,
        > {
            self.inner.get_game(id)
        }
    }

    let storage_dyn: Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage> =
        Arc::clone(&storage)
            as Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage>;
    let snapshot_storage: Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage> =
        Arc::new(FailingLoadStorage { inner: storage_dyn });
    let message_storage: Arc<dyn chronicler_engine::storage::message_storage::MessageStorage> =
        Arc::clone(&storage)
            as Arc<dyn chronicler_engine::storage::message_storage::MessageStorage>;
    let llm_storage =
        Arc::new(chronicler_engine::storage::llm_message_storage::InMemoryLlmMessageStorage::new())
            as Arc<dyn chronicler_engine::storage::llm_message_storage::LlmMessageStorage>;

    let app = chronicler_engine::server::create_app_with_storage(
        state,
        snapshot_storage,
        message_storage,
        llm_storage,
        AppSettings::default(),
    );

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
    let app = chronicler_engine::create_app_for_testing(create_test_state());

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
    let mut state = create_test_state();
    state.narrative.last_trigger = Some(chronicler_engine::model::state::StoredTriggerContext {
        npc_id: "test_npc".to_string(),
        trigger_idx: 0,
        trigger_name: "Greeting".to_string(),
        trigger_repeat: false,
        trigger_narration_prompt: "Hello".to_string(),
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        max_tokens: None,
    });

    let storage = Arc::new(chronicler_engine::test_support::InMemoryGameStorage::new());
    let snapshot =
        chronicler_engine::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    storage.save(&snapshot).unwrap();
    // Intentionally do NOT insert any messages

    let llm_storage =
        Arc::new(chronicler_engine::storage::llm_message_storage::InMemoryLlmMessageStorage::new())
            as Arc<dyn chronicler_engine::storage::llm_message_storage::LlmMessageStorage>;

    let app = chronicler_engine::server::create_app_with_storage(
        state,
        Arc::clone(&storage) as Arc<dyn SnapshotStorage>,
        Arc::clone(&storage) as Arc<dyn MessageStorage>,
        llm_storage,
        AppSettings::default(),
    );

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
    let mut state = create_test_state();
    state.narrative.last_trigger = Some(chronicler_engine::model::state::StoredTriggerContext {
        npc_id: "test_npc".to_string(),
        trigger_idx: 0,
        trigger_name: "Greeting".to_string(),
        trigger_repeat: false,
        trigger_narration_prompt: "Hello".to_string(),
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        max_tokens: None,
    });
    state.add_log(
        "look around".to_string(),
        Some("Test Player".to_string()),
        LogType::Input,
    );

    let storage = Arc::new(chronicler_engine::test_support::InMemoryGameStorage::new());
    let snapshot =
        chronicler_engine::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    storage.save(&snapshot).unwrap();
    for mut msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        let _ = storage.insert_message(&mut msg);
    }

    let llm_storage =
        Arc::new(chronicler_engine::storage::llm_message_storage::InMemoryLlmMessageStorage::new())
            as Arc<dyn chronicler_engine::storage::llm_message_storage::LlmMessageStorage>;

    let app = chronicler_engine::server::create_app_with_storage(
        state,
        Arc::clone(&storage) as Arc<dyn SnapshotStorage>,
        Arc::clone(&storage) as Arc<dyn MessageStorage>,
        llm_storage,
        AppSettings::default(),
    );

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
    let app = chronicler_engine::create_app_for_testing(create_test_state());

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
    let mut state = create_test_state();
    state.add_log(
        "look around".to_string(),
        Some("Test Player".to_string()),
        LogType::Input,
    );

    let storage = Arc::new(chronicler_engine::test_support::InMemoryGameStorage::new());
    let snapshot =
        chronicler_engine::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    storage.save(&snapshot).unwrap();

    // Insert two messages so the first is NOT the last
    let mut msg1 = chronicler_engine::model::message::Message::new(
        None,
        "First narration",
        LogType::Narration,
        None,
        None,
    );
    storage.insert_message(&mut msg1).unwrap();

    let mut msg2 = chronicler_engine::model::message::Message::new(
        None,
        "Second narration",
        LogType::Narration,
        None,
        None,
    );
    storage.insert_message(&mut msg2).unwrap();

    let llm_storage =
        Arc::new(chronicler_engine::storage::llm_message_storage::InMemoryLlmMessageStorage::new())
            as Arc<dyn chronicler_engine::storage::llm_message_storage::LlmMessageStorage>;

    let app = chronicler_engine::server::create_app_with_storage(
        state,
        Arc::clone(&storage) as Arc<dyn SnapshotStorage>,
        Arc::clone(&storage) as Arc<dyn MessageStorage>,
        llm_storage,
        AppSettings::default(),
    );

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

    let mut state = create_test_state();
    state.add_log(
        "look around".to_string(),
        Some("Test Player".to_string()),
        LogType::Input,
    );

    let storage = Arc::new(chronicler_engine::test_support::InMemoryGameStorage::new());
    let snapshot =
        chronicler_engine::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    storage.save(&snapshot).unwrap();

    // Insert a narration with a swipe that has NO snapshot_id
    let mut msg = Message::new(None, "Narration", LogType::Narration, None, None);
    msg.swipes.push(Swipe {
        text: "Second swipe".to_string(),
        snapshot_id: None, // Deliberately missing
        location_header: None,
        event_header: None,
    });
    storage.insert_message(&mut msg).unwrap();

    let llm_storage =
        Arc::new(chronicler_engine::storage::llm_message_storage::InMemoryLlmMessageStorage::new())
            as Arc<dyn chronicler_engine::storage::llm_message_storage::LlmMessageStorage>;

    let app = chronicler_engine::server::create_app_with_storage(
        state,
        Arc::clone(&storage) as Arc<dyn SnapshotStorage>,
        Arc::clone(&storage) as Arc<dyn MessageStorage>,
        llm_storage,
        AppSettings::default(),
    );

    let req = Request::builder()
        .uri(format!("/message/{}/swipe/1", msg.id))
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
    use chronicler_engine::model::message::{Message, Swipe};
    use chronicler_engine::model::state::LogType;
    use chronicler_engine::model::state_snapshot::GameStateSnapshot;
    use chronicler_engine::storage::message_storage::MessageStorage;
    use chronicler_engine::storage::snapshot_storage::SnapshotStorage;

    let mut state = create_test_state();
    state.add_log(
        "look around".to_string(),
        Some("Test Player".to_string()),
        LogType::Input,
    );

    let mut narration = Message::new(None, "First narration", LogType::Narration, None, None);

    // Build state that includes the narration so snapshot covers it.
    state.narrative.history.append(narration.clone());

    let storage = Arc::new(chronicler_engine::test_support::InMemoryGameStorage::new());

    let snapshot = GameStateSnapshot::from_game_state(&state);
    let snapshot_id = storage.save(&snapshot).unwrap();
    narration.snapshot_id = Some(snapshot_id);
    narration.swipes[0].snapshot_id = Some(snapshot_id);

    for mut msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        if msg.log_type == LogType::Narration {
            msg.snapshot_id = Some(snapshot_id);
            if let Some(swipe) = msg.swipes.first_mut() {
                swipe.snapshot_id = Some(snapshot_id);
            }
        }
        storage.insert_message(&mut msg).unwrap();
    }

    let narration_id = storage
        .load_messages()
        .unwrap()
        .into_iter()
        .find(|m| m.log_type == LogType::Narration)
        .map(|m| m.id)
        .expect("narration message should exist");

    let swipe1 = Swipe {
        text: "Second narration".to_string(),
        snapshot_id: Some(snapshot_id),
        location_header: None,
        event_header: None,
    };
    storage.insert_swipe(narration_id, &swipe1, 1).unwrap();
    storage.update_active_swipe(narration_id, 1).unwrap();

    let llm_storage =
        Arc::new(chronicler_engine::storage::llm_message_storage::InMemoryLlmMessageStorage::new())
            as Arc<dyn chronicler_engine::storage::llm_message_storage::LlmMessageStorage>;

    let app = chronicler_engine::server::create_app_with_storage(
        state,
        Arc::clone(&storage) as Arc<dyn SnapshotStorage>,
        Arc::clone(&storage) as Arc<dyn MessageStorage>,
        llm_storage,
        AppSettings::default(),
    );

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
    let mut state = create_test_state();
    state.add_log(
        "look around".to_string(),
        Some("Test Player".to_string()),
        LogType::Input,
    );
    let storage = Arc::new(chronicler_engine::test_support::InMemoryGameStorage::new());
    let snapshot =
        chronicler_engine::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    storage.save(&snapshot).unwrap();
    for mut msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        let _ = storage.insert_message(&mut msg);
    }

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

        fn list_games(
            &self,
        ) -> Result<Vec<chronicler_engine::model::game::Game>, chronicler_engine::error::EngineError>
        {
            self.inner.list_games()
        }
        fn create_game(
            &self,
            world_name: &str,
            name: &str,
        ) -> Result<u64, chronicler_engine::error::EngineError> {
            self.inner.create_game(world_name, name)
        }
        fn delete_game(&self, id: u64) -> Result<(), chronicler_engine::error::EngineError> {
            self.inner.delete_game(id)
        }
        fn get_game(
            &self,
            id: u64,
        ) -> Result<
            Option<chronicler_engine::model::game::Game>,
            chronicler_engine::error::EngineError,
        > {
            self.inner.get_game(id)
        }
    }

    let storage_dyn: Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage> =
        Arc::clone(&storage)
            as Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage>;
    let snapshot_storage: Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage> =
        Arc::new(FailingSaveStorage { inner: storage_dyn });
    let message_storage: Arc<dyn chronicler_engine::storage::message_storage::MessageStorage> =
        Arc::clone(&storage)
            as Arc<dyn chronicler_engine::storage::message_storage::MessageStorage>;
    let llm_storage =
        Arc::new(chronicler_engine::storage::llm_message_storage::InMemoryLlmMessageStorage::new())
            as Arc<dyn chronicler_engine::storage::llm_message_storage::LlmMessageStorage>;

    let app = chronicler_engine::server::create_app_with_storage(
        state,
        snapshot_storage,
        message_storage,
        llm_storage,
        AppSettings::default(),
    );

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
    let state = create_test_state();
    let storage = Arc::new(chronicler_engine::test_support::InMemoryGameStorage::new());

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

        fn list_games(
            &self,
        ) -> Result<Vec<chronicler_engine::model::game::Game>, chronicler_engine::error::EngineError>
        {
            self.inner.list_games()
        }
        fn create_game(
            &self,
            world_name: &str,
            name: &str,
        ) -> Result<u64, chronicler_engine::error::EngineError> {
            self.inner.create_game(world_name, name)
        }
        fn delete_game(&self, id: u64) -> Result<(), chronicler_engine::error::EngineError> {
            self.inner.delete_game(id)
        }
        fn get_game(
            &self,
            id: u64,
        ) -> Result<
            Option<chronicler_engine::model::game::Game>,
            chronicler_engine::error::EngineError,
        > {
            self.inner.get_game(id)
        }
    }

    let storage_dyn: Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage> =
        Arc::clone(&storage)
            as Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage>;
    let snapshot_storage: Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage> =
        Arc::new(FailingSaveStorage { inner: storage_dyn });
    let message_storage: Arc<dyn chronicler_engine::storage::message_storage::MessageStorage> =
        Arc::clone(&storage)
            as Arc<dyn chronicler_engine::storage::message_storage::MessageStorage>;
    let llm_storage =
        Arc::new(chronicler_engine::storage::llm_message_storage::InMemoryLlmMessageStorage::new())
            as Arc<dyn chronicler_engine::storage::llm_message_storage::LlmMessageStorage>;

    let app = chronicler_engine::server::create_app_with_storage(
        state,
        snapshot_storage,
        message_storage,
        llm_storage,
        AppSettings::default(),
    );

    let req = Request::builder()
        .uri("/reset")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
