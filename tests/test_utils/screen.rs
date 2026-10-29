//! Waits on the stamp of the page: the page shows the screen of the server when `<html data-revision>` equals the `server:revision` of `GET /changes` and no ask or load is in flight.

use std::time::Duration;

use playwright_rs::Page;

use super::browser::capture_failure_state;
use super::wait::wait_for_condition_async;

const SCREEN_CURRENT_TIMEOUT: Duration = Duration::from_secs(10);

/// A poll interval that no check reaches: the page keeps its timer, and it asks
/// only when a check or an action makes it ask.
const PAUSED_POLL_MS: &str = "3600000";

const SCREEN_IS_CURRENT: &str = r#"async () => {
    const root = document.documentElement;
    if (root.hasAttribute('data-screen-busy')) return false;
    const response = await fetch('/changes', { cache: 'no-store' });
    if (!response.ok) return false;
    const answer = await response.json();
    return root.dataset.revision === answer.server + ':' + answer.revision;
}"#;

/// Installed in every stub page, so a script that a check runs in the page can wait
/// for the screen without going through the test: it returns when the page shows the
/// screen of the server.
pub const WAIT_FOR_SCREEN_INIT_SCRIPT: &str = r#"(() => {
  window.__waitForScreen = async () => {
    const root = document.documentElement;
    for (;;) {
      if (!root.hasAttribute('data-screen-busy')) {
        const response = await fetch('/changes', { cache: 'no-store' });
        const answer = await response.json();
        if (root.dataset.revision === answer.server + ':' + answer.revision) return;
      }
      await new Promise((resolve) => setTimeout(resolve, 20));
    }
  };
})();"#;

/// Waits until the page shows the screen of the server. A page whose engine is
/// unreachable never does, so a check that stops the engine does not call this.
pub async fn wait_for_screen_current(page: &Page) {
    let current = wait_for_condition_async(
        SCREEN_CURRENT_TIMEOUT,
        Duration::from_millis(25),
        || async {
            page.evaluate::<(), bool>(SCREEN_IS_CURRENT, None)
                .await
                .unwrap_or(false)
        },
    )
    .await;
    if !current {
        capture_failure_state(page, "wait_for_screen_current").await;
        panic!("the page did not show the screen of the server within {SCREEN_CURRENT_TIMEOUT:?}");
    }
}

/// Makes the page ask now, as it does when the tab becomes visible, and waits until
/// the screen is current.
pub async fn ask_once(page: &Page) {
    page.evaluate::<(), ()>(
        "() => { document.dispatchEvent(new Event('visibilitychange')); }",
        None,
    )
    .await
    .expect("the page takes the visibility event");
    wait_for_screen_current(page).await;
}

pub async fn load_screen_now(page: &Page) {
    page.evaluate::<(), ()>("() => fetch('/stub/touch').then(() => {})", None)
        .await
        .expect("the stub moves its revision");
    ask_once(page).await;
}

/// The page asks only when a check makes it ask, and waits until the asks and
/// loads that are in flight have ended. The tick that the old interval scheduled still
/// runs once; a page timer of the old interval fires after it, because timers fire in
/// the order of their due time.
pub async fn stop_page_timer(page: &Page) {
    page.evaluate::<String, ()>(
        "async (ms) => { const root = document.documentElement; const old = Number(root.dataset.pollMs) || 1000; root.dataset.pollMs = ms; if (old < Number(ms)) await new Promise((resolve) => setTimeout(resolve, old)); }",
        Some(&PAUSED_POLL_MS.to_string()),
    )
    .await
    .expect("the page takes the poll interval");
    wait_for_screen_current(page).await;
}
