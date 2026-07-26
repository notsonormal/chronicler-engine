//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
//! Port management helpers used by `bind_with_retry`.

/// Finds the process ID listening on the given port (Windows only).
pub(crate) fn find_process_on_port(addr: &str) -> Option<u32> {
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
pub(crate) fn kill_process(pid: u32) -> std::io::Result<std::process::Output> {
    std::process::Command::new("taskkill")
        .args(["/F", "/PID", &pid.to_string()])
        .output()
}
