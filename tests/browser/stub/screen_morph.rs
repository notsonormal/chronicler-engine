//! Stub-browser tests for the direct morph of the screen script: htmx processes the morphed nodes, and the Idiomorph callbacks keep what a load keeps. Tagged against `docs/specs/browser_screen.md`.

use std::time::Duration;

use super::support::{
    eval_string, hold_requests_ending_with, record_requests_to, recorded_requests,
    release_held_requests_and_wait_for_answers, start_current, StubRunner,
};
use super::*;

const PROBE_HOST: &str = "probe-host";

/// A host element outside every part, so the screen loop never writes into it.
async fn add_probe_host(page: &playwright_rs::Page) {
    page.evaluate::<String, ()>(
        r#"(id) => {
            const host = document.createElement('div');
            host.id = id;
            document.body.appendChild(host);
        }"#,
        Some(&PROBE_HOST.to_string()),
    )
    .await
    .unwrap();
}

/// Makes each `GET /screen` answer carry `window.__probeHeader` at the start of the
/// header part, so a check changes what a load writes without a second stub.
async fn add_probe_to_the_header_answer(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>(
        r#"() => {
            const realFetch = window.fetch;
            window.__probeHeader = '';
            window.fetch = async function (url, init) {
                const response = await realFetch.apply(this, arguments);
                if (!String(url).startsWith('/screen')) return response;
                const body = (await response.text()).replace(
                    '<template data-part="header">',
                    '<template data-part="header">' + window.__probeHeader,
                );
                return new Response(body, { status: response.status, headers: response.headers });
            };
        }"#,
        None,
    )
    .await
    .unwrap();
}

async fn set_probe_header(page: &playwright_rs::Page, html: &str) {
    page.evaluate::<String, ()>(
        "(html) => { window.__probeHeader = html; }",
        Some(&html.to_string()),
    )
    .await
    .unwrap();
    load_screen_now(page).await;
}

// A load fires no htmx swap for a part, so a new row has no `hx-post` listener until
// the screen script lets htmx process the part, and a kept node keeps the listener
// of its old `hx-post` unless htmx processes it again.
// [docs/specs/browser_screen.md] SCENARIO: 44.30
async fn check_a_load_makes_a_new_rows_post_work_and_a_changed_post_use_its_new_value(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    start_current(&page).await;
    add_probe_to_the_header_answer(&page).await;
    set_probe_header(
        &page,
        r#"<button id="kept" type="button" hx-post="/probe/old" hx-swap="none">Kept</button>"#,
    )
    .await;
    page.evaluate::<(), ()>(
        "() => { window.__keptNode = document.getElementById('kept'); }",
        None,
    )
    .await
    .unwrap();

    set_probe_header(
        &page,
        concat!(
            r#"<button id="kept" type="button" hx-post="/probe/changed" hx-swap="none">Kept</button>"#,
            r#"<button id="added" type="button" hx-post="/probe/added" hx-swap="none">Added</button>"#,
        ),
    )
    .await;
    assert_eq!(
        eval_string(
            &page,
            "() => document.getElementById('kept') === window.__keptNode ? '' : 'a new node'"
        )
        .await,
        "",
        "the load keeps the node"
    );

    record_requests_to(&page, "/probe/").await;
    page.evaluate::<(), ()>(
        r#"async () => {
            document.getElementById('added').click();
            document.getElementById('kept').click();
            await new Promise((resolve) => setTimeout(resolve, 250));
        }"#,
        None,
    )
    .await
    .unwrap();
    assert_eq!(
        recorded_requests(&page).await,
        vec![
            "POST /probe/added".to_string(),
            "POST /probe/changed".to_string()
        ],
        "after the load the new button posts, and the kept button posts to its new value"
    );

    set_probe_header(&page, "").await;
}

async fn probe_text(page: &playwright_rs::Page, id: &str) -> String {
    page.evaluate::<String, String>(
        "(id) => { const node = document.getElementById(id); return node ? node.textContent.trim() : '<missing>'; }",
        Some(&id.to_string()),
    )
    .await
    .unwrap()
}

// [docs/specs/browser_screen.md] SCENARIO: 44.31
async fn check_a_direct_morph_applies_the_keep_rules(page: playwright_rs::Page, _stub: StubServer) {
    add_probe_host(&page).await;
    let answer = page
        .evaluate::<(), String>(
            r#"() => {
                const host = document.getElementById('probe-host');
                host.innerHTML =
                    '<div id="view" data-keep="data-view" data-view="open">View</div>'
                    + '<div id="entry" data-editing>Draft</div>';
                Idiomorph.morph(
                    host,
                    '<div id="view" data-keep="data-view" data-view="closed">View</div>'
                    + '<div id="entry">Server text</div>',
                    { morphStyle: 'innerHTML' },
                );
                const entry = document.getElementById('entry');
                return JSON.stringify({
                    view: document.getElementById('view').dataset.view,
                    entryEditing: entry.hasAttribute('data-editing'),
                    entryText: entry.textContent.trim(),
                });
            }"#,
            None,
        )
        .await
        .unwrap();
    let report: serde_json::Value = serde_json::from_str(&answer).unwrap();
    assert_eq!(
        report["view"], "open",
        "data-keep keeps the view state: {answer}"
    );
    assert_eq!(
        report["entryEditing"], true,
        "the entry in edit stays: {answer}"
    );
    assert_eq!(report["entryText"], "Draft", "its draft stays: {answer}");
}

// [docs/specs/browser_screen.md] SCENARIO: 44.32
async fn check_a_direct_morph_skips_a_control_in_flight(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    add_probe_host(&page).await;
    hold_requests_ending_with(&page, "/probe/inflight").await;
    page.evaluate::<(), ()>(
        r#"() => {
            const host = document.getElementById('probe-host');
            host.innerHTML =
                '<button id="busy" type="button" hx-post="/probe/inflight" hx-swap="none">Sending</button>';
            htmx.process(host);
            document.getElementById('busy').click();
        }"#,
        None,
    )
    .await
    .unwrap();
    let morph = r#"() => {
        Idiomorph.morph(
            document.getElementById('probe-host'),
            '<button id="busy" type="button" hx-post="/probe/inflight" hx-swap="none">Server value</button>',
            { morphStyle: 'innerHTML' },
        );
    }"#;

    page.evaluate::<(), ()>(morph, None).await.unwrap();
    assert_eq!(
        probe_text(&page, "busy").await,
        "Sending",
        "a morph skips the control while its request is in flight"
    );

    release_held_requests_and_wait_for_answers(&page).await;
    let settled = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(25),
        || async {
            page.evaluate::<(), ()>(morph, None).await.unwrap();
            probe_text(&page, "busy").await == "Server value"
        },
    )
    .await;
    assert!(
        settled,
        "the next morph after the request sets the server value, got {:?}",
        probe_text(&page, "busy").await
    );
}

#[tokio::test]
async fn run_screen_morph_checks() {
    let mut runner = StubRunner::launch().await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_load_makes_a_new_rows_post_work_and_a_changed_post_use_its_new_value,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_direct_morph_applies_the_keep_rules,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_direct_morph_skips_a_control_in_flight,
        )
        .await;
    runner.finish().await;
}
