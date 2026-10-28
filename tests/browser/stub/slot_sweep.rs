//! Stub-browser sweep of the places that hold a report: each form with typed input renders one slot, and no other element but the report container of the notice holds a slot. Tagged against `docs/specs/browser_screen.md`.

use std::time::Duration;

use super::support::{dialog_is_open, eval_string, open_section, wait_until_dialog_closed, StubRunner};
use super::*;

const WAIT: Duration = Duration::from_secs(5);

/// The forms that DS3 lists: each form in the dialog, the Action input and the
/// Autoplay cycles, and each `.edit-actions` group of a message edit. The rule:
/// each of them holds one slot as a direct child, and a slot is a direct child of
/// one of them, or the report container of the notice. The slot of the story-log
/// strip is allowed until the story-log controls become htmx controls.
const SWEEP: &str = r#"() => {
    const places = '#edit-dialog form, #command-form, #autoplay-form, .edit-actions';
    const problems = [];
    let examined = 0;
    for (const place of document.querySelectorAll(places)) {
        examined += 1;
        const slots = place.querySelectorAll(':scope > [data-error-slot]').length;
        if (slots !== 1) {
            problems.push('<' + place.tagName.toLowerCase() + ' ' + (place.id || place.className) + '> holds ' + slots + ' direct slots');
        }
    }
    for (const slot of document.querySelectorAll('[data-error-slot]')) {
        const parent = slot.parentElement;
        const inPlace = !!parent && parent.matches(places);
        const report = !!parent && parent.id === 'screen-error' && slot.hasAttribute('data-report');
        const storyLog = slot.dataset.errorSlot === 'story-log';
        if (!inPlace && !report && !storyLog) {
            const holder = parent ? (parent.id || parent.className || parent.tagName) : 'nothing';
            problems.push('the slot "' + slot.dataset.errorSlot + '" sits in ' + holder + ', not in a form of the list');
        }
    }
    const dialogForms = document.querySelectorAll('#edit-dialog form').length;
    return JSON.stringify({ examined, dialogForms, problems });
}"#;

const OPENERS: &str = r##"[hx-target="#edit-dialog-body"]"##;

#[derive(serde::Deserialize)]
struct Sweep {
    examined: usize,
    #[serde(rename = "dialogForms")]
    dialog_forms: usize,
    problems: Vec<String>,
}

async fn sweep(page: &playwright_rs::Page, problems: &mut Vec<String>) -> (usize, usize) {
    let found: Sweep = serde_json::from_str(&eval_string(page, SWEEP).await).unwrap();
    problems.extend(found.problems);
    (found.examined, found.dialog_forms)
}

/// The dialogs that hold no form: the View of a preset, the record of an LLM message
/// and the Test result of a connection.
fn opens_a_dialog_without_a_form(opener: &str) -> bool {
    opener.ends_with("/view") || opener.contains("/llm-messages/") || opener.ends_with("/test")
}

async fn opener_count(page: &playwright_rs::Page) -> usize {
    eval_string(
        page,
        &format!("() => String(document.querySelectorAll('[data-part] {OPENERS}').length)"),
    )
    .await
    .parse()
    .unwrap()
}

async fn open_dialog_through(page: &playwright_rs::Page, index: usize) -> String {
    let described = eval_string(
        page,
        &format!(
            r#"() => {{
                const opener = document.querySelectorAll('[data-part] {OPENERS}')[{index}];
                const verb = opener.hasAttribute('hx-get') ? 'get ' + opener.getAttribute('hx-get') : 'post ' + opener.getAttribute('hx-post');
                opener.click();
                return verb;
            }}"#
        ),
    )
    .await;
    let opened = wait_for_condition_async(WAIT, Duration::from_millis(25), || async {
        dialog_is_open(page).await
    })
    .await;
    assert!(opened, "the opener `{described}` opens the dialog");
    wait_for_condition_async(WAIT, Duration::from_millis(25), || async {
        eval_string(
            page,
            "() => String(document.getElementById('edit-dialog-body').childElementCount > 0)",
        )
        .await
            == "true"
    })
    .await;
    described
}

// [docs/specs/browser_screen.md] SCENARIO: 44.45
async fn check_each_form_with_typed_input_renders_one_slot_and_no_other_element_holds_one(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    let mut problems = Vec::new();
    let mut examined = 0;
    let mut dialogs = 0;
    let mut dialog_forms = 0;
    let mut dialogs_that_lack_a_form = Vec::new();
    for (tab, part) in [
        ("game", "#story-log"),
        ("settings", ".settings-panel"),
        ("prompt-presets", ".prompt-presets-panel"),
        ("worlds", ".worlds-panel"),
        ("games", ".games-panel"),
        ("llm-messages", ".llm-messages-panel"),
    ] {
        open_section(&page, tab, part).await;
        examined += sweep(&page, &mut problems).await.0;
        for index in 0..opener_count(&page).await {
            let opener = open_dialog_through(&page, index).await;
            let (swept, forms) = sweep(&page, &mut problems).await;
            examined += swept;
            dialog_forms += forms;
            dialogs += 1;
            if forms == 0 && !opens_a_dialog_without_a_form(&opener) {
                dialogs_that_lack_a_form.push(opener);
            }
            page.locator("#edit-dialog [data-close-dialog]")
                .await
                .first()
                .click(None)
                .await
                .unwrap();
            wait_until_dialog_closed(&page, "Cancel closes the dialog").await;
        }
    }
    assert!(
        dialogs_that_lack_a_form.is_empty(),
        "each opener of a dialog with a form reaches a form that the sweep examines, but these did not: {dialogs_that_lack_a_form:?} ({dialog_forms} dialog forms in {dialogs} dialogs, {examined} forms examined)"
    );
    problems.sort();
    problems.dedup();
    assert!(
        problems.is_empty(),
        "the places of a report break the rule:\n{}",
        problems.join("\n")
    );
}

#[tokio::test]
async fn run_slot_sweep() {
    let mut runner = StubRunner::launch().await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_each_form_with_typed_input_renders_one_slot_and_no_other_element_holds_one,
        )
        .await;
    runner.finish().await;
}
