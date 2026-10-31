//! Stub-browser tests for failed form requests. The form keeps its place. An invalid answer shows in its slot. A refusal or a failure shows in the notice. Tagged against `docs/specs/browser_dashboard.md`.

use std::time::Duration;

use super::*;
use super::support::{eval_string, open_settings, StubRunner, NOTICE};

async fn connection_form_node_is_still_present(page: &playwright_rs::Page) -> bool {
    page.evaluate::<(), bool>(
        r#"() => {
            const now = document.querySelector('#edit-dialog .connection-form');
            return !!now && now === window.__connectionFormBefore;
        }"#,
        None,
    )
    .await
    .unwrap_or(false)
}

async fn stash_connection_form_node(page: &playwright_rs::Page) -> bool {
    page.evaluate::<(), bool>(
        r#"() => {
            window.__connectionFormBefore = document.querySelector('#edit-dialog .connection-form');
            return !!window.__connectionFormBefore;
        }"#,
        None,
    )
    .await
    .unwrap()
}

async fn submit_connection_form(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>(
        r#"() => document.querySelector('#edit-dialog .connection-form form').requestSubmit()"#,
        None,
    )
    .await
    .unwrap();
}

async fn wait_for_inline_slot(page: &playwright_rs::Page, slot: &str) -> bool {
    wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(50),
        || async { read_error_disclosure(page, slot).await.0 },
    )
    .await
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.29
async fn check_failed_connection_add_keeps_the_form_and_renders_inline(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    open_settings(&page).await;

    click_and_settle(&page, ".connection-add button", "#edit-dialog-body").await;
    wait_until_visible(
        &page,
        "#edit-dialog .connection-form",
        Duration::from_secs(5),
    )
    .await;
    assert!(
        stash_connection_form_node(&page).await,
        "the Add form should be loaded"
    );

    submit_connection_form(&page).await;

    let slot = r#"#edit-dialog .connection-form [data-error-slot="connection-form"]"#;
    assert!(
        wait_for_inline_slot(&page, slot).await,
        "an invalid connection add should render into the form's inline slot"
    );
    let (visible, message, _) = read_error_disclosure(&page, slot).await;
    assert!(visible, "the inline slot should be shown");
    assert_eq!(
        message, "The Connection was not saved: Unknown LLM backend 'bogus_provider'.",
        "the short line is the sentence that the server wrote, with the engine's explanation"
    );
    assert!(
        connection_form_node_is_still_present(&page).await,
        "an invalid connection add must leave the form in place in the dialog"
    );
    assert!(
        !read_error_disclosure(&page, NOTICE).await.0,
        "an invalid answer shows in the slot of the form, and the notice stays empty"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.35
async fn check_failed_connection_edit_keeps_the_form_and_renders_inline(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    open_settings(&page).await;

    click_and_settle(
        &page,
        ".connection-list .connection-row:first-child button:has-text('Edit')",
        "#edit-dialog-body",
    )
    .await;
    wait_until_visible(
        &page,
        "#edit-dialog .connection-form",
        Duration::from_secs(5),
    )
    .await;
    assert!(
        stash_connection_form_node(&page).await,
        "the Edit form should be loaded"
    );

    submit_connection_form(&page).await;

    let slot = r#"#edit-dialog .connection-form [data-error-slot="connection-form"]"#;
    assert!(
        wait_for_inline_slot(&page, slot).await,
        "an invalid connection edit should render into the form's inline slot"
    );
    let (visible, message, _) = read_error_disclosure(&page, slot).await;
    assert!(visible, "the inline slot should be shown");
    assert_eq!(
        message, "The Connection was not saved: Unknown LLM backend 'bogus_provider'.",
        "the short line is the sentence that the server wrote, with the engine's explanation"
    );
    assert!(
        connection_form_node_is_still_present(&page).await,
        "an invalid connection edit must leave the form in place in the dialog"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.30
async fn check_failed_preset_add_keeps_the_dialog_and_renders_inline(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    open_prompt_presets_tab(&page).await;

    click_and_settle(
        &page,
        r#"button[hx-get="/fragment/prompt-presets/new?type=system"]"#,
        "#edit-dialog-body",
    )
    .await;
    wait_until_visible(
        &page,
        "#edit-dialog .preset-add-form",
        Duration::from_secs(5),
    )
    .await;

    page.evaluate::<(), ()>(
        r#"() => {
                window.__presetFormBefore = document.querySelector('#edit-dialog .preset-add-form form');
                const form = window.__presetFormBefore;
                form.querySelector('input[name="name"]').value = 'Bad Type';
                form.requestSubmit();
            }"#,
        None,
    )
    .await
    .unwrap();

    let slot = r#"#edit-dialog form [data-error-slot="preset-add-system"]"#;
    assert!(
        wait_for_inline_slot(&page, slot).await,
        "an invalid preset add should render into the form's inline slot"
    );
    let (visible, message, _) = read_error_disclosure(&page, slot).await;
    assert!(visible, "the inline slot should be shown");
    assert!(
        message.contains("Invalid preset type"),
        "the sentence carries the engine's explanation, got {message:?}"
    );

    let kept = page
        .evaluate::<(), bool>(
            r#"() => {
                    const now = document.querySelector('#edit-dialog .preset-add-form form');
                    return document.getElementById('edit-dialog').open
                        && now === window.__presetFormBefore
                        && now.querySelector('input[name="name"]').value === 'Bad Type';
                }"#,
            None,
        )
        .await
        .unwrap();
    assert!(
        kept,
        "an invalid preset add must leave the dialog open with the same form and its typed text"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.31
async fn check_failed_preset_edit_keeps_the_dialog_and_reports_in_the_notice(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    open_prompt_presets_tab(&page).await;

    let edit_selector =
        r#".preset-card:has(.card-title:text-is("My Custom Prompt")) button:has-text('Edit')"#;
    click_and_settle(&page, edit_selector, "#edit-dialog-body").await;
    wait_until_visible(&page, "#edit-dialog .edit-form", Duration::from_secs(5)).await;

    let stashed = page
        .evaluate::<(), bool>(
            r#"() => {
                    window.__presetFormBefore = document.querySelector('#edit-dialog .edit-form form');
                    return !!window.__presetFormBefore;
                }"#,
            None,
        )
        .await
        .unwrap();
    assert!(stashed, "the preset edit form should be loaded");

    page.evaluate::<(), ()>(
        r#"() => document.querySelector('#edit-dialog .edit-form form').requestSubmit()"#,
        None,
    )
    .await
    .unwrap();

    wait_for_error_disclosure(&page, NOTICE, "a failed preset edit reports in the notice").await;
    let (visible, message, raw) = read_error_disclosure(&page, NOTICE).await;
    assert!(visible, "the notice should be shown");
    assert_eq!(message, "The preset was not saved.");
    assert!(
        raw.contains("preset save failure"),
        "the raw failure belongs in the disclosure, got {raw:?}"
    );
    assert!(
        !read_error_disclosure(
            &page,
            r#"#edit-dialog [data-error-slot="preset-edit-custom_ref"]"#
        )
        .await
        .0,
        "a failure shows in the notice, and the form's slot stays empty"
    );

    let kept = page
        .evaluate::<(), bool>(
            r#"() => {
                    const now = document.querySelector('#edit-dialog .edit-form form');
                    return document.getElementById('edit-dialog').open
                        && now === window.__presetFormBefore;
                }"#,
            None,
        )
        .await
        .unwrap();
    assert!(
        kept,
        "a failed preset edit must leave the dialog open with the same form"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.32
async fn check_an_invalid_preset_delete_keeps_the_card_and_reports_in_the_notice(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    open_prompt_presets_tab(&page).await;

    let stashed = page
            .evaluate::<(), bool>(
                r#"() => {
                    const card = Array.from(document.querySelectorAll('.preset-card')).find(
                        (c) => c.querySelector('.card-title') &&
                            c.querySelector('.card-title').textContent.trim() === 'My Custom Prompt',
                    );
                    window.__presetCardBefore = card || null;
                    window.confirm = () => true;
                    return !!card;
                }"#,
                None,
            )
            .await
            .unwrap();
    assert!(stashed, "the non-default preset card should be loaded");

    page.locator(
        r#".preset-card:has(.card-title:text-is("My Custom Prompt")) button:has-text('Delete')"#,
    )
    .await
    .click(None)
    .await
    .unwrap();

    wait_for_error_disclosure(
        &page,
        NOTICE,
        "an invalid preset delete reports in the notice",
    )
    .await;
    let (visible, message, _) = read_error_disclosure(&page, NOTICE).await;
    assert!(visible, "the notice should be shown");
    assert_eq!(
        message,
        "The preset was not deleted: Preset is a mode default; change the default before deleting."
    );

    let kept = page
            .evaluate::<(), bool>(
                r#"() => {
                    const now = Array.from(document.querySelectorAll('.preset-card')).find(
                        (c) => c.querySelector('.card-title') &&
                            c.querySelector('.card-title').textContent.trim() === 'My Custom Prompt',
                    );
                    return !!now && now === window.__presetCardBefore;
                }"#,
                None,
            )
            .await
            .unwrap();
    assert!(
        kept,
        "an invalid preset delete must leave the card in place"
    );
}

// A one-line sentence is the whole report: it has no Details, so the sentence is the only
// place that holds its end. It wraps onto more lines instead of being cut.
const LONG_SENTENCE: &str = "The World \"Typed Name\" was not created: Invalid scenarios JSON: invalid type: map, expected a sequence at line 1 column 0, so the Scenarios field holds text that no World can read and the player must see all of this sentence to fix it.";

/// Whether the sentence of the report in `place` is drawn whole: no cut at the right edge, no
/// ellipsis, and more than one line, because the sentence is longer than the place is wide.
async fn sentence_reads_in_full(page: &playwright_rs::Page, place: &str) -> String {
    eval_string(
        page,
        &format!(
            r#"() => {{
                const message = document.querySelector('{place} .error-disclosure-message');
                const text = document.querySelector('{place} .error-disclosure-text');
                if (!message || !text) return 'no report in {place}';
                const style = getComputedStyle(message);
                const lineHeight = parseFloat(getComputedStyle(text).lineHeight) || 16;
                const cut = message.scrollWidth > message.clientWidth
                    || text.scrollWidth > text.clientWidth
                    || message.scrollHeight > message.clientHeight + 1;
                if (cut) return 'cut: scrollWidth ' + message.scrollWidth + ' clientWidth ' + message.clientWidth;
                if (style.textOverflow === 'ellipsis') return 'ellipsis';
                if (text.getBoundingClientRect().height < lineHeight * 1.5) return 'one line: ' + text.getBoundingClientRect().height;
                return 'ok';
            }}"#
        ),
    )
    .await
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.80
async fn check_a_long_sentence_without_details_reads_in_full(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    open_settings(&page).await;
    click_and_settle(&page, ".connection-add button", "#edit-dialog-body").await;
    wait_until_visible(
        &page,
        "#edit-dialog .connection-form",
        Duration::from_secs(5),
    )
    .await;

    eval_string(
        &page,
        &format!(
            r#"() => {{
                const slot = document.querySelector('#edit-dialog .connection-form [data-error-slot="connection-form"]');
                renderInlineError(slot, {LONG_SENTENCE:?}, '');
                showNotice({LONG_SENTENCE:?}, '', 'request');
                return '';
            }}"#
        ),
    )
    .await;

    for width in [1280, 1024] {
        page.set_viewport_size(playwright_rs::Viewport { width, height: 800 })
            .await
            .unwrap();
        for place in [
            r#"#edit-dialog .connection-form [data-error-slot="connection-form"]"#,
            NOTICE,
        ] {
            let outcome = wait_for_condition_async(
                Duration::from_secs(2),
                Duration::from_millis(50),
                || async { sentence_reads_in_full(&page, place).await == "ok" },
            )
            .await;
            assert!(
                outcome,
                "at {width} px the sentence in {place} reads in full, got {}",
                sentence_reads_in_full(&page, place).await
            );
        }
    }
}

#[tokio::test]
async fn run_form_failures_checks() {
    let mut runner = StubRunner::launch().await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_failed_connection_add_keeps_the_form_and_renders_inline,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_failed_connection_edit_keeps_the_form_and_renders_inline,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_failed_preset_add_keeps_the_dialog_and_renders_inline,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_failed_preset_edit_keeps_the_dialog_and_reports_in_the_notice,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_an_invalid_preset_delete_keeps_the_card_and_reports_in_the_notice,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_long_sentence_without_details_reads_in_full,
        )
        .await;
    runner.finish().await;
}
