//! Test Utilities for Chronicler Engine UI Tests
//!
//! Shared utilities for managing test server lifecycle and smart waiting

#![allow(dead_code)]

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
    std::thread::sleep(std::time::Duration::from_millis(300));
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
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
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

pub struct TestServer {
    child: Child,
}

impl TestServer {
    /// Create a test server with real LLM backend
    pub fn new(port: u16, world: &str) -> Self {
        Self::with_config(port, world, false)
    }

    /// Create a test server with mock LLM backend
    pub fn new_with_mock(port: u16, world: &str) -> Self {
        Self::with_config(port, world, true)
    }

    /// Internal: create server with specified config
    fn with_config(port: u16, world: &str, use_mock: bool) -> Self {
        if port_in_use(port) {
            kill_existing_server();
        }
        let child = start_server_with_env(port, world, use_mock);
        assert!(wait_for_server(port, 30), "Server failed to start");
        SERVER_MANAGED.store(true, Ordering::SeqCst);
        TestServer { child }
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        std::thread::sleep(std::time::Duration::from_millis(200));
        let _ = self.child.kill();
        let _ = self.child.wait();
        SERVER_MANAGED.store(false, Ordering::SeqCst);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
