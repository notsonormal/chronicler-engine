//! [DOC: docs/system/dashboard.md]
//! Port management utilities

use tokio::net::TcpListener;
use tracing;

/// Binds to `addr`, killing any process found on the port between attempts.
/// `max_attempts: None` retries forever; `Some(n)` fails after `n` attempts.
pub async fn bind_with_retry(
    addr: &str,
    max_attempts: Option<u32>,
) -> std::io::Result<TcpListener> {
    let mut attempts: u32 = 0;
    loop {
        attempts += 1;
        match TcpListener::bind(addr).await {
            Ok(listener) => return Ok(listener),
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                if max_attempts.is_some_and(|limit| attempts >= limit) {
                    return Err(e);
                }
                tracing::error!("Port in use, attempting to free it...");
                if let Some(pid) = find_process_on_port(addr) {
                    tracing::error!("Found process on port, attempting to kill PID {pid}...");
                    let _ = kill_process(pid);
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
            Err(e) => {
                tracing::error!("Bind error: {e:?}");
                return Err(e);
            }
        }
    }
}

/// Finds the process ID listening on the given port (Windows only).
fn find_process_on_port(addr: &str) -> Option<u32> {
    let port = addr.split(':').next_back()?.parse::<u16>().ok()?;
    let output = std::process::Command::new("netstat")
        .args(["-ano"])
        .output()
        .ok()?;
    let output_str = String::from_utf8_lossy(&output.stdout);
    for line in output_str.lines() {
        if line.contains(&format!(":{port}")) && line.contains("LISTENING") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(pid_str) = parts.last() {
                return pid_str.parse().ok();
            }
        }
    }
    None
}

/// Kills a process by PID (Windows only).
fn kill_process(pid: u32) -> std::io::Result<std::process::Output> {
    std::process::Command::new("taskkill")
        .args(["/F", "/PID", &pid.to_string()])
        .output()
}
