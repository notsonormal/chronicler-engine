//! Browser test helpers: Playwright bootstrap (`TestServer`, `LaunchOptions`), page builders, and the tab/panel open helpers.

use std::time::Duration;

use playwright_rs::LaunchOptions;
use playwright_rs::Playwright;

use super::htmx_settle::{await_panel_ready, click_and_settle, install_htmx_settle};
use super::server::{buffer_text, get_config_port, registered_server_logs, tail_lines, TestServer};
use super::stub_server::{StubActionOutcome, StubServer};
#[allow(unused_imports)]
pub use super::wait::wait_for_element_children;
use super::wait::wait_until_visible;
#[allow(unused_imports)]
pub use super::wait::wait_for_status_ready;
use super::wait::wait_for_status_generating;
pub use super::wait::wait_for_story_log;

pub async fn goto_with_connection_check(
    page: &playwright_rs::Page,
    port: u16,
) -> Result<(), String> {
    let url = format!("http://127.0.0.1:{port}");

    install_htmx_settle(page).await;

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
pub async fn with_test_page<F, Fut>(config_path: &str, world: &str, persona: &str, test_fn: F)
where
    F: FnOnce(playwright_rs::Page, u16) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let port = get_config_port(config_path).expect("Failed to get config port");
    let _server = TestServer::new_with_mock(port, world, persona).await;

    let (_playwright, browser) = launch_chrome().await;
    let page = browser.new_page().await.unwrap();

    goto_with_connection_check(&page, port)
        .await
        .expect("Failed to connect to server");

    test_fn(page, port).await;

    let _ = browser.close().await;
}

/// A shared Chromium process for the stub-browser tests, so the
/// browser launch cost is paid once per test binary rather than per test.
/// Dropping it closes the browser.
pub struct SharedBrowser {
    _playwright: playwright_rs::Playwright,
    browser: playwright_rs::Browser,
}

impl SharedBrowser {
    /// Launch the shared browser once.
    pub async fn launch() -> Self {
        let (playwright, browser) = launch_chrome().await;
        Self {
            _playwright: playwright,
            browser,
        }
    }

    /// Open a fresh page on the stub, with the htmx settle counter installed
    /// and the shell's initial fragments loaded. The page is the caller's to close.
    pub async fn open_page(&self, stub: &StubServer) -> playwright_rs::Page {
        let page = self.browser.new_page().await.unwrap();
        let url = stub.url();
        install_htmx_settle(&page).await;
        page.goto(&url, None)
            .await
            .unwrap_or_else(|e| panic!("navigate to stub at {url}: {e}"));
        // The shell's `load`-triggered fragments settle the story log; wait for
        // the canned entry so the test starts from the loaded state.
        wait_for_story_log(&page).await;
        page
    }
}

/// Run a stub-browser check: start a stub (default outcome: pending),
/// drive one fresh page against it, and tear both down. This is the stub-browser
/// entry point — the sibling of `with_test_page` for tests that do not need a
/// real server.
pub async fn with_stub_page<F, Fut>(outcome: StubActionOutcome, test_fn: F)
where
    F: FnOnce(playwright_rs::Page, &StubServer) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let stub = StubServer::start(outcome).await;
    let browser = SharedBrowser::launch().await;
    let page = browser.open_page(&stub).await;
    test_fn(page.clone(), &stub).await;
    let _ = page.close().await;
}

/// Open a tab panel and wait for that panel's own `hx-trigger="load"` swap to
/// settle. A hidden panel's load swap lands before any test arms a baseline, so
/// the wait goes through `await_panel_ready`, which searches the whole recorded
/// target list.
async fn open_tab(page: &playwright_rs::Page, tab: &str, gate_target: &str) {
    await_panel_ready(page, gate_target)
        .await
        .expect_settled(tab);
    page.locator(&format!(r#"[data-tab="{tab}"]"#))
        .await
        .click(None)
        .await
        .unwrap_or_else(|e| panic!("click tab '{tab}' failed: {e}"));
}

/// Open the Worlds tab (`.worlds-panel` load swap awaited).
pub async fn open_worlds_tab(page: &playwright_rs::Page) {
    open_tab(page, "worlds", ".worlds-panel").await;
    wait_for_element_children(page, "#worlds-tab .world-item", 1).await;
}

/// Open the Games tab, then wait for the posture fragment's selects.
pub async fn open_games_tab(page: &playwright_rs::Page) {
    open_tab(page, "games", ".games-panel").await;
    wait_until_visible(page, "#game-posture-controls", Duration::from_millis(5000)).await;
}

/// Open the Prompt Presets tab (`.prompt-presets-panel` load swap awaited).
pub async fn open_prompt_presets_tab(page: &playwright_rs::Page) {
    open_tab(page, "prompt-presets", ".prompt-presets-panel").await;
}

/// Open the edit form for the seeded world "test" and wait for the posture
/// selects.
///
/// The Edit button is scoped to the "Test Realm" world item, not `.first()`:
/// the engine seeds two worlds, so an unscoped Edit selector matches twice and
/// strict mode rejects it, and name-scoping is order-independent.
///
/// The Edit click's swap is awaited through the htmx settle counter: htmx
/// attaches the new form's `hx-trigger` listeners at the end of that settle,
/// so a `change` fired earlier is lost.
pub async fn open_world_edit(page: &playwright_rs::Page) {
    open_worlds_tab(page).await;
    let edit_selector = r#".world-item:has(strong:text-is("Test Realm")) button:has-text('Edit')"#;
    click_and_settle(page, edit_selector, ".worlds-panel").await;
    // Wait on the posture selects, not #world-posture-status: an empty
    // (zero-sized) span never becomes visible.
    wait_until_visible(
        page,
        r#"#worlds-tab select[name="narrator_mode"]"#,
        Duration::from_millis(5000),
    )
    .await;
}

/// Click a preset card's Edit button and wait for that card's swap. The card
/// carries no `data-id`, so `card_selector` is the caller's stable anchor,
/// e.g. `.preset-card:has(.card-title:text-is("My Preset"))`.
pub async fn open_preset_editor(page: &playwright_rs::Page, card_selector: &str) {
    let edit_selector = format!("{card_selector} button:has-text('Edit')");
    click_and_settle(page, &edit_selector, ".preset-card").await;
    wait_until_visible(page, ".preset-card.edit-form", Duration::from_millis(5000)).await;
}

/// Create a system preset through the engine's own HTTP API, before the
/// browser interacts with it — seeding goes through the same `POST
/// /prompt-presets` route the Create form uses, not a test-only endpoint.
pub async fn seed_system_preset(port: u16, name: &str, instructions: &str) {
    let client = reqwest::Client::new();
    let form = [
        ("preset_type", "system"),
        ("name", name),
        ("instructions", instructions),
        // Seed a single mode on purpose. With both flags absent the handler
        // falls back to `default_allowed_modes()` (Novel + IF), which would
        // render both activation buttons before the guard's edit — making the
        // guard's post-save assertion vacuous.
        ("allowed_mode_novel", "true"),
    ];
    let response = client
        .post(format!("http://127.0.0.1:{port}/prompt-presets"))
        .form(&form)
        .send()
        .await
        .unwrap_or_else(|e| panic!("seed_system_preset('{name}') request failed: {e}"));
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    assert!(
        status.is_success(),
        "seed_system_preset('{name}') returned {status}: {body}"
    );
    assert!(
        preset_card_rendered(&body, name),
        "seed_system_preset('{name}') found no card for the new preset in the panel: {body}"
    );
}

/// True when the returned panel HTML carries a card titled `name` with a
/// duplicate route. Cards carry no `data-id`, so the duplicate button is the
/// stable anchor.
fn preset_card_rendered(body: &str, name: &str) -> bool {
    let title_anchor = format!(r#"<span class="card-title">{name}</span>"#);
    let Some(title_pos) = body.find(&title_anchor) else {
        return false;
    };
    let card_start = body[..title_pos]
        .rfind("<div class=\"preset-card")
        .unwrap_or(0);
    let card_end = body[title_pos..]
        .find("<div class=\"preset-card")
        .map(|offset| title_pos + offset)
        .unwrap_or(body.len());
    body[card_start..card_end].contains(r#"hx-post="/prompt-presets/"#)
        && body[card_start..card_end].contains("/duplicate")
}

/// Selector addressing a preset card by its title — cards carry no `data-id`.
pub fn preset_card_selector(name: &str) -> String {
    format!(r#".preset-card:has(.card-title:text-is("{name}"))"#)
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

    // Confirm the action was acknowledged before returning: the
    // `/action/check` response swaps "Thinking..." into #status-display. Without
    // this wait, a subsequent `wait_for_status_ready` can return on the STALE
    // pre-action "Ready" (before the swap arrives) and race the test — see
    // `wait_for_status_generating`.
    wait_for_status_generating(page).await;

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
        let _ = locator
            .wait_for(Some(playwright_rs::WaitForOptions {
                state: Some(playwright_rs::WaitForState::Hidden),
                timeout: Some(5000.0),
            }))
            .await;
    }
}

/// Count log entries in story log (instant snapshot)
pub async fn count_log_entries(page: &playwright_rs::Page) -> usize {
    page.query_selector_all("#story-log .log-entry")
        .await
        .unwrap_or_default()
        .len()
}

/// Capture failure diagnostics before panicking: screenshot to
/// `tmp/screenshots/`, DOM dump + per-server engine logs to
/// `tmp/test_diagnostics/`, and log tails printed into the failure output.
/// Read these before re-running a failed test.
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

    // Engine output: full buffers to files, a tail into the test output.
    // Locks recover from poisoning so one panicking test cannot break
    // another test's diagnostics.
    let logs = registered_server_logs();
    if logs.is_empty() {
        println!("⚠️  No engine logs registered (server already dropped?)");
    }
    for (port, buffers) in logs {
        for (label, buffer) in [("stdout", &buffers.stdout), ("stderr", &buffers.stderr)] {
            let text = buffer_text(buffer);
            let log_path = diagnostics_dir.join(format!("{safe_name}_{port}_{label}.log"));
            if let Err(e) = std::fs::write(&log_path, &text) {
                println!("⚠️  Failed to write engine {label} dump: {e}");
            } else {
                println!(
                    "🧾 Engine {label} (port {port}) saved: {}",
                    log_path.display()
                );
            }
            let tail = tail_lines(&text, 30);
            if !tail.is_empty() {
                eprintln!("--- engine {label} tail (port {port}) ---\n{tail}");
            }
        }
    }
}
