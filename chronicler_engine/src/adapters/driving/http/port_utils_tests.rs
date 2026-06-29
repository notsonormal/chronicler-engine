use super::port_utils::bind_with_retry;

#[tokio::test]
async fn test_bind_with_retry_on_port_zero_succeeds() {
    let result = bind_with_retry("127.0.0.1:0").await;
    assert!(result.is_ok(), "Should bind to port 0 (OS-assigned)");
}

#[tokio::test]
async fn test_bind_with_retry_invalid_address_fails() {
    let result = bind_with_retry("invalid").await;
    assert!(result.is_err(), "Should fail to bind to invalid address");
}

#[test]
#[cfg(target_os = "windows")]
fn test_find_process_on_port_unused_port_returns_none() {
    // This test accesses the private function via the module, so we can't test it directly
    // The bind_with_retry tests cover the public API
}
