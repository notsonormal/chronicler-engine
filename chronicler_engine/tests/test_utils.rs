//! Test Utilities for Chronicler Engine UI Tests
//!
//! Shared utilities for managing test server lifecycle and smart waiting

#![allow(dead_code)]

use playwright_rs::Playwright;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::net::TcpListener;
use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

static SERVER_MANAGED: AtomicBool = AtomicBool::new(false);

pub fn port_in_use(port: u16) -> bool {
    std::net::TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok()
}

pub fn kill_existing_server() {
    let _ = Command::new("taskkill")
        .args(&["/F", "/IM", "chronicler_engine.exe"])
        .output();
    // Wait longer for the port to be released
    std::thread::sleep(std::time::Duration::from_millis(500));
}

/// Start the server with optional mock LLM backend
/// If use_mock is true, sets LLM_BACKEND=mock env var
pub fn start_server_with_env(port: u16, world: &str, use_mock: bool) -> Child {
    let mut cmd = Command::new("cargo");
    cmd.args(&["run", "--", "--world", world, "--port", &port.to_string()]);

    // Set env var for mock LLM if requested
    if use_mock {
        cmd.env("LLM_BACKEND", "mock");
    }

    cmd.current_dir(".")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    cmd.spawn().expect("Failed to start server")
}

pub fn wait_for_server(port: u16, max_attempts: usize) -> bool {
    for _ in 0..max_attempts {
        if port_in_use(port) {
            // Port is open - give it more time to be fully ready
            std::thread::sleep(std::time::Duration::from_millis(300));

            // Try to connect and verify server responds
            if std::net::TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok() {
                // Give extra time for server to initialize
                std::thread::sleep(std::time::Duration::from_millis(200));
                return true;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    false
}

/// Wait for LLM to finish generating a response by polling the generating status endpoint.
/// Returns Ok(()) if LLM became idle within timeout, Err(()) if timeout exceeded.
pub async fn wait_for_llm_idle(port: u16, timeout: Duration) -> Result<(), ()> {
    let start = std::time::Instant::now();
    let client = reqwest::Client::new();

    while start.elapsed() < timeout {
        match client
            .get(&format!("http://127.0.0.1:{}/status/generating", port))
            .send()
            .await
        {
            Ok(resp) => {
                if let Ok(text) = resp.text().await {
                    if text == "idle" {
                        return Ok(());
                    }
                }
            }
            Err(_) => {}
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    Err(())
}

/// Poll until story log has at least the expected number of entries
/// Returns the actual count after polling, or the expected count if timeout
pub async fn wait_for_story_log_entries(page: &playwright_rs::Page, min_count: u32) -> u32 {
    use tokio::time::sleep;

    for _ in 0..20 {
        let count: u32 = page
            .evaluate::<(), u32>(
                "document.querySelectorAll('#story-log .log-entry').length",
                None,
            )
            .await
            .unwrap_or(0);

        if count >= min_count {
            return count;
        }
        sleep(Duration::from_millis(500)).await;
    }
    page.evaluate::<(), u32>(
        "document.querySelectorAll('#story-log .log-entry').length",
        None,
    )
    .await
    .unwrap_or(0)
}

/// Poll until location text changes from the initial value
/// Returns the new location text, or empty string if timeout
pub async fn wait_for_location_change(page: &playwright_rs::Page, initial: &str) -> String {
    use tokio::time::sleep;

    for _ in 0..20 {
        let location: String = page
            .evaluate::<(), String>("document.querySelector('.location')?.innerText || ''", None)
            .await
            .unwrap_or_default();

        if location != initial {
            return location;
        }
        sleep(Duration::from_millis(500)).await;
    }
    String::new()
}

/// Poll until story log content changes from initial content
/// Returns the new content, or empty string if timeout
pub async fn wait_for_story_log_change(page: &playwright_rs::Page, initial: &str) -> String {
    use tokio::time::sleep;

    for _ in 0..20 {
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
        sleep(Duration::from_millis(500)).await;
    }
    String::new()
}

/// Poll until story log has more messages than initial count
/// Returns the new message count, or initial_count if timeout
pub async fn wait_for_more_messages(page: &playwright_rs::Page, initial_count: usize) -> usize {
    use tokio::time::sleep;

    for _ in 0..20 {
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
        sleep(Duration::from_millis(500)).await;
    }
    initial_count
}

/// Poll until element has a non-empty value (not "Loading...")
/// Returns the element's inner text, or empty string if timeout
pub async fn wait_for_non_loading_value(page: &playwright_rs::Page, selector: &str) -> String {
    use tokio::time::sleep;

    for _ in 0..20 {
        let value: String = page
            .evaluate::<(), String>(
                &format!("document.querySelector('{}')?.innerText || ''", selector),
                None,
            )
            .await
            .unwrap_or_default();

        if !value.is_empty() {
            return value;
        }
        sleep(Duration::from_millis(500)).await;
    }
    String::new()
}

/// Poll until element exists and has at least min_count children
/// Returns the count of children, or 0 if timeout
pub async fn wait_for_element_children(
    page: &playwright_rs::Page,
    selector: &str,
    min_count: u32,
) -> u32 {
    use tokio::time::sleep;

    for _ in 0..20 {
        let count: u32 = page
            .evaluate::<(), u32>(
                &format!("document.querySelectorAll('{}').length", selector),
                None,
            )
            .await
            .unwrap_or(0);

        if count >= min_count {
            return count;
        }
        sleep(Duration::from_millis(500)).await;
    }
    0
}

/// Poll until element has text content (not empty)
/// Returns the text content, or empty string if timeout
pub async fn wait_for_element_text(page: &playwright_rs::Page, selector: &str) -> String {
    use tokio::time::sleep;

    for _ in 0..20 {
        let text: String = page
            .evaluate::<(), String>(
                &format!("document.querySelector('{}')?.innerText || ''", selector),
                None,
            )
            .await
            .unwrap_or_default();

        if !text.is_empty() {
            return text;
        }
        sleep(Duration::from_millis(500)).await;
    }
    String::new()
}

/// Wait for LLM backend to finish processing (direct HTTP check, not UI)
/// Returns Ok(()) if LLM became idle within timeout, Err(()) if timeout exceeded.
pub async fn wait_for_llm_backend_idle(port: u16, timeout: Duration) -> Result<(), ()> {
    let start = std::time::Instant::now();
    let client = reqwest::Client::new();

    while start.elapsed() < timeout {
        match client
            .get(&format!("http://127.0.0.1:{}/status/generating", port))
            .send()
            .await
        {
            Ok(resp) => {
                if let Ok(text) = resp.text().await {
                    if text == "idle" {
                        return Ok(());
                    }
                }
            }
            Err(_) => {}
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    Err(())
}

/// Poll until status display shows "Ready" (not "Thinking")
/// This indicates synchronous action processing is complete
pub async fn wait_for_status_ready(page: &playwright_rs::Page) {
    use tokio::time::sleep;

    for i in 0..50 {
        let status: String = page
            .evaluate::<(), String>(
                "document.querySelector('#status-display')?.innerText || ''",
                None,
            )
            .await
            .unwrap_or_default();

        if status.contains("Ready") {
            return;
        }
        if i == 49 {
            println!("WAIT_FOR_STATUS_READY TIMEOUT - status: '{}'", status);
        }
        sleep(Duration::from_millis(100)).await;
    }
}

/// Launch Chrome browser for testing
pub async fn launch_chrome() -> (playwright_rs::Playwright, playwright_rs::Browser) {
    use playwright_rs::LaunchOptions;

    let playwright = Playwright::launch().await.unwrap();
    let browser = playwright
        .chromium()
        .launch_with_options(LaunchOptions {
            channel: Some("chrome".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();
    (playwright, browser)
}

/// Test configuration loaded from JSON file
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
    /// Load test configuration from JSON file
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read config file: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))
    }

    /// Get the backend for a specific test file
    pub fn get_backend(&self, test_name: &str) -> String {
        self.test_specific
            .get(test_name)
            .and_then(|c| c.backend.clone())
            .unwrap_or_else(|| self.default_backend.clone())
    }
}

/// Find an available port in the given range
pub fn get_available_port(min: u16, max: u16) -> Result<u16, String> {
    for port in min..=max {
        match TcpListener::bind(format!("127.0.0.1:{}", port)) {
            Ok(_) => return Ok(port),
            Err(_) => continue,
        }
    }
    Err(format!("No available ports in range {}-{}", min, max))
}

/// Get a dynamic port from config file (convenience function)
pub fn get_config_port(config_path: &str) -> Result<u16, String> {
    let config = TestConfig::from_file(config_path)?;
    get_available_port(config.port_range.min, config.port_range.max)
}

pub struct TestServer {
    child: Child,
}

impl TestServer {
    /// Create a test server with config (dynamic port + config-based backend)
    pub fn with_config(port: u16, world: &str, use_mock: bool) -> Self {
        Self::start(port, world, use_mock)
    }

    /// Create a test server using config file for port and backend
    /// Returns (TestServer, port_used) - port is dynamically allocated
    pub fn from_config(
        world: &str,
        config_path: &str,
        test_name: &str,
    ) -> Result<(Self, u16), String> {
        let config = TestConfig::from_file(config_path)?;
        let port = get_available_port(config.port_range.min, config.port_range.max)?;
        let use_mock = config.get_backend(test_name) == "mock";
        let server = Self::start(port, world, use_mock);
        Ok((server, port))
    }

    /// Internal: start the server with given parameters
    fn start(port: u16, world: &str, use_mock: bool) -> Self {
        if port_in_use(port) {
            kill_existing_server();
        }
        let child = start_server_with_env(port, world, use_mock);
        // Increased wait time for server to be fully ready
        let started = wait_for_server(port, 100); // 100 * 100ms = 10s total
        assert!(started, "Server failed to start on port {}", port);
        SERVER_MANAGED.store(true, Ordering::SeqCst);
        TestServer { child }
    }

    /// Create a test server with real LLM backend
    pub fn new(port: u16, world: &str) -> Self {
        Self::start(port, world, false)
    }

    /// Create a test server with mock LLM backend
    pub fn new_with_mock(port: u16, world: &str) -> Self {
        Self::start(port, world, true)
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        std::thread::sleep(std::time::Duration::from_millis(500));
        let _ = self.child.kill();
        let _ = self.child.wait();
        SERVER_MANAGED.store(false, Ordering::SeqCst);
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}
