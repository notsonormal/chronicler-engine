//! Polling helpers: `wait_for_llm_idle`, `wait_for_status_ready`, and `wait_for_element_children` — retry-based waits used by browser and HTTP tests.

use std::time::Duration;

use tokio::time::sleep;

use super::browser::capture_failure_state;

pub async fn wait_for_llm_idle(port: u16, timeout: Duration) -> Result<(), ()> {
    let start = std::time::Instant::now();
    let client = reqwest::Client::new();

    while start.elapsed() < timeout {
        if let Ok(resp) = client
            .get(format!("http://127.0.0.1:{port}/status/generating"))
            .send()
            .await
        {
            if let Ok(text) = resp.text().await {
                if text == "idle" {
                    return Ok(());
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    // Timeout reached - try to reset the flag by posting to a reset endpoint
    // This ensures is_generating doesn't get stuck
    let _ = client
        .post(format!("http://127.0.0.1:{port}/status/reset-generating"))
        .send()
        .await;

    Err(())
}

pub async fn wait_for_non_loading_value(page: &playwright_rs::Page, selector: &str) -> String {
    let locator = page.locator(selector).await;
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(10);

    while start.elapsed() < timeout {
        match locator.inner_text().await {
            Ok(text) if !text.is_empty() => return text,
            _ => sleep(Duration::from_millis(200)).await,
        }
    }

    capture_failure_state(page, &format!("wait_for_non_loading_value_{selector}")).await;
    String::new()
}

pub async fn wait_for_element_children(
    page: &playwright_rs::Page,
    selector: &str,
    min_count: u32,
) -> u32 {
    let locator = page.locator(selector).await;
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(10);
    let mut last_count = 0;

    while start.elapsed() < timeout {
        match locator.count().await {
            Ok(count) => {
                last_count = count as u32;
                if last_count >= min_count {
                    return last_count;
                }
            }
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
    last_count
}

/// Wait for an element to become visible
pub async fn wait_for_element_exists(
    page: &playwright_rs::Page,
    selector: &str,
    max_attempts: u32,
) {
    let locator = page.locator(selector).await;
    let timeout_ms = max_attempts as f64 * 50.0;
    if let Err(e) = playwright_rs::expect(locator)
        .with_timeout(std::time::Duration::from_millis(timeout_ms as u64))
        .to_be_visible()
        .await
    {
        capture_failure_state(page, &format!("wait_for_element_exists_{selector}")).await;
        panic!("Element '{selector}' did not become visible: {e}");
    }
}

/// Wait for an element to become hidden
pub async fn wait_for_element_not_exists(
    page: &playwright_rs::Page,
    selector: &str,
    max_attempts: u32,
) {
    let locator = page.locator(selector).await;
    let timeout_ms = max_attempts as f64 * 50.0;
    if let Err(e) = locator
        .wait_for(Some(playwright_rs::WaitForOptions {
            state: Some(playwright_rs::WaitForState::Hidden),
            timeout: Some(timeout_ms),
        }))
        .await
    {
        let still_visible = locator.is_visible().await.unwrap_or(true);
        eprintln!(
            "⏱️  wait_for_element_not_exists('{selector}') TIMED OUT after {}ms \
             (still visible: {still_visible})",
            timeout_ms as u64
        );
        capture_failure_state(page, &format!("wait_for_element_not_exists_{selector}")).await;
        panic!("Element '{selector}' did not become hidden: {e}");
    }
}

fn extract_port_from_url(url: &str) -> Option<u16> {
    url.split("://")
        .nth(1)
        .and_then(|port_str| port_str.split('/').next())
        .and_then(|port_part| port_part.split(':').nth(1))
        .and_then(|port| port.parse::<u16>().ok())
}

pub async fn wait_for_status_ready(page: &playwright_rs::Page) {
    let locator = page.locator("#status-display").await;
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(5);

    while start.elapsed() < timeout {
        match locator.inner_text().await {
            Ok(text) if text.contains("Ready") => return,
            _ => sleep(Duration::from_millis(200)).await,
        }
    }
    // One final check — the locator may have just updated
    if let Ok(text) = locator.inner_text().await {
        if text.contains("Ready") {
            return;
        }
        tracing::debug!("Final status-display='{text}'");
    }
    capture_failure_state(page, "wait_for_status_ready").await;
    panic!("Status did not become Ready");
}

pub async fn wait_for_status_ready_or_error(page: &playwright_rs::Page) -> String {
    let locator = page.locator("#status-display").await;
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(15);

    while start.elapsed() < timeout {
        match locator.inner_text().await {
            Ok(text) if text.contains("Ready") || text.contains("Error") => return text,
            _ => sleep(Duration::from_millis(500)).await,
        }
    }

    let final_text = locator.inner_text().await.unwrap_or_default();
    capture_failure_state(page, "wait_for_status_ready_or_error").await;
    final_text
}

/// Wait for an element to persist (exist continuously) for a duration
/// Useful for verifying UI state survives polling cycles or time-based operations
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
            return false; // Element disappeared during wait
        }
    }

    // Element must exist in all checks
    exists_count == check_count
}

/// Generic async condition wait
/// Polls the condition until it returns true or timeout expires
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

/// Generic sync condition wait (for std::thread tests)
/// Polls the condition until it returns true or timeout expires
pub fn wait_for_condition_sync<F>(timeout: Duration, poll_interval: Duration, condition: F) -> bool
where
    F: Fn() -> bool,
{
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if condition() {
            return true;
        }
        std::thread::sleep(poll_interval);
    }
    false
}
