use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use std::sync::Arc;
use tower::ServiceExt;

use chronicler_engine::TestAppBuilder;

#[tokio::test]
async fn test_action_handler_load_state_failure() {
    let snapshot_storage_inner: Arc<
        dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage,
    > = Arc::new(chronicler_engine::test_support::InMemorySnapshotRepository::new());

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

    let snapshot_storage: Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage> =
        Arc::new(FailingLoadStorage {
            inner: snapshot_storage_inner,
        });
    let message_storage: Arc<dyn chronicler_engine::storage::message_storage::MessageStorage> =
        Arc::new(chronicler_engine::test_support::InMemoryMessageRepository::new());

    let app = TestAppBuilder::default_test()
        .snapshot_storage(snapshot_storage)
        .message_storage(message_storage)
        .build();

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
async fn test_action_handler_snapshot_save_failure() {
    let snapshot_storage_inner: Arc<
        dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage,
    > = Arc::new(chronicler_engine::test_support::InMemorySnapshotRepository::new());

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

    let snapshot_storage: Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage> =
        Arc::new(FailingSaveStorage {
            inner: snapshot_storage_inner,
        });
    let message_storage: Arc<dyn chronicler_engine::storage::message_storage::MessageStorage> =
        Arc::new(chronicler_engine::test_support::InMemoryMessageRepository::new());

    let app = TestAppBuilder::default_test()
        .snapshot_storage(snapshot_storage)
        .message_storage(message_storage)
        .build();

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
async fn test_action_confirm_handler_render_error_fallback() {
    let snapshot_storage_inner: Arc<
        dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage,
    > = Arc::new(chronicler_engine::test_support::InMemorySnapshotRepository::new());

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

    let snapshot_storage: Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage> =
        Arc::new(FailingLoadStorage {
            inner: snapshot_storage_inner,
        });
    let message_storage: Arc<dyn chronicler_engine::storage::message_storage::MessageStorage> =
        Arc::new(chronicler_engine::test_support::InMemoryMessageRepository::new());

    let app = TestAppBuilder::default_test()
        .snapshot_storage(snapshot_storage)
        .message_storage(message_storage)
        .build();

    let req = Request::builder()
        .uri("/action/confirm")
        .method(Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from("command=look"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("error-message"),
        "Expected error message in fallback: {body_str}"
    );
}
