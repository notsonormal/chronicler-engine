//! Browser test helpers: Playwright bootstrap (`TestServer`, `LaunchOptions`), page builders, and the tab/panel open helpers.

use std::time::Duration;

use playwright_rs::LaunchOptions;
use playwright_rs::Playwright;

use super::convergence::assert_screen_converged;
use super::htmx_settle::{await_panel_ready, click_and_settle, install_htmx_settle};
use super::html::preset_card_html_slice;
use super::server::{buffer_text, get_config_port, registered_server_logs, tail_lines, TestServer};
use super::stub_server::StubServer;
#[allow(unused_imports)]
pub use super::wait::wait_for_element_children;
use super::wait::{wait_for_condition_async, wait_until_visible};
#[allow(unused_imports)]
pub use super::wait::wait_for_status_ready;
use super::wait::wait_for_action_acknowledged;
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
                 Check whether another process is listening on port {port}",
            )
        } else {
            format!("Navigation failed to {url}: {err_str}")
        }
    })?;
    Ok(())
}

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

/// The driver process inherits the test's stdio, so it has to be gone before the
/// test returns: nextest reports a child that outlives it as a leak.
async fn close_chrome(playwright: &playwright_rs::Playwright, browser: &playwright_rs::Browser) {
    let _ = browser.close().await;
    let _ = playwright.shutdown().await;
}

/// After the test, the page must converge: each visible live region equals a
/// fresh GET of its route.
pub async fn with_test_page<F, Fut>(config_path: &str, world: &str, persona: &str, test_fn: F)
where
    F: FnOnce(playwright_rs::Page, u16) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let port = get_config_port(config_path).expect("Failed to get config port");
    let _server = TestServer::new_with_mock(port, world, persona).await;

    let (playwright, browser) = launch_chrome().await;
    let page = browser.new_page().await.unwrap();

    goto_with_connection_check(&page, port)
        .await
        .expect("Failed to connect to server");

    test_fn(page.clone(), port).await;
    assert_screen_converged(&page).await;

    close_chrome(&playwright, &browser).await;
}

/// Shared per test binary, so the launch cost is paid once rather than per test.
pub struct SharedBrowser {
    _playwright: playwright_rs::Playwright,
    browser: playwright_rs::Browser,
}

impl SharedBrowser {
    pub async fn launch() -> Self {
        let (playwright, browser) = launch_chrome().await;
        Self {
            _playwright: playwright,
            browser,
        }
    }

    pub async fn close(&self) {
        close_chrome(&self._playwright, &self.browser).await;
    }

    pub async fn open_page(&self, stub: &StubServer) -> playwright_rs::Page {
        let page = self.browser.new_page().await.unwrap();
        let url = stub.url();
        install_htmx_settle(&page).await;
        page.goto(&url, None)
            .await
            .unwrap_or_else(|e| panic!("navigate to stub at {url}: {e}"));
        wait_for_story_log(&page).await;
        make_regions_current(&page).await;
        page
    }
}

/// The first ask of the page loads every region once. A stub test starts after
/// that ask has settled: only a number the test moves can load a region
/// afterwards, and a state the test writes into a region is not replaced by a
/// late first ask.
async fn make_regions_current(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>(
        "async () => { await window.chroniclerLive.settle(); await window.chroniclerLive.askOnce(); }",
        None,
    )
    .await
    .expect("the stub page makes its live regions current");
}

/// A panel that is not a live region loads at page load, and a live region in a
/// hidden section loads when the section opens. Either swap is on the recorded
/// target list by the time `await_panel_ready` reads it, because that wait
/// searches the whole list, so the wait follows the click for both kinds.
async fn open_tab(page: &playwright_rs::Page, tab: &str, gate_target: &str) {
    page.locator(&format!(r#"[data-tab="{tab}"]"#))
        .await
        .click(None)
        .await
        .unwrap_or_else(|e| panic!("click tab '{tab}' failed: {e}"));
    await_panel_ready(page, gate_target)
        .await
        .expect_settled(tab);
}

pub async fn open_worlds_tab(page: &playwright_rs::Page) {
    open_tab(page, "worlds", ".worlds-panel").await;
    wait_for_element_children(page, "#worlds-tab .world-item", 1).await;
}

pub async fn open_games_tab(page: &playwright_rs::Page) {
    open_tab(page, "games", ".games-panel").await;
    wait_until_visible(page, "#game-posture-controls", Duration::from_millis(5000)).await;
}

pub async fn open_prompt_presets_tab(page: &playwright_rs::Page) {
    open_tab(page, "prompt-presets", ".prompt-presets-panel").await;
}

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
    click_and_settle(page, edit_selector, "#world-editor").await;
    // Wait on the posture selects, not #world-posture-status: an empty
    // (zero-sized) span never becomes visible.
    wait_until_visible(
        page,
        r#"#worlds-tab select[name="narrator_mode"]"#,
        Duration::from_millis(5000),
    )
    .await;
}

/// Cards carry no `data-id`, so `card_selector` is the caller's stable anchor,
/// e.g. `.preset-card:has(.card-title:text-is("My Preset"))`.
pub async fn open_preset_editor(page: &playwright_rs::Page, card_selector: &str) {
    let edit_selector = format!("{card_selector} button:has-text('Edit')");
    click_and_settle(page, &edit_selector, ".preset-card").await;
    wait_until_visible(page, ".preset-card.edit-form", Duration::from_millis(5000)).await;
}

/// Choose `value` in an auto-save select whose answer swaps nothing, and wait for
/// the browser to show "Saved" on `#flag_id`: the line that the player reads,
/// which the browser draws once the request ended well. An empty answer fires no
/// htmx settle, so the visible line is the observable end of the save.
pub async fn select_option_and_await_saved(
    page: &playwright_rs::Page,
    selector: &str,
    value: &str,
    flag_id: &str,
) {
    page.locator(selector)
        .await
        .select_option(value, None)
        .await
        .unwrap_or_else(|e| panic!("select_option('{selector}' -> '{value}') failed: {e}"));
    let script = format!(
        "() => {{ const flag = document.getElementById('{flag_id}'); return getComputedStyle(flag, '::before').content === '\"Saved\"'; }}"
    );
    let saved = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(25),
        || async {
            page.evaluate::<(), bool>(&script, None)
                .await
                .unwrap_or(false)
        },
    )
    .await;
    assert!(
        saved,
        "select_option('{selector}' -> '{value}'): #{flag_id} never showed the text Saved"
    );
}

/// Click a control that closes an editor in the browser, and wait until the
/// editor slot is empty. Closing sends no request, so no htmx settle follows.
pub async fn click_and_close_editor(page: &playwright_rs::Page, selector: &str, slot: &str) {
    page.locator(selector)
        .await
        .click(None)
        .await
        .unwrap_or_else(|e| panic!("click('{selector}') failed: {e}"));
    let script = format!("() => document.querySelector('{slot}').children.length === 0");
    let closed = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(25),
        || async {
            page.evaluate::<(), bool>(&script, None)
                .await
                .unwrap_or(false)
        },
    )
    .await;
    assert!(closed, "click('{selector}') never emptied {slot}");
}

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
        body.is_empty(),
        "seed_system_preset('{name}') must answer nothing: {body}"
    );
    let panel = client
        .get(format!("http://127.0.0.1:{port}/fragment/prompt-presets"))
        .send()
        .await
        .unwrap_or_else(|e| panic!("load the preset panel after seeding '{name}': {e}"))
        .text()
        .await
        .unwrap_or_default();
    assert!(
        preset_card_rendered(&panel, name),
        "seed_system_preset('{name}') found no card for the new preset in the panel: {panel}"
    );
}

fn preset_card_rendered(body: &str, name: &str) -> bool {
    let title_anchor = format!(r#"<span class="card-title">{name}</span>"#);
    preset_card_html_slice(body, &title_anchor).is_some_and(|card| {
        card.contains(r#"hx-post="/prompt-presets/"#) && card.contains("/duplicate")
    })
}

pub fn preset_card_selector(name: &str) -> String {
    format!(r#".preset-card:has(.card-title:text-is("{name}"))"#)
}

pub async fn fill_command_input(page: &playwright_rs::Page, command: &str) {
    let command = command.to_string();
    let _: Result<(), _> = page
        .evaluate::<String, ()>(
            r#"
            (command) => {
                const input = document.querySelector('#command-form input[name="command"]');
                if (input) {
                    input.value = command;
                }
            }
            "#,
            Some(&command),
        )
        .await;
}

pub async fn send_action(page: &playwright_rs::Page, text: &str) {
    fill_command_input(page, text).await;
    let _: Result<(), _> = page
        .evaluate::<(), ()>(
            r#"
            () => {
                const btn = document.querySelector('#command-form button[type="submit"]');
                if (btn) {
                    btn.click();
                }
            }
            "#,
            None,
        )
        .await;

    // Confirm the action was acknowledged before returning: the action area
    // consumes the input of a command the engine took. Without this wait, a
    // subsequent `wait_for_status_ready` can return on the STALE pre-action
    // "Ready" — see `wait_for_action_acknowledged`.
    wait_for_action_acknowledged(page).await;

    dismiss_text_check_if_present(page).await;
}

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

pub async fn count_log_entries(page: &playwright_rs::Page) -> usize {
    page.query_selector_all("#story-log .log-entry")
        .await
        .unwrap_or_default()
        .len()
}

/// The tuple is (shown, short line, raw text behind the Details control).
pub async fn read_error_disclosure(
    page: &playwright_rs::Page,
    selector: &str,
) -> (bool, String, String) {
    page.evaluate::<String, (bool, String, String)>(
        r#"(selector) => {
            const container = document.querySelector(selector);
            if (!container) return [false, '', ''];
            const message = container.querySelector('.error-disclosure-message');
            const raw = container.querySelector('.error-detail-raw');
            return [
                !container.hidden && !!message,
                message ? message.textContent.trim() : '',
                raw ? raw.textContent.trim() : '',
            ];
        }"#,
        Some(&selector.to_string()),
    )
    .await
    .unwrap()
}

/// Waits until `selector` shows an error disclosure. `shows` is the failure
/// message of the wait.
pub async fn wait_for_error_disclosure(page: &playwright_rs::Page, selector: &str, shows: &str) {
    assert!(
        wait_for_condition_async(
            Duration::from_secs(5),
            Duration::from_millis(50),
            || async { read_error_disclosure(page, selector).await.0 }
        )
        .await,
        "{shows}"
    );
}

pub async fn error_details_open(page: &playwright_rs::Page, selector: &str) -> bool {
    page.evaluate::<String, bool>(
        r#"(selector) => {
            const container = document.querySelector(selector);
            if (!container) return false;
            const popover = container.querySelector('.error-detail-popover');
            return !!popover && !popover.hidden;
        }"#,
        Some(&selector.to_string()),
    )
    .await
    .unwrap()
}

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

pub async fn status_display_text(page: &playwright_rs::Page) -> String {
    page.locator("#status-display")
        .await
        .inner_text()
        .await
        .unwrap_or_default()
        .trim()
        .to_string()
}
