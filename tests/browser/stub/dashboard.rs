//! Stub-browser tests for dashboard chrome: the action-area state machine, the tab bar, the landmarks and the status display. Tagged against `docs/specs/browser_dashboard.md`.

use std::time::Duration;

use chronicler_engine::domain::model::state::generation_status::GenerationFailureKind;

use chronicler_engine::domain::model::state::generation_status::Step;

use super::*;
use super::support::{
    NOTICE, StubRunner, active_element_is, assert_busy_refusal, read_banner, read_command_input,
    submit_intercepted_command,
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
    // The page timer would load the generating status into the button between the
    // `set` and the click, and a click on a disabled button sends nothing. The
    // action itself still makes the page ask.
    stop_page_timer(&page).await;
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
async fn check_refused_concurrent_send_keeps_the_command_and_reports_in_the_notice(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    wait_for_status_ready(&page).await;
    let status_before = status_display_text(&page).await;
    fill_command_input(&page, "look around").await;
    page.evaluate::<(), ()>(
        r#"() => document.getElementById('command-form').requestSubmit()"#,
        None,
    )
    .await
    .unwrap();

    assert_busy_refusal(
        &page,
        NOTICE,
        "a refused concurrent send must report in the notice",
    )
    .await;
    assert_eq!(
        read_command_input(&page).await,
        "look around",
        "a command the engine refused must keep the text the player typed"
    );
    assert_eq!(
        status_display_text(&page).await,
        status_before,
        "a refusal leaves the status display as it was"
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
async fn check_confirm_refused_for_a_busy_game_reports_in_the_notice_and_keeps_the_preview(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    submit_intercepted_command(&page).await;
    page.locator(".text-check-preview .btn-original")
        .await
        .click(None)
        .await
        .unwrap();

    assert_busy_refusal(&page, NOTICE, "a refused confirm must report in the notice").await;
    wait_until_visible(&page, ".text-check-preview", Duration::from_secs(5)).await;
    assert_eq!(
        page.locator("#corrected-textarea")
            .await
            .input_value(None)
            .await
            .unwrap(),
        "look at the castle",
        "the preview stays open with the corrected text"
    );
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
            check_confirm_refused_for_a_busy_game_reports_in_the_notice_and_keeps_the_preview,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Refused,
            check_refused_concurrent_send_keeps_the_command_and_reports_in_the_notice,
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
