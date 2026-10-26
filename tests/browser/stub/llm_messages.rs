//! Stub-browser tests for the LLM Messages panel: the shipped client's keyboard path over a canned row, the record that opens in the dialog, and the load that leaves the open dialog alone. Tagged against `docs/specs/browser_llm_messages.md`.

// The Open control is a native button with an `hx-get` into the edit dialog, so
// Enter and Space fire the same click a mouse uses. The stub server renders the
// list and the record with the engine's own templates, so a row holds what the
// real list holds.
//
// The panel loads when the `llm` number moves. The page's timer stops in the
// checks that move it, so each load is one the check asked for. Every assertion
// polls the state it wants rather than sampling once, and reads the live DOM
// rather than a Playwright element handle.

use std::time::Duration;

use chronicler_engine::domain::model::change::ChangeKind;

use super::support::{wait_until_focused, StubRunner};
use super::*;

const OPEN_CONTROL: &str = ".llm-message-open";
const RECORD: &str = "#edit-dialog .llm-message-record";
const WAIT: Duration = Duration::from_millis(2000);

/// The section loads its part when it opens, so the wait is for the card the load renders.
async fn open_llm_messages_tab(page: &playwright_rs::Page) {
    page.locator(r#"[data-tab="llm-messages"]"#)
        .await
        .click(None)
        .await
        .unwrap();
    wait_until_visible(page, ".llm-message-card", Duration::from_millis(2000)).await;
}

async fn dialog_is_open(page: &playwright_rs::Page) -> bool {
    page.evaluate::<(), bool>(
        "() => { const d = document.getElementById('edit-dialog'); return !!d && d.open && d.matches(':modal'); }",
        None,
    )
    .await
    .unwrap_or(false)
}

async fn wait_until_dialog_closed(page: &playwright_rs::Page, why: &str) {
    let closed = wait_for_condition_async(WAIT, Duration::from_millis(25), || async {
        page.evaluate::<(), bool>(
            "() => { const d = document.getElementById('edit-dialog'); return !d.open && d.querySelector('#edit-dialog-body').childElementCount === 0; }",
            None,
        )
        .await
        .unwrap_or(false)
    })
    .await;
    assert!(closed, "{why}");
}

async fn text_of(page: &playwright_rs::Page, selector: &str) -> String {
    page.evaluate::<String, String>(
        "(selector) => { const el = document.querySelector(selector); return el ? el.textContent : ''; }",
        Some(&selector.to_string()),
    )
    .await
    .unwrap()
}

async fn card_count(page: &playwright_rs::Page) -> usize {
    page.query_selector_all(".llm-message-card")
        .await
        .map(|cards| cards.len())
        .unwrap_or(0)
}

async fn press_open_and_wait_for_the_record(page: &playwright_rs::Page) {
    page.locator(OPEN_CONTROL)
        .await
        .press("Enter", None)
        .await
        .unwrap();
    wait_until_visible(page, RECORD, Duration::from_secs(5)).await;
}

// [docs/specs/browser_llm_messages.md] SCENARIO: 35.1
async fn check_open_shows_the_record_in_the_dialog_from_the_keyboard(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    open_llm_messages_tab(&page).await;
    let row = text_of(&page, ".llm-messages-panel").await;
    assert!(
        !row.contains("the system prompt") && !row.contains("canned narration"),
        "the row holds no record text: {row:?}"
    );

    // Tab from the LLM tab button: the first focusable in the panel is the Open
    // control, so this proves it is keyboard-reachable, not just focusable.
    page.locator(r#"[data-tab="llm-messages"]"#)
        .await
        .press("Tab", None)
        .await
        .unwrap();
    assert!(
        wait_until_focused(&page, OPEN_CONTROL, Duration::from_millis(500)).await,
        "Tab must reach the Open control"
    );
    press_open_and_wait_for_the_record(&page).await;

    assert!(dialog_is_open(&page).await, "Open must open the dialog");
    let record = text_of(&page, RECORD).await;
    for text in ["the system prompt", "the user prompt", "canned narration"] {
        assert!(
            record.contains(text),
            "the dialog must show the whole record ({text}): {record:?}"
        );
    }
    let titled = page
        .evaluate::<(), bool>(
            "() => document.getElementById('edit-dialog-title').closest('dialog') === document.getElementById('edit-dialog')",
            None,
        )
        .await
        .unwrap();
    assert!(titled, "the record carries the title that names the dialog");
}

// [docs/specs/browser_llm_messages.md] SCENARIO: 35.2
async fn check_close_returns_focus_to_the_open_control(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    open_llm_messages_tab(&page).await;
    page.locator(OPEN_CONTROL).await.focus().await.unwrap();
    press_open_and_wait_for_the_record(&page).await;

    page.locator("#edit-dialog [data-close-dialog]")
        .await
        .click(None)
        .await
        .unwrap();

    wait_until_dialog_closed(&page, "Close must close the dialog and empty its body").await;
    assert!(
        wait_until_focused(&page, OPEN_CONTROL, WAIT).await,
        "focus must return to the Open control"
    );
}

// [docs/specs/browser_llm_messages.md] SCENARIO: 35.3
async fn check_focused_open_control_keeps_focus_across_panel_load(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    open_llm_messages_tab(&page).await;
    stop_page_timer(&page).await;

    page.locator(OPEN_CONTROL).await.focus().await.unwrap();
    page.evaluate::<(), ()>(
        r#"() => { window.__openBefore = document.querySelector('.llm-message-open'); }"#,
        None,
    )
    .await
    .unwrap();
    let changes = stub.changes_handle();
    let before = changes.loads("/fragment/llm-messages");
    changes.bump(ChangeKind::LLM_MESSAGES);
    ask_once(&page).await;
    assert_eq!(
        changes.loads("/fragment/llm-messages"),
        before + 1,
        "a moved `llm` number must load the panel once"
    );

    let same_node = page
        .evaluate::<(), bool>(
            r#"() => document.querySelector('.llm-message-open') === window.__openBefore"#,
            None,
        )
        .await
        .unwrap();
    assert!(same_node, "the load must keep the row's Open control node");
    assert!(
        wait_until_focused(&page, OPEN_CONTROL, Duration::from_millis(1000)).await,
        "the panel load must keep focus on the Open control"
    );
}

// [docs/specs/browser_llm_messages.md] SCENARIO: 35.4
async fn check_a_new_llm_message_appears_after_a_record_was_opened_and_closed(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    open_llm_messages_tab(&page).await;
    for _ in 0..2 {
        press_open_and_wait_for_the_record(&page).await;
        page.locator("#edit-dialog [data-close-dialog]")
            .await
            .click(None)
            .await
            .unwrap();
        wait_until_dialog_closed(&page, "Close must close the dialog").await;
    }

    // The page's own timer loads the panel, with nothing paused by the opens.
    stub.llm_handle().record_message();
    let appeared = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(50),
        || async { card_count(&page).await == 2 },
    )
    .await;
    assert!(
        appeared,
        "a new LLM message must appear in the panel with no reload of the page"
    );
}

// [docs/specs/browser_llm_messages.md] SCENARIO: 35.5
async fn check_an_open_record_stays_as_it_is_when_the_panel_loads(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    open_llm_messages_tab(&page).await;
    stop_page_timer(&page).await;
    press_open_and_wait_for_the_record(&page).await;
    let before = text_of(&page, RECORD).await;

    stub.llm_handle().record_message();
    ask_once(&page).await;
    let appeared = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(50),
        || async { card_count(&page).await == 2 },
    )
    .await;
    assert!(
        appeared,
        "the load must list the new message behind the dialog"
    );

    assert!(
        dialog_is_open(&page).await,
        "a panel load must leave the dialog open"
    );
    assert_eq!(
        text_of(&page, RECORD).await,
        before,
        "a panel load must leave the record as it was"
    );
}

// The record of a row can be gone when Open asks for it, because retention keeps
// 50 records. The engine answers a 404: no dialog opens, the row's own slot holds
// the refusal, and focus stays on the Open control.
// [docs/specs/browser_llm_messages.md] SCENARIO: 35.6
async fn check_a_missing_record_reports_in_the_rows_slot_and_keeps_focus(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    open_llm_messages_tab(&page).await;
    stub.llm_handle().prune_records();
    page.locator(OPEN_CONTROL).await.focus().await.unwrap();
    page.locator(OPEN_CONTROL).await.click(None).await.unwrap();

    let slot = ".llm-message-card [data-error-slot]";
    wait_until_visible(&page, slot, Duration::from_secs(5)).await;
    let (shown, _, raw) = read_error_disclosure(&page, slot).await;
    assert!(shown, "the refusal shows in the row's own slot");
    assert!(
        raw.contains("LLM message not found"),
        "the slot carries the refusal, got {raw:?}"
    );
    assert!(
        !dialog_is_open(&page).await,
        "a missing record opens no dialog"
    );
    assert!(
        wait_until_focused(&page, OPEN_CONTROL, WAIT).await,
        "a missing record must leave focus on the Open control"
    );
}

#[tokio::test]
async fn run_llm_messages_checks() {
    let mut runner = StubRunner::launch().await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_open_shows_the_record_in_the_dialog_from_the_keyboard,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_close_returns_focus_to_the_open_control,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_focused_open_control_keeps_focus_across_panel_load,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_new_llm_message_appears_after_a_record_was_opened_and_closed,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_an_open_record_stays_as_it_is_when_the_panel_loads,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_missing_record_reports_in_the_rows_slot_and_keeps_focus,
        )
        .await;
    runner.finish().await;
}
