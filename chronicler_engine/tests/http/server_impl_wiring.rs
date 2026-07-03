//! HTTP wiring tests for `server_impl.rs`.
//!
//! Verifies that `run_server_with_config` builds app state correctly,
//! binds a listener, and propagates bind errors. The happy-path test
//! confirms the spawned server task is cancelable; real request routing
//! is covered by `tests/http/fragment.rs` via `TestAppBuilder`.

use std::sync::{Arc, RwLock};

use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::adapters::driving::http::{ServerConfig, ServerResources};
use chronicler_engine::adapters::driving::http::server_impl::run_server_with_config;
use chronicler_engine::domain::model::settings::AppSettings;

fn build_test_resources() -> ServerResources {
    let storage = Arc::new(Storage::new_in_memory());
    let preset_storage = Arc::new(Storage::new_in_memory());
    let settings = Arc::new(RwLock::new(AppSettings::default()));
    ServerResources {
        storage,
        preset_storage,
        settings,
    }
}

#[tokio::test]
async fn run_server_spawns_and_can_be_cancelled() {
    let resources = build_test_resources();
    let config = ServerConfig { port: 0 };

    let server_handle =
        tokio::spawn(async move { run_server_with_config(resources, config).await });

    // Give the server time to bind.
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // Cancel by aborting; assert the JoinError reflects cancellation.
    server_handle.abort();
    let result = server_handle.await;
    assert!(
        matches!(result, Err(ref e) if e.is_cancelled()),
        "server task should finish with cancellation after abort, got: {result:?}"
    );
}

#[tokio::test]
async fn run_server_propagates_bind_error_for_privileged_port() {
    // Port 1 is privileged (< 1024) and typically unbound; on Linux binding
    // to it without CAP_NET_BIND_SERVICE returns an error.
    let resources = build_test_resources();
    let config = ServerConfig { port: 1 };

    let result = run_server_with_config(resources, config).await;

    assert!(result.is_err(), "expected bind error on privileged port");
}
