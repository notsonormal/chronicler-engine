//! Stub-browser tests for the Prompt Presets panel: the keyboard-focus landing after a preset card delete and when an editor opens. Tagged against `docs/specs/browser_prompt_presets.md`.

use std::time::Duration;

use super::support::{StubRunner, accept_delete_confirmation, wait_for_count, wait_until_focused};
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
        "opening the editor must land focus in the new card's Name field"
    );
    assert_eq!(
        focus_landings(&page).await,
        vec![
            "/fragment/prompt-presets/custom_ref/edit".to_string(),
            "edit-name-custom_ref".to_string(),
        ],
        "the editor must claim focus once, with no neighbouring card in between"
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

// A card's Edit load refused with a 200 whose body stands in for the card
// (a default preset) leaves the message where the card was, so focus lands on it.
// [docs/specs/browser_prompt_presets.md] SCENARIO: 28.5
async fn check_a_refused_card_load_lands_focus_on_its_message(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    stub.failure_handle().set_preset_card_refused(true);
    open_prompt_presets_tab(&page).await;

    let cards = ".prompt-presets-panel .preset-section .preset-card";
    let cards_before = page
        .query_selector_all(cards)
        .await
        .expect("count the preset cards the fixture shows")
        .len();
    let refusal = ".prompt-presets-panel .preset-section .error-message";
    let edit = r#"button[hx-get="/fragment/prompt-presets/custom_ref/edit"]"#;
    page.locator(edit).await.focus().await.unwrap();
    page.locator(edit).await.click(None).await.unwrap();

    wait_until_visible(&page, refusal, Duration::from_secs(5)).await;
    assert_eq!(
        page.query_selector_all(cards)
            .await
            .unwrap_or_default()
            .len(),
        cards_before - 1,
        "the refusal must replace the card it could not open, not remove it"
    );
    assert_eq!(
        page.query_selector_all(r#"button[hx-get="/fragment/prompt-presets/custom_two/edit"]"#)
            .await
            .unwrap_or_default()
            .len(),
        1,
        "the cards beside the refusal keep their controls"
    );
    assert!(
        wait_until_focused(&page, refusal, Duration::from_secs(5)).await,
        "a refused card load must land focus on the refusal message, not on a neighbouring card"
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
            check_a_refused_card_load_lands_focus_on_its_message,
        )
        .await;
    runner.finish().await;
}
