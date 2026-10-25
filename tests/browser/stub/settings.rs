//! Stub-browser tests for the Settings panel: the client-side sub-tabs and the shared connection form page. Tagged against `docs/specs/browser_settings.md`.

use std::time::Duration;

use super::support::wait_until_focused;
use super::*;
use super::support::StubRunner;

async fn element_is_visible(page: &playwright_rs::Page, selector: &str) -> bool {
    page.evaluate::<String, bool>(
        "(selector) => { const el = document.querySelector(selector); return !!el && !!(el.offsetWidth || el.offsetHeight || el.getClientRects().length); }",
        Some(&selector.to_string()),
    )
    .await
    .unwrap()
}

async fn open_settings(page: &playwright_rs::Page) {
    page.locator(r#"[data-tab="settings"]"#)
        .await
        .click(None)
        .await
        .unwrap();
    wait_until_visible(page, "#subtab-connections", Duration::from_secs(5)).await;
}

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
    page.locator(r#"[data-tab="settings"]"#)
        .await
        .click(None)
        .await
        .unwrap();

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
async fn check_add_and_edit_open_the_shared_connection_form_page(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let stub = stub.changes_handle();
    open_settings(&page).await;

    page.locator(".connection-add button")
        .await
        .click(None)
        .await
        .unwrap();
    wait_until_visible(&page, ".connection-form-page", Duration::from_secs(5)).await;

    let back_link = page
        .locator(".connection-form-page .back-link")
        .await
        .inner_text()
        .await
        .unwrap_or_default();
    assert!(
        back_link.contains("Connections"),
        "the form page must offer a back link to Connections, got {back_link:?}"
    );
    assert!(
        !element_is_visible(&page, ".connection-list").await,
        "while the editor is open, the list of its panel is hidden"
    );

    page.locator(".connection-form-page .back-link")
        .await
        .click(None)
        .await
        .unwrap();
    wait_until_visible(&page, ".connection-list", Duration::from_secs(5)).await;
    assert_eq!(
        stub.loads("/fragment/connections/new"),
        1,
        "closing the editor sends no request: the only request was the one that opened it"
    );

    let edit_selector = r#".connection-row:has(.connection-name:text-is("openrouter-gpt-4o-mini")) button:has-text('Edit')"#;
    page.locator(edit_selector).await.click(None).await.unwrap();
    wait_until_visible(&page, ".connection-form-page", Duration::from_secs(5)).await;

    let heading = page
        .locator(".connection-form-page h2")
        .await
        .inner_text()
        .await
        .unwrap_or_default();
    assert!(
        heading.contains("openrouter-gpt-4o-mini"),
        "the Edit form must name the connection, got {heading:?}"
    );

    page.locator(".connection-form-page .back-link")
        .await
        .click(None)
        .await
        .unwrap();
    wait_until_visible(&page, ".connection-list", Duration::from_secs(5)).await;
}

// [docs/specs/browser_settings.md] SCENARIO: 40.3
async fn check_opening_and_closing_the_connection_editor_keeps_keyboard_focus_in_the_panel(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    open_settings(&page).await;

    let edit = r#"#connection-openrouter-gpt-4o-mini button[hx-get$="/edit"]"#;
    page.locator(edit).await.focus().await.unwrap();
    page.locator(edit).await.click(None).await.unwrap();
    wait_until_visible(&page, ".connection-form-page", Duration::from_secs(5)).await;
    assert!(
        wait_until_focused(&page, "#conn_name", Duration::from_secs(2)).await,
        "opening the connection form must land focus in its first field"
    );

    let back = ".connection-form-page .back-link";
    page.locator(back).await.focus().await.unwrap();
    page.locator(back).await.click(None).await.unwrap();
    wait_until_visible(&page, ".connection-list", Duration::from_secs(5)).await;
    assert!(
        wait_until_focused(&page, edit, Duration::from_secs(2)).await,
        "the back link must land focus on the control that opened the editor, not on the page body"
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
            check_add_and_edit_open_the_shared_connection_form_page,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_opening_and_closing_the_connection_editor_keeps_keyboard_focus_in_the_panel,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_saved_text_check_change_shows_the_saved_line,
        )
        .await;
    runner.finish().await;
}
