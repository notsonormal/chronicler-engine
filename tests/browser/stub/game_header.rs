//! Stub-browser tests for the Game header of a request: the page names the Game it shows, and the engine refuses a request for a Game that is no longer active. Tagged against `docs/specs/browser_dashboard.md`.

use std::time::Duration;

use super::support::{
    accept_delete_confirmation, ask_once_without_waiting, hold_requests_ending_with,
    hold_screen_loads, read_command_input, release_held_requests_and_wait_for_answers,
    release_screen_loads, submit_intercepted_command, wait_for_held_screen_load, StubRunner,
    NOTICE,
};
use super::*;

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
async fn check_each_request_carries_the_game_the_page_shows(
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
    stub.changes_handle().set_game_of_requests(2);
    fill_command_input(&page, "look around").await;
    page.evaluate::<(), ()>(
        r#"() => document.getElementById('command-form').requestSubmit()"#,
        None,
    )
    .await
    .unwrap();

    let slot = NOTICE;
    wait_for_error_disclosure(&page, slot, "the refused send reports in the notice").await;
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
    stub.changes_handle().set_game_of_requests(2);
    page.locator("[data-editing] .save-btn")
        .await
        .click(None)
        .await
        .unwrap();

    wait_for_error_disclosure(&page, NOTICE, "the refused save reports in the notice").await;
    let (_, line, _) = read_error_disclosure(&page, NOTICE).await;
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
async fn check_a_send_after_a_switch_from_this_page_gets_the_game_refusal_and_keeps_the_text(
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
    hold_screen_loads(&page).await;

    page.evaluate::<(), ()>(
        r#"() => {
            document.getElementById('command-input').value = 'look around';
            document.querySelector('button[hx-post="/games/2/switch"]').click();
        }"#,
        None,
    )
    .await
    .unwrap();
    wait_for_held_screen_load(&page).await;
    assert_eq!(
        page.evaluate::<(), String>("() => document.documentElement.dataset.game || ''", None)
            .await
            .unwrap(),
        "1",
        "the switch answer does not remove the Game the page names"
    );

    page.evaluate::<(), ()>(
        "() => document.getElementById('command-form').requestSubmit()",
        None,
    )
    .await
    .unwrap();
    let slot = NOTICE;
    wait_for_error_disclosure(
        &page,
        slot,
        "the send before the next load reports in the notice",
    )
    .await;
    let (_, line, _) = read_error_disclosure(&page, slot).await;
    assert!(
        line.contains("The active Game changed"),
        "the line says the Game changed, got {line:?}"
    );
    assert_eq!(
        stub.changes_handle().game_headers("/action/check"),
        vec![Some("1".to_string())],
        "the send names the Game that the page still shows"
    );
    assert_eq!(
        read_command_input(&page).await,
        "look around",
        "a refused send keeps the text the player typed"
    );

    release_screen_loads(&page).await;
    wait_until_page_shows_game(&page, "2").await;
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
async fn check_a_stop_for_a_game_that_is_no_longer_active_reports_in_the_notice(
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
    stub.changes_handle().set_game_of_requests(2);
    page.locator("#stop-btn").await.click(None).await.unwrap();

    let slot = NOTICE;
    wait_for_error_disclosure(&page, slot, "the refused Stop reports in the notice").await;
    let (_, line, _) = read_error_disclosure(&page, slot).await;
    assert_eq!(
        line, "The active Game changed, so the Autoplay run was not stopped.",
        "the line says what was refused and why"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.59
async fn check_a_reset_for_a_game_that_is_no_longer_active_reports_in_the_notice(
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
    stub.changes_handle().set_game_of_requests(2);
    page.locator(reset).await.click(None).await.unwrap();

    let slot = NOTICE;
    wait_for_error_disclosure(&page, slot, "the refused Reset reports in the notice").await;
    let (_, line, _) = read_error_disclosure(&page, slot).await;
    assert_eq!(
        line, "The active Game changed, so the Game was not reset.",
        "the line says what was refused and why"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.81
async fn check_a_command_typed_for_a_game_names_that_game_after_a_switch(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    wait_until_page_shows_game(&page, "1").await;
    page.locator("#command-input")
        .await
        .fill("look around", None)
        .await
        .unwrap();
    stub.changes_handle().set_game(Some(2));
    wait_until_page_shows_game(&page, "2").await;

    page.locator("#command-input")
        .await
        .press("Enter", None)
        .await
        .unwrap();

    wait_for_error_disclosure(&page, NOTICE, "the refused send reports in the notice").await;
    let (_, line, _) = read_error_disclosure(&page, NOTICE).await;
    assert_eq!(
        line, "The active Game changed, so the Action was not sent.",
        "the line says what was refused and why"
    );
    assert_eq!(
        stub.changes_handle().game_headers("/action/check"),
        vec![Some("1".to_string())],
        "the send names the Game that the text was typed for"
    );
    assert_eq!(
        stub.action_handle().count(),
        0,
        "the engine took nothing for the other Game"
    );
    assert_eq!(
        read_command_input(&page).await,
        "look around",
        "a refused send keeps the text the player typed"
    );

    wait_for_screen_current(&page).await;
    page.locator("#command-input")
        .await
        .press("Enter", None)
        .await
        .unwrap();
    let sent = wait_for_condition_async(
        Duration::from_secs(5),
        Duration::from_millis(25),
        || async { stub.changes_handle().game_headers("/action/check").len() == 2 },
    )
    .await;
    assert!(sent, "the second send reaches the stub");
    assert_eq!(
        stub.changes_handle().game_headers("/action/check")[1],
        Some("2".to_string()),
        "after the refusal, the draft takes the Game that the screen shows, so a second Enter names it"
    );
    assert_eq!(
        stub.action_handle().count(),
        1,
        "the engine took the second send for the new Game"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.85
async fn check_an_option_click_after_a_game_change_names_the_new_game(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    wait_until_page_shows_game(&page, "1").await;
    wait_for_element_children(&page, "#options-dock .option-item", 3).await;
    page.locator("#command-input")
        .await
        .fill("typed for the first Game", None)
        .await
        .unwrap();
    stub.changes_handle().set_game(Some(2));
    wait_until_page_shows_game(&page, "2").await;
    wait_for_screen_current(&page).await;

    page.evaluate::<(), ()>(
        "() => document.querySelector('#options-dock .option-btn').click()",
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
    assert!(sent, "the Option reaches the stub");
    assert_eq!(
        stub.changes_handle().game_headers("/action/check"),
        vec![Some("2".to_string())],
        "an Option comes from the screen of the current Game, so it starts a new draft"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.86
async fn check_an_option_edited_after_a_game_change_names_the_new_game(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    wait_until_page_shows_game(&page, "1").await;
    wait_for_element_children(&page, "#options-dock .option-item", 3).await;
    page.locator("#command-input")
        .await
        .fill("typed for the first Game", None)
        .await
        .unwrap();
    stub.changes_handle().set_game(Some(2));
    wait_until_page_shows_game(&page, "2").await;
    wait_for_screen_current(&page).await;

    page.evaluate::<(), ()>(
        "() => document.querySelector('#options-dock .option-item [aria-label=\"Edit before send\"]').click()",
        None,
    )
    .await
    .unwrap();
    page.locator("#command-input")
        .await
        .press("Enter", None)
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
    assert!(sent, "the edited Option reaches the stub");
    assert_eq!(
        stub.changes_handle().game_headers("/action/check"),
        vec![Some("2".to_string())],
        "an edited Option comes from the screen of the current Game, so it starts a new draft"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.82
async fn check_an_edit_opened_for_a_game_saves_to_that_game_after_a_switch(
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
    stub.changes_handle().set_game(Some(2));
    wait_until_page_shows_game(&page, "2").await;

    page.locator("[data-editing] .save-btn")
        .await
        .click(None)
        .await
        .unwrap();

    wait_for_error_disclosure(&page, NOTICE, "the refused save reports in the notice").await;
    let (_, line, _) = read_error_disclosure(&page, NOTICE).await;
    assert_eq!(
        line, "The active Game changed, so the edit was not saved.",
        "the line says what was refused and why"
    );
    assert!(
        stub.save_handle().saved_texts().is_empty(),
        "the engine saved nothing for the other Game"
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

    page.locator("[data-editing] .cancel-btn")
        .await
        .click(None)
        .await
        .unwrap();
    let draft_game: Option<String> = page
        .evaluate::<(), Option<String>>(
            "() => document.querySelector('#story-log [data-draft-game]')?.getAttribute('data-draft-game') ?? null",
            None,
        )
        .await
        .unwrap();
    assert_eq!(draft_game, None, "Cancel ends the draft");
    let entries_in_edit: u32 = page
        .evaluate::<(), u32>(
            "() => document.querySelectorAll('#story-log [data-editing]').length",
            None,
        )
        .await
        .unwrap();
    assert_eq!(entries_in_edit, 0, "after Cancel no entry is in edit");
}

#[tokio::test]
async fn run_game_header_checks() {
    let mut runner = StubRunner::launch().await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_each_request_carries_the_game_the_page_shows,
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
            check_a_send_after_a_switch_from_this_page_gets_the_game_refusal_and_keeps_the_text,
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
            check_a_stop_for_a_game_that_is_no_longer_active_reports_in_the_notice,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_reset_for_a_game_that_is_no_longer_active_reports_in_the_notice,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_a_command_typed_for_a_game_names_that_game_after_a_switch,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_an_edit_opened_for_a_game_saves_to_that_game_after_a_switch,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_an_option_click_after_a_game_change_names_the_new_game,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_an_option_edited_after_a_game_change_names_the_new_game,
        )
        .await;
    runner.finish().await;
}
