//! Shared stub-browser helpers: the check runner, page probes, node reads and the on-demand region load used by more than one stub test module.

use std::panic::AssertUnwindSafe;
use std::time::{Duration, Instant};

use futures_util::future::FutureExt;

use super::*;

const POLL_NOW_TIMEOUT: Duration = Duration::from_secs(5);

struct CheckReport {
    name: String,
    duration: Duration,
    passed: bool,
}

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

    /// The browser is closed before the panic so no driver process outlives the
    /// test.
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

/// The summary and the failed-check list cannot drift from the `check_*` function
/// they name.
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

/// Fire a region's own htmx request now, whether or not a number of its kinds
/// moved, and return once htmx has handled the response.
///
/// The page asks first, so the loop has already loaded what the stub's numbers
/// say is due and holds no later load that would replace what the test does
/// next. `htmx.ajax` with the element as its `source` then issues that
/// element's own request: the element's `hx-get` path, its `hx-target`, its
/// `hx-swap` (`morph:innerHTML` included), its `hx-sync` and its `hx-on::`
/// handlers all take the same path a load of the loop takes.
///
/// htmx queues a request behind one already in flight on the same element and
/// fires `htmx:afterRequest` for the earlier one first. So the helper waits
/// until the element is idle, fires, and then waits for `htmx:afterRequest`
/// carrying its own `xhr`. htmx fires that event after the swap is applied and
/// also for a refused load that swaps nothing.
///
/// The request goes out whatever the region holds, and the hold still applies
/// to its swap, so the helper also runs where an edit holds the region.
pub(super) async fn poll_now(page: &playwright_rs::Page, selector: &str) {
    page.evaluate::<(), ()>("() => window.chroniclerLive.askOnce()", None)
        .await
        .expect("the page asks before the forced load");
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

/// Stops the page's own timer, so only the asks a check makes load a region.
pub(super) async fn stop_page_timer(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>("() => window.chroniclerLive.stopTimer()", None)
        .await
        .expect("the page stops its timer");
}

/// One ask of the page, which returns when every load that the ask started has
/// settled.
pub(super) async fn ask_page_once(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>("() => window.chroniclerLive.askOnce()", None)
        .await
        .expect("the page runs one ask");
}

/// Focus lands only at the end of a swap.
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

/// Focus lands inside `selector` or on the element itself: a load that removes
/// the focused node and every row near it leaves focus on the region.
pub(super) async fn wait_until_focus_within(
    page: &playwright_rs::Page,
    selector: &str,
    timeout: Duration,
) -> bool {
    wait_for_condition_async(timeout, Duration::from_millis(25), || async {
        page.evaluate::<String, bool>(
            r#"(selector) => {
                const el = document.querySelector(selector);
                return !!el && el.contains(document.activeElement);
            }"#,
            Some(&selector.to_string()),
        )
        .await
        .unwrap()
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

/// A delete's `swap:0.3s` delay removes its row after the click resolves, so the
/// count the swap changes — not the click — is what says the swap has landed.
/// A query that errors panics rather than counting as zero, so a wait for "none
/// left" cannot pass on a failed query.
pub(super) async fn wait_for_count(
    page: &playwright_rs::Page,
    selector: &str,
    expected: usize,
) -> bool {
    wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(25),
        || async move {
            page.query_selector_all(selector)
                .await
                .expect("count the elements the selector names")
                .len()
                == expected
        },
    )
    .await
}

pub(super) async fn accept_delete_confirmation(page: &playwright_rs::Page) {
    page.on_dialog(|dialog| async move { dialog.accept(None).await })
        .await
        .unwrap();
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
