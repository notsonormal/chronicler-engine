//! HTTP wiring tests for `server_impl.rs` (real request routing lives in `tests/http/fragment.rs`).

use std::sync::{Arc, RwLock};
use std::time::Duration;

use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::adapters::driving::http::ServerConfig;
use chronicler_engine::adapters::driving::http::bootstrap::run_server_with_config;
use chronicler_engine::bootstrap::wiring::{WiredApp, build_app_graph_for_tests};
use chronicler_engine::domain::model::settings::AppSettings;

fn build_test_wired_app() -> WiredApp {
    build_app_graph_for_tests(
        Arc::new(RwLock::new(AppSettings::default())),
        Arc::new(Storage::new_in_memory()),
        Arc::new(Storage::new_in_memory()),
        None,
    )
    .expect("build_app_graph_for_tests should succeed")
}

#[tokio::test]
async fn run_server_binds_and_accepts_connections() {
    let config = ServerConfig {
        port: 0,
        bind_attempts: Some(1),
    };

    let (addr, server_handle) = run_server_with_config(build_test_wired_app(), config)
        .await
        .expect("server should bind to an OS-assigned port");

    tokio::time::timeout(Duration::from_secs(2), tokio::net::TcpStream::connect(addr))
        .await
        .expect("server should accept connections within 2s")
        .expect("TCP connect should succeed");

    server_handle.abort();
    let _ = server_handle.await;
}

#[tokio::test]
async fn run_server_propagates_bind_error_for_occupied_port() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("should bind a real port");
    let port = listener.local_addr().expect("local_addr").port();

    let config = ServerConfig {
        port,
        bind_attempts: Some(1),
    };

    let result = run_server_with_config(build_test_wired_app(), config).await;

    assert!(
        result.is_err(),
        "expected bind error when port {port} is already in use"
    );

    drop(listener);
}
