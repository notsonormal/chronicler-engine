//! [DOC: docs/reference/testing.md]

#![allow(dead_code)]

use std::collections::HashMap;
use std::fs;
use std::net::TcpListener;
use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use playwright_rs::LaunchOptions;
use playwright_rs::Playwright;
use playwright_rs::expect;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

static SERVER_MANAGED: AtomicBool = AtomicBool::new(false);

pub fn port_in_use(port: u16) -> bool {
    std::net::TcpStream::connect(format!("127.0.0.1:{port}")).is_ok()
}

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

pub fn kill_existing_server() {
    // Only kill if we manage the server (to avoid killing other test instances)
    if SERVER_MANAGED.load(Ordering::SeqCst) {
        let _ = Command::new("taskkill")
            .args(["/F", "/IM", "chronicler_engine.exe"])
            .output();
        // Wait for the port to be released (2 seconds for Windows)
        std::thread::sleep(std::time::Duration::from_millis(2000));
    }
}

/// Start the server with optional mock LLM backend.
/// When use_mock is true, writes a temporary settings file with Mock
/// connections and points the server to it via CHRONICLER_SETTINGS_PATH.
/// Returns the spawned child process and an optional temp directory path
/// that should be cleaned up when the server shuts down.
pub fn start_server_with_env(
    port: u16,
    world: &str,
    use_mock: bool,
) -> (Child, Option<std::path::PathBuf>) {
    // Prefer pre-built binary to avoid per-test compilation overhead.
    // Fall back to cargo run for fresh clones or after cargo clean.
    // Respect CARGO_TARGET_DIR for concurrent builds with custom target directories.
    let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string());
    let binary_path = if cfg!(windows) {
        format!("{target_dir}/debug/chronicler_engine.exe")
    } else {
        format!("{target_dir}/debug/chronicler_engine")
    };

    let mut cmd = if std::path::Path::new(&binary_path).exists() {
        let mut c = Command::new(&binary_path);
        c.args(["--world", world, "--port", &port.to_string()]);
        c
    } else {
        let mut c = Command::new("cargo");
        c.args(["run", "--", "--world", world, "--port", &port.to_string()]);
        c
    };

    let tmp_dir = if use_mock {
        let tmp = std::env::temp_dir().join(format!(
            "chronicler_test_settings_{}_{}",
            std::process::id(),
            port
        ));
        let _ = std::fs::create_dir_all(&tmp);
        let settings_path = tmp.join("settings.json");
        let mock_settings = serde_json::json!({
            "connections": [
                {
                    "id": "openrouter-gpt-4o-mini",
                    "name": "openrouter-gpt-4o-mini",
                    "provider": "Mock",
                    "model": "mock-model",
                    "api_key": null,
                    "base_url": null
                },
                {
                    "id": "openrouter-euryale",
                    "name": "openrouter-euryale",
                    "provider": "Mock",
                    "model": "mock-model",
                    "api_key": null,
                    "base_url": null
                }
            ],
            "narration_connection_id": "openrouter-gpt-4o-mini",
            "quantifier_connection_id": "openrouter-gpt-4o-mini"
        });
        std::fs::write(
            &settings_path,
            serde_json::to_string_pretty(&mock_settings).unwrap(),
        )
        .expect("Failed to write mock settings");
        cmd.env("CHRONICLER_SETTINGS_PATH", &settings_path);
        Some(tmp)
    } else {
        None
    };

    cmd.current_dir(".")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let child = cmd.spawn().expect("Failed to start server");
    (child, tmp_dir)
}

pub async fn wait_for_server(port: u16, max_attempts: usize) -> bool {
    for _ in 0..max_attempts {
        if port_in_use(port) {
            // Port is open - give it more time to be fully ready
            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

            // Try to connect and verify server responds
            if std::net::TcpStream::connect(format!("127.0.0.1:{port}")).is_ok() {
                // Give extra time for server to initialize
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                return true;
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    false
}

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

pub async fn wait_for_location_change(page: &playwright_rs::Page, initial: &str) -> String {
    for _ in 0..50 {
        let location: String = page
            .evaluate::<(), String>("document.querySelector('.location')?.innerText || ''", None)
            .await
            .unwrap_or_default();

        if location != initial {
            return location;
        }
        sleep(Duration::from_millis(200)).await;
    }
    String::new()
}

pub async fn wait_for_story_log_change(page: &playwright_rs::Page, initial: &str) -> String {
    for _ in 0..50 {
        let content: String = page
            .evaluate::<(), String>(
                "document.querySelector('#story-log')?.innerText || ''",
                None,
            )
            .await
            .unwrap_or_default();

        if content != initial {
            return content;
        }
        sleep(Duration::from_millis(200)).await;
    }
    String::new()
}

pub async fn wait_for_more_messages(page: &playwright_rs::Page, initial_count: usize) -> usize {
    for _ in 0..50 {
        let messages: Vec<String> = page
            .evaluate::<(), Vec<String>>(
                "Array.from(document.querySelectorAll('#story-log .log-entry .text')).map(el => el.innerText)",
                None,
            )
            .await
            .unwrap_or_default();

        if messages.len() > initial_count {
            return messages.len();
        }
        sleep(Duration::from_millis(200)).await;
    }
    initial_count
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

pub async fn wait_for_element_class(
    page: &playwright_rs::Page,
    selector: &str,
    class_name: &str,
    max_attempts: u32,
) -> bool {
    for _ in 0..max_attempts {
        let has_class: bool = page
            .evaluate::<(), bool>(
                &format!("document.querySelector('{selector}')?.classList.contains('{class_name}') ?? false"),
                None,
            )
            .await
            .unwrap_or(false);

        if has_class {
            return true;
        }
        sleep(Duration::from_millis(250)).await;
    }
    false
}

pub async fn wait_for_element_children(
    page: &playwright_rs::Page,
    selector: &str,
    min_count: u32,
) -> u32 {
    let locator = page.locator(selector).await;
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(10);

    while start.elapsed() < timeout {
        match locator.count().await {
            Ok(count) if count as u32 >= min_count => return count as u32,
            _ => sleep(Duration::from_millis(200)).await,
        }
    }

    capture_failure_state(page, &format!("wait_for_element_children_{selector}")).await;
    0
}

pub async fn wait_for_element_text(page: &playwright_rs::Page, selector: &str) -> String {
    let locator = page.locator(selector).await;
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(10);

    while start.elapsed() < timeout {
        match locator.inner_text().await {
            Ok(text) if !text.is_empty() => return text,
            _ => sleep(Duration::from_millis(200)).await,
        }
    }

    capture_failure_state(page, &format!("wait_for_element_text_{selector}")).await;
    String::new()
}

// ============ Shared Helper Functions ============
// These functions are duplicated across multiple test files.
// Centralized here to ensure consistency and reduce duplication.

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

/// Wait for an element to become visible
pub async fn wait_for_element_exists(
    page: &playwright_rs::Page,
    selector: &str,
    max_attempts: u32,
) {
    let locator = page.locator(selector).await;
    let timeout_ms = max_attempts as f64 * 50.0;
    if let Err(e) = expect(locator)
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
        capture_failure_state(page, &format!("wait_for_element_not_exists_{selector}")).await;
        panic!("Element '{selector}' did not become hidden: {e}");
    }
}

pub async fn wait_for_status_ready(page: &playwright_rs::Page) {
    let locator = page.locator("#status-display").await;
    if let Err(e) = expect(locator).to_contain_text("Ready").await {
        capture_failure_state(page, "wait_for_status_ready").await;
        panic!("Status did not become Ready: {e}");
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub port_range: PortRange,
    pub default_backend: String,
    #[serde(default)]
    pub test_specific: HashMap<String, TestSpecificConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortRange {
    pub min: u16,
    pub max: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSpecificConfig {
    #[serde(default)]
    pub backend: Option<String>,
}

impl TestConfig {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read config file: {e}"))?;
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse config: {e}"))
    }

    /// Get the backend for a specific test file
    pub fn get_backend(&self, test_name: &str) -> String {
        self.test_specific
            .get(test_name)
            .and_then(|c| c.backend.clone())
            .unwrap_or_else(|| self.default_backend.clone())
    }
}

pub fn get_available_port(min: u16, max: u16) -> Result<u16, String> {
    let lock_dir = std::env::temp_dir().join("chronicler_test_ports");
    let _ = std::fs::create_dir_all(&lock_dir);

    let mut attempts = 20;
    let mut delay_ms = 50;

    while attempts > 0 {
        for port in min..=max {
            let lock_path = lock_dir.join(format!("port_{port}.lock"));

            // Try to create lock file exclusively (atomic on most filesystems)
            if std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&lock_path)
                .is_err()
            {
                // Lock exists, port reserved by another test
                continue;
            }

            // We have the lock file — verify port is actually free
            match TcpListener::bind(format!("127.0.0.1:{port}")) {
                Ok(listener) => {
                    drop(listener);
                    // Write PID to lock file for debugging stale locks
                    let _ = std::fs::write(&lock_path, format!("{}", std::process::id()));
                    return Ok(port);
                }
                Err(_) => {
                    // Port not actually available, release lock
                    let _ = std::fs::remove_file(&lock_path);
                    continue;
                }
            }
        }

        // All ports in range were locked — clean stale locks and retry
        if let Ok(entries) = std::fs::read_dir(&lock_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(pid) = content.trim().parse::<u32>() {
                        // Check if the process is still alive
                        if !is_process_alive(pid) {
                            let _ = std::fs::remove_file(&path);
                        }
                    }
                }
            }
        }

        if attempts > 1 {
            attempts -= 1;
            std::thread::sleep(std::time::Duration::from_millis(delay_ms));
            delay_ms = (delay_ms * 2).min(500);
        }
    }
    Err(format!(
        "No available ports in range {}-{} after {} attempts",
        min, max, 20
    ))
}

pub fn release_port_lock(port: u16) {
    let lock_dir = std::env::temp_dir().join("chronicler_test_ports");
    let lock_path = lock_dir.join(format!("port_{port}.lock"));
    let _ = std::fs::remove_file(&lock_path);
}

fn is_process_alive(pid: u32) -> bool {
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/NH"])
            .output();
        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.contains(&pid.to_string())
        } else {
            false
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        unsafe { libc::kill(pid as i32, 0) == 0 }
    }
}

pub fn get_config_port(config_path: &str) -> Result<u16, String> {
    let config = TestConfig::from_file(config_path)?;
    get_available_port(config.port_range.min, config.port_range.max)
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

pub struct TestServer {
    child: Child,
    port: u16,
    temp_dir: Option<std::path::PathBuf>,
}

impl TestServer {
    pub async fn with_config(port: u16, world: &str, use_mock: bool) -> Self {
        Self::start(port, world, use_mock).await
    }

    pub async fn from_config(
        world: &str,
        config_path: &str,
        test_name: &str,
    ) -> Result<(Self, u16), String> {
        let config = TestConfig::from_file(config_path)?;
        let port = get_available_port(config.port_range.min, config.port_range.max)?;
        let use_mock = config.get_backend(test_name) == "mock";
        let server = Self::start(port, world, use_mock).await;
        Ok((server, port))
    }

    async fn start(port: u16, world: &str, use_mock: bool) -> Self {
        if port_in_use(port) {
            kill_existing_server();
        }
        let (child, temp_dir) = start_server_with_env(port, world, use_mock);
        // Increased wait time for server to be fully ready
        let started = wait_for_server(port, 100).await; // 100 * 100ms = 10s total
        assert!(started, "Server failed to start on port {port}");
        SERVER_MANAGED.store(true, Ordering::SeqCst);
        TestServer {
            child,
            port,
            temp_dir,
        }
    }

    pub async fn new(port: u16, world: &str) -> Self {
        Self::start(port, world, false).await
    }

    pub async fn new_with_mock(port: u16, world: &str) -> Self {
        Self::start(port, world, true).await
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        std::thread::sleep(std::time::Duration::from_millis(500));
        let _ = self.child.kill();
        let _ = self.child.wait();
        SERVER_MANAGED.store(false, Ordering::SeqCst);
        release_port_lock(self.port);
        if let Some(tmp) = &self.temp_dir {
            let _ = std::fs::remove_dir_all(tmp);
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}
