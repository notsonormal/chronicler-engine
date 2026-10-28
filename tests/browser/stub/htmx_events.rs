//! Probes of the vendored htmx (declared exemption in the spec-coverage validator): the order and the reach of the events that the one answer reader relies on. No spec link: the probes pin the contract of the library, and the scenarios of `docs/specs/browser_dashboard.md` rely on it.

use std::time::Duration;

use super::support::{eval_string, StubRunner};
use super::*;

/// Records each event of a request on `window.__events`: the name, the status of
/// the request, and whether the element that the event names is in the page. A
/// listener on the body records an event only when it reaches the body.
const RECORD_EVENTS: &str = r#"() => {
    window.__events = [];
    for (const name of ['htmx:afterRequest', 'htmx:sendError', 'htmx:sendAbort', 'htmx:timeout', 'htmx:onLoadError']) {
        document.body.addEventListener(name, (evt) => {
            window.__events.push({
                name: evt.type,
                status: evt.detail.xhr ? evt.detail.xhr.status : null,
                attached: evt.detail.elt.isConnected,
            });
        });
    }
}"#;

async fn record_events(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>(RECORD_EVENTS, None).await.unwrap();
}

async fn events_of_probe(page: &playwright_rs::Page) -> Vec<serde_json::Value> {
    serde_json::from_str(&eval_string(page, "() => JSON.stringify(window.__events)").await).unwrap()
}

async fn forget_events(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>("() => { window.__events = []; }", None)
        .await
        .unwrap();
}

async fn add_probe(page: &playwright_rs::Page, attributes: &str) {
    let script = format!(
        r#"() => {{
            document.querySelectorAll('#probe').forEach((old) => old.remove());
            const probe = document.createElement('div');
            probe.id = 'probe';
            probe.innerHTML = '<button type="button" id="probe-button" {attributes}>probe</button>';
            document.body.appendChild(probe);
            htmx.process(probe);
        }}"#
    );
    page.evaluate::<(), ()>(&script, None).await.unwrap();
}

async fn press_probe(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>(
        "() => { htmx.trigger(document.getElementById('probe-button'), 'click'); }",
        None,
    )
    .await
    .unwrap();
}

async fn wait_for_events(page: &playwright_rs::Page, count: usize) -> Vec<serde_json::Value> {
    let arrived = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(25),
        || async { events_of_probe(page).await.len() >= count },
    )
    .await;
    let seen = events_of_probe(page).await;
    assert!(arrived, "expected {count} events, saw {seen:?}");
    seen
}

fn names(seen: &[serde_json::Value]) -> Vec<String> {
    seen.iter()
        .map(|event| event["name"].as_str().unwrap().to_string())
        .collect()
}

// A swap that throws fires `htmx:onLoadError`, the kebab-case
// twin of that event reaches `hx-on::on-load-error`, and no `htmx:afterRequest` follows.
async fn check_a_swap_that_throws_runs_the_load_error_attribute(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    wait_for_screen_current(&page).await;
    record_events(&page).await;
    page.evaluate::<(), ()>(
        r#"() => {
            window.__attribute_runs = 0;
            document.body.addEventListener('htmx:beforeSwap', (evt) => {
                if (evt.detail.requestConfig.path === '/probe/a') {
                    evt.detail.shouldSwap = true;
                    evt.detail.target = null;
                }
            });
        }"#,
        None,
    )
    .await
    .unwrap();
    add_probe(
        &page,
        r##"hx-post="/probe/a" hx-swap="innerHTML" hx-target="#probe" hx-on::on-load-error="window.__attribute_runs += 1""##,
    )
    .await;
    press_probe(&page).await;

    let seen = wait_for_events(&page, 1).await;
    assert_eq!(
        names(&seen),
        ["htmx:onLoadError"],
        "a swap that throws ends in onLoadError only"
    );
    let runs = eval_string(&page, "() => String(window.__attribute_runs)").await;
    assert_eq!(
        runs, "1",
        "hx-on::on-load-error runs for the swap that threw"
    );
    page.evaluate::<(), ()>("() => { document.getElementById('probe').remove(); }", None)
        .await
        .unwrap();
}

// `htmx:afterRequest` fires with status 0 for an abort, a
// timeout and a network error, and `htmx:sendError` fires for the network error only.
async fn check_after_request_fires_with_status_zero_for_every_request_that_got_no_answer(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    wait_for_screen_current(&page).await;
    stub.changes_handle().set_story_log_delay_ms(3000);
    record_events(&page).await;

    add_probe(&page, r#"hx-get="/fragment/story-log" hx-swap="none""#).await;
    press_probe(&page).await;
    page.evaluate::<(), ()>(
        "() => { htmx.trigger(document.getElementById('probe-button'), 'htmx:abort'); }",
        None,
    )
    .await
    .unwrap();
    let seen = wait_for_events(&page, 2).await;
    assert_eq!(
        names(&seen),
        ["htmx:afterRequest", "htmx:sendAbort"],
        "an abort"
    );
    assert_eq!(seen[0]["status"], 0, "an abort has status 0");

    forget_events(&page).await;
    add_probe(&page, r#"hx-get="/fragment/story-log" hx-swap="none""#).await;
    page.evaluate::<(), ()>(
        "() => { document.body.addEventListener('htmx:beforeSend', (evt) => { evt.detail.xhr.timeout = 100; }, { once: true }); }",
        None,
    )
    .await
    .unwrap();
    press_probe(&page).await;
    let seen = wait_for_events(&page, 2).await;
    assert_eq!(
        names(&seen),
        ["htmx:afterRequest", "htmx:timeout"],
        "a timeout"
    );
    assert_eq!(seen[0]["status"], 0, "a timeout has status 0");

    forget_events(&page).await;
    add_probe(&page, r#"hx-post="/probe/b" hx-swap="none""#).await;
    let lifecycle = stub.lifecycle_handle();
    lifecycle.stop();
    wait_until_port_closed(lifecycle.addr()).await;
    press_probe(&page).await;
    let seen = wait_for_events(&page, 2).await;
    assert_eq!(
        names(&seen),
        ["htmx:afterRequest", "htmx:sendError"],
        "a network error"
    );
    assert_eq!(seen[0]["status"], 0, "a network error has status 0");
}

// The `htmx:afterRequest` of a network error reaches the body
// while the sender is in the page. A site handler that detaches the sender then hides
// `htmx:sendError` from the body, although the element itself still hears it.
async fn check_a_send_error_on_a_detached_sender_misses_the_body(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    wait_for_screen_current(&page).await;
    record_events(&page).await;
    add_probe(
        &page,
        r#"hx-post="/probe/c" hx-swap="none" hx-on::after-request="this.remove()""#,
    )
    .await;
    page.evaluate::<(), ()>(
        r#"() => {
            window.__element_events = [];
            const button = document.getElementById('probe-button');
            button.addEventListener('htmx:sendError', () => window.__element_events.push('sendError'));
        }"#,
        None,
    )
    .await
    .unwrap();
    let lifecycle = stub.lifecycle_handle();
    lifecycle.stop();
    wait_until_port_closed(lifecycle.addr()).await;
    press_probe(&page).await;

    let seen = wait_for_events(&page, 1).await;
    assert_eq!(
        names(&seen),
        ["htmx:afterRequest"],
        "the body hears the afterRequest first"
    );
    assert_eq!(seen[0]["status"], 0);
    assert_eq!(
        seen[0]["attached"], true,
        "the sender is in the page when the body hears it"
    );
    // A later event has no signal to wait on: the check proves that nothing arrives.
    tokio::time::sleep(Duration::from_millis(300)).await;
    let later = events_of_probe(&page).await;
    assert_eq!(
        names(&later),
        ["htmx:afterRequest"],
        "the sendError of the detached sender does not reach the body, saw {later:?}"
    );
    let on_element = eval_string(&page, "() => window.__element_events.join(',')").await;
    assert_eq!(
        on_element, "sendError",
        "the detached element still hears its own sendError"
    );
}

#[tokio::test]
async fn run_htmx_events() {
    let mut runner = StubRunner::launch().await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_swap_that_throws_runs_the_load_error_attribute,
        )
        .await;
    runner
        .run_without_convergence(
            "the check stops the stub to get a network error",
            StubActionOutcome::Pending,
            check_after_request_fires_with_status_zero_for_every_request_that_got_no_answer,
        )
        .await;
    runner
        .run_without_convergence(
            "the check stops the stub to get a network error",
            StubActionOutcome::Pending,
            check_a_send_error_on_a_detached_sender_misses_the_body,
        )
        .await;
    runner.finish().await;
}
