//! Stub-browser tests for the Worlds panel: the keyboard-focus landing after a world row delete. Tagged against `docs/specs/browser_worlds.md`.

use std::time::Duration;

use super::support::{
    StubRunner, accept_delete_confirmation, active_element_is, wait_for_count,
    wait_until_focus_within, wait_until_focused,
};
use super::*;

// [docs/specs/browser_worlds.md] SCENARIO: 29.4
async fn check_deleting_a_world_keeps_keyboard_focus_in_the_panel(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    stub.row_delete_handle().set_row_deletes_succeed(true);
    accept_delete_confirmation(&page).await;
    open_worlds_tab(&page).await;

    let rows = ".worlds-list .world-item";
    let rows_before = page
        .query_selector_all(rows)
        .await
        .expect("count the worlds the fixture shows")
        .len();
    assert_eq!(
        rows_before, 2,
        "the fixture must show the two worlds the scenario names"
    );

    let first_delete = ".worlds-list .world-item:first-child .btn-danger";
    page.locator(first_delete).await.focus().await.unwrap();
    page.locator(first_delete).await.click(None).await.unwrap();

    assert!(
        wait_for_count(&page, rows, rows_before - 1).await,
        "the delete must remove the world's row"
    );
    assert!(
        wait_until_focused(
            &page,
            ".worlds-list .world-item .btn-cyan",
            Duration::from_secs(5)
        )
        .await,
        "deleting a world must move focus to the surviving world's Edit control"
    );

    let last_delete = ".worlds-list .world-item .btn-danger";
    page.locator(last_delete).await.focus().await.unwrap();
    page.locator(last_delete).await.click(None).await.unwrap();

    // The clicked control sits inside the panel, so a focus wait is satisfied
    // before the swap lands. Wait for the removal itself first.
    assert!(
        wait_for_count(&page, rows, 0).await,
        "the last delete must remove the final world"
    );
    assert!(
        wait_until_focus_within(&page, ".worlds-panel", Duration::from_secs(5)).await,
        "deleting the last world must leave focus inside the Worlds panel"
    );
}

// A refused delete (the world still has games) answers a 400 error fragment,
// which the world's own row reports in its inline slot.
// [docs/specs/browser_worlds.md] SCENARIO: 29.5
async fn check_a_refused_world_delete_reports_in_its_row_and_keeps_focus(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    stub.row_delete_handle().set_world_delete_refused(true);
    accept_delete_confirmation(&page).await;
    open_worlds_tab(&page).await;

    let rows = ".worlds-list .world-item";
    let slot = r#"#world-redmist_estate [data-error-slot="world-redmist_estate"]"#;
    let first_delete = "#world-redmist_estate .btn-danger";
    let row_before = page
        .evaluate::<(), bool>(
            "() => { window.__worldRow = document.getElementById('world-redmist_estate'); return !!window.__worldRow; }",
            None,
        )
        .await
        .unwrap();
    assert!(row_before, "the first world's row must carry its stable id");
    page.locator(first_delete).await.focus().await.unwrap();
    page.locator(first_delete).await.click(None).await.unwrap();

    wait_until_visible(&page, slot, Duration::from_secs(5)).await;
    let (visible, message, raw) = read_error_disclosure(&page, slot).await;
    assert!(visible, "the refusal shows in the world's own slot");
    assert!(
        message.contains("action failed") && raw.contains("World still has games"),
        "the slot must carry the refusal, got {message:?} / {raw:?}"
    );
    assert_eq!(
        page.query_selector_all(rows)
            .await
            .unwrap_or_default()
            .len(),
        2,
        "the refusal must leave both rows in place"
    );
    assert!(
        page.evaluate::<(), bool>(
            "() => document.getElementById('world-redmist_estate') === window.__worldRow",
            None,
        )
        .await
        .unwrap(),
        "the refused row must be the same element"
    );
    assert!(
        active_element_is(&page, first_delete).await,
        "a refused delete must leave focus on its Delete control"
    );
}

// [docs/specs/browser_worlds.md] SCENARIO: 29.6
async fn check_a_world_posture_save_leaves_the_dialog_open_with_the_draft(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    open_worlds_tab(&page).await;
    let edit = r#"#world-test button[hx-get$="/edit"]"#;
    click_and_settle(&page, edit, "#edit-dialog-body").await;

    let drafted: bool = page
        .evaluate::<(), bool>(
            r#"() => {
                const input = document.querySelector('#edit-dialog input[name="name"]');
                if (!input) return false;
                input.value = 'A typed world name';
                window.__worldForm = document.getElementById('world-form-container');
                window.__worldInput = input;
                return true;
            }"#,
            None,
        )
        .await
        .unwrap();
    assert!(drafted, "the World form opens in the dialog");

    let saved: String = page
        .evaluate::<(), String>(
            r#"async () => {
                const select = document.querySelector('#edit-dialog select[name="narrative_tense"]');
                const ended = new Promise((resolve) =>
                    select.addEventListener('htmx:afterRequest', resolve, { once: true }));
                select.value = 'present';
                select.dispatchEvent(new Event('change', { bubbles: true }));
                await ended;
                await window.chroniclerLive.settle();
                const dialog = document.getElementById('edit-dialog');
                const input = document.querySelector('#edit-dialog input[name="name"]');
                if (!dialog.open) return 'the posture save closed the dialog';
                if (document.getElementById('world-form-container') !== window.__worldForm) return 'the form is a new element';
                if (input !== window.__worldInput || input.value !== 'A typed world name') return 'the typed name did not survive';
                return document.getElementById('world-posture-status').hasAttribute('data-saved')
                    ? '' : 'the Saved flag does not show';
            }"#,
            None,
        )
        .await
        .unwrap();
    assert_eq!(saved, "");
}

#[tokio::test]
async fn run_world_checks() {
    let mut runner = StubRunner::launch().await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_deleting_a_world_keeps_keyboard_focus_in_the_panel,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_refused_world_delete_reports_in_its_row_and_keeps_focus,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_world_posture_save_leaves_the_dialog_open_with_the_draft,
        )
        .await;
    runner.finish().await;
}
