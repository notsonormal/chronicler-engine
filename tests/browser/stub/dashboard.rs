//! Stub-browser tests for dashboard chrome: the action-area state machine, the tab bar, the landmarks and the status display. Tagged against `docs/specs/browser_dashboard.md`.

use std::time::Duration;

use chronicler_engine::domain::model::change::ChangeKind;
use chronicler_engine::domain::model::state::generation_status::GenerationFailureKind;

use chronicler_engine::domain::model::state::generation_status::Step;

use super::*;
use super::support::{
    StubRunner, accept_delete_confirmation, active_element_is, ask_once_without_waiting,
    assert_busy_refusal, delay_screen_answers, hold_requests_ending_with, hold_screen_loads,
    read_banner, read_command_input, release_held_requests_and_wait_for_answers,
    release_screen_answers, release_screen_loads, wait_for_held_screen_load,
};

async fn stash_action_area_nodes(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>(
        r#"(() => {
            window.__formBefore = document.getElementById('command-form');
            window.__statusBefore = document.getElementById('status-display');
        })()"#,
        None,
    )
    .await
    .unwrap();
}

/// Compares node identity: a swapped-in form or status keeps its id.
async fn action_area_nodes_unchanged(page: &playwright_rs::Page) -> (bool, bool) {
    page.evaluate::<(), (bool, bool)>(
        r#"(() => {
            const form = document.getElementById('command-form');
            const status = document.getElementById('status-display');
            return [
                !!form && form === window.__formBefore,
                !!status && status === window.__statusBefore,
            ];
        })()"#,
        None,
    )
    .await
    .unwrap()
}

async fn submit_intercepted_command(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>(
        r#"(() => {
            const input = document.querySelector('#command-form input[name="command"]');
            input.value = 'look at the casle';
            document.querySelector('#submit-btn').click();
        })()"#,
        None,
    )
    .await
    .unwrap();

    wait_until_visible(page, ".text-check-preview", Duration::from_secs(5)).await;
}

/// The player can still submit the form while the primary button is disabled mid-generation.
async fn submit_intercepted_command_direct(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>(
        r#"(() => {
            const input = document.querySelector('#command-form input[name="command"]');
            input.value = 'look at the casle';
            document.getElementById('command-form').requestSubmit();
        })()"#,
        None,
    )
    .await
    .unwrap();

    wait_until_visible(page, ".text-check-preview", Duration::from_secs(5)).await;
}

async fn confirm_text_check_preview(page: &playwright_rs::Page) {
    page.locator(".text-check-preview .btn-original")
        .await
        .click(None)
        .await
        .unwrap();

    wait_until_hidden(page, ".text-check-preview", Duration::from_secs(5)).await;
    wait_for_status_busy(page).await;
}

async fn read_submit_button(page: &playwright_rs::Page) -> (bool, String) {
    page.evaluate::<(), (bool, String)>(
        r#"(() => {
            const btn = document.getElementById('submit-btn');
            if (!btn) return [false, ''];
            return [btn.disabled, btn.textContent.trim()];
        })()"#,
        None,
    )
    .await
    .unwrap()
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.9
async fn check_primary_button_locks_and_unlocks_after_confirm(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let status = stub.status_handle();
    // Keep the stub generating so no load unlocks the button before the assertion reads it.
    status.set(StubStatus::Step(Step::Narrating));
    stash_action_area_nodes(&page).await;

    submit_intercepted_command(&page).await;
    confirm_text_check_preview(&page).await;

    let (disabled, label) = read_submit_button(&page).await;
    assert!(
        disabled,
        "Send should lock while the turn runs (label {label:?})"
    );
    assert!(
        !label.contains("Stop"),
        "the locked button must not claim a Stop action, got {label:?}"
    );
    assert!(
        label.contains("Generating"),
        "the locked button should read as a generating indicator, got {label:?}"
    );

    let (form_same, status_same) = action_area_nodes_unchanged(&page).await;
    assert!(form_same, "confirming the preview replaced #command-form");
    assert!(
        status_same,
        "confirming the preview replaced #status-display"
    );

    status.set(StubStatus::Idle);
    load_screen_now(&page).await;
    wait_for_status_ready(&page).await;
    let (disabled, label) = read_submit_button(&page).await;
    assert!(
        !disabled,
        "Send should unlock once the status returns to Ready"
    );
    assert!(
        label.contains("Send"),
        "unlocked button should read Send, got {label:?}"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.10
async fn check_status_error_shows_in_the_status_display_after_confirm(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let status = stub.status_handle();
    submit_intercepted_command(&page).await;
    confirm_text_check_preview(&page).await;

    status.set(StubStatus::Error(
        GenerationFailureKind::Other,
        "narration failed".to_string(),
    ));
    load_screen_now(&page).await;
    assert!(
        wait_for_condition_async(
            Duration::from_secs(8),
            Duration::from_millis(100),
            || async { read_error_disclosure(&page, "#status-display").await.0 },
        )
        .await,
        "the status error should render in the status display after confirming a preview"
    );

    let (_, message, raw) = read_error_disclosure(&page, "#status-display").await;
    assert!(
        !message.is_empty() && !message.contains("narration failed"),
        "the status display must show a short line, got {message:?}"
    );
    assert!(
        raw.contains("narration failed"),
        "the raw text belongs behind the Details disclosure, got {raw:?}"
    );
    assert!(
        !error_details_open(&page, "#status-display").await,
        "the raw text must not be on screen until the client opens Details"
    );
    let form_present: bool = page
        .evaluate::<(), bool>("() => !!document.getElementById('command-form')", None)
        .await
        .unwrap();
    assert!(
        form_present,
        "confirming the preview must leave the command form in place"
    );
    let (banner_visible, _, _) = read_banner(&page).await;
    assert!(
        !banner_visible,
        "a generation error on a reachable engine must not raise the banner"
    );

    status.set(StubStatus::Idle);
    load_screen_now(&page).await;
    wait_for_status_ready(&page).await;
    assert!(
        wait_for_condition_async(
            Duration::from_secs(5),
            Duration::from_millis(50),
            || async { !read_error_disclosure(&page, "#status-display").await.0 },
        )
        .await,
        "returning to Ready must clear the status error"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.13
async fn check_preview_opens_beside_status_display(page: playwright_rs::Page, stub: StubServer) {
    let status = stub.status_handle();
    status.set(StubStatus::Step(Step::Narrating));
    load_screen_now(&page).await;
    wait_for_status_busy(&page).await;

    submit_intercepted_command_direct(&page).await;

    assert!(
        wait_for_condition_async(
            Duration::from_secs(3),
            Duration::from_millis(50),
            || async { active_element_is(&page, "#corrected-textarea").await },
        )
        .await,
        "focus should land in the correction textarea even mid-generation"
    );

    let (preview_visible, status_present, status_text) = page
        .evaluate::<(), (bool, bool, String)>(
            r#"(() => {
                        const preview = document.querySelector('.text-check-preview');
                        const status = document.getElementById('status-display');
                        return [
                            !!preview,
                            !!status,
                            status ? status.textContent.trim() : '',
                        ];
                    })()"#,
            None,
        )
        .await
        .unwrap();
    assert!(preview_visible, "the send preview should be open");
    assert!(
        status_present,
        "opening the preview must not take the status display off screen"
    );
    assert!(
        status_text.contains("Generating") || status_text.contains("Thinking"),
        "the status display should still show the in-flight phase, got {status_text:?}"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.15
async fn check_preview_focus_moves_to_correction_and_back(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    submit_intercepted_command(&page).await;

    assert!(
        wait_for_condition_async(
            Duration::from_secs(3),
            Duration::from_millis(50),
            || async { active_element_is(&page, "#corrected-textarea").await },
        )
        .await,
        "focus should land in the correction textarea when the preview opens"
    );

    page.locator(".preview-cancel")
        .await
        .click(None)
        .await
        .unwrap();
    wait_until_hidden(&page, ".text-check-preview", Duration::from_secs(5)).await;

    assert!(
        wait_for_condition_async(
            Duration::from_secs(3),
            Duration::from_millis(50),
            || async { active_element_is(&page, "#command-form input[name=\"command\"]").await },
        )
        .await,
        "cancelling the preview should return focus to the command input"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.16
async fn check_tab_bar_exposes_a_tablist_and_panels(page: playwright_rs::Page, _stub: StubServer) {
    let list_role: String = page
        .evaluate::<(), String>(
            "() => document.querySelector('.tab-bar').getAttribute('role') || ''",
            None,
        )
        .await
        .unwrap();
    assert_eq!(list_role, "tablist", "the tab bar must expose a tablist");

    let tabs_ok: bool = page
        .evaluate::<(), bool>(
            r#"() => Array.from(document.querySelectorAll('.tab-bar .tab')).every((t) => {
                    const panel = document.getElementById(t.getAttribute('aria-controls'));
                    return t.getAttribute('role') === 'tab'
                        && panel
                        && panel.getAttribute('role') === 'tabpanel'
                        && panel.getAttribute('aria-labelledby') === t.id;
                })"#,
            None,
        )
        .await
        .unwrap();
    assert!(tabs_ok, "every tab must control a labelled tabpanel");

    let game_selected: bool = page
            .evaluate::<(), bool>(
                r#"() => document.querySelector('.tab[data-tab="game"]').getAttribute('aria-selected') === 'true'"#,
                None,
            )
            .await
            .unwrap();
    assert!(game_selected, "the Game tab must start selected");

    page.locator(r#"[data-tab="settings"]"#)
        .await
        .click(None)
        .await
        .unwrap();

    let after: bool = page
        .evaluate::<(), bool>(
            r#"() => {
                    const settings = document.querySelector('.tab[data-tab="settings"]');
                    const game = document.querySelector('.tab[data-tab="game"]');
                    return settings.getAttribute('aria-selected') === 'true'
                        && game.getAttribute('aria-selected') === 'false'
                        && document.getElementById('settings-tab').classList.contains('active');
                }"#,
            None,
        )
        .await
        .unwrap();
    assert!(
        after,
        "activating Settings must move the selection and show its panel"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.39
async fn check_active_tab_survives_a_reload(page: playwright_rs::Page, _stub: StubServer) {
    page.locator(r#"[data-tab="worlds"]"#)
        .await
        .click(None)
        .await
        .unwrap();

    page.reload(None).await.expect("reload the dashboard");

    let restored: bool = page
        .evaluate::<(), bool>(
            r#"() => {
                    const worlds = document.querySelector('.tab[data-tab="worlds"]');
                    return worlds.getAttribute('aria-selected') === 'true'
                        && document.getElementById('worlds-tab').classList.contains('active')
                        && !document.getElementById('game-tab').classList.contains('active');
                }"#,
            None,
        )
        .await
        .unwrap();
    assert!(
        restored,
        "a reload must land back on the tab the user was on"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.17
async fn check_dashboard_exposes_landmarks_and_a_labelled_command_input(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    let landmark_ok: bool = page
        .evaluate::<(), bool>(
            r#"() => {
                    const link = document.querySelector('.skip-link');
                    const main = document.getElementById('main-content');
                    return !!link
                        && !!main
                        && main.tagName === 'MAIN'
                        && link.getAttribute('href') === '#main-content';
                }"#,
            None,
        )
        .await
        .unwrap();
    assert!(landmark_ok, "a skip link must target the main landmark");

    let label_ok: bool = page
        .evaluate::<(), bool>(
            r#"() => {
                    const input = document.getElementById('command-input');
                    if (!input) return false;
                    const label = document.querySelector('label[for="command-input"]');
                    return !!label
                        && label.textContent.trim().length > 0
                        && label.textContent.trim() !== input.placeholder;
                }"#,
            None,
        )
        .await
        .unwrap();
    assert!(
        label_ok,
        "the command input must have an accessible name that is not its placeholder"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.28
async fn check_status_load_leaves_generating_and_enter_submits(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let status = stub.status_handle();
    let action = stub.action_handle();
    status.set(StubStatus::Step(Step::Narrating));
    load_screen_now(&page).await;
    assert!(
        wait_for_condition_async(
            Duration::from_secs(8),
            Duration::from_millis(100),
            || async { read_submit_button(&page).await.0 },
        )
        .await,
        "the status load's phase should lock Send"
    );
    let status_text = page
        .locator("#status-display")
        .await
        .inner_text()
        .await
        .unwrap_or_default();
    assert!(
        !status_text.contains("Ready"),
        "the display should leave Ready, got {status_text:?}"
    );

    status.set(StubStatus::Idle);
    load_screen_now(&page).await;
    wait_for_status_ready(&page).await;
    let (disabled, _) = read_submit_button(&page).await;
    assert!(
        !disabled,
        "Send should unlock once the status load reports idle"
    );

    // Only a real Enter runs the browser's disabled-default-button check;
    // `requestSubmit()` bypasses it.
    let before = action.count();
    fill_command_input(&page, "look around").await;
    let input = page.locator(r#"#command-form input[name="command"]"#).await;
    input.focus().await.unwrap();
    input.press("Enter", None).await.unwrap();
    assert!(
        wait_for_condition_async(
            Duration::from_secs(5),
            Duration::from_millis(50),
            || async { action.count() > before },
        )
        .await,
        "a real Enter should submit the command form"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.36
async fn check_confirmed_preview_consumes_the_command_text(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    submit_intercepted_command(&page).await;
    assert_eq!(
        read_command_input(&page).await,
        "look at the casle",
        "an open preview must keep the submitted text in the command input"
    );

    page.locator(".text-check-preview form .btn-cyan")
        .await
        .click(None)
        .await
        .unwrap();
    wait_until_hidden(&page, ".text-check-preview", Duration::from_secs(5)).await;
    assert!(
        wait_for_condition_async(
            Duration::from_secs(5),
            Duration::from_millis(25),
            || async { read_command_input(&page).await.is_empty() }
        )
        .await,
        "Send with edits must consume the command text, once the screen shows the send"
    );

    submit_intercepted_command_direct(&page).await;
    page.locator(".text-check-preview .btn-original")
        .await
        .click(None)
        .await
        .unwrap();
    wait_until_hidden(&page, ".text-check-preview", Duration::from_secs(5)).await;
    assert!(
        wait_for_condition_async(
            Duration::from_secs(5),
            Duration::from_millis(25),
            || async { read_command_input(&page).await.is_empty() }
        )
        .await,
        "Send Original must consume the command text, once the screen shows the send"
    );

    submit_intercepted_command_direct(&page).await;
    page.locator(".text-check-preview .preview-cancel")
        .await
        .click(None)
        .await
        .unwrap();
    wait_until_hidden(&page, ".text-check-preview", Duration::from_secs(5)).await;
    assert_eq!(
        read_command_input(&page).await,
        "look at the casle",
        "Cancel must keep the command text"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.42
async fn check_refused_concurrent_send_keeps_the_command_and_reports_in_the_form_slot(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    fill_command_input(&page, "look around").await;
    page.evaluate::<(), ()>(
        r#"() => document.getElementById('command-form').requestSubmit()"#,
        None,
    )
    .await
    .unwrap();

    let slot = r#"#command-form [data-error-slot="action"]"#;
    assert_busy_refusal(
        &page,
        slot,
        "a refused concurrent send must report in the command form's own slot",
    )
    .await;
    assert_eq!(
        read_command_input(&page).await,
        "look around",
        "a command the engine refused must keep the text the player typed"
    );
    let display_class: String = page
        .evaluate::<(), String>(
            "(() => document.getElementById('status-display').className)()",
            None,
        )
        .await
        .unwrap();
    assert!(
        display_class
            .split_whitespace()
            .any(|class| class == "ready"),
        "a refusal leaves the status display as it was, got {display_class:?}"
    );
    let preview_children: f64 = page
        .evaluate::<(), f64>(
            "(() => document.getElementById('action-preview').children.length)()",
            None,
        )
        .await
        .unwrap();
    assert_eq!(preview_children, 0.0, "a refusal opens no preview");
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.44
async fn check_confirm_refused_for_a_busy_game_reports_on_the_command_form_slot(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    submit_intercepted_command(&page).await;
    page.locator(".text-check-preview .btn-original")
        .await
        .click(None)
        .await
        .unwrap();
    wait_until_hidden(&page, ".text-check-preview", Duration::from_secs(5)).await;

    let slot = r#"#command-form [data-error-slot="action"]"#;
    assert_busy_refusal(
        &page,
        slot,
        "a refused confirm must report in the command form's own slot",
    )
    .await;
    assert_eq!(
        read_command_input(&page).await,
        "look at the casle",
        "a command the engine refused must keep the text the player typed"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.45
async fn check_the_action_area_shows_its_own_state_until_the_status_display_shows_the_send(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    changes.set_loads_failing(true);
    fill_command_input(&page, "look around").await;
    page.evaluate::<(), ()>(
        r#"() => document.getElementById('command-form').requestSubmit()"#,
        None,
    )
    .await
    .unwrap();

    let own_text = || async {
        page.evaluate::<(), String>(
            "(() => document.getElementById('action-status').textContent.trim())()",
            None,
        )
        .await
        .unwrap()
    };
    assert!(
        wait_for_condition_async(
            Duration::from_secs(5),
            Duration::from_millis(25),
            || async { own_text().await == "Thinking..." },
        )
        .await,
        "an accepted send shows Thinking... in the action area's own element"
    );
    let (state, send_takes_clicks) = page
        .evaluate::<(), (String, bool)>(
            r#"(() => {
                const area = document.getElementById('action-area');
                const send = document.getElementById('submit-btn');
                return [
                    area.dataset.state,
                    getComputedStyle(send).pointerEvents !== 'none',
                ];
            })()"#,
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        state, "generating",
        "the area's view state says that a send is pending"
    );
    assert!(
        !send_takes_clicks,
        "Send takes no click while the send is pending: the part still renders it as idle"
    );

    changes.set_loads_failing(false);
    ask_once(&page).await;
    assert_eq!(
        own_text().await,
        "",
        "the status display shows the send, so the own state ends"
    );
    assert_eq!(
        status_display_text(&page).await,
        "Generating narration...",
        "the status display shows the label of the Job's step"
    );
    let (disabled, label) = read_submit_button(&page).await;
    assert!(disabled, "Send stays locked while the Job runs");
    assert!(
        label.contains("Generating"),
        "the server renders the locked button as a generating indicator, got {label:?}"
    );
}

async fn wait_until_page_shows_game(page: &playwright_rs::Page, game: &str) {
    let shown = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(25),
        || async move {
            page.evaluate::<(), String>("() => document.documentElement.dataset.game || ''", None)
                .await
                .unwrap()
                == game
        },
    )
    .await;
    assert!(shown, "the page records the Game it shows: {game}");
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.53
async fn check_each_action_carries_the_game_the_page_shows(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    wait_until_page_shows_game(&page, "1").await;
    fill_command_input(&page, "look around").await;
    page.evaluate::<(), ()>(
        r#"() => document.getElementById('command-form').requestSubmit()"#,
        None,
    )
    .await
    .unwrap();
    let sent = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(25),
        || async {
            !stub
                .changes_handle()
                .game_headers("/action/check")
                .is_empty()
        },
    )
    .await;
    assert!(sent, "the command reaches the engine");
    assert_eq!(
        stub.changes_handle().game_headers("/action/check"),
        vec![Some("1".to_string())],
        "the command names the Game the page shows"
    );

    stub.save_handle().set_delete_succeeds(true);
    accept_delete_confirmation(&page).await;
    let delete = "#story-log .log-entry:last-child .delete-btn";
    page.locator(delete).await.click(None).await.unwrap();
    let sent = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(25),
        || async {
            !stub
                .changes_handle()
                .game_headers("/history/delete")
                .is_empty()
        },
    )
    .await;
    assert!(sent, "the delete reaches the engine");
    assert_eq!(
        stub.changes_handle().game_headers("/history/delete"),
        vec![Some("1".to_string())],
        "a request that a script sends names the Game the page shows"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.54
async fn check_a_send_for_a_game_that_is_no_longer_active_keeps_the_typed_command(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    wait_until_page_shows_game(&page, "1").await;
    stub.changes_handle().set_game_of_actions(2);
    fill_command_input(&page, "look around").await;
    page.evaluate::<(), ()>(
        r#"() => document.getElementById('command-form').requestSubmit()"#,
        None,
    )
    .await
    .unwrap();

    let slot = r#"#command-form [data-error-slot="action"]"#;
    wait_for_error_disclosure(
        &page,
        slot,
        "the refused send reports in the command form's slot",
    )
    .await;
    let (_, line, _) = read_error_disclosure(&page, slot).await;
    assert!(
        line.contains("The active Game changed"),
        "the line says the Game changed, got {line:?}"
    );
    assert_eq!(
        read_command_input(&page).await,
        "look around",
        "a refused send keeps the text the player typed"
    );
    let preview_children: f64 = page
        .evaluate::<(), f64>(
            "(() => document.getElementById('action-preview').children.length)()",
            None,
        )
        .await
        .unwrap();
    assert_eq!(preview_children, 0.0, "a refusal opens no preview");
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.55
async fn check_an_edit_saved_for_a_game_that_is_no_longer_active_stays_open(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    wait_until_page_shows_game(&page, "1").await;
    page.locator(".log-entry.narration .edit-btn")
        .await
        .click(None)
        .await
        .unwrap();
    wait_until_visible(&page, "#edit-textarea", Duration::from_millis(500)).await;
    page.locator("#edit-textarea")
        .await
        .fill("Text typed before the switch", None)
        .await
        .unwrap();
    stub.changes_handle().set_game_of_actions(2);
    page.locator(".log-entry.narration .save-btn")
        .await
        .click(None)
        .await
        .unwrap();

    wait_for_error_disclosure(
        &page,
        "#story-log-error",
        "the refused save reports in the story log",
    )
    .await;
    let (_, line, _) = read_error_disclosure(&page, "#story-log-error").await;
    assert!(
        line.contains("The active Game changed"),
        "the line says the Game changed, got {line:?}"
    );
    assert_eq!(
        page.locator("#edit-textarea")
            .await
            .input_value(None)
            .await
            .unwrap(),
        "Text typed before the switch",
        "the editor keeps the player's text"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.56
async fn check_the_page_names_a_game_only_when_the_screen_shows_it(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    wait_until_page_shows_game(&page, "1").await;
    stop_page_timer(&page).await;
    hold_screen_loads(&page).await;
    stub.changes_handle().set_game(Some(2));

    ask_once_without_waiting(&page).await;
    wait_for_held_screen_load(&page).await;
    let shown_while_loading: String = page
        .evaluate::<(), String>("() => document.documentElement.dataset.game || ''", None)
        .await
        .unwrap();
    assert_eq!(
        shown_while_loading, "1",
        "the load is still in flight, so the page names no new Game yet"
    );

    release_screen_loads(&page).await;
    wait_until_page_shows_game(&page, "2").await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.57
async fn check_a_switch_sent_from_this_page_clears_the_game_until_the_next_ask(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    wait_until_page_shows_game(&page, "1").await;
    page.locator(r#"[data-tab="games"]"#)
        .await
        .click(None)
        .await
        .unwrap();
    let switch = r#"button[hx-post="/games/2/switch"]"#;
    wait_until_visible(&page, switch, Duration::from_secs(5)).await;
    stop_page_timer(&page).await;

    let seen: (String, bool) = page
        .evaluate::<(), (String, bool)>(
            r#"async () => {
                let seen = null;
                document.body.addEventListener('htmx:afterRequest', () => {
                    if (seen) return;
                    seen = document.documentElement.dataset.game || '';
                    actionFetch('/retrigger', { method: 'POST' }).catch(() => {});
                }, { once: true });
                document.querySelector('button[hx-post="/games/2/switch"]').click();
                while (seen === null) await new Promise((resolve) => setTimeout(resolve, 5));
                return [seen, true];
            }"#,
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        seen.0, "",
        "the switch answer removed the Game the page named"
    );
    assert_eq!(
        stub.changes_handle().game_headers("/retrigger"),
        vec![None],
        "a send before the next ask carries no Game header, so it meets no false refusal"
    );
    wait_until_page_shows_game(&page, "2").await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.60
async fn check_a_load_that_started_before_a_switch_does_not_name_the_old_game(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    wait_until_page_shows_game(&page, "1").await;
    page.locator(r#"[data-tab="games"]"#)
        .await
        .click(None)
        .await
        .unwrap();
    let switch = r#"button[hx-post="/games/2/switch"]"#;
    wait_until_visible(&page, switch, Duration::from_secs(5)).await;
    stop_page_timer(&page).await;

    delay_screen_answers(&page).await;
    let requests = changes.screen_requests().len();
    changes.bump(ChangeKind::SETTINGS);
    ask_once_without_waiting(&page).await;
    assert!(
        wait_for_condition_async(
            Duration::from_secs(5),
            Duration::from_millis(20),
            || async { changes.screen_requests().len() > requests }
        )
        .await,
        "the engine must receive the load before the switch"
    );

    page.evaluate::<(), ()>(
        r#"() => {
            window.__gamesNamed = [];
            new MutationObserver(() => {
                window.__gamesNamed.push(document.documentElement.dataset.game || '');
            }).observe(document.documentElement, { attributes: true, attributeFilter: ['data-game'] });
            document.querySelector('button[hx-post="/games/2/switch"]').click();
        }"#,
        None,
    )
    .await
    .unwrap();
    let cleared = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(20),
        || async {
            page.evaluate::<(), String>("() => document.documentElement.dataset.game || ''", None)
                .await
                .unwrap()
                .is_empty()
        },
    )
    .await;
    assert!(cleared, "the switch removes the Game the page named");

    release_screen_answers(&page).await;
    wait_until_page_shows_game(&page, "2").await;
    let named: Vec<String> = page
        .evaluate::<(), Vec<String>>("() => window.__gamesNamed", None)
        .await
        .unwrap();
    assert!(
        !named.iter().any(|game| game == "1"),
        "the load that started before the switch must not name Game 1 again, the page named {named:?}"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.61
async fn check_a_preview_send_clears_the_input_only_if_it_still_holds_the_text_the_preview_sent(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    stop_page_timer(&page).await;
    submit_intercepted_command(&page).await;
    hold_requests_ending_with(&page, "/action/confirm").await;
    page.locator(".text-check-preview .btn-original")
        .await
        .click(None)
        .await
        .unwrap();
    fill_command_input(&page, "look at the casle and then wait").await;

    release_held_requests_and_wait_for_answers(&page).await;
    wait_for_screen_current(&page).await;
    assert_eq!(
        read_command_input(&page).await,
        "look at the casle and then wait",
        "the player typed on while the preview send was in flight, and the screen showing the send must not clear the new text"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.58
async fn check_a_stop_for_a_game_that_is_no_longer_active_reports_in_the_command_form_slot(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    wait_until_page_shows_game(&page, "1").await;
    stub.status_handle().set(StubStatus::Autoplay {
        done: 1,
        budget: 10,
        step: None,
    });
    load_screen_now(&page).await;
    wait_until_visible(&page, "#stop-btn", Duration::from_secs(5)).await;
    stub.changes_handle().set_game_of_actions(2);
    page.locator("#stop-btn").await.click(None).await.unwrap();

    let slot = r#"#command-form [data-error-slot="action"]"#;
    wait_for_error_disclosure(
        &page,
        slot,
        "the refused Stop reports in the command form's slot",
    )
    .await;
    let (_, line, _) = read_error_disclosure(&page, slot).await;
    assert!(
        line.contains("The active Game changed") && line.contains("the run was not stopped"),
        "the line says what was refused and why, got {line:?}"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.59
async fn check_a_reset_for_a_game_that_is_no_longer_active_reports_in_the_game_row(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    wait_until_page_shows_game(&page, "1").await;
    accept_delete_confirmation(&page).await;
    page.locator(r#"[data-tab="games"]"#)
        .await
        .click(None)
        .await
        .unwrap();
    let reset = ".game-item.active .btn-reset-small";
    wait_until_visible(&page, reset, Duration::from_secs(5)).await;
    stub.changes_handle().set_game_of_actions(2);
    page.locator(reset).await.click(None).await.unwrap();

    let slot = ".game-item.active > [data-error-slot]";
    wait_for_error_disclosure(&page, slot, "the refused Reset reports in its game row").await;
    let (_, line, _) = read_error_disclosure(&page, slot).await;
    assert!(
        line.contains("The active Game changed") && line.contains("the game was not reset"),
        "the line says what was refused and why, got {line:?}"
    );
}

#[tokio::test]
async fn run_dashboard_action_area_checks() {
    let mut runner = StubRunner::launch().await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_primary_button_locks_and_unlocks_after_confirm,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_status_error_shows_in_the_status_display_after_confirm,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_preview_opens_beside_status_display,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_preview_focus_moves_to_correction_and_back,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_status_load_leaves_generating_and_enter_submits,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_confirmed_preview_consumes_the_command_text,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_the_action_area_shows_its_own_state_until_the_status_display_shows_the_send,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Refused,
            check_confirm_refused_for_a_busy_game_reports_on_the_command_form_slot,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Refused,
            check_refused_concurrent_send_keeps_the_command_and_reports_in_the_form_slot,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_each_action_carries_the_game_the_page_shows,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_send_for_a_game_that_is_no_longer_active_keeps_the_typed_command,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_an_edit_saved_for_a_game_that_is_no_longer_active_stays_open,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_the_page_names_a_game_only_when_the_screen_shows_it,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_switch_sent_from_this_page_clears_the_game_until_the_next_ask,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_load_that_started_before_a_switch_does_not_name_the_old_game,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_preview_send_clears_the_input_only_if_it_still_holds_the_text_the_preview_sent,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_stop_for_a_game_that_is_no_longer_active_reports_in_the_command_form_slot,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_reset_for_a_game_that_is_no_longer_active_reports_in_the_game_row,
        )
        .await;
    runner.finish().await;
}

#[tokio::test]
async fn run_dashboard_chrome_checks() {
    let mut runner = StubRunner::launch().await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_tab_bar_exposes_a_tablist_and_panels,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_active_tab_survives_a_reload,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_dashboard_exposes_landmarks_and_a_labelled_command_input,
        )
        .await;
    runner.finish().await;
}
