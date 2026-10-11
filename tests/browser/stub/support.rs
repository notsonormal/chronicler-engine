//! Shared stub-browser helpers: the check runner, page probes, node reads and the on-demand poll trigger used by more than one stub test module.

use std::panic::AssertUnwindSafe;
use std::time::{Duration, Instant};

use futures_util::future::FutureExt;

use super::*;

/// How long a poll the test fired itself may take to be answered.
const POLL_NOW_TIMEOUT: Duration = Duration::from_secs(5);

/// One check's outcome, for the runner's per-check timing summary.
struct CheckReport {
    name: String,
    duration: Duration,
    passed: bool,
}

/// Runs one stub module's checks in a single browser.
///
/// Each check gets a fresh `StubServer` carrying the action outcome it asked
/// for and its own page, so one check's server state cannot reach the next. The
/// browser launch is paid once per runner instead of once per check.
pub(super) struct StubRunner {
    browser: SharedBrowser,
    reports: Vec<CheckReport>,
}

impl StubRunner {
    pub(super) async fn launch() -> Self {
        Self {
            browser: SharedBrowser::launch().await,
            reports: Vec::new(),
        }
    }

    /// Run one check and keep its report. A check that panics is caught here, so
    /// the module's remaining checks still run and the summary names the one
    /// that failed.
    pub(super) async fn run<Fut>(
        &mut self,
        outcome: StubActionOutcome,
        check: impl FnOnce(playwright_rs::Page, StubServer) -> Fut,
    ) where
        Fut: std::future::Future<Output = ()>,
    {
        let name = check_name(&check);
        let start = Instant::now();
        let stub = StubServer::start(outcome).await;
        let page = self.browser.open_page(&stub).await;

        let result = AssertUnwindSafe(check(page.clone(), stub))
            .catch_unwind()
            .await;
        let _ = page.close().await;

        self.reports.push(CheckReport {
            name,
            duration: start.elapsed(),
            passed: result.is_ok(),
        });
    }

    /// Print the per-check summary, close the browser, and panic if any check
    /// failed. The browser is closed before the panic so no driver process
    /// outlives the test.
    pub(super) async fn finish(self) {
        let failed: Vec<String> = self
            .reports
            .iter()
            .filter(|report| !report.passed)
            .map(|report| report.name.clone())
            .collect();
        print_summary(&self.reports);
        self.browser.close().await;
        if !failed.is_empty() {
            panic!(
                "{} stub check(s) failed: {}",
                failed.len(),
                failed.join(", ")
            );
        }
    }
}

/// The report name of a check is the check function's own name, so the summary
/// and the failed-check list cannot drift from the `check_*` function they
/// describe.
fn check_name<Check>(_check: &Check) -> String {
    let full = std::any::type_name::<Check>();
    full.rsplit("::").next().unwrap_or(full).to_string()
}

fn print_summary(reports: &[CheckReport]) {
    eprintln!("--- Stub checks ({}) ---", reports.len());
    for report in reports {
        let status = if report.passed { "OK" } else { "FAIL" };
        eprintln!(
            "  {:>7.3}s  [{status}]  {}",
            report.duration.as_secs_f64(),
            report.name
        );
    }
    let total: f64 = reports.iter().map(|r| r.duration.as_secs_f64()).sum();
    eprintln!("  {total:>7.3}s  total (shared browser)");
}

/// Fire a polled container's own htmx request now, instead of waiting out its
/// `every Ns` timer, and return once htmx has handled the response.
///
/// htmx 1.9 runs a poll by calling the same request handler its `load` trigger
/// calls: `load` is an immediate call and `every` is a `setTimeout`, so neither
/// registers a DOM listener and no event re-fires them. `htmx.ajax` with the
/// element as its `source` issues that element's own request instead: the
/// element's `hx-get` path, its `hx-target`, its `hx-swap` (`morph:innerHTML`
/// included), its `hx-sync`, its `hx-on::` handlers and the response's
/// out-of-band swaps all take the same path the timer would have taken.
///
/// htmx queues a request behind one already in flight on the same element and
/// fires `htmx:afterRequest` for the earlier one first. So the helper waits
/// until the element is idle, fires, and then waits for `htmx:afterRequest`
/// carrying its own `xhr`. htmx fires that event after the swap is applied and
/// also for a refused poll that swaps nothing.
///
/// The request goes out whatever the container's `hx-trigger` currently says,
/// so a test that asserts a *paused* poll (the story log during an edit) must
/// not call this.
pub(super) async fn poll_now(page: &playwright_rs::Page, selector: &str) {
    let fired = page
        .evaluate::<String, String>(
            r#"async (selector) => {
                const el = document.querySelector(selector);
                if (!el) return 'no element matches the selector';
                const verb = el.hasAttribute('hx-get')
                    ? 'get'
                    : (el.hasAttribute('hx-post') ? 'post' : '');
                const path = verb ? el.getAttribute('hx-' + verb) : '';
                if (!path) return 'the container declares no hx-get/hx-post';
                const inFlight = () => (el['htmx-internal-data'] || {}).xhr;
                const deadline = Date.now() + 5000;
                while (inFlight()) {
                    if (Date.now() > deadline) return 'an earlier request never finished';
                    await new Promise((resolve) => setTimeout(resolve, 10));
                }
                window.__pollNowHandled = false;
                let ownXhr = null;
                const once = (evt) => {
                    if (ownXhr && evt.detail && evt.detail.xhr === ownXhr) {
                        window.__pollNowHandled = true;
                        document.body.removeEventListener('htmx:afterRequest', once);
                    }
                };
                document.body.addEventListener('htmx:afterRequest', once);
                htmx.ajax(verb, path, { source: el });
                ownXhr = inFlight();
                return ownXhr ? '' : 'htmx did not send the request';
            }"#,
            Some(&selector.to_string()),
        )
        .await
        .unwrap();
    assert!(fired.is_empty(), "poll_now('{selector}'): {fired}");

    let handled = wait_for_condition_async(POLL_NOW_TIMEOUT, Duration::from_millis(10), || async {
        page.evaluate::<(), bool>("() => window.__pollNowHandled === true", None)
            .await
            .unwrap_or(false)
    })
    .await;
    assert!(
        handled,
        "poll_now('{selector}'): the poll was never answered within {POLL_NOW_TIMEOUT:?}"
    );
}

/// A focus move lands at the end of a swap, so a test polls for it rather than
/// reading focus once.
pub(super) async fn wait_until_focused(
    page: &playwright_rs::Page,
    selector: &str,
    timeout: Duration,
) -> bool {
    wait_for_condition_async(timeout, Duration::from_millis(25), || async {
        active_element_is(page, selector).await
    })
    .await
}

pub(super) async fn active_element_is(page: &playwright_rs::Page, selector: &str) -> bool {
    page.evaluate::<String, bool>(
        r#"(selector) => {
            const el = document.querySelector(selector);
            return !!el && document.activeElement === el;
        }"#,
        Some(&selector.to_string()),
    )
    .await
    .unwrap()
}

pub(super) async fn read_command_input(page: &playwright_rs::Page) -> String {
    page.evaluate::<(), String>(
        r#"() => {
            const input = document.querySelector('#command-form input[name="command"]');
            return input ? input.value : '';
        }"#,
        None,
    )
    .await
    .unwrap()
}

pub(super) async fn read_banner(page: &playwright_rs::Page) -> (bool, String, String) {
    page.evaluate::<(), (bool, String, String)>(
        r#"() => {
            const banner = document.getElementById('failure-banner');
            if (!banner) return [false, '', ''];
            return [
                !banner.hidden,
                banner.getAttribute('role') || '',
                banner.textContent.trim(),
            ];
        }"#,
        None,
    )
    .await
    .unwrap()
}

pub(super) async fn story_poll_count(page: &playwright_rs::Page) -> f64 {
    page.evaluate::<(), f64>("(() => window.__storyPolls || 0)()", None)
        .await
        .unwrap_or(0.0)
}

pub(super) async fn install_story_poll_counter(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>(
        r#"(() => {
            window.__storyPolls = 0;
            document.body.addEventListener('htmx:afterRequest', (evt) => {
                const elt = evt.detail && evt.detail.elt;
                if (elt && elt.id === 'story-log') window.__storyPolls += 1;
            });
        })()"#,
        None,
    )
    .await
    .unwrap();
}
