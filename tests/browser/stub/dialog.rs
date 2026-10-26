//! Stub-browser tests for the edit dialog: an htmx swap into its body opens it, Cancel and Escape close it and return focus to the opener, a save closes it only when the engine accepts it, and an answer from outside never replaces its form. Tagged against `docs/specs/browser_dashboard.md`.

use std::time::Duration;

use super::support::{
    hold_requests_ending_with, release_held_requests_and_wait_for_answers, wait_until_focused,
    StubRunner,
};
use super::*;

const DIALOG_WAIT: Duration = Duration::from_secs(5);

async fn open_settings(page: &playwright_rs::Page) {
    page.locator(r#"[data-tab="settings"]"#)
        .await
        .click(None)
        .await
        .unwrap();
    wait_until_visible(page, "#subtab-connections", DIALOG_WAIT).await;
}

async fn dialog_is_open(page: &playwright_rs::Page) -> bool {
    page.evaluate::<(), bool>(
        "() => { const d = document.getElementById('edit-dialog'); return !!d && d.open && d.matches(':modal'); }",
        None,
    )
    .await
    .unwrap_or(false)
}

async fn dialog_body_is_empty(page: &playwright_rs::Page) -> bool {
    page.evaluate::<(), bool>(
        "() => { const b = document.getElementById('edit-dialog-body'); return !!b && b.childElementCount === 0 && b.textContent.trim() === ''; }",
        None,
    )
    .await
    .unwrap_or(false)
}

/// The `close` event comes after the dialog stops being open, and it is the event
/// that discards the form, so the wait ends when the body is empty.
async fn wait_until_dialog_closed(page: &playwright_rs::Page, why: &str) {
    let closed = wait_for_condition_async(DIALOG_WAIT, Duration::from_millis(25), || async {
        !dialog_is_open(page).await && dialog_body_is_empty(page).await
    })
    .await;
    assert!(closed, "{why}");
}

async fn submit_dialog_form(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>(
        "() => document.querySelector('#edit-dialog form').requestSubmit()",
        None,
    )
    .await
    .unwrap();
}

const EDIT_BUTTON: &str = r#"#connection-openrouter-gpt-4o-mini button[hx-get$="/edit"]"#;
const ADD_BUTTON: &str = ".connection-add button";

// [docs/specs/browser_dashboard.md] SCENARIO: 16.46
async fn check_a_swap_into_the_dialog_body_opens_the_modal_dialog(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    open_settings(&page).await;
    assert!(
        !dialog_is_open(&page).await,
        "the dialog is closed before an opener is activated"
    );

    click_and_settle(&page, ADD_BUTTON, "#edit-dialog-body").await;

    assert!(
        dialog_is_open(&page).await,
        "an htmx swap into #edit-dialog-body must open #edit-dialog as a modal"
    );
    assert!(
        wait_until_focused(&page, "#conn_name", Duration::from_secs(2)).await,
        "the first field of the form must have keyboard focus"
    );
    let title = page
        .evaluate::<(), String>(
            r#"() => {
                const dialog = document.getElementById('edit-dialog');
                const heading = document.getElementById(dialog.getAttribute('aria-labelledby'));
                return heading ? heading.textContent.trim() : '';
            }"#,
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        title, "Add Connection",
        "the dialog is named by the heading of its form"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.47
async fn check_closing_the_dialog_returns_focus_to_its_opener(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let stub = stub.changes_handle();
    open_settings(&page).await;

    page.locator(EDIT_BUTTON).await.focus().await.unwrap();
    click_and_settle(&page, EDIT_BUTTON, "#edit-dialog-body").await;
    assert!(dialog_is_open(&page).await, "Edit opens the dialog");
    page.keyboard().press("Escape", None).await.unwrap();
    wait_until_dialog_closed(&page, "Escape must close the dialog").await;
    assert!(
        dialog_body_is_empty(&page).await,
        "closing the dialog discards its form"
    );
    assert!(
        wait_until_focused(&page, EDIT_BUTTON, Duration::from_secs(2)).await,
        "Escape must return keyboard focus to the Edit control that opened the dialog"
    );

    page.locator(ADD_BUTTON).await.focus().await.unwrap();
    click_and_settle(&page, ADD_BUTTON, "#edit-dialog-body").await;
    page.mouse().click(2, 2, None).await.unwrap();
    assert!(
        dialog_is_open(&page).await,
        "a click on the backdrop must not close the dialog"
    );
    page.locator("#edit-dialog [data-close-dialog]")
        .await
        .first()
        .click(None)
        .await
        .unwrap();
    wait_until_dialog_closed(&page, "Cancel must close the dialog").await;
    assert!(dialog_body_is_empty(&page).await);
    assert!(
        wait_until_focused(&page, ADD_BUTTON, Duration::from_secs(2)).await,
        "Cancel must return keyboard focus to the Add control that opened the dialog"
    );
    assert_eq!(
        stub.loads("/fragment/connections/new"),
        1,
        "closing the dialog sends no request: the only request was the one that opened it"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.48
async fn check_a_save_that_the_engine_accepts_closes_the_dialog(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    stub.save_handle().set_connection_saves_succeed(true);
    open_settings(&page).await;

    click_and_settle(&page, ADD_BUTTON, "#edit-dialog-body").await;
    submit_dialog_form(&page).await;
    wait_until_dialog_closed(&page, "a save that the engine accepts closes the dialog").await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.49
async fn check_a_late_answer_from_outside_does_not_replace_the_open_dialog(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    open_settings(&page).await;
    hold_requests_ending_with(&page, "/test").await;

    let test_button = r#"#connection-openrouter-gpt-4o-mini button[hx-post$="/test"]"#;
    page.locator(test_button).await.click(None).await.unwrap();
    assert!(
        !dialog_is_open(&page).await,
        "the Test request is in flight, so no dialog is open yet"
    );
    click_and_settle(&page, ADD_BUTTON, "#edit-dialog-body").await;
    page.locator("#conn_name")
        .await
        .fill("Typed while the test ran", None)
        .await
        .unwrap();

    release_held_requests_and_wait_for_answers(&page).await;

    assert!(dialog_is_open(&page).await, "the dialog stays open");
    let typed = page
        .evaluate::<(), String>(
            "() => { const f = document.getElementById('conn_name'); return f ? f.value : ''; }",
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        typed, "Typed while the test ran",
        "the answer of the Test must not replace the form the player is typing in"
    );
    assert_eq!(
        page.locator("#edit-dialog .connection-test-result")
            .await
            .count()
            .await
            .unwrap_or(0),
        0,
        "the late result is dropped"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.50
async fn check_escape_closes_an_open_error_detail_before_it_closes_the_dialog(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    open_settings(&page).await;
    click_and_settle(&page, ADD_BUTTON, "#edit-dialog-body").await;
    page.locator("#conn_name")
        .await
        .fill("Typed before the refusal", None)
        .await
        .unwrap();
    submit_dialog_form(&page).await;
    let slot = r#"#edit-dialog [data-error-slot="connection-form"]"#;
    wait_for_error_disclosure(&page, slot, "a refused save reports in the form's own slot").await;
    page.locator(&format!("{slot} .error-details-toggle"))
        .await
        .click(None)
        .await
        .unwrap();
    assert!(error_details_open(&page, slot).await, "Details opens");

    page.keyboard().press("Escape", None).await.unwrap();

    assert!(
        !error_details_open(&page, slot).await,
        "Escape closes the error detail"
    );
    assert!(
        dialog_is_open(&page).await,
        "the Escape that closed the error detail must not close the dialog"
    );
    let typed = page
        .evaluate::<(), String>("() => document.getElementById('conn_name').value", None)
        .await
        .unwrap();
    assert_eq!(typed, "Typed before the refusal", "the draft stays");

    page.keyboard().press("Escape", None).await.unwrap();
    wait_until_dialog_closed(&page, "a second Escape closes the dialog").await;
}

#[tokio::test]
async fn run_dialog_checks() {
    let mut runner = StubRunner::launch().await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_swap_into_the_dialog_body_opens_the_modal_dialog,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_closing_the_dialog_returns_focus_to_its_opener,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_save_that_the_engine_accepts_closes_the_dialog,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_late_answer_from_outside_does_not_replace_the_open_dialog,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_escape_closes_an_open_error_detail_before_it_closes_the_dialog,
        )
        .await;
    runner.finish().await;
}
