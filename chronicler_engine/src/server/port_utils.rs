use tokio::net::TcpListener;
use tracing;

/// Attempts to bind to the given address, retrying if the port is in use.
/// If a process is found on the port, it attempts to kill it.
pub async fn bind_with_retry(addr: &str) -> std::io::Result<TcpListener> {
    // [DOC: docs/architecture/system.md]
    loop {
        match TcpListener::bind(addr).await {
            Ok(listener) => return Ok(listener),
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                tracing::error!("Port in use, attempting to free it...");
                if let Some(pid) = find_process_on_port(addr) {
                    tracing::error!("Found process on port, attempting to kill PID {pid}...");
                    let _ = kill_process(pid);
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    continue;
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
