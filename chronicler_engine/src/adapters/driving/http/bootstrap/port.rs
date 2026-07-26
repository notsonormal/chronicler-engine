//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
//! Port management utilities

use tokio::net::TcpListener;
use tracing;

use crate::adapters::driving::http::utils::port_utils::{find_process_on_port, kill_process};

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
