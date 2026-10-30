//! Polling helpers: `wait_for_llm_idle`, `wait_for_status_ready`, and `wait_for_element_children` — retry-based waits used by browser and HTTP tests.

use std::time::Duration;

use tokio::time::sleep;

use super::browser::capture_failure_state;
use super::html::part_of_screen;

pub async fn wait_for_llm_idle(port: u16, timeout: Duration) -> Result<(), ()> {
    let start = std::time::Instant::now();
    let client = reqwest::Client::new();

    while start.elapsed() < timeout {
        if let Ok(resp) = client
            .get(format!(
                "http://127.0.0.1:{port}/screen?parts=status-display"
            ))
            .send()
            .await
        {
            if let Ok(text) = resp.text().await {
                let status = part_of_screen(&text, "status-display").unwrap_or_default();
                if status.contains(r#"data-status="idle""#) {
                    return Ok(());
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    // Reset the busy state after a timeout so a stuck Game does not hang the test.
    let _ = client
        .post(format!("http://127.0.0.1:{port}/status/reset-busy"))
        .send()
        .await;

    Err(())
}

/// Count-based wait, no strict mode — the helper for selectors that
/// legitimately match several elements (visibility waits fail on multi-match).
/// Panics on timeout after `capture_failure_state` — a missing element is a
/// failed test, never a soft skip.
pub async fn wait_for_element_children(page: &playwright_rs::Page, selector: &str, min_count: u32) {
    let locator = page.locator(selector).await;
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(10);
    let mut last_count = 0;

    while start.elapsed() < timeout {
        match locator.count().await {
            Ok(count) if count >= min_count as usize => return,
            Ok(count) => last_count = count as u32,
            Err(e) => {
                eprintln!("⚠️  wait_for_element_children('{selector}') count() failed: {e}");
            }
        }
        sleep(Duration::from_millis(200)).await;
    }

    let elapsed = start.elapsed().as_secs_f32();
    eprintln!(
        "⏱️  wait_for_element_children('{selector}') TIMED OUT after {elapsed:.1}s \
         (expected ≥ {min_count}, found {last_count})"
    );
    capture_failure_state(page, &format!("wait_for_element_children_{selector}")).await;
    panic!("wait_for_element_children('{selector}'): expected ≥ {min_count}, found {last_count}");
}

/// Wait for the story log to render at least one entry.
///
/// The explicit story-log readiness contract: a test that reads the log calls
/// this, so a missing render is attributable to that test rather than to every
/// test that passed through a shared precondition.
pub async fn wait_for_story_log(page: &playwright_rs::Page) {
    wait_for_element_children(page, "#story-log .log-entry", 1).await;
}

/// Poll-until-visible for a uniquely-matching selector (Playwright strict mode
/// rejects a multi-element match — scope panel selectors by tab, e.g.
/// `#worlds-tab select[name=..]`). An empty inline element stays invisible;
/// wait on its populated container, then assert text on the status span.
pub async fn wait_until_visible(page: &playwright_rs::Page, selector: &str, timeout: Duration) {
    let locator = page.locator(selector).await;
    if let Err(e) = playwright_rs::expect(locator)
        .with_timeout(timeout)
        .to_be_visible()
        .await
    {
        capture_failure_state(page, &format!("wait_until_visible_{selector}")).await;
        panic!("Element '{selector}' did not become visible within {timeout:?}: {e}");
    }
}

pub async fn wait_until_hidden(page: &playwright_rs::Page, selector: &str, timeout: Duration) {
    let locator = page.locator(selector).await;
    let timeout_ms = timeout.as_millis() as f64;
    if let Err(e) = locator
        .wait_for(Some(playwright_rs::WaitForOptions {
            state: Some(playwright_rs::WaitForState::Hidden),
            timeout: Some(timeout_ms),
        }))
        .await
    {
        let still_visible = locator.is_visible().await.unwrap_or(true);
        eprintln!(
            "⏱️  wait_until_hidden('{selector}') TIMED OUT after {}ms \
             (still visible: {still_visible})",
            timeout_ms as u64
        );
        capture_failure_state(page, &format!("wait_until_hidden_{selector}")).await;
        panic!("Element '{selector}' did not become hidden: {e}");
    }
}

/// The action area is idle when no send is pending, no check runs and no preview
/// is open. A stale "Ready" in the status display, before the load that shows
/// the generation a send started, is not idle: the area keeps its own state until
/// that load arrives.
async fn action_area_is_idle(page: &playwright_rs::Page) -> bool {
    page.evaluate::<(), bool>(
        "() => { const area = document.getElementById('action-area'); return !!area && area.dataset.state === 'idle'; }",
        None,
    )
    .await
    .unwrap_or(false)
}

pub async fn wait_for_status_ready(page: &playwright_rs::Page) {
    let locator = page.locator("#status-display").await;
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(12);

    while start.elapsed() < timeout {
        match locator.inner_text().await {
            Ok(text) if text.contains("Ready") && action_area_is_idle(page).await => return,
            _ => sleep(Duration::from_millis(200)).await,
        }
    }
    if let Ok(text) = locator.inner_text().await {
        if text.contains("Ready") && action_area_is_idle(page).await {
            return;
        }
        tracing::debug!("Final status-display='{text}'");
    }
    capture_failure_state(page, "wait_for_status_ready").await;
    panic!("Status did not become Ready");
}

/// Wait until the engine has taken or refused a typed command: the command input
/// is empty (an accepted send consumes it) or the text-check preview is open.
///
/// `send_action` must call this before any `wait_for_status_ready`. The mock model
/// answers at once, so the Job can end before the status display ever shows it:
/// the consumed input is the one signal that stays.
pub async fn wait_for_action_acknowledged(page: &playwright_rs::Page) {
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(5);
    let acknowledged = || async {
        page.evaluate::<(), bool>(
            r#"() => {
                const input = document.querySelector('#command-form input[name="command"]');
                return (!!input && input.value === '')
                    || !!document.querySelector('.text-check-preview');
            }"#,
            None,
        )
        .await
        .unwrap_or(false)
    };
    while start.elapsed() < timeout {
        if acknowledged().await {
            return;
        }
        sleep(Duration::from_millis(50)).await;
    }
    capture_failure_state(page, "wait_for_action_acknowledged").await;
    panic!("The command was neither consumed nor held in a preview after 5s");
}

/// Wait for `#status-display` to leave the "Ready" state: the page asked after a
/// send, the busy number moved, and the display loaded the generation. A stub test
/// that sets the stub's status uses it; a full-stack test whose model answers at
/// once uses `wait_for_action_acknowledged`.
pub async fn wait_for_status_busy(page: &playwright_rs::Page) {
    let locator = page.locator("#status-display").await;
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(5);

    while start.elapsed() < timeout {
        match locator.inner_text().await {
            Ok(text) if !text.contains("Ready") => return,
            _ => sleep(Duration::from_millis(100)).await,
        }
    }
    if let Ok(text) = locator.inner_text().await {
        if !text.contains("Ready") {
            return;
        }
        tracing::debug!("Final status-display='{text}'");
    }
    capture_failure_state(page, "wait_for_status_busy").await;
    panic!(
        "Status never left Ready after action — the /action/check acknowledgement did not arrive"
    );
}

pub async fn wait_for_status_ready_or_error(page: &playwright_rs::Page) -> StatusOutcome {
    let locator = page.locator("#status-display").await;
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(15);

    while start.elapsed() < timeout {
        let has_error = has_error_disclosure(page).await;
        match locator.inner_text().await {
            Ok(text) if text.contains("Ready") || has_error => {
                return StatusOutcome {
                    ready: text.contains("Ready"),
                    error: has_error,
                    text,
                };
            }
            _ => sleep(Duration::from_millis(500)).await,
        }
    }

    let final_text = locator.inner_text().await.unwrap_or_default();
    let final_error = has_error_disclosure(page).await;
    capture_failure_state(page, "wait_for_status_ready_or_error").await;
    StatusOutcome {
        ready: final_text.contains("Ready"),
        error: final_error,
        text: final_text,
    }
}

#[derive(Debug, Clone)]
pub struct StatusOutcome {
    pub text: String,
    pub ready: bool,
    pub error: bool,
}

impl StatusOutcome {
    pub fn is_terminal(&self) -> bool {
        self.ready || self.error
    }
}

async fn has_error_disclosure(page: &playwright_rs::Page) -> bool {
    page.evaluate::<(), bool>(
        "() => !!document.querySelector('#status-display .error-disclosure')",
        None,
    )
    .await
    .unwrap_or(false)
}

pub async fn wait_for_element_persist(
    page: &playwright_rs::Page,
    selector: &str,
    duration: Duration,
) -> bool {
    let start = std::time::Instant::now();
    let poll_interval = Duration::from_millis(200);
    let mut exists_count = 0;
    let mut check_count = 0;

    while start.elapsed() < duration {
        sleep(poll_interval).await;
        check_count += 1;
        let count = element_count(page, selector).await;
        if count > 0 {
            exists_count += 1;
        } else {
            return false;
        }
    }

    exists_count == check_count
}

pub async fn wait_for_condition_async<F, Fut>(
    timeout: Duration,
    poll_interval: Duration,
    condition: F,
) -> bool
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if condition().await {
            return true;
        }
        sleep(poll_interval).await;
    }
    false
}

async fn element_count(page: &playwright_rs::Page, selector: &str) -> u32 {
    page.locator(selector).await.count().await.unwrap_or(0) as u32
}
