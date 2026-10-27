//! Stub-browser tests for the Settings panel: the client-side sub-tabs and the connection form in the dialog. Tagged against `docs/specs/browser_settings.md`.

use std::time::Duration;

use chronicler_engine::domain::model::change::ChangeKind;

use super::support::{
    dialog_is_open, open_settings, wait_for_count, wait_until_dialog_closed, wait_until_focused,
    StubRunner,
};
use super::*;

// [docs/specs/browser_settings.md] SCENARIO: 40.1
async fn check_settings_sub_tabs_keep_their_place_across_a_top_level_tab_change(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    open_settings(&page).await;

    page.locator("#subtab-text-check")
        .await
        .click(None)
        .await
        .unwrap();

    let after_switch: (String, bool, bool) = page
        .evaluate::<(), (String, bool, bool)>(
            r#"(() => {
                    const tab = document.getElementById('subtab-text-check');
                    const textPanel = document.getElementById('settings-text-check');
                    const connectionsPanel = document.getElementById('settings-connections');
                    return [
                        tab.getAttribute('aria-selected'),
                        !textPanel.hidden,
                        !connectionsPanel.hidden,
                    ];
                })()"#,
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        after_switch.0, "true",
        "the sub-tab must report itself selected"
    );
    assert!(after_switch.1, "the Text Check panel must be visible");
    assert!(
        !after_switch.2,
        "the Connections panel must not be the visible one"
    );

    page.locator(r#"[data-tab="game"]"#)
        .await
        .click(None)
        .await
        .unwrap();
    open_settings(&page).await;

    let kept: bool = page
        .evaluate::<(), bool>(
            r#"(() => {
                    const tab = document.getElementById('subtab-text-check');
                    return tab.getAttribute('aria-selected') === 'true'
                        && !document.getElementById('settings-text-check').hidden
                        && document.getElementById('settings-connections').hidden;
                })()"#,
            None,
        )
        .await
        .unwrap();
    assert!(
        kept,
        "the selected sub-tab must survive a top-level tab change"
    );
}

// [docs/specs/browser_settings.md] SCENARIO: 40.2
async fn check_add_and_edit_open_the_connection_form_in_the_dialog(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let stub = stub.changes_handle();
    open_settings(&page).await;

    click_and_settle(&page, ".connection-add button", "#edit-dialog-body").await;
    wait_until_visible(
        &page,
        "#edit-dialog .connection-form",
        Duration::from_secs(5),
    )
    .await;
    let heading = page
        .locator("#edit-dialog h2")
        .await
        .inner_text()
        .await
        .unwrap_or_default();
    assert_eq!(heading, "Add Connection");

    page.locator("#edit-dialog [data-close-dialog]")
        .await
        .click(None)
        .await
        .unwrap();
    wait_until_hidden(&page, "#edit-dialog", Duration::from_secs(5)).await;
    assert!(
        wait_for_count(&page, "#edit-dialog-body > *", 0).await,
        "closing the dialog empties its body"
    );
    assert_eq!(
        stub.loads("/fragment/connections/new"),
        1,
        "Cancel sent no request: the only request was the one that opened the dialog"
    );

    let edit_selector = r#".connection-row:has(.connection-name:text-is("openrouter-gpt-4o-mini")) button:has-text('Edit')"#;
    click_and_settle(&page, edit_selector, "#edit-dialog-body").await;
    wait_until_visible(
        &page,
        "#edit-dialog .connection-form",
        Duration::from_secs(5),
    )
    .await;
    let heading = page
        .locator("#edit-dialog h2")
        .await
        .inner_text()
        .await
        .unwrap_or_default();
    assert!(
        heading.contains("openrouter-gpt-4o-mini"),
        "the Edit form must name the connection, got {heading:?}"
    );

    page.locator("#edit-dialog [data-close-dialog]")
        .await
        .click(None)
        .await
        .unwrap();
    wait_until_hidden(&page, "#edit-dialog", Duration::from_secs(5)).await;
}

// [docs/specs/browser_settings.md] SCENARIO: 40.3
async fn check_opening_and_closing_the_connection_dialog_keeps_keyboard_focus_where_the_player_works(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    open_settings(&page).await;

    let edit = r#"#connection-openrouter-gpt-4o-mini button[hx-get$="/edit"]"#;
    page.locator(edit).await.focus().await.unwrap();
    click_and_settle(&page, edit, "#edit-dialog-body").await;
    wait_until_visible(
        &page,
        "#edit-dialog .connection-form",
        Duration::from_secs(5),
    )
    .await;
    assert!(
        wait_until_focused(&page, "#conn_name", Duration::from_secs(2)).await,
        "opening the connection form must land focus in its first field"
    );

    page.locator("#edit-dialog [data-close-dialog]")
        .await
        .click(None)
        .await
        .unwrap();
    wait_until_hidden(&page, "#edit-dialog", Duration::from_secs(5)).await;
    assert!(
        wait_until_focused(&page, edit, Duration::from_secs(2)).await,
        "Cancel must land focus on the control that opened the dialog, not on the page body"
    );

    // A role change swaps nothing, so the select keeps focus.
    page.locator("#role-select-quantifier")
        .await
        .focus()
        .await
        .unwrap();
    page.locator("#role-select-quantifier")
        .await
        .select_option("openrouter-euryale", None)
        .await
        .unwrap();
    assert!(
        wait_until_focused(&page, "#role-select-quantifier", Duration::from_secs(2)).await,
        "changing a role must keep focus on that role's select"
    );
}

// [docs/specs/browser_settings.md] SCENARIO: 40.5
async fn check_the_test_result_of_a_saved_connection_opens_in_the_dialog_and_survives_a_load(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    open_settings(&page).await;

    let test_button = r#"#connection-openrouter-gpt-4o-mini button[hx-post$="/test"]"#;
    page.locator(test_button).await.focus().await.unwrap();
    click_and_settle(&page, test_button, "#edit-dialog-body").await;
    assert!(dialog_is_open(&page).await, "the row Test opens the dialog");
    let result = page
        .locator("#edit-dialog .connection-test-result.success")
        .await
        .inner_text()
        .await
        .unwrap_or_default();
    assert!(
        result.contains("OK"),
        "the dialog shows the result of the test, got {result:?}"
    );

    let loads_before = changes.loads("/fragment/settings");
    changes.bump(ChangeKind::SETTINGS);
    let loaded = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(25),
        || async { changes.loads("/fragment/settings") > loads_before },
    )
    .await;
    assert!(loaded, "the Settings panel loads after its number moved");
    assert!(
        dialog_is_open(&page).await
            && page
                .locator("#edit-dialog .connection-test-result.success")
                .await
                .count()
                .await
                .unwrap_or(0)
                == 1,
        "a Settings load must leave the result in the dialog"
    );

    page.keyboard().press("Escape", None).await.unwrap();
    wait_until_dialog_closed(&page, "Escape closes the result").await;
    assert!(
        wait_until_focused(&page, test_button, Duration::from_secs(2)).await,
        "closing the result returns focus to the Test control"
    );
}

// [docs/specs/browser_settings.md] SCENARIO: 40.6
async fn check_a_busy_refused_connection_save_shows_its_refusal_in_the_dialog(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    stub.save_handle()
        .set_connection_saves_refused_as_busy(true);
    open_settings(&page).await;
    click_and_settle(&page, ".connection-add button", "#edit-dialog-body").await;
    page.locator("#conn_name")
        .await
        .fill("Typed while Autoplay runs", None)
        .await
        .unwrap();

    page.evaluate::<(), ()>(
        "() => document.querySelector('#edit-dialog form').requestSubmit()",
        None,
    )
    .await
    .unwrap();

    let slot = r#"#edit-dialog [data-error-slot="connection-form"]"#;
    wait_for_error_disclosure(&page, slot, "a busy refusal shows in the form's own slot").await;
    let (_, message, _) = read_error_disclosure(&page, slot).await;
    assert!(
        message.contains("The Game is busy"),
        "the refusal names the busy Game, got {message:?}"
    );
    assert!(
        dialog_is_open(&page).await,
        "a refused save keeps the dialog open"
    );
    let typed = page
        .evaluate::<(), String>("() => document.getElementById('conn_name').value", None)
        .await
        .unwrap();
    assert_eq!(typed, "Typed while Autoplay runs", "the draft stays");
}

// [docs/specs/browser_settings.md] SCENARIO: 40.4
async fn check_a_saved_text_check_change_shows_the_saved_line(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    open_settings(&page).await;
    page.locator("#subtab-text-check")
        .await
        .click(None)
        .await
        .unwrap();
    wait_until_visible(&page, "#check_mode", Duration::from_secs(5)).await;

    let shown_before: String = page
        .evaluate::<(), String>(
            "() => getComputedStyle(document.getElementById('text-check-status'), '::before').content",
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        shown_before, "none",
        "the status line must show no text before a save"
    );

    select_option_and_await_saved(&page, "#check_mode", "spell", "text-check-status").await;
}

#[tokio::test]
async fn run_settings_checks() {
    let mut runner = StubRunner::launch().await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_settings_sub_tabs_keep_their_place_across_a_top_level_tab_change,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_add_and_edit_open_the_connection_form_in_the_dialog,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_opening_and_closing_the_connection_dialog_keeps_keyboard_focus_where_the_player_works,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_saved_text_check_change_shows_the_saved_line,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_the_test_result_of_a_saved_connection_opens_in_the_dialog_and_survives_a_load,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_busy_refused_connection_save_shows_its_refusal_in_the_dialog,
        )
        .await;
    runner.finish().await;
}
