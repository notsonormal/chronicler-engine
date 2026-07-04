use super::port_utils::bind_with_retry;

#[tokio::test]
async fn test_bind_with_retry_on_port_zero_succeeds() {
    let result = bind_with_retry("127.0.0.1:0", Some(1)).await;
    assert!(result.is_ok(), "Should bind to port 0 (OS-assigned)");
}

#[tokio::test]
async fn test_bind_with_retry_invalid_address_fails() {
    let result = bind_with_retry("invalid", Some(1)).await;
    assert!(result.is_err(), "Should fail to bind to invalid address");
}
