# Plan: Fragment Handler Integration Tests

## Problem

Current integration tests verify HTML output via Playwright browser, but never call the HTMX endpoints that use `fragments.rs`. This results in 0% coverage for:
- `server/fragments.rs`
- `server/mod.rs`

Tests load the initial page but don't exercise fragment handlers via HTTP.

## Solution

Add HTTP-based integration tests that call fragment endpoints directly, not through browser.

## Files to Change

### New File
- `tests/fragment_tests.rs` - HTTP tests for fragment handlers

## Test Coverage Target

Endpoint → Tests needed:
- `GET /header` → test_header_fragment_returns_html
- `GET /story-log` → test_story_log_fragment_returns_html  
- `GET /visual-sidebar` → test_visual_sidebar_fragment_returns_html
- `GET /action-area` → test_action_area_fragment_returns_html
- `POST /action` → test_action_handler_accepts_command

## Implementation

```rust
#[tokio::test]
async fn test_action_handler_accepts_command() {
    let server = TestServer::new_with_mock(PORT, WORLD);
    
    let client = reqwest::Client::new();
    let response = client
        .post(format!("http://127.0.0.1:{}/action", PORT))
        .form(&[("command", "look")])
        .send()
        .await
        .unwrap();
    
    assert!(response.status().is_success());
}
```

## Acceptance Criteria

- [ ] `server/fragments.rs` coverage > 50%
- [ ] `server/mod.rs` coverage > 50%
- [ ] Tests run with flow_mock_tests suite