# Smart Wait Helpers

Catalog of polling/wait helpers in `tests/test_utils/wait.rs` and `tests/test_utils/browser.rs`.

**Use these instead of bare `sleep()`** for any wait that depends on a runtime condition (UI state, server status, async completion). Bare sleeps hide flakiness and slow the suite.

## Acceptable bare sleeps

- **500ms** in polling loops (between condition checks)
- **200ms** in helper retry logic

Any other bare wait must be justified with a comment explaining why.

## `tests/test_utils/wait.rs`

### LLM state

```rust
// Wait for LLM to report idle via /status/generating.
// On timeout, attempts to POST /status/reset-generating to unstick the flag.
pub async fn wait_for_llm_idle(port: u16, timeout: Duration) -> Result<(), ()>
```

### Element polling (Playwright)

```rust
// Poll until `selector` matches ≥ min_count elements (10s timeout).
// Story-log growth idiom: wait_for_element_children(&page, "#story-log .log-entry", 2).
// Panics on timeout after `capture_failure_state` — a missing element is a
// failed test, never a soft skip. No return value to assert on.
pub async fn wait_for_element_children(page: &Page, selector: &str, min_count: u32)

// Wait for a uniquely-matching selector to become visible; panics on timeout.
// Playwright strict mode rejects multi-element matches — scope the selector
// (e.g. `#worlds-tab select[name=..]`).
pub async fn wait_until_visible(page: &Page, selector: &str, timeout: Duration)

// Wait for a uniquely-matching selector to become hidden OR detached;
// panics on timeout. Use for "element left the DOM" assertions — the
// story-log refresh replaces #story-log innerHTML, detaching old entries.
pub async fn wait_until_hidden(page: &Page, selector: &str, timeout: Duration)

// Verify element exists continuously for `duration` (poll every 200ms).
// Returns false if the element disappears at any check. Does not panic.
pub async fn wait_for_element_persist(
    page: &Page,
    selector: &str,
    duration: Duration,
) -> bool
```

### Status-display polling

```rust
// Poll until `#status-display` contains "Ready" (12s timeout, panics on miss).
pub async fn wait_for_status_ready(page: &Page)

// Poll until `#status-display` LEAVES "Ready" (5s timeout, panics on miss).
// `send_action` calls this before any wait_for_status_ready so the latter
// cannot return on the STALE pre-action "Ready".
pub async fn wait_for_status_generating(page: &Page)

// Poll until `#status-display` contains "Ready" or "Error" (15s timeout).
// Captures failure state on timeout and returns the final text — does not panic.
pub async fn wait_for_status_ready_or_error(page: &Page) -> String
```

### Generic condition waits

```rust
// Async: poll `condition` until true or timeout. Returns false on timeout.
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

## `tests/test_utils/server.rs`

```rust
// Wait for server to become reachable on a port.
pub async fn wait_for_server(port: u16, max_attempts: usize) -> bool

// Read the port the test server was started on from the config file.
pub fn get_config_port(config_path: &str) -> Option<u16>
```

## `tests/test_utils/browser.rs`

Test-server and page bootstrap (no polling loops except as noted):

```rust
// Launch Chrome via Playwright.
pub async fn launch_chrome() -> (Playwright, Browser)

// Start a TestServer, open a page, navigate with a connection check, and run
// `test_fn` with the page. Tears the server down afterwards.
pub async fn with_test_page<F, Fut>(config_path: &str, world: &str, persona: &str, test_fn: F)

// Fill the command input, submit, and wait for the action to be acknowledged
// (status leaves "Ready" via wait_for_status_generating), then dismiss any
// text-check dialog.
pub async fn send_action(page: &Page, text: &str)

// Click "Send Original" if a text-check preview dialog is visible.
pub async fn dismiss_text_check_if_present(page: &Page)

// Instant snapshot: number of `#story-log .log-entry` elements right now.
pub async fn count_log_entries(page: &Page) -> usize

// Capture failure diagnostics (screenshot, DOM dump, engine log tails).
// Called by the panicking helpers before they panic.
pub async fn capture_failure_state(page: &Page, test_name: &str)
```

## Example usage

```rust
use crate::test_utils::wait::{
    wait_for_element_children, wait_until_hidden, wait_for_status_ready,
    wait_for_status_generating, wait_for_llm_idle,
};
use crate::test_utils::server::get_config_port;
use std::time::Duration;

let port = get_config_port(CONFIG_PATH).expect("Failed to get config port");

// Wait for the story log to render at least 2 entries; panics on timeout.
wait_for_element_children(&page, "#story-log .log-entry", 2).await;

// Wait for a specific entry to leave the DOM after a delete.
wait_until_hidden(&page, ".log-entry[data-id='42']", Duration::from_secs(10)).await;

let llm_result = wait_for_llm_idle(port, Duration::from_secs(30)).await;
```

## Failure state capture

All UI-wait helpers that panic on timeout (`wait_until_visible`,
`wait_until_hidden`, `wait_for_element_children`, `wait_for_status_ready`,
`wait_for_status_generating`) call `capture_failure_state(page, <label>)`
from `tests/test_utils/browser.rs` first, saving diagnostic context. The
returning helpers (`wait_for_status_ready_or_error`, `wait_for_condition_*`)
also capture on timeout but hand the decision back to the caller — always
assert on what they return. When a wait times out, check the captured
failure state before re-running.
