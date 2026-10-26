//! Stub-browser tests for the rules that keep a draft out of a region: a load skips a control whose request is in flight, and no region of any tab holds a form that waits for a Save. Tagged against `docs/specs/browser_live_regions.md`.

use std::time::Duration;

use chronicler_engine::domain::model::change::ChangeKind;

use super::support::{
    hold_requests_ending_with, release_held_requests_and_wait_for_answers, StubRunner,
};
use super::*;

const WAIT: Duration = Duration::from_secs(5);
const QUANTIFIER_SELECT: &str = "#role-select-quantifier";

async fn open_settings(page: &playwright_rs::Page) {
    page.locator(r#"[data-tab="settings"]"#)
        .await
        .click(None)
        .await
        .unwrap();
    wait_until_visible(page, "#subtab-connections", WAIT).await;
}

async fn select_state(page: &playwright_rs::Page) -> (String, bool) {
    page.evaluate::<(), (String, bool)>(
        r#"() => {
            const select = document.getElementById('role-select-quantifier');
            return [select.value, select.classList.contains('htmx-request')];
        }"#,
        None,
    )
    .await
    .unwrap()
}

async fn server_value_of_quantifier(page: &playwright_rs::Page) -> String {
    page.evaluate::<(), String>(
        r#"async () => {
            const answer = await fetch('/fragment/settings', { cache: 'no-store' });
            const template = document.createElement('template');
            template.innerHTML = await answer.text();
            return template.content.querySelector('#role-select-quantifier option[selected]').value;
        }"#,
        None,
    )
    .await
    .unwrap()
}

async fn choose_the_other_connection_and_hold_its_save(page: &playwright_rs::Page) -> String {
    open_settings(page).await;
    hold_requests_ending_with(page, "/connections/set-quantifier").await;
    let before = server_value_of_quantifier(page).await;
    let other = if before == "openrouter-euryale" {
        "openrouter-gpt-4o-mini"
    } else {
        "openrouter-euryale"
    };
    page.locator(QUANTIFIER_SELECT)
        .await
        .select_option(other, None)
        .await
        .unwrap();
    let in_flight = wait_for_condition_async(WAIT, Duration::from_millis(25), || async {
        select_state(page).await.1
    })
    .await;
    assert!(
        in_flight,
        "the save is in flight, so the select shows htmx-request"
    );
    other.to_string()
}

async fn wait_for_a_settings_load_after(
    page: &playwright_rs::Page,
    changes: &StubChangesHandle,
    loads_before: usize,
    ends_in_state: &str,
) {
    let loaded = wait_for_condition_async(WAIT, Duration::from_millis(25), || async {
        changes.loads("/fragment/settings") > loads_before
            && page
                .evaluate::<String, bool>(
                    "(state) => document.querySelector('.settings-panel').dataset.liveState === state",
                    Some(&ends_in_state.to_string()),
                )
                .await
                .unwrap_or(false)
    })
    .await;
    assert!(
        loaded,
        "the Settings panel loads and ends in the {ends_in_state} state"
    );
}

// [docs/specs/browser_live_regions.md] SCENARIO: 44.24
async fn check_a_load_skips_a_select_whose_request_is_in_flight(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    let chosen = choose_the_other_connection_and_hold_its_save(&page).await;
    let server_value = server_value_of_quantifier(&page).await;
    assert_ne!(
        server_value, chosen,
        "the engine has not seen the save, so it still renders the old value"
    );

    // The hold of a region with a dirty field keeps the region from loading at
    // all while the save is in flight, so it is switched off: this check is about
    // what a load that does run does to a control whose request is in flight.
    page.evaluate::<(), ()>("() => { window.regionHoldsEdit = () => false; }", None)
        .await
        .unwrap();
    let loads_before = changes.loads("/fragment/settings");
    changes.bump(ChangeKind::SETTINGS);
    wait_for_a_settings_load_after(&page, &changes, loads_before, "due").await;

    let (shown, in_flight) = select_state(&page).await;
    assert!(in_flight, "the save is still in flight after the load");
    assert_eq!(
        shown, chosen,
        "a load of the panel must leave the select whose save is in flight on the player's choice"
    );

    release_held_requests_and_wait_for_answers(&page).await;
}

// [docs/specs/browser_live_regions.md] SCENARIO: 44.25
async fn check_after_the_save_the_next_load_sets_the_select_to_the_server_value(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    stub.save_handle().set_role_saves_keep_stored_value(true);
    let chosen = choose_the_other_connection_and_hold_its_save(&page).await;
    let server_value = server_value_of_quantifier(&page).await;
    assert_ne!(
        server_value, chosen,
        "the engine keeps its own value, so the next render differs from the choice"
    );
    let loads_before = changes.loads("/fragment/settings");

    release_held_requests_and_wait_for_answers(&page).await;

    let settled = wait_for_condition_async(WAIT, Duration::from_millis(25), || async {
        changes.loads("/fragment/settings") > loads_before && !select_state(&page).await.1
    })
    .await;
    assert!(
        settled,
        "the save ends, the request leaves the select, and the panel loads again"
    );
    let shown = wait_for_select_value(&page, &server_value).await;
    assert_eq!(
        shown, server_value,
        "after the save, the select shows what the engine renders, not the player's choice"
    );
}

// [docs/specs/browser_live_regions.md] SCENARIO: 44.27
async fn check_a_load_that_skipped_a_select_in_flight_leaves_the_region_due(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    let save = stub.save_handle();
    save.set_role_saves_keep_stored_value(true);
    save.hold_role_save_answers();
    open_settings(&page).await;
    let server_value = server_value_of_quantifier(&page).await;
    let chosen = if server_value == "openrouter-euryale" {
        "openrouter-gpt-4o-mini"
    } else {
        "openrouter-euryale"
    };
    // The hold of a region with a dirty field would keep the panel from loading
    // while the save is in flight: it is switched off, as in 44.24.
    page.evaluate::<(), ()>("() => { window.regionHoldsEdit = () => false; }", None)
        .await
        .unwrap();
    let loads_before = changes.loads("/fragment/settings");

    page.locator(QUANTIFIER_SELECT)
        .await
        .select_option(chosen, None)
        .await
        .unwrap();
    // The engine committed the save and moved its number, and the answer is held:
    // the panel loads while the select still has its request in flight.
    wait_for_a_settings_load_after(&page, &changes, loads_before, "due").await;
    let (shown, in_flight) = select_state(&page).await;
    assert!(
        in_flight,
        "the answer of the save is held, so it is in flight"
    );
    assert_eq!(shown, chosen, "the load skipped the select");

    save.release_role_save_answer();

    let shown = wait_for_select_value(&page, &server_value).await;
    let dbg = page.evaluate::<(), String>("() => document.querySelector('.settings-panel').dataset.liveState + ' ' + document.getElementById('role-select-quantifier').className", None).await.unwrap();
    eprintln!(
        "DEBUG {dbg} loads {} before {}",
        changes.loads("/fragment/settings"),
        loads_before
    );
    assert_eq!(
        shown, server_value,
        "after the save ended, the panel loaded again and set the select to the engine's value"
    );
    wait_for_a_settings_load_after(&page, &changes, loads_before + 1, "current").await;
}

async fn wait_for_select_value(page: &playwright_rs::Page, expected: &str) -> String {
    wait_for_condition_async(WAIT, Duration::from_millis(25), || async {
        select_state(page).await.0 == expected
    })
    .await;
    select_state(page).await.0
}

/// The tabs, each with the regions it shows.
const TABS: [&str; 6] = [
    "game",
    "settings",
    "prompt-presets",
    "worlds",
    "games",
    "llm-messages",
];

/// A form that waits for a Save and still sits in a region: the selector of its
/// root, why it is still there, and the stage of the plan that moves it into the
/// dialog. The list only shrinks, and it is empty after that stage.
struct DraftFormInRegion {
    selector: &'static str,
    reason: &'static str,
    moved_in_stage: u8,
}

const DRAFT_FORMS_STILL_IN_A_REGION: &[DraftFormInRegion] = &[
    DraftFormInRegion {
        selector: ".game-rename",
        reason: "the Rename form of a Game is still inside the Games panel",
        moved_in_stage: 3,
    },
    DraftFormInRegion {
        selector: ".new-game-add",
        reason: "the New Game form is still inside the Games panel",
        moved_in_stage: 3,
    },
    DraftFormInRegion {
        selector: ".preset-add",
        reason: "the Add forms of a preset category are still inside the Prompt Presets panel",
        moved_in_stage: 3,
    },
    DraftFormInRegion {
        selector: ".preset-card.edit-form",
        reason: "a preset card in edit still replaces its card inside the Prompt Presets panel",
        moved_in_stage: 3,
    },
    DraftFormInRegion {
        selector: "#world-editor",
        reason: "the World editor is still inside the Worlds panel",
        moved_in_stage: 3,
    },
];

// [docs/specs/browser_live_regions.md] SCENARIO: 44.26
async fn check_no_region_of_any_tab_holds_a_draft_control(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    for row in DRAFT_FORMS_STILL_IN_A_REGION {
        assert!(
            !row.reason.is_empty() && row.moved_in_stage > 0,
            "each allowed form names its reason and the stage that moves it: {}",
            row.selector
        );
    }
    for tab in TABS {
        page.locator(&format!(r#"[data-tab="{tab}"]"#))
            .await
            .click(None)
            .await
            .unwrap();
        wait_until_visible(&page, &format!("#{tab}-tab.active"), WAIT).await;
        let loaded = wait_for_condition_async(WAIT, Duration::from_millis(25), || async {
            page.evaluate::<String, bool>(
                r#"(tab) => {
                    const regions = document.querySelectorAll('#' + tab + '-tab [data-live]');
                    return regions.length > 0 && Array.from(regions).every((r) => r.dataset.liveState === 'current');
                }"#,
                Some(&tab.to_string()),
            )
            .await
            .unwrap_or(false)
        })
        .await;
        assert!(loaded, "the regions of the {tab} tab load");

        let arguments = serde_json::json!({
            "tab": tab,
            "allowed": DRAFT_FORMS_STILL_IN_A_REGION
                .iter()
                .map(|row| row.selector)
                .collect::<Vec<_>>(),
        });
        let drafts: Vec<String> = page
            .evaluate::<serde_json::Value, Vec<String>>(
                r#"({ tab, allowed }) => {
                    const controls = 'input:not([type=hidden]):not([type=button]):not([type=submit]), textarea, select';
                    const savesOnChange = (control) =>
                        control.matches('[hx-trigger="change"]') || !!control.closest('form[hx-trigger="change"]');
                    const allow = allowed.join(', ');
                    return Array.from(document.querySelectorAll('#' + tab + '-tab [data-live]'))
                        .flatMap((region) => Array.from(region.querySelectorAll(controls)))
                        .filter((control) => !savesOnChange(control))
                        .filter((control) => !control.disabled && !control.readOnly)
                        .filter((control) => !control.closest(allow))
                        .map((control) => control.outerHTML.slice(0, 120));
                }"#,
                Some(&arguments),
            )
            .await
            .unwrap();
        assert!(
            drafts.is_empty(),
            "the {tab} tab holds draft controls in a region, so a load could erase a draft: {drafts:?}"
        );
    }
}

#[tokio::test]
async fn run_draft_checks() {
    let mut runner = StubRunner::launch().await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_load_skips_a_select_whose_request_is_in_flight,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_after_the_save_the_next_load_sets_the_select_to_the_server_value,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_load_that_skipped_a_select_in_flight_leaves_the_region_due,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_no_region_of_any_tab_holds_a_draft_control,
        )
        .await;
    runner.finish().await;
}
