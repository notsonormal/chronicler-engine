//! Stub-browser tests for dashboard chrome: the action-area state machine, the tab bar, the landmarks and the status display. Tagged against `docs/specs/browser_dashboard.md`.

use std::time::Duration;

use super::*;
use super::support::{active_element_is, read_banner};

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

/// Node identity, not the id string a replacement would also carry.
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
            document.querySelector('#command-form button[type="submit"]').click();
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
    wait_for_status_generating(page).await;
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
#[tokio::test]
async fn test_primary_button_locks_and_unlocks_after_confirm() {
    with_stub_page(StubActionOutcome::Pending, |page, stub| {
        let status = stub.status_handle();
        async move {
            // Keep the stub generating so no poll unlocks the button before the assertion reads it.
            status.set(StubStatus::Phase("narrating".to_string()));
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
    })
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.10
#[tokio::test]
async fn test_status_error_shows_in_the_status_display_after_confirm() {
    with_stub_page(StubActionOutcome::Pending, |page, stub| {
        let status = stub.status_handle();
        async move {
            submit_intercepted_command(&page).await;
            confirm_text_check_preview(&page).await;

            // The error fragment lands in the #status-display the confirm already swapped.
            status.set(StubStatus::Error("narration failed".to_string()));
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
    })
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.13
#[tokio::test]
async fn test_preview_opens_beside_status_display() {
    with_stub_page(StubActionOutcome::Pending, |page, stub| {
        let status = stub.status_handle();
        async move {
            status.set(StubStatus::Phase("narrating".to_string()));
            wait_for_status_generating(&page).await;

            // The button is a disabled generating indicator here, so submit the form
            // directly (the player's Enter path).
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
    })
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.15
#[tokio::test]
async fn test_preview_focus_moves_to_correction_and_back() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
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
                || async {
                    active_element_is(&page, "#command-form input[name=\"command\"]").await
                },
            )
            .await,
            "cancelling the preview should return focus to the command input"
        );
    })
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.16
#[tokio::test]
async fn test_tab_bar_exposes_a_tablist_and_panels() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
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
    })
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.17
#[tokio::test]
async fn test_dashboard_exposes_landmarks_and_a_labelled_command_input() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
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
    })
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.28
#[tokio::test]
async fn test_status_poll_leaves_generating_and_enter_submits() {
    with_stub_page(StubActionOutcome::Pending, |page, stub| {
        let status = stub.status_handle();
        let action = stub.action_handle();
        async move {
            status.set(StubStatus::Phase("narrating".to_string()));
            assert!(
                wait_for_condition_async(
                    Duration::from_secs(8),
                    Duration::from_millis(100),
                    || async { read_submit_button(&page).await.0 },
                )
                .await,
                "the poll's phase should lock Send"
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

            // The button's label belongs to 16.9; this test owns the poll-driven transition.
            status.set(StubStatus::Idle);
            wait_for_status_ready(&page).await;
            let (disabled, _) = read_submit_button(&page).await;
            assert!(!disabled, "Send should unlock once the poll reports idle");

            // No other test presses Enter: `requestSubmit()` does not reproduce the
            // disabled-default-button block a real Enter hits.
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
    })
    .await;
}
