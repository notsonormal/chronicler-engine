//! Shared stub-browser helpers: the check runner, page probes and node reads used by more than one stub test module.

use std::panic::AssertUnwindSafe;
use std::time::{Duration, Instant};

use futures_util::future::FutureExt;

use super::*;

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

    /// After the check, the page must converge: each visible part equals a fresh
    /// `GET /screen` of that part.
    pub(super) async fn run<Fut>(
        &mut self,
        outcome: StubActionOutcome,
        check: impl FnOnce(playwright_rs::Page, StubServer) -> Fut,
    ) where
        Fut: std::future::Future<Output = ()>,
    {
        self.run_checked(true, outcome, check).await;
    }

    /// For a check that leaves the page in a state that no fresh render can match
    /// on purpose, such as an engine that is unreachable. `reason` names that state.
    pub(super) async fn run_without_convergence<Fut>(
        &mut self,
        _reason: &'static str,
        outcome: StubActionOutcome,
        check: impl FnOnce(playwright_rs::Page, StubServer) -> Fut,
    ) where
        Fut: std::future::Future<Output = ()>,
    {
        self.run_checked(false, outcome, check).await;
    }

    async fn run_checked<Fut>(
        &mut self,
        converges: bool,
        outcome: StubActionOutcome,
        check: impl FnOnce(playwright_rs::Page, StubServer) -> Fut,
    ) where
        Fut: std::future::Future<Output = ()>,
    {
        let name = check_name(&check);
        let start = Instant::now();
        let stub = StubServer::start(outcome).await;
        let page = self.browser.open_page(&stub).await;

        let result = AssertUnwindSafe(async {
            check(page.clone(), stub.lent()).await;
            if converges {
                assert_screen_converged(&page).await;
            }
        })
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

pub(super) async fn eval_string(page: &playwright_rs::Page, script: &str) -> String {
    page.evaluate::<(), String>(script, None)
        .await
        .unwrap_or_else(|e| panic!("evaluate failed: {e}\n{script}"))
}

pub(super) async fn open_section(page: &playwright_rs::Page, tab: &str, part: &str) {
    page.locator(&format!(r#"[data-tab="{tab}"]"#))
        .await
        .click(None)
        .await
        .unwrap_or_else(|e| panic!("click tab '{tab}' failed: {e}"));
    wait_for_screen_current(page).await;
    await_panel_ready(page, part).await.expect_settled(tab);
}

pub(super) async fn open_settings(page: &playwright_rs::Page) {
    page.locator(r#"[data-tab="settings"]"#)
        .await
        .click(None)
        .await
        .unwrap();
    wait_until_visible(page, "#subtab-connections", Duration::from_secs(5)).await;
}

pub(super) async fn dialog_is_open(page: &playwright_rs::Page) -> bool {
    page.evaluate::<(), bool>(
        "() => { const d = document.getElementById('edit-dialog'); return !!d && d.open && d.matches(':modal'); }",
        None,
    )
    .await
    .unwrap_or(false)
}

pub(super) async fn dialog_body_is_empty(page: &playwright_rs::Page) -> bool {
    page.evaluate::<(), bool>(
        "() => { const b = document.getElementById('edit-dialog-body'); return !!b && b.childElementCount === 0 && b.textContent.trim() === ''; }",
        None,
    )
    .await
    .unwrap_or(false)
}

/// The `close` event comes after the dialog stops being open, and it is the event
/// that empties the body, so closed means both.
pub(super) async fn wait_until_dialog_closed(page: &playwright_rs::Page, why: &str) {
    let closed = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(25),
        || async { !dialog_is_open(page).await && dialog_body_is_empty(page).await },
    )
    .await;
    assert!(closed, "{why}");
}

pub(super) async fn ask_once_without_waiting(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>(
        "() => { document.dispatchEvent(new Event('visibilitychange')); }",
        None,
    )
    .await
    .unwrap();
}

/// The page asks only when a check makes it ask, and its first load has ended.
pub(super) async fn start_current(page: &playwright_rs::Page) {
    stop_page_timer(page).await;
}

/// Keeps each `GET /screen` of the page until `window.__releaseScreen()`, so a
/// load stays in flight while a check works. `window.__screenHeld.length` counts
/// the loads that wait.
pub(super) async fn hold_screen_loads(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>(
        r#"() => {
            const realFetch = window.fetch;
            window.__holdScreen = true;
            window.__screenHeld = [];
            window.fetch = function (url, init) {
                if (window.__holdScreen && String(url).startsWith('/screen')) {
                    return new Promise((resolve) => {
                        window.__screenHeld.push(() => resolve(realFetch.call(window, url, init)));
                    });
                }
                return realFetch.apply(this, arguments);
            };
            window.__releaseScreen = () => {
                window.__holdScreen = false;
                window.__screenHeld.splice(0).forEach((resume) => resume());
            };
        }"#,
        None,
    )
    .await
    .unwrap();
}

/// Records the `VERB url` of each request that the page sends from now on through
/// `XMLHttpRequest` to a URL that starts with `prefix`.
pub(super) async fn record_requests_to(page: &playwright_rs::Page, prefix: &str) {
    page.evaluate::<String, ()>(
        r#"(prefix) => {
            window.__recordedRequests = [];
            const open = XMLHttpRequest.prototype.open;
            XMLHttpRequest.prototype.open = function (method, url) {
                if (String(url).startsWith(prefix)) {
                    window.__recordedRequests.push(method + ' ' + url);
                }
                return open.apply(this, arguments);
            };
        }"#,
        Some(&prefix.to_string()),
    )
    .await
    .unwrap();
}

pub(super) async fn recorded_requests(page: &playwright_rs::Page) -> Vec<String> {
    page.evaluate::<(), Vec<String>>("() => window.__recordedRequests", None)
        .await
        .unwrap()
}

pub(super) async fn wait_for_held_screen_load(page: &playwright_rs::Page) {
    let held = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(10),
        || async {
            page.evaluate::<(), bool>("() => window.__screenHeld.length > 0", None)
                .await
                .unwrap_or(false)
        },
    )
    .await;
    assert!(held, "a screen load must be held in the page");
}

pub(super) async fn release_screen_loads(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>("() => window.__releaseScreen()", None)
        .await
        .unwrap();
}

/// Keeps each request whose URL ends with `suffix` in the page until
/// `window.releaseHeldRequests()`, so it stays in flight while a check works.
/// `window.heldRequestsSeen` counts the requests that the page tried to send and the
/// hold kept; `window.heldRequestsAnswered` counts the held requests whose answer
/// came back.
pub(super) async fn hold_requests_ending_with(page: &playwright_rs::Page, suffix: &str) {
    page.evaluate::<String, ()>(
        r#"(suffix) => {
            const open = XMLHttpRequest.prototype.open;
            const send = XMLHttpRequest.prototype.send;
            const held = [];
            window.heldRequestsAnswered = 0;
            window.heldRequestsSeen = 0;
            XMLHttpRequest.prototype.open = function (method, url) {
                this.heldUrl = String(url);
                return open.apply(this, arguments);
            };
            XMLHttpRequest.prototype.send = function (body) {
                if (this.heldUrl.endsWith(suffix)) {
                    window.heldRequestsSeen += 1;
                    this.addEventListener('loadend', () => { window.heldRequestsAnswered += 1; });
                    held.push(() => send.call(this, body));
                    return;
                }
                return send.call(this, body);
            };
            window.releaseHeldRequests = () => held.splice(0).forEach((resume) => resume());
        }"#,
        Some(&suffix.to_string()),
    )
    .await
    .unwrap();
}

pub(super) async fn release_held_requests_and_wait_for_answers(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>(
        "() => { window.expectedHeldAnswers = window.heldRequestsAnswered + 1; window.releaseHeldRequests(); }",
        None,
    )
    .await
    .unwrap();
    let answered = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(25),
        || async {
            page.evaluate::<(), bool>(
                "() => window.heldRequestsAnswered >= window.expectedHeldAnswers",
                None,
            )
            .await
            .unwrap_or(false)
        },
    )
    .await;
    assert!(
        answered,
        "the held request must be answered after its release"
    );
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
/// the focused node and every row near it leaves focus on the part.
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

/// A delete removes its row when the next load lands, after the click resolves, so
/// the count the load changes — not the click — says the load has landed.
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

pub(super) async fn story_load_count(page: &playwright_rs::Page) -> f64 {
    page.evaluate::<(), f64>("(() => window.__storyLoads || 0)()", None)
        .await
        .unwrap_or(0.0)
}

pub(super) async fn install_story_load_counter(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>(
        r#"(() => {
            window.__storyLoads = 0;
            document.addEventListener('screen:loaded', (evt) => {
                if (evt.detail.parts.includes('story-log')) window.__storyLoads += 1;
            });
        })()"#,
        None,
    )
    .await
    .unwrap();
}

pub(super) const NOTICE: &str = "#notice";

/// Waits until `slot` shows the refusal of a busy Game, and checks that its line
/// names the busy Game and its details carry the engine's answer. `shows` is the
/// failure message of the wait.
pub(super) async fn assert_busy_refusal(page: &playwright_rs::Page, slot: &str, shows: &str) {
    wait_for_error_disclosure(page, slot, shows).await;
    let (_, line, detail) = read_error_disclosure(page, slot).await;
    assert!(
        line.contains("busy"),
        "the line names the busy Game, got {line:?}"
    );
    assert!(
        detail.ends_with("Still thinking..."),
        "the details carry the engine's answer, got {detail:?}"
    );
    if slot == NOTICE {
        assert_eq!(
            detail, "Still thinking...",
            "the notice's Details hold only the line after the sentence"
        );
    }
}

pub(super) async fn submit_intercepted_command(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>(
        r#"(() => {
            const input = document.querySelector('#command-form input[name="command"]');
            input.value = 'look at the casle';
            document.querySelector('#submit-btn').click();
        })()"#,
        None,
    )
    .await
    .unwrap();

    wait_until_visible(page, ".text-check-preview", Duration::from_secs(5)).await;
}
