//! Stub-browser sweeps of the server's markup: each form with typed input renders one slot, no other element but the report container of the notice holds a slot, and each control that opens the dialog renders an id. Tagged against `docs/specs/browser_screen.md`.

use std::time::Duration;

use super::support::{dialog_is_open, eval_string, open_section, wait_until_dialog_closed, StubRunner};
use super::*;

const WAIT: Duration = Duration::from_secs(5);

/// Each place holds one slot as a direct child, and a slot is a direct child of one of
/// them, or the report container of the notice.
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
        const report = !!parent && parent.id === 'notice' && slot.hasAttribute('data-report');
        if (!inPlace && !report) {
            const holder = parent ? (parent.id || parent.className || parent.tagName) : 'nothing';
            problems.push('the slot "' + slot.dataset.errorSlot + '" sits in ' + holder + ', not in a form of the list');
        }
    }
    const dialogForms = document.querySelectorAll('#edit-dialog form').length;
    const editActions = document.querySelectorAll('.edit-actions').length;
    return JSON.stringify({ examined, dialogForms, editActions, problems });
}"#;

const OPENERS: &str = r##"[hx-target="#edit-dialog-body"]"##;

#[derive(serde::Deserialize)]
struct Sweep {
    examined: usize,
    #[serde(rename = "dialogForms")]
    dialog_forms: usize,
    #[serde(rename = "editActions")]
    edit_actions: usize,
    problems: Vec<String>,
}

async fn sweep(page: &playwright_rs::Page, problems: &mut Vec<String>) -> (usize, usize, usize) {
    let found: Sweep = serde_json::from_str(&eval_string(page, SWEEP).await).unwrap();
    problems.extend(found.problems);
    (found.examined, found.dialog_forms, found.edit_actions)
}

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
    let mut edit_groups = 0;
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
        let (swept, _, groups) = sweep(&page, &mut problems).await;
        examined += swept;
        edit_groups += groups;
        if tab == "game" {
            page.locator(".log-entry.narration .edit-btn")
                .await
                .click(None)
                .await
                .unwrap();
            wait_until_visible(&page, "#edit-textarea", WAIT).await;
            examined += sweep(&page, &mut problems).await.0;
            page.locator("[data-editing] .cancel-btn")
                .await
                .click(None)
                .await
                .unwrap();
        }
        for index in 0..opener_count(&page).await {
            let opener = open_dialog_through(&page, index).await;
            let (swept, forms, _) = sweep(&page, &mut problems).await;
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
    assert!(
        edit_groups > 0,
        "the Game tab renders the edit group of each story-log entry, and the sweep examines them"
    );
    problems.sort();
    problems.dedup();
    assert!(
        problems.is_empty(),
        "the places of a report break the rule:\n{}",
        problems.join("\n")
    );
}

const OPENER_IDS: &str = r##"() => JSON.stringify(
    Array.from(document.querySelectorAll('[data-part] [hx-target="#edit-dialog-body"]'))
        .map((opener) => ({ request: opener.getAttribute('hx-get') || opener.getAttribute('hx-post'), id: opener.id }))
)"##;

#[derive(serde::Deserialize)]
struct OpenerId {
    request: String,
    id: String,
}

// [docs/specs/browser_screen.md] SCENARIO: 44.46
async fn check_each_control_that_opens_the_dialog_renders_a_unique_id(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    let mut seen = Vec::new();
    let mut problems = Vec::new();
    for (tab, part) in [
        ("settings", ".settings-panel"),
        ("prompt-presets", ".prompt-presets-panel"),
        ("worlds", ".worlds-panel"),
        ("games", ".games-panel"),
        ("llm-messages", ".llm-messages-panel"),
    ] {
        open_section(&page, tab, part).await;
        let openers: Vec<OpenerId> =
            serde_json::from_str(&eval_string(&page, OPENER_IDS).await).unwrap();
        assert!(
            !openers.is_empty(),
            "the {tab} tab renders a control that opens the dialog"
        );
        for opener in openers {
            if opener.id.is_empty() {
                problems.push(format!(
                    "the opener `{}` of the {tab} tab has no id",
                    opener.request
                ));
            } else if seen.contains(&opener.id) {
                problems.push(format!(
                    "the id `{}` of the {tab} tab names two openers",
                    opener.id
                ));
            }
            seen.push(opener.id);
        }
    }
    assert!(
        problems.is_empty(),
        "each opener renders its own id ({} openers examined):\n{}",
        seen.len(),
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
    runner
        .run(
            StubActionOutcome::Pending,
            check_each_control_that_opens_the_dialog_renders_a_unique_id,
        )
        .await;
    runner.finish().await;
}
