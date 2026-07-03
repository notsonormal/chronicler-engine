# Smart Wait Helpers

Catalog of polling/wait helpers in `tests/test_utils/wait.rs` and `tests/test_utils/browser.rs`.

**Use these instead of bare `sleep()`** for any wait that depends on a runtime condition (UI state, server status, async completion). Bare sleeps hide flakiness and slow the suite.

## Acceptable bare sleeps

- **500ms** in polling loops (between condition checks)
- **200ms** in helper retry logic

Any other bare wait must be justified with a comment explaining why.

## `tests/test_utils/wait.rs`

### LLM / server state

```rust
// Wait for server to become reachable on a port.
pub async fn wait_for_server(port: u16, max_attempts: usize) -> bool

// Wait for LLM to report idle via /status/generating.
// On timeout, attempts to POST /status/reset-generating to unstick the flag.
pub async fn wait_for_llm_idle(port: u16, timeout: Duration) -> Result<(), ()>
```

### UI element polling (Playwright)

```rust
// Poll until `.location` innerText differs from `initial`.
pub async fn wait_for_location_change(page: &Page, initial: &str) -> String

// Poll until `#story-log` innerText differs from `initial`.
pub async fn wait_for_story_log_change(page: &Page, initial: &str) -> String

// Poll until count of `#story-log .log-entry .text` exceeds initial_count.
pub async fn wait_for_more_messages(page: &Page, initial_count: usize) -> usize

// Poll until selector's innerText is non-empty (10s timeout).
pub async fn wait_for_non_loading_value(page: &Page, selector: &str) -> String

// Poll until selector has given class (max_attempts iterations, 250ms each).
pub async fn wait_for_element_class(
    page: &Page,
    selector: &str,
    class_name: &str,
    max_attempts: u32,
) -> bool

// Poll until selector has ≥ min_count children (10s timeout).
pub async fn wait_for_element_children(
    page: &Page,
    selector: &str,
    min_count: u32,
) -> u32

// Poll until selector innerText is non-empty (10s timeout).
pub async fn wait_for_element_text(page: &Page, selector: &str) -> String

// Wait for element to become visible; panics on timeout.
pub async fn wait_for_element_exists(page: &Page, selector: &str, max_attempts: u32)

// Wait for element to become hidden; panics on timeout.
pub async fn wait_for_element_not_exists(page: &Page, selector: &str, max_attempts: u32)

// Poll until `#status-display` contains "Ready" (5s timeout, panics on miss).
pub async fn wait_for_status_ready(page: &Page)

// Poll until `#status-display` contains "Ready" or "Error" (15s timeout).
pub async fn wait_for_status_ready_or_error(page: &Page) -> String

// Verify element exists continuously for `duration` (poll every 200ms).
// Returns false if the element disappears at any check.
pub async fn wait_for_element_persist(
    page: &Page,
    selector: &str,
    duration: Duration,
) -> bool
```

### Generic condition waits

```rust
// Async: poll `condition` until true or timeout.
pub async fn wait_for_condition_async<F, Fut>(
    timeout: Duration,
    poll_interval: Duration,
    condition: F,
) -> bool
where
    F: Fn() -> Fut,
    Fut: Future<Output = bool>,

// Sync: poll `condition` until true or timeout (for std::thread tests).
pub fn wait_for_condition_sync<F>(
    timeout: Duration,
    poll_interval: Duration,
    condition: F,
) -> bool
where
    F: Fn() -> bool,
```

## `tests/test_utils/browser.rs`

```rust
// Poll until `#story-log .log-entry` has ≥ min_count children.
pub async fn wait_for_log_entries(page: &Page, min_count: usize) -> usize
```

## Example usage

```rust
use crate::test_utils::wait::{
    wait_for_element_children, wait_for_element_text, wait_for_story_log_change,
    wait_for_more_messages, wait_for_llm_idle,
};
use crate::test_utils::server::get_config_port;
use crate::test_utils::CONFIG_PATH;
use std::time::Duration;

let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");
let entries = wait_for_element_children(&page, "#story-log .log-entry", 1).await;
let status = wait_for_element_text(&page, "#status-display").await;
let content = wait_for_story_log_change(&page, &initial).await;
let count = wait_for_more_messages(&page, initial_count).await;
let llm_result = wait_for_llm_idle(port, Duration::from_secs(30)).await;
```

## Failure state capture

All UI-wait helpers that panic on timeout call `capture_failure_state(page, <label>)` from `tests/test_utils/browser.rs` first, saving diagnostic context. When a wait times out, check the captured failure state before re-running.
