//! Stub-browser tests for the rules that keep a draft out of a part: a load skips a control whose request is in flight, and no part of any tab holds a form that waits for a Save. Tagged against `docs/specs/browser_screen.md`.

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
) {
    // A load records the stamp of what it rendered, also when it skipped a control in
    // flight, so the stamp of the page equals the stamp of the stub after it.
    let loaded = wait_for_condition_async(WAIT, Duration::from_millis(25), || async {
        changes.loads("/fragment/settings") > loads_before
            && page
                .evaluate::<(), bool>(
                    r#"async () => {
                        const answer = await (await fetch('/changes', { cache: 'no-store' })).json();
                        return document.documentElement.dataset.revision === answer.server + ':' + answer.revision;
                    }"#,
                    None,
                )
                .await
                .unwrap_or(false)
    })
    .await;
    assert!(
        loaded,
        "the Settings panel loads and the stamp of the page equals the stamp of the engine"
    );
}

// [docs/specs/browser_screen.md] SCENARIO: 44.24
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

    let loads_before = changes.loads("/fragment/settings");
    changes.bump(ChangeKind::SETTINGS);
    wait_for_a_settings_load_after(&page, &changes, loads_before).await;

    let (shown, in_flight) = select_state(&page).await;
    assert!(in_flight, "the save is still in flight after the load");
    assert_eq!(
        shown, chosen,
        "a load of the panel must leave the select whose save is in flight on the player's choice"
    );

    release_held_requests_and_wait_for_answers(&page).await;
}

// [docs/specs/browser_screen.md] SCENARIO: 44.25
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

// [docs/specs/browser_screen.md] SCENARIO: 44.27
async fn check_a_load_that_skipped_a_select_in_flight_records_the_stamp_and_the_end_of_the_request_loads_again(
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
    let loads_before = changes.loads("/fragment/settings");

    page.locator(QUANTIFIER_SELECT)
        .await
        .select_option(chosen, None)
        .await
        .unwrap();
    // The engine committed the save and moved its number, and the answer is held:
    // the panel loads while the select still has its request in flight.
    wait_for_a_settings_load_after(&page, &changes, loads_before).await;
    let (shown, in_flight) = select_state(&page).await;
    assert!(
        in_flight,
        "the answer of the save is held, so it is in flight"
    );
    assert_eq!(shown, chosen, "the load skipped the select");

    save.release_role_save_answer();

    let shown = wait_for_select_value(&page, &server_value).await;
    assert_eq!(
        shown, server_value,
        "after the save ended, the panel loaded again and set the select to the engine's value"
    );
    wait_for_a_settings_load_after(&page, &changes, loads_before + 1).await;
}

async fn wait_for_select_value(page: &playwright_rs::Page, expected: &str) -> String {
    wait_for_condition_async(WAIT, Duration::from_millis(25), || async {
        select_state(page).await.0 == expected
    })
    .await;
    select_state(page).await.0
}

/// The tabs.
const TABS: [&str; 6] = [
    "game",
    "settings",
    "prompt-presets",
    "worlds",
    "games",
    "llm-messages",
];

// [docs/specs/browser_screen.md] SCENARIO: 44.26
async fn check_no_part_of_any_tab_holds_a_draft_control(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
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
                    const parts = document.querySelectorAll('#' + tab + '-tab [data-part]');
                    return parts.length > 0 && Array.from(parts).every((part) => part.childNodes.length > 0);
                }"#,
                Some(&tab.to_string()),
            )
            .await
            .unwrap_or(false)
        })
        .await;
        assert!(loaded, "the parts of the {tab} tab load");

        let drafts: Vec<String> = page
            .evaluate::<String, Vec<String>>(
                r#"(tab) => {
                    const controls = 'input:not([type=hidden]):not([type=button]):not([type=submit]), textarea, select';
                    const savesOnChange = (control) =>
                        control.matches('[hx-trigger="change"]') || !!control.closest('form[hx-trigger="change"]');
                    return Array.from(document.querySelectorAll('#' + tab + '-tab [data-part]'))
                        .flatMap((part) => Array.from(part.querySelectorAll(controls)))
                        .filter((control) => !savesOnChange(control))
                        .filter((control) => !control.disabled && !control.readOnly)
                        .map((control) => control.outerHTML.slice(0, 120));
                }"#,
                Some(&tab.to_string()),
            )
            .await
            .unwrap();
        assert!(
            drafts.is_empty(),
            "the {tab} tab holds draft controls in a part, so a load could erase a draft: {drafts:?}"
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
            check_a_load_that_skipped_a_select_in_flight_records_the_stamp_and_the_end_of_the_request_loads_again,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_no_part_of_any_tab_holds_a_draft_control,
        )
        .await;
    runner.finish().await;
}
