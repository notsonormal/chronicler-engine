//! Stub-browser tests for the Worlds panel: the keyboard-focus landing after a world row delete. Tagged against `docs/specs/browser_worlds.md`.

use std::time::Duration;

use super::support::{StubRunner, accept_delete_confirmation, wait_for_count, wait_until_focused};
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
        wait_until_focused(&page, ".worlds-panel :focus", Duration::from_secs(5)).await,
        "deleting the last world must leave focus inside the Worlds panel"
    );
}

/// The refused delete waits out `swap:0.3s`, so a poll can settle in that gap:
/// htmx fires the poll's own `beforeSwap` and `afterSettle`, and this dispatches
/// both as the row swap lands, before the row's own settle. The delete's landing
/// spot has to survive them.
async fn settle_a_poll_during_the_refused_delete(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>(
        r#"() => {
            const once = (evt) => {
                const target = evt.detail && evt.detail.target;
                if (!target || !target.classList || !target.classList.contains('world-item')) {
                    return;
                }
                document.body.removeEventListener('htmx:afterSwap', once);
                const display = document.getElementById('status-display');
                const detail = { elt: display, target: display };
                display.dispatchEvent(new CustomEvent('htmx:beforeSwap', {
                    bubbles: true,
                    detail: Object.assign({}, detail, {
                        shouldSwap: true,
                        serverResponse: 'idle',
                    }),
                }));
                display.dispatchEvent(new CustomEvent('htmx:afterSettle', {
                    bubbles: true,
                    detail,
                }));
            };
            document.body.addEventListener('htmx:afterSwap', once);
        }"#,
        None,
    )
    .await
    .unwrap();
}

// A refused delete (the world still has games) answers a 200 whose body stands
// in for the row, so the refusal's own message is where focus belongs.
// [docs/specs/browser_worlds.md] SCENARIO: 29.5
async fn check_a_refused_world_delete_lands_focus_on_its_message(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    stub.row_delete_handle().set_world_delete_refused(true);
    accept_delete_confirmation(&page).await;
    open_worlds_tab(&page).await;
    settle_a_poll_during_the_refused_delete(&page).await;

    let rows = ".worlds-list .world-item";
    let refusal = ".worlds-list .world-item:first-child .error-message";
    let first_delete = ".worlds-list .world-item:first-child .btn-danger";
    page.locator(first_delete).await.focus().await.unwrap();
    page.locator(first_delete).await.click(None).await.unwrap();

    wait_until_visible(&page, refusal, Duration::from_secs(5)).await;
    assert_eq!(
        page.query_selector_all(rows)
            .await
            .unwrap_or_default()
            .len(),
        2,
        "the refusal must replace the world's row rather than remove it"
    );
    assert_eq!(
        page.query_selector_all(".worlds-list .world-item .btn-cyan")
            .await
            .unwrap_or_default()
            .len(),
        1,
        "only the refused row takes the message: the surviving world keeps its controls"
    );
    assert!(
        wait_until_focused(&page, refusal, Duration::from_secs(5)).await,
        "a refused delete must land focus on the refusal message, not on the neighbouring row"
    );
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
            check_a_refused_world_delete_lands_focus_on_its_message,
        )
        .await;
    runner.finish().await;
}
