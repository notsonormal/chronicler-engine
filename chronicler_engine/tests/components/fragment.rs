use std::sync::Arc;

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use tower::util::ServiceExt;

use chronicler_engine::TestAppBuilder;
use chronicler_engine::model::state::{GenerationPhase, GenerationStatus, LogType};
use chronicler_engine::storage::message_storage::MessageStorage;
use chronicler_engine::storage::snapshot_storage::SnapshotStorage;

async fn fetch_body(app: Router, uri: &str) -> String {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert!(response.status().is_success(), "Expected success for {uri}");
    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    String::from_utf8_lossy(&body).to_string()
}

#[tokio::test]
async fn test_header_fragment_returns_html() {
    let body = fetch_body(TestAppBuilder::default_app(), "/fragment/header").await;
    assert!(body.contains("class=\"header\""));
    assert!(body.contains("Chronicler Engine"));
}

#[tokio::test]
async fn test_story_log_fragment_returns_html() {
    let body = fetch_body(TestAppBuilder::default_app(), "/fragment/story-log").await;
    assert!(body.contains("id=\"story-log\""));
}

#[tokio::test]
async fn test_visual_sidebar_fragment_returns_html() {
    let body = fetch_body(TestAppBuilder::default_app(), "/fragment/visual-sidebar").await;
    assert!(body.contains("id=\"visual-sidebar\""));
}

#[tokio::test]
async fn test_visual_sidebar_renders_room_image() {
    let body = fetch_body(TestAppBuilder::default_app(), "/fragment/visual-sidebar").await;
    // Should contain the image, not "No Location Image"
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

    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Enter a command"),
        "Expected empty command error: {body_str}"
    );
}

#[tokio::test]
async fn test_hints_handler() {
    let body = fetch_body(TestAppBuilder::default_app(), "/hints").await;
    assert!(body.is_empty(), "Expected empty hints but got: {body}");
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
    // The test state has npc_1 with a profile_image, so headshots should render
    assert!(
        body.contains("headshot"),
        "Expected headshot in fragment: {body}"
    );
}

#[tokio::test]
async fn test_generating_status_handler_idle() {
    let body = fetch_body(TestAppBuilder::default_app(), "/status/generating").await;
    // Should return "idle" when not generating
    assert!(body.contains("idle"));
}

#[tokio::test]
async fn test_generating_status_handler_narrating() {
    let body = fetch_body(
        TestAppBuilder::default_test()
            .generation_status(GenerationStatus::Generating, GenerationPhase::Narrating)
            .build(),
        "/status/generating",
    )
    .await;
    assert!(body.contains("narrating"));
}

#[tokio::test]
async fn test_generating_status_handler_quantifying() {
    let body = fetch_body(
        TestAppBuilder::default_test()
            .generation_status(GenerationStatus::Generating, GenerationPhase::Quantifying)
            .build(),
        "/status/generating",
    )
    .await;
    assert!(body.contains("quantifying"));
}

#[tokio::test]
async fn test_reset_generating_handler() {
    let app = TestAppBuilder::default_app();

    // reset-generating is POST, not GET
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
    // Should return "reset" on success
    assert!(body_str.contains("reset"));
}

#[tokio::test]
async fn test_edit_history_handler_success() {
    let snapshot_storage: Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage> =
        Arc::new(chronicler_engine::test_support::InMemorySnapshotRepository::new());
    let message_storage: Arc<dyn chronicler_engine::storage::message_storage::MessageStorage> =
        Arc::new(chronicler_engine::test_support::InMemoryMessageRepository::new());

    let app = TestAppBuilder::default_test()
        .log("Original text", Some("Test"), LogType::Narration)
        .snapshot_storage(Arc::clone(&snapshot_storage))
        .message_storage(Arc::clone(&message_storage))
        .build();

    let entry_id = message_storage.load_messages().unwrap().last().unwrap().id;

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

    // Try to edit a non-existent log entry (ID 9999) - correct path is /history/:id
    let req = Request::builder()
        .uri("/history/9999")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("text=Edited text"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    // Should return NOT_FOUND for non-existent entry
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_history_handler_success() {
    let app = TestAppBuilder::default_test()
        .log("Test message", Some("Test"), LogType::Narration)
        .build();

    let req = Request::builder()
        .uri("/history/delete")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify the entry was deleted by fetching the story log fragment
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

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
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

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Enter a command"),
        "Expected empty command error: {body_str}"
    );
}

#[tokio::test]
async fn test_action_concurrent_rejection() {
    // Use a mock backend with a small delay so the first action remains
    // in-flight when the second request arrives, ensuring deterministic
    // concurrent rejection.
    let llm: Arc<dyn chronicler_engine::narrative::llm::LlmBackend> =
        Arc::new(chronicler_engine::narrative::llm::MockBackend::with_delay(50));
    let game_service = Arc::new(chronicler_engine::application::game_service::DefaultGameService::with_mock_quantifier(
        Arc::clone(&llm),
        Arc::new(chronicler_engine::narrative::llm::MockBackend::default()),
    ));
    let app = TestAppBuilder::default_test()
        .game_service(game_service)
        .build();

    // First async action sets is_generating = true
    let req1 = Request::builder()
        .uri("/action")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=go north"))
        .unwrap();
    let response1 = app.clone().oneshot(req1).await.unwrap();
    assert!(response1.status().is_success());

    // Second async action while first is in flight
    let req2 = Request::builder()
        .uri("/action")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=go south"))
        .unwrap();
    let response2 = app.oneshot(req2).await.unwrap();
    assert!(response2.status().is_success());
    let body = axum::body::to_bytes(response2.into_body(), 1024)
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
    // Inventory is no longer a sync action — it goes through async generation.
    let hx_trigger = response.headers().get("HX-Trigger");
    assert!(hx_trigger.is_none(), "Inventory should be async, not sync");
}

#[tokio::test]
async fn test_list_games_fragment_empty() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/fragment/games")
        .method(http::Method::GET)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let html = String::from_utf8_lossy(&body);
    assert!(
        html.contains(r#"class="games-list""#),
        "Should render games list container: {html}"
    );
}

#[tokio::test]
async fn test_list_games_fragment_populated() {
    let app = TestAppBuilder::default_app();

    // Create two games — the first will be active after the second is created
    // (since create_game_handler switches to the new game).
    let req = Request::builder()
        .uri("/games")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let req = Request::builder()
        .uri("/games")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Now fetch the fragment — the first game should appear in Saved Games
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
    let snapshot_storage: Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage> =
        Arc::new(chronicler_engine::test_support::InMemorySnapshotRepository::new());
    let message_storage: Arc<dyn chronicler_engine::storage::message_storage::MessageStorage> =
        Arc::new(chronicler_engine::test_support::InMemoryMessageRepository::new());

    let _ = snapshot_storage
        .create_game("Test World", "<script>alert('xss')</script>")
        .unwrap();

    let app = TestAppBuilder::default_test()
        .snapshot_storage(Arc::clone(&snapshot_storage))
        .message_storage(Arc::clone(&message_storage))
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
        html.contains("&lt;script&gt;"),
        "Should contain escaped script tag: {html}"
    );
}

#[tokio::test]
async fn test_create_game_handler() {
    let snapshot_storage: Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage> =
        Arc::new(chronicler_engine::test_support::InMemorySnapshotRepository::new());
    let message_storage: Arc<dyn chronicler_engine::storage::message_storage::MessageStorage> =
        Arc::new(chronicler_engine::test_support::InMemoryMessageRepository::new());

    let app = TestAppBuilder::default_test()
        .snapshot_storage(Arc::clone(&snapshot_storage))
        .message_storage(Arc::clone(&message_storage))
        .build();

    let old_id = SnapshotStorage::current_game_id(&*snapshot_storage);

    let req = Request::builder()
        .uri("/games")
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

    // Verify the new game was created, switched to, and initialized.
    let new_id = SnapshotStorage::current_game_id(&*snapshot_storage);
    assert_ne!(new_id, old_id, "Should have switched to the new game");

    let latest = snapshot_storage.load_latest().unwrap();
    assert!(latest.is_some(), "New game should have an initial snapshot");

    // The test world has no scenario, so messages may be empty.
    // What matters is that the snapshot was saved and the game was switched.
}

#[tokio::test]
async fn test_switch_game_handler_success() {
    let snapshot_storage: Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage> =
        Arc::new(chronicler_engine::test_support::InMemorySnapshotRepository::new());
    let message_storage: Arc<dyn chronicler_engine::storage::message_storage::MessageStorage> =
        Arc::new(chronicler_engine::test_support::InMemoryMessageRepository::new());

    let app = TestAppBuilder::default_test()
        .snapshot_storage(Arc::clone(&snapshot_storage))
        .message_storage(Arc::clone(&message_storage))
        .build();

    let other_id = snapshot_storage
        .create_game("Test World", "Test World_2026-01-01_1")
        .unwrap();
    assert_ne!(
        other_id,
        SnapshotStorage::current_game_id(&*snapshot_storage)
    );

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
    assert_eq!(
        SnapshotStorage::current_game_id(&*snapshot_storage),
        other_id
    );
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
async fn test_switch_game_handler_wrong_world() {
    let snapshot_storage: Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage> =
        Arc::new(chronicler_engine::test_support::InMemorySnapshotRepository::new());
    let message_storage: Arc<dyn chronicler_engine::storage::message_storage::MessageStorage> =
        Arc::new(chronicler_engine::test_support::InMemoryMessageRepository::new());

    let other_id = snapshot_storage
        .create_game("Other World", "Other World_1")
        .unwrap();

    let app = TestAppBuilder::default_test()
        .snapshot_storage(Arc::clone(&snapshot_storage))
        .message_storage(Arc::clone(&message_storage))
        .build();

    let req = Request::builder()
        .uri(format!("/games/{other_id}/switch"))
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_game_handler_success() {
    let snapshot_storage: Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage> =
        Arc::new(chronicler_engine::test_support::InMemorySnapshotRepository::new());
    let message_storage: Arc<dyn chronicler_engine::storage::message_storage::MessageStorage> =
        Arc::new(chronicler_engine::test_support::InMemoryMessageRepository::new());

    let app = TestAppBuilder::default_test()
        .snapshot_storage(Arc::clone(&snapshot_storage))
        .message_storage(Arc::clone(&message_storage))
        .build();

    let other_id = snapshot_storage
        .create_game("Test World", "Test World_2026-01-01_1")
        .unwrap();
    assert_ne!(
        other_id,
        SnapshotStorage::current_game_id(&*snapshot_storage)
    );

    let req = Request::builder()
        .uri(format!("/games/{other_id}/delete"))
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(snapshot_storage.get_game(other_id).unwrap().is_none());
}

#[tokio::test]
async fn test_delete_game_handler_active_game() {
    let app = TestAppBuilder::default_app();

    // create_app_for_testing initializes storage with game_id = 1
    let active_id = 1u64;

    let req = Request::builder()
        .uri(format!("/games/{active_id}/delete"))
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_game_handler_generating() {
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

    // Deletion should be rejected while generation is in progress
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
    struct FailingListGamesStorage {
        inner: Arc<dyn SnapshotStorage>,
        message_inner: Arc<dyn MessageStorage>,
    }

    impl SnapshotStorage for FailingListGamesStorage {
        fn set_game_id(&self, game_id: u64) {
            self.inner.set_game_id(game_id);
        }

        fn current_game_id(&self) -> u64 {
            self.inner.current_game_id()
        }

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

        fn list_games(
            &self,
        ) -> Result<Vec<chronicler_engine::model::game::Game>, chronicler_engine::error::EngineError>
        {
            Err(chronicler_engine::error::EngineError::Config(
                "list_games failed".to_string(),
            ))
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

    impl MessageStorage for FailingListGamesStorage {
        fn set_game_id(&self, game_id: u64) {
            self.message_inner.set_game_id(game_id);
        }

        fn current_game_id(&self) -> u64 {
            self.message_inner.current_game_id()
        }

        fn insert_message(
            &self,
            msg: &chronicler_engine::model::message::Message,
        ) -> Result<u64, chronicler_engine::error::EngineError> {
            self.message_inner.insert_message(msg)
        }

        fn update_message(
            &self,
            id: u64,
            text: &str,
        ) -> Result<(), chronicler_engine::error::EngineError> {
            self.message_inner.update_message(id, text)
        }

        fn delete_message(&self, id: u64) -> Result<(), chronicler_engine::error::EngineError> {
            self.message_inner.delete_message(id)
        }

        fn load_messages(
            &self,
        ) -> Result<
            Vec<chronicler_engine::model::message::Message>,
            chronicler_engine::error::EngineError,
        > {
            self.message_inner.load_messages()
        }

        fn soft_delete_message(
            &self,
            id: u64,
        ) -> Result<(), chronicler_engine::error::EngineError> {
            self.message_inner.soft_delete_message(id)
        }

        fn restore_soft_deleted(
            &self,
            ids: &[u64],
        ) -> Result<(), chronicler_engine::error::EngineError> {
            self.message_inner.restore_soft_deleted(ids)
        }

        fn purge_soft_deleted(
            &self,
            ids: &[u64],
        ) -> Result<(), chronicler_engine::error::EngineError> {
            self.message_inner.purge_soft_deleted(ids)
        }

        fn insert_swipe(
            &self,
            message_id: u64,
            swipe: &chronicler_engine::model::message::Swipe,
            index: usize,
        ) -> Result<(), chronicler_engine::error::EngineError> {
            self.message_inner.insert_swipe(message_id, swipe, index)
        }

        fn update_active_swipe(
            &self,
            message_id: u64,
            index: usize,
        ) -> Result<(), chronicler_engine::error::EngineError> {
            self.message_inner.update_active_swipe(message_id, index)
        }

        fn shift_swipe_indices(
            &self,
            message_id: u64,
            offset: usize,
        ) -> Result<(), chronicler_engine::error::EngineError> {
            self.message_inner.shift_swipe_indices(message_id, offset)
        }

        fn migrate_swipes(
            &self,
            message_id: u64,
            pending_swipes: &[chronicler_engine::model::message::Swipe],
            new_active_index: usize,
            to_delete: &[u64],
        ) -> Result<(), chronicler_engine::error::EngineError> {
            self.message_inner.migrate_swipes(
                message_id,
                pending_swipes,
                new_active_index,
                to_delete,
            )
        }
    }

    let inner_snapshot =
        Arc::new(chronicler_engine::test_support::InMemorySnapshotRepository::new());
    let inner_message = Arc::new(chronicler_engine::test_support::InMemoryMessageRepository::new());
    let storage = Arc::new(FailingListGamesStorage {
        inner: Arc::clone(&inner_snapshot) as Arc<dyn SnapshotStorage>,
        message_inner: Arc::clone(&inner_message) as Arc<dyn MessageStorage>,
    });

    let app = TestAppBuilder::default_test()
        .snapshot_storage(storage)
        .message_storage(Arc::clone(&inner_message) as Arc<dyn MessageStorage>)
        .build();

    let req = Request::builder()
        .uri("/fragment/games")
        .method(http::Method::GET)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
