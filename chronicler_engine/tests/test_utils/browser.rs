use std::time::Duration;

use playwright_rs::LaunchOptions;
use playwright_rs::Playwright;
use playwright_rs::expect;

use super::server::{TestServer, get_config_port, wait_for_server};
use super::wait::wait_for_element_children;

pub async fn goto_with_connection_check(
    page: &playwright_rs::Page,
    port: u16,
) -> Result<(), String> {
    let url = format!("http://127.0.0.1:{port}");

    // Explicit wait for server before navigation
    if !wait_for_server(port, 100).await {
        return Err(format!("Server failed to start on port {port}"));
    }

    let _: Option<_> = page.goto(&url, None).await.map_err(|e| {
        let err_str = e.to_string();
        if err_str.contains("ERR_CONNECTION_REFUSED") {
            format!(
                "CONNECTION REFUSED: Server not running on port {port}. \
                 Likely causes: port conflict (another process using port {port}), \
                 server failed to start, or server crashed. \
                 Check with: netstat -ano | Select-String {port}",
            )
        } else {
            format!("Navigation failed to {url}: {err_str}")
        }
    })?;
    Ok(())
}

/// Launch Chromium browser for UI tests
pub async fn launch_chrome() -> (playwright_rs::Playwright, playwright_rs::Browser) {
    let headed = std::env::var("HEADED").map(|v| v == "1").unwrap_or(false);
    let slow_mo = std::env::var("SLOW_MO")
        .ok()
        .and_then(|v| v.parse::<f64>().ok());

    let playwright = Playwright::launch().await.unwrap();
    let mut options = LaunchOptions {
        channel: Some("chrome".to_string()),
        ..Default::default()
    };
    if headed {
        options.headless = Some(false);
        println!("🖥️  Running browser in headed mode (HEADED=1)");
    }
    if let Some(ms) = slow_mo {
        options.slow_mo = Some(ms);
        println!("⏱️  Slowing operations by {ms}ms (SLOW_MO={ms})");
    }
    let browser = playwright
        .chromium()
        .launch_with_options(options)
        .await
        .unwrap();
    (playwright, browser)
}

/// Run an E2E test with a fully set up browser page.
///
/// Handles: port allocation, mock server startup, browser launch, navigation,
/// and waiting for initial content. Cleans up the browser on completion.
pub async fn with_test_page<F, Fut>(config_path: &str, world: &str, test_fn: F)
where
    F: FnOnce(playwright_rs::Page, u16) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let port = get_config_port(config_path).expect("Failed to get config port");
    let _server = TestServer::new_with_mock(port, world).await;

    let (_playwright, browser) = launch_chrome().await;
    let page = browser.new_page().await.unwrap();

    goto_with_connection_check(&page, port)
        .await
        .expect("Failed to connect to server");

    let _ = wait_for_element_children(&page, "#story-log .log-entry", 1).await;

    test_fn(page, port).await;

    let _ = browser.close().await;
}

/// Send an action via the command form
pub async fn send_action(page: &playwright_rs::Page, text: &str) {
    let text_owned = text.to_string();
    let _: Result<(), _> = page
        .evaluate(
            r#"
            (text) => {
                const input = document.querySelector('#command-form input[name="command"]');
                const btn = document.querySelector('#command-form button[type="submit"]');
                if (input) {
                    input.value = text;
                }
                if (btn) {
                    btn.click();
                }
            }
            "#,
            Some(&text_owned),
        )
        .await;

    // Wait for status to leave "Ready" (proves the action was received).
    // For sync actions this times out harmlessly since status stays Ready.
    let status_locator = page.locator("#status-display").await;
    let _ = expect(status_locator)
        .with_timeout(Duration::from_millis(500))
        .not()
        .to_contain_text("Ready")
        .await;

    // If text check dialog appeared, click "Send Original" to proceed.
    dismiss_text_check_if_present(page).await;
}

/// Detect and dismiss the text check "Did you mean?" dialog by clicking
/// "Send Original". Without it, the dialog replaces the action-area and
/// removes #status-display, breaking status polling.
pub async fn dismiss_text_check_if_present(page: &playwright_rs::Page) {
    let locator = page.locator(".text-check-preview .btn-original").await;
    if let Ok(true) = locator.is_visible().await {
        eprintln!("⚠️  Text check dialog detected — clicking 'Send Original'");
        let _ = locator.click(None).await;
        // Wait for dialog to disappear and action-area to be restored
        let _ = locator
            .wait_for(Some(playwright_rs::WaitForOptions {
                state: Some(playwright_rs::WaitForState::Hidden),
                timeout: Some(5000.0),
            }))
            .await;
    }
}

/// Get current status text from the status display element
pub async fn get_status(page: &playwright_rs::Page) -> String {
    page.locator("#status-display")
        .await
        .inner_text()
        .await
        .unwrap_or_default()
}

/// Check if a DOM element exists.
pub async fn element_exists(page: &playwright_rs::Page, selector: &str) -> bool {
    let s = selector.to_string();
    page.evaluate::<String, bool>(
        "(selector) => document.querySelector(selector) !== null",
        Some(&s),
    )
    .await
    .unwrap()
}

/// Count elements matching a selector.
pub async fn element_count(page: &playwright_rs::Page, selector: &str) -> i32 {
    let s = selector.to_string();
    page.evaluate::<String, i32>(
        "(selector) => document.querySelectorAll(selector).length",
        Some(&s),
    )
    .await
    .unwrap()
}

/// Count log entries in story log (instant snapshot)
pub async fn count_log_entries(page: &playwright_rs::Page) -> usize {
    page.query_selector_all("#story-log .log-entry")
        .await
        .unwrap_or_default()
        .len()
}

/// Wait until story log has at least `min_count` entries
pub async fn wait_for_log_entries(page: &playwright_rs::Page, min_count: usize) -> usize {
    wait_for_element_children(page, "#story-log .log-entry", min_count as u32).await as usize
}

/// Capture screenshot and DOM dump when a test fails for debugging.
/// Saves to `tmp/screenshots/` and `tmp/test_diagnostics/`.
pub async fn capture_failure_state(page: &playwright_rs::Page, test_name: &str) {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let safe_name = test_name.replace(|c: char| !c.is_alphanumeric() && c != '_', "_");

    let screenshots_dir = std::path::PathBuf::from("tmp/screenshots");
    let diagnostics_dir = std::path::PathBuf::from("tmp/test_diagnostics");
    let _ = std::fs::create_dir_all(&screenshots_dir);
    let _ = std::fs::create_dir_all(&diagnostics_dir);

    let screenshot_path = screenshots_dir.join(format!("{timestamp}_{safe_name}.png"));
    match page.screenshot_to_file(&screenshot_path, None).await {
        Ok(_) => println!("📸 Screenshot saved: {}", screenshot_path.display()),
        Err(e) => println!("⚠️  Failed to capture screenshot: {e}"),
    }

    let html_path = diagnostics_dir.join(format!("{safe_name}.html"));
    match page.content().await {
        Ok(html) => {
            if let Err(e) = std::fs::write(&html_path, html) {
                println!("⚠️  Failed to write DOM dump: {e}");
            } else {
                println!("📄 DOM dump saved: {}", html_path.display());
            }
        }
        Err(e) => println!("⚠️  Failed to get page content: {e}"),
    }
}
