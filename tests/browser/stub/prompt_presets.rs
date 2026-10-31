//! Stub-browser tests for the Prompt Presets panel: the keyboard-focus landing after a preset card delete and when an editor opens. They also cover the report of a refused editor. Tagged against `docs/specs/browser_prompt_presets.md`.

use std::time::Duration;

use super::support::{
    NOTICE, StubRunner, accept_delete_confirmation, wait_for_count, wait_until_focused,
};
use super::*;

/// The focus moves a page has made, in order. `focusin` is synchronous with the
/// move, so the list is complete once the move the test waits for has happened:
/// a landing that passed through another card is visible rather than assumed.
async fn install_focus_recorder(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>(
        r#"() => {
            window.__focusLandings = [];
            document.addEventListener('focusin', (evt) => {
                const el = evt.target;
                window.__focusLandings.push(
                    el.id ||
                        el.getAttribute('hx-get') ||
                        el.getAttribute('hx-post') ||
                        el.tagName.toLowerCase()
                );
            });
        }"#,
        None,
    )
    .await
    .unwrap();
}

async fn focus_landings(page: &playwright_rs::Page) -> Vec<String> {
    page.evaluate::<(), Vec<String>>("() => window.__focusLandings", None)
        .await
        .unwrap()
}

// [docs/specs/browser_prompt_presets.md] SCENARIO: 28.4
async fn check_opening_a_preset_editor_moves_focus_straight_into_the_form(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    open_prompt_presets_tab(&page).await;
    install_focus_recorder(&page).await;

    let edit = r#"button[hx-get="/fragment/prompt-presets/custom_ref/edit"]"#;
    page.locator(edit).await.focus().await.unwrap();
    page.locator(edit).await.click(None).await.unwrap();

    assert!(
        wait_until_focused(&page, "input#edit-name-custom_ref", Duration::from_secs(5)).await,
        "opening the editor must land focus in the dialog's Name field"
    );
    assert_eq!(
        focus_landings(&page).await,
        vec![
            "preset-edit-open-custom_ref".to_string(),
            "edit-name-custom_ref".to_string(),
        ],
        "the editor must claim focus once, with no other control in between"
    );
}

// [docs/specs/browser_prompt_presets.md] SCENARIO: 28.3
async fn check_deleting_a_preset_card_keeps_keyboard_focus_on_a_card(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    stub.row_delete_handle().set_row_deletes_succeed(true);
    accept_delete_confirmation(&page).await;
    open_prompt_presets_tab(&page).await;

    let deletes = ".prompt-presets-panel .preset-section:first-child .preset-card .btn-danger";
    let deletable_before = page
        .query_selector_all(deletes)
        .await
        .expect("count the deletable preset cards the fixture shows")
        .len();
    assert_eq!(
        deletable_before, 2,
        "the fixture must show two deletable preset cards in one section"
    );

    let first_delete = r#"button[hx-post="/prompt-presets/custom_ref/delete"]"#;
    page.locator(first_delete).await.focus().await.unwrap();
    page.locator(first_delete).await.click(None).await.unwrap();

    assert!(
        wait_for_count(&page, deletes, deletable_before - 1).await,
        "the delete must remove the card the control sat in"
    );
    assert!(
        wait_until_focused(
            &page,
            "#preset-card-custom_two :focus",
            Duration::from_secs(5),
        )
        .await,
        "deleting a preset card must move focus to a control of the next card"
    );

    let last_delete = r#"button[hx-post="/prompt-presets/custom_two/delete"]"#;
    page.locator(last_delete).await.focus().await.unwrap();
    page.locator(last_delete).await.click(None).await.unwrap();

    assert!(
        wait_for_count(&page, deletes, deletable_before - 2).await,
        "the second delete must remove the last non-default card"
    );
    assert!(
        wait_until_focused(
            &page,
            r#"button[hx-get="/fragment/prompt-presets/system_if_default/view"]"#,
            Duration::from_secs(5),
        )
        .await,
        "deleting the last card of its kind must move focus to the card before it"
    );
}

// A default preset refuses its Edit load, so the page takes a refusal from a dialog
// opener.
// [docs/specs/browser_prompt_presets.md] SCENARIO: 28.5
async fn check_an_invalid_editor_open_reports_in_the_notice_and_keeps_focus(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    stub.failure_handle().set_preset_card_refused(true);
    open_prompt_presets_tab(&page).await;

    let cards = ".prompt-presets-panel .preset-section .preset-card";
    let cards_before = page
        .evaluate::<(), bool>(
            r#"() => {
                window.__cards = Array.from(document.querySelectorAll('.prompt-presets-panel .preset-section .preset-card'));
                return window.__cards.length > 1;
            }"#,
            None,
        )
        .await
        .unwrap();
    assert!(cards_before, "the fixture shows several preset cards");
    let edit = r#"button[hx-get="/fragment/prompt-presets/custom_ref/edit"]"#;
    page.locator(edit).await.focus().await.unwrap();
    page.locator(edit).await.click(None).await.unwrap();

    wait_for_error_disclosure(&page, NOTICE, "the refusal shows in the notice").await;
    let (visible, message, _) = read_error_disclosure(&page, NOTICE).await;
    assert!(visible, "the refusal shows in the notice");
    assert_eq!(
        message, "The form was not opened: Cannot edit default presets.",
        "the sentence names what was not done, with the engine's explanation"
    );
    let dialog_open = page
        .evaluate::<(), bool>("() => document.getElementById('edit-dialog').open", None)
        .await
        .unwrap();
    assert!(!dialog_open, "a refused editor opens no dialog");
    let same_cards = page
        .evaluate::<String, bool>(
            r#"(selector) => {
                const now = Array.from(document.querySelectorAll(selector));
                return now.length === window.__cards.length
                    && now.every((card, index) => card === window.__cards[index]);
            }"#,
            Some(&cards.to_string()),
        )
        .await
        .unwrap();
    assert!(same_cards, "every card stays in place, as the same element");
    assert!(
        wait_until_focused(&page, edit, Duration::from_secs(5)).await,
        "a refused editor must leave focus on its Edit control"
    );
}

// [docs/specs/browser_prompt_presets.md] SCENARIO: 28.6
async fn check_the_view_of_a_default_preset_opens_in_the_dialog_and_close_returns_focus(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    open_prompt_presets_tab(&page).await;

    let view = r#"button[hx-get="/fragment/prompt-presets/system_if_default/view"]"#;
    page.locator(view).await.focus().await.unwrap();
    page.locator(view).await.click(None).await.unwrap();

    wait_until_visible(&page, "#edit-dialog .view-form", Duration::from_secs(5)).await;
    let (open, writable_fields): (bool, usize) = page
        .evaluate::<(), (bool, usize)>(
            r#"() => [
                document.getElementById('edit-dialog').open,
                document.querySelectorAll('#edit-dialog textarea:not([disabled]), #edit-dialog input:not([readonly])').length,
            ]"#,
            None,
        )
        .await
        .unwrap();
    assert!(open, "the View opens the dialog");
    assert_eq!(writable_fields, 0, "the view form is read-only");

    page.locator("#edit-dialog [data-close-dialog]")
        .await
        .click(None)
        .await
        .unwrap();
    assert!(
        wait_until_focused(&page, view, Duration::from_secs(5)).await,
        "closing the dialog returns focus to the View control"
    );
}

#[tokio::test]
async fn run_prompt_preset_checks() {
    let mut runner = StubRunner::launch().await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_opening_a_preset_editor_moves_focus_straight_into_the_form,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_deleting_a_preset_card_keeps_keyboard_focus_on_a_card,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_an_invalid_editor_open_reports_in_the_notice_and_keeps_focus,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_the_view_of_a_default_preset_opens_in_the_dialog_and_close_returns_focus,
        )
        .await;
    runner.finish().await;
}
