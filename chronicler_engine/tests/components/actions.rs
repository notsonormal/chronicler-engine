use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use std::sync::Arc;
use tower::ServiceExt;

use chronicler_engine::model::settings::{AppSettings, TextCheckMode, TextCheckSettings};
use chronicler_engine::model::state::LogType;

use crate::create_test_state;

#[tokio::test]
async fn test_action_check_handler_empty_command() {
    let app = chronicler_engine::create_app_for_testing(create_test_state());

    let req = Request::builder()
        .uri("/action/check")
        .method(Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from("command="))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_action_check_handler_disabled_mode() {
    let mut state = create_test_state();
    state.add_log(
        "look".to_string(),
        Some("Player".to_string()),
        LogType::Input,
    );
    let app = chronicler_engine::create_app_for_testing_with_settings(
        state,
        AppSettings {
            text_check: TextCheckSettings {
                mode: TextCheckMode::Disabled,
                enable_auto_check: true,
                ignored_words: vec![],
            },
            ..AppSettings::default()
        },
    );

    let req = Request::builder()
        .uri("/action/check")
        .method(Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from("command=walk+north"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    // When disabled, it falls through to process_action
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_action_handler_load_state_failure() {
    let state = create_test_state();
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
        fn commit(&self, snapshot_id: u64) -> Result<(), chronicler_engine::error::EngineError> {
            self.inner.commit(snapshot_id)
        }
        fn reset(&self) -> Result<(), chronicler_engine::error::EngineError> {
            self.inner.reset()
        }
        fn save_checkpoint(
            &self,
            checkpoint: &chronicler_engine::model::checkpoint::Checkpoint,
        ) -> Result<(), chronicler_engine::error::EngineError> {
            self.inner.save_checkpoint(checkpoint)
        }
        fn load_checkpoint(
            &self,
            id: &str,
        ) -> Result<
            Option<chronicler_engine::model::checkpoint::Checkpoint>,
            chronicler_engine::error::EngineError,
        > {
            self.inner.load_checkpoint(id)
        }
        fn list_checkpoints(
            &self,
        ) -> Result<
            Vec<chronicler_engine::model::checkpoint::Checkpoint>,
            chronicler_engine::error::EngineError,
        > {
            self.inner.list_checkpoints()
        }
        fn delete_checkpoint(&self, id: &str) -> Result<(), chronicler_engine::error::EngineError> {
            self.inner.delete_checkpoint(id)
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
        fn commit(&self, snapshot_id: u64) -> Result<(), chronicler_engine::error::EngineError> {
            self.inner.commit(snapshot_id)
        }
        fn reset(&self) -> Result<(), chronicler_engine::error::EngineError> {
            self.inner.reset()
        }
        fn save_checkpoint(
            &self,
            checkpoint: &chronicler_engine::model::checkpoint::Checkpoint,
        ) -> Result<(), chronicler_engine::error::EngineError> {
            self.inner.save_checkpoint(checkpoint)
        }
        fn load_checkpoint(
            &self,
            id: &str,
        ) -> Result<
            Option<chronicler_engine::model::checkpoint::Checkpoint>,
            chronicler_engine::error::EngineError,
        > {
            self.inner.load_checkpoint(id)
        }
        fn list_checkpoints(
            &self,
        ) -> Result<
            Vec<chronicler_engine::model::checkpoint::Checkpoint>,
            chronicler_engine::error::EngineError,
        > {
            self.inner.list_checkpoints()
        }
        fn delete_checkpoint(&self, id: &str) -> Result<(), chronicler_engine::error::EngineError> {
            self.inner.delete_checkpoint(id)
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
    let state = create_test_state();
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
        fn commit(&self, snapshot_id: u64) -> Result<(), chronicler_engine::error::EngineError> {
            self.inner.commit(snapshot_id)
        }
        fn reset(&self) -> Result<(), chronicler_engine::error::EngineError> {
            self.inner.reset()
        }
        fn save_checkpoint(
            &self,
            checkpoint: &chronicler_engine::model::checkpoint::Checkpoint,
        ) -> Result<(), chronicler_engine::error::EngineError> {
            self.inner.save_checkpoint(checkpoint)
        }
        fn load_checkpoint(
            &self,
            id: &str,
        ) -> Result<
            Option<chronicler_engine::model::checkpoint::Checkpoint>,
            chronicler_engine::error::EngineError,
        > {
            self.inner.load_checkpoint(id)
        }
        fn list_checkpoints(
            &self,
        ) -> Result<
            Vec<chronicler_engine::model::checkpoint::Checkpoint>,
            chronicler_engine::error::EngineError,
        > {
            self.inner.list_checkpoints()
        }
        fn delete_checkpoint(&self, id: &str) -> Result<(), chronicler_engine::error::EngineError> {
            self.inner.delete_checkpoint(id)
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
