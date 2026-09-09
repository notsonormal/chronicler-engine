//! Test server helpers: spawn the real engine binary on a free port, track lifecycle via `SERVER_MANAGED`, and expose `TestServer` / `wait_for_server` / `get_config_port`.

use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use serde::{Deserialize, Serialize};

static SERVER_MANAGED: AtomicBool = AtomicBool::new(false);

/// Registry of `port -> spawned_child_pid` so `kill_existing_server` can terminate
/// only the server we spawned (not other test instances or unrelated dev servers).
static PORT_PIDS: OnceLock<Mutex<HashMap<u16, u32>>> = OnceLock::new();

fn port_pids() -> &'static Mutex<HashMap<u16, u32>> {
    PORT_PIDS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Live stdout/stderr buffers for every engine this process spawned, keyed by
/// port. `capture_failure_state` reads it to dump engine output on browser-
/// test failure; `Drop` deregisters so a later test in this process never
/// sees a dead server's logs.
static SERVER_LOG_REGISTRY: OnceLock<Mutex<HashMap<u16, ServerLogBuffers>>> = OnceLock::new();

#[derive(Clone)]
pub struct ServerLogBuffers {
    pub stdout: Arc<Mutex<Vec<u8>>>,
    pub stderr: Arc<Mutex<Vec<u8>>>,
}

fn server_log_registry() -> &'static Mutex<HashMap<u16, ServerLogBuffers>> {
    SERVER_LOG_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn register_server_logs(port: u16, buffers: ServerLogBuffers) {
    server_log_registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .insert(port, buffers);
}

fn unregister_server_logs(port: u16) {
    server_log_registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .remove(&port);
}

/// Snapshot of every registered server's log buffers.
pub fn registered_server_logs() -> Vec<(u16, ServerLogBuffers)> {
    server_log_registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .iter()
        .map(|(port, buffers)| (*port, buffers.clone()))
        .collect()
}

/// Full decoded text of a log buffer. Recovers from a poisoned lock so one
/// panicking test cannot break a later test's failure dump.
pub fn buffer_text(buffer: &Arc<Mutex<Vec<u8>>>) -> String {
    let bytes = match buffer.lock() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    };
    String::from_utf8_lossy(&bytes).to_string()
}

/// Last `max_lines` lines of `text`, for failure-output tails.
pub fn tail_lines(text: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.len().saturating_sub(max_lines);
    lines[start..].join("\n")
}

/// Tee target under `tmp/test_server_logs/` (gitignored scratch; files are
/// never removed — delete the directory to prune); `None` disables the tee
/// when the directory cannot be created.
fn tee_log_path(port: u16, stream: &str) -> Option<std::path::PathBuf> {
    let dir = std::path::PathBuf::from("tmp/test_server_logs");
    fs::create_dir_all(&dir).ok()?;
    Some(dir.join(format!("{port}_{stream}.log")))
}

/// Incremental drain: appends every chunk to the in-memory buffer (read by
/// failure dumps and the startup-failure dump) and — best effort — to the
/// tee file, so hung or timed-out tests still leave artifacts. A file write
/// failure disables the tee only; the memory buffer keeps filling.
fn drain_to_buffer_and_tee(
    stream: &mut impl Read,
    buffer: Arc<Mutex<Vec<u8>>>,
    tee_path: Option<std::path::PathBuf>,
) {
    let mut tee = tee_path.and_then(|path| fs::File::create(&path).ok());
    let mut chunk = [0u8; 4096];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                if let Ok(mut guard) = buffer.lock() {
                    guard.extend_from_slice(&chunk[..n]);
                }
                if let Some(file) = tee.as_mut() {
                    if file.write_all(&chunk[..n]).is_err() {
                        tee = None;
                    }
                }
            }
        }
    }
}

fn register_port_pid(port: u16, pid: u32) {
    if let Ok(mut g) = port_pids().lock() {
        g.insert(port, pid);
    }
}

fn take_port_pid(port: u16) -> Option<u32> {
    port_pids().lock().ok().and_then(|mut g| g.remove(&port))
}

/// Shared HTTP probe client. `reqwest::Client` is internally Arc; one instance
/// per process avoids rebuilding connection pools on every readiness attempt.
static HTTP_PROBE: OnceLock<reqwest::Client> = OnceLock::new();

fn http_probe_client() -> &'static reqwest::Client {
    HTTP_PROBE.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(200))
            .build()
            .expect("reqwest client build")
    })
}

async fn probe_http(port: u16) -> bool {
    let client = http_probe_client();
    match client.get(format!("http://127.0.0.1:{port}/")).send().await {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}

fn terminate_pid(pid: u32) {
    #[cfg(not(target_os = "windows"))]
    {
        // SAFETY: SIGTERM is the standard graceful-termination signal. `pid`
        // was inserted by `register_port_pid` from a `Child::id()` for a
        // server we ourselves spawned.
        unsafe {
            libc::kill(pid as i32, libc::SIGTERM);
        }
    }
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("taskkill")
            .args(["/F", "/PID", &pid.to_string()])
            .output();
    }
}

pub fn port_in_use(port: u16) -> bool {
    std::net::TcpStream::connect(format!("127.0.0.1:{port}")).is_ok()
}

pub fn kill_existing_server(port: u16) {
    // Only kill if we manage the server (to avoid killing other test instances)
    if !SERVER_MANAGED.load(Ordering::SeqCst) {
        return;
    }
    if let Some(pid) = take_port_pid(port) {
        terminate_pid(pid);
    }
    // Poll for port release instead of fixed 2s sleep
    for _ in 0..40 {
        if !port_in_use(port) {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

/// Start the server with optional mock LLM backend.
/// When use_mock is true, writes a temporary settings file with Mock
/// connections and passes it via `--settings-path` CLI flag.
/// Returns the spawned child process, temp dir (if mock settings were written),
/// the path to the SQLite database file the server will create, and the
/// stdout/stderr drain buffers (so the caller can dump them on startup timeout).
type ChildOutputBuffer = Arc<Mutex<Vec<u8>>>;
type StartServerResult = (
    Child,
    Option<std::path::PathBuf>,
    std::path::PathBuf,
    ChildOutputBuffer,
    ChildOutputBuffer,
);

pub fn start_server_with_env(
    port: u16,
    world: &str,
    persona: &str,
    use_mock: bool,
) -> StartServerResult {
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
        c.env("RUST_LOG", "chronicler_engine=debug,tower_http=debug");
        c.env("CHRONICLER_LOG_CONSOLE", "1");
        c.args([
            "--world",
            world,
            "--persona",
            persona,
            "--port",
            &port.to_string(),
        ]);
        c
    } else {
        let mut c = Command::new("cargo");
        c.env("RUST_LOG", "chronicler_engine=debug,tower_http=debug");
        c.env("CHRONICLER_LOG_CONSOLE", "1");
        c.args([
            "run",
            "--",
            "--world",
            world,
            "--persona",
            persona,
            "--port",
            &port.to_string(),
        ]);
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
        cmd.arg("--settings-path")
            .arg(settings_path.to_str().unwrap());
        Some(tmp)
    } else {
        None
    };

    cmd.current_dir(".")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut child = cmd.spawn().expect("Failed to start server");
    register_port_pid(port, child.id());

    let stdout_buf = Arc::new(Mutex::new(Vec::<u8>::new()));
    let stderr_buf = Arc::new(Mutex::new(Vec::<u8>::new()));

    if let Some(mut out) = child.stdout.take() {
        let buf = Arc::clone(&stdout_buf);
        let tee = tee_log_path(port, "stdout");
        std::thread::spawn(move || {
            drain_to_buffer_and_tee(&mut out, buf, tee);
        });
    }
    if let Some(mut err) = child.stderr.take() {
        let buf = Arc::clone(&stderr_buf);
        let tee = tee_log_path(port, "stderr");
        std::thread::spawn(move || {
            drain_to_buffer_and_tee(&mut err, buf, tee);
        });
    }

    let db_path = std::path::PathBuf::from(&target_dir)
        .join("debug")
        .join(format!("chronicler_{port}.db"));

    (child, tmp_dir, db_path, stdout_buf, stderr_buf)
}

pub async fn wait_for_server(port: u16, max_attempts: usize) -> bool {
    for _ in 0..max_attempts {
        if probe_http(port).await {
            return true;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    false
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
        // SAFETY: signal 0 is POSIX no-op — only checks process existence / permission.
        // Does not deliver a signal or modify process state. `pid` is bounded to u32
        // range by the caller; cast to i32 is sound for valid PIDs.
        unsafe { libc::kill(pid as i32, 0) == 0 }
    }
}

pub fn get_config_port(config_path: &str) -> Result<u16, String> {
    let config = TestConfig::from_file(config_path)?;
    get_available_port(config.port_range.min, config.port_range.max)
}

pub struct TestServer {
    child: Child,
    port: u16,
    temp_dir: Option<std::path::PathBuf>,
    db_path: std::path::PathBuf,
    log_buffers: ServerLogBuffers,
}

impl TestServer {
    pub async fn with_config(port: u16, world: &str, persona: &str, use_mock: bool) -> Self {
        Self::start(port, world, persona, use_mock).await
    }

    /// Remove any stale SQLite database for this port before starting.
    fn cleanup_stale_db(port: u16, db_path: &std::path::Path) {
        if db_path.exists() {
            let size = std::fs::metadata(db_path).map(|m| m.len()).unwrap_or(0);
            eprintln!(
                "🧹 Cleaning stale DB for port {port} ({size} bytes): {}",
                db_path.display()
            );
            let _ = std::fs::remove_file(db_path);
        }
    }

    pub async fn from_config(
        world: &str,
        persona: &str,
        config_path: &str,
        test_name: &str,
    ) -> Result<(Self, u16), String> {
        let config = TestConfig::from_file(config_path)?;
        let port = get_available_port(config.port_range.min, config.port_range.max)?;
        let use_mock = config.get_backend(test_name) == "mock";
        let server = Self::start(port, world, persona, use_mock).await;
        Ok((server, port))
    }

    async fn start(port: u16, world: &str, persona: &str, use_mock: bool) -> Self {
        if port_in_use(port) {
            kill_existing_server(port);
        }
        let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string());
        let db_path = std::path::PathBuf::from(&target_dir)
            .join("debug")
            .join(format!("chronicler_{port}.db"));
        // Remove stale database BEFORE starting server so we don't inherit old state.
        Self::cleanup_stale_db(port, &db_path);
        let (mut child, temp_dir, _db_path, stdout_buf, stderr_buf) =
            start_server_with_env(port, world, persona, use_mock);
        let log_buffers = ServerLogBuffers {
            stdout: Arc::clone(&stdout_buf),
            stderr: Arc::clone(&stderr_buf),
        };
        register_server_logs(port, log_buffers.clone());
        let started = wait_for_server(port, 300).await; // 300 * 100ms = 30s total — CI under load can take >10s
        if !started {
            eprintln!(
                "🛑 Server failed to start on port {port} within 30s. Draining child output for diagnostics:"
            );
            for (label, buffer) in [("stdout", &stdout_buf), ("stderr", &stderr_buf)] {
                let text = buffer_text(buffer);
                if text.is_empty() {
                    eprintln!(
                        "--- child {label} empty (binary may not have written anything yet) ---"
                    );
                } else {
                    eprintln!("--- child {label} ({} bytes) ---", text.len());
                    eprintln!("{text}");
                }
            }
            let _ = child.kill();
            let _ = child.wait();
            unregister_server_logs(port);
            panic!("Server failed to start on port {port}");
        }
        SERVER_MANAGED.store(true, Ordering::SeqCst);
        TestServer {
            child,
            port,
            temp_dir,
            db_path,
            log_buffers,
        }
    }

    pub async fn new(port: u16, world: &str, persona: &str) -> Self {
        Self::start(port, world, persona, false).await
    }

    pub async fn new_with_mock(port: u16, world: &str, persona: &str) -> Self {
        Self::start(port, world, persona, true).await
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        unregister_server_logs(self.port);
        let _ = self.child.kill();
        let _ = self.child.wait();
        // Clear PID registry so kill_existing_server on a future test does not
        // try to SIGTERM a process we just terminated.
        let _ = take_port_pid(self.port);
        SERVER_MANAGED.store(false, Ordering::SeqCst);
        release_port_lock(self.port);
        if let Some(tmp) = &self.temp_dir {
            let _ = std::fs::remove_dir_all(tmp);
        }
        // Delete the SQLite database so the next test on this port starts clean.
        if self.db_path.exists() {
            let _ = std::fs::remove_file(&self.db_path);
        }
    }
}
