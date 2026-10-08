//! Stub-browser tests for dashboard chrome: the failure surfaces and the action-area state machine. Tagged against `docs/specs/browser_dashboard.md`.

// The engine's failing action route answers 500, so htmx fires
// `htmx:responseError` without a dispatched event.

use std::time::Duration;

use playwright_rs::AriaRole;

use super::*;

/// Unlike `send_action`, submits directly rather than clicking Send — the failure
/// path leaves Send locked until the next idle poll — and does not wait for the
/// status span: a 500 is not swapped.
async fn submit_command(page: &playwright_rs::Page, command: &str) {
    fill_command_input(page, command).await;
    page.evaluate::<(), ()>(
        r#"() => {
            const form = document.getElementById('command-form');
            form.requestSubmit();
        }"#,
        None,
    )
    .await
    .unwrap();
}

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

async fn active_element_is(page: &playwright_rs::Page, selector: &str) -> bool {
    page.evaluate::<String, bool>(
        r#"(selector) => {
            const el = document.querySelector(selector);
            return !!el && document.activeElement === el;
        }"#,
        Some(&selector.to_string()),
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

// [docs/specs/browser_dashboard.md] SCENARIO: 16.22
#[tokio::test]
async fn test_degraded_role_raises_status_banner() {
    with_stub_page(StubActionOutcome::Pending, |page, stub| {
        let failures = stub.failure_handle();
        failures.set_header_degraded(true);
        async move {
            assert!(
                wait_for_condition_async(
                    Duration::from_secs(10),
                    Duration::from_millis(100),
                    || async { read_banner(&page).await.0 },
                )
                .await,
                "the server-rendered degraded banner should become visible"
            );
            let (visible, role, text) = read_banner(&page).await;
            assert!(visible, "the degraded banner should be up");
            assert_eq!(
                role, "status",
                "a degraded-role banner reports role status, not alert"
            );
            assert!(
                text.contains("Quantifier failed"),
                "the banner should name the degraded role, got {text:?}"
            );

            page.locator("#failure-banner-degraded .error-details-toggle")
                .await
                .click(None)
                .await
                .unwrap();
            let (open, detail) = page
                .evaluate::<(), (bool, String)>(
                    r#"() => {
                        const popover = document.getElementById('failure-banner-popover');
                        return [!!popover && popover.classList.contains('open'), popover ? popover.textContent : ''];
                    }"#,
                    None,
                )
                .await
                .unwrap();
            assert!(open, "Details should open the banner's anchored disclosure");
            assert!(
                detail.contains("Mock mock"),
                "the disclosure should name each role's backend/model, got {detail:?}"
            );
        }
    })
    .await;
}

async fn read_banner(page: &playwright_rs::Page) -> (bool, String, String) {
    page.evaluate::<(), (bool, String, String)>(
        r#"() => {
            const banner = document.getElementById('failure-banner');
            if (!banner) return [false, '', ''];
            return [
                !banner.hidden,
                banner.getAttribute('role') || '',
                banner.textContent.trim(),
            ];
        }"#,
        None,
    )
    .await
    .unwrap()
}

async fn error_detail_popover_open(page: &playwright_rs::Page, popover_id: &str) -> bool {
    page.evaluate::<String, bool>(
        r#"(id) => {
            const popover = document.getElementById(id);
            return !!popover && popover.classList.contains('open') && !popover.hidden;
        }"#,
        Some(&popover_id.to_string()),
    )
    .await
    .unwrap_or(false)
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.18
#[tokio::test]
async fn test_failed_poll_keeps_region_and_marks_banner() {
    with_stub_page(StubActionOutcome::Pending, |page, stub| {
        let failures = stub.failure_handle();
        async move {
            let before: String = page
                .evaluate::<(), String>(
                    "() => document.getElementById('story-log').innerHTML",
                    None,
                )
                .await
                .unwrap();

            failures.set_polls_failing(true);
            assert!(
                wait_for_condition_async(
                    Duration::from_secs(10),
                    Duration::from_millis(100),
                    || async { read_banner(&page).await.0 },
                )
                .await,
                "a failed poll should raise the banner"
            );
            let (visible, role, text) = read_banner(&page).await;
            assert!(visible, "the banner should be up on a failed poll");
            assert_eq!(role, "alert", "an unreachable banner reports role alert");
            assert!(
                text.contains("unreachable"),
                "the banner should say the engine is unreachable, got {text:?}"
            );

            let after: String = page
                .evaluate::<(), String>(
                    "() => document.getElementById('story-log').innerHTML",
                    None,
                )
                .await
                .unwrap();
            assert_eq!(
                after, before,
                "a failed poll must leave the story log's last good content in place"
            );

            failures.set_polls_failing(false);
            assert!(
                wait_for_condition_async(
                    Duration::from_secs(10),
                    Duration::from_millis(100),
                    || async { !read_banner(&page).await.0 },
                )
                .await,
                "a later successful poll should clear the banner"
            );
        }
    })
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.19
#[tokio::test]
async fn test_failed_action_renders_inline_slot() {
    with_stub_page(StubActionOutcome::Error, |page, _stub| async move {
        submit_command(&page, "Internal server error").await;

        assert!(
            wait_for_condition_async(
                Duration::from_secs(5),
                Duration::from_millis(50),
                || async {
                    read_error_disclosure(&page, "#command-form [data-error-slot]")
                        .await
                        .0
                },
            )
            .await,
            "a failed action should render into the form's inline slot"
        );
        let (visible, message, raw) =
            read_error_disclosure(&page, "#command-form [data-error-slot]").await;
        assert!(visible, "the inline slot should be shown");
        assert!(
            message.contains("action failed"),
            "the inline message should name the failed action, got {message:?}"
        );
        assert!(
            raw.contains("Failed to process action"),
            "the raw server text belongs in the disclosure, got {raw:?}"
        );
        let (banner_visible, _, _) = read_banner(&page).await;
        assert!(
            !banner_visible,
            "an action failure with the server up must not raise the banner"
        );
    })
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.25
#[tokio::test]
async fn test_failed_non_poll_action_does_not_raise_unreachable_banner() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        wait_for_element_children(&page, "#game-posture-controls select", 1).await;
        let stashed = page
            .evaluate::<(), bool>(
                r#"() => {
                    window.__postureBefore = document.getElementById('game-posture-controls');
                    return !!window.__postureBefore;
                }"#,
                None,
            )
            .await
            .unwrap();
        assert!(stashed, "the posture controls should be loaded");

        page.evaluate::<(), ()>(
            r#"() => {
                const select = document.querySelector(
                    '#game-posture-controls select[name="narrative_tense"]');
                select.value = 'present';
                select.dispatchEvent(new Event('change', { bubbles: true }));
            }"#,
            None,
        )
        .await
        .unwrap();

        assert!(
            wait_for_condition_async(
                Duration::from_secs(8),
                Duration::from_millis(100),
                || async {
                    read_error_disclosure(&page, "#game-posture-controls [data-error-slot]").await.0
                },
            )
            .await,
            "a failed posture save must report in the posture controls' own slot"
        );
        let (slot_visible, slot_message, slot_raw) =
            read_error_disclosure(&page, "#game-posture-controls [data-error-slot]").await;
        assert!(
            slot_visible,
            "a failed non-poll action must report on its own surface"
        );
        assert!(
            slot_message.contains("action failed"),
            "the slot carries a short line, got {slot_message:?}"
        );
        assert!(
            slot_raw.contains("Failed to save posture"),
            "the server's own text belongs in the disclosure, got {slot_raw:?}"
        );

        let (banner_visible, role, banner_text) = read_banner(&page).await;
        assert!(
            !banner_visible,
            "a failed action on a reachable server must not raise the banner, got {banner_text:?} ({role})"
        );

        let region_kept = page
            .evaluate::<(), bool>(
                r#"() => {
                    const now = document.getElementById('game-posture-controls');
                    return !!now && now === window.__postureBefore;
                }"#,
                None,
            )
            .await
            .unwrap();
        assert!(
            region_kept,
            "a failed action must leave its region in place"
        );
    })
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.26
#[tokio::test]
async fn test_status_details_popover_survives_a_poll() {
    with_stub_page(StubActionOutcome::Pending, |page, stub| {
        let status = stub.status_handle();
        async move {
            status.set(StubStatus::Error("narration failed".to_string()));
            assert!(
                wait_for_condition_async(
                    Duration::from_secs(10),
                    Duration::from_millis(100),
                    || async {
                        page.evaluate::<(), bool>(
                            "() => !!document.querySelector('#status-display .error-disclosure')",
                            None,
                        )
                        .await
                        .unwrap_or(false)
                    },
                )
                .await,
                "the status poll should swap in the clamped error disclosure"
            );

            watch_htmx_requests(&page).await;
            page.locator("#status-display .error-details-toggle")
                .await
                .click(None)
                .await
                .unwrap();
            assert!(
                error_detail_popover_open(&page, "status-error-popover").await,
                "Details should open the status display's disclosure"
            );
            assert!(
                active_element_is(&page, "#status-display .error-details-toggle").await,
                "the toggle should hold focus while the disclosure is open"
            );
            page.evaluate::<(), ()>(
                "() => { window.__statusToggleBefore = document.querySelector('#status-display .error-details-toggle'); }",
                None,
            )
            .await
            .unwrap();

            wait_for_htmx_requests(&page, "/status/generating", 1, Duration::from_secs(8)).await;

            assert!(
                wait_for_condition_async(
                    Duration::from_secs(3),
                    Duration::from_millis(50),
                    || async {
                        page.evaluate::<(), bool>(
                            r#"() => {
                                const now = document.querySelector('#status-display .error-details-toggle');
                                const popover = document.getElementById('status-error-popover');
                                return !!now
                                    && now !== window.__statusToggleBefore
                                    && !!popover
                                    && popover.classList.contains('open');
                            }"#,
                            None,
                        )
                        .await
                        .unwrap_or(false)
                    },
                )
                .await,
                "a poll that replaced the disclosure must not collapse it"
            );
            assert!(
                active_element_is(&page, "#status-display .error-details-toggle").await,
                "focus should return to the disclosure's toggle after the poll"
            );

            page.keyboard().press("Escape", None).await.unwrap();
            assert!(
                wait_for_condition_async(
                    Duration::from_secs(3),
                    Duration::from_millis(50),
                    || async { !error_detail_popover_open(&page, "status-error-popover").await },
                )
                .await,
                "Escape should dismiss the disclosure"
            );
        }
    })
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.27
#[tokio::test]
async fn test_banner_details_popover_survives_a_header_poll() {
    with_stub_page(StubActionOutcome::Pending, |page, stub| {
        let failures = stub.failure_handle();
        failures.set_header_degraded(true);
        async move {
            assert!(
                wait_for_condition_async(
                    Duration::from_secs(10),
                    Duration::from_millis(100),
                    || async { read_banner(&page).await.0 },
                )
                .await,
                "the degraded-role banner should become visible"
            );

            watch_htmx_requests(&page).await;
            page.locator("#failure-banner-degraded .error-details-toggle")
                .await
                .click(None)
                .await
                .unwrap();
            assert!(
                error_detail_popover_open(&page, "failure-banner-popover").await,
                "Details should open the banner's anchored disclosure"
            );
            assert!(
                active_element_is(&page, "#failure-banner-degraded .error-details-toggle").await,
                "the toggle should hold focus while the disclosure is open"
            );
            page.evaluate::<(), ()>(
                "() => { window.__bannerToggleBefore = document.querySelector('#failure-banner-degraded .error-details-toggle'); }",
                None,
            )
            .await
            .unwrap();

            wait_for_htmx_requests(&page, "/fragment/header", 1, Duration::from_secs(8)).await;

            assert!(
                wait_for_condition_async(
                    Duration::from_secs(3),
                    Duration::from_millis(50),
                    || async {
                        page.evaluate::<(), bool>(
                            r#"() => {
                                const now = document.querySelector('#failure-banner-degraded .error-details-toggle');
                                const popover = document.getElementById('failure-banner-popover');
                                return !!now
                                    && now !== window.__bannerToggleBefore
                                    && !!popover
                                    && popover.classList.contains('open');
                            }"#,
                            None,
                        )
                        .await
                        .unwrap_or(false)
                    },
                )
                .await,
                "the header poll's out-of-band banner refresh must not collapse the open disclosure"
            );
            assert!(
                active_element_is(&page, "#failure-banner-degraded .error-details-toggle").await,
                "focus should return to the banner disclosure's toggle after the poll"
            );
        }
    })
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.20
#[tokio::test]
async fn test_dead_engine_renders_inline_error_for_pending_action() {
    with_stub_page(StubActionOutcome::Pending, |page, stub| {
        stub.stop();
        async move {
            tokio::time::sleep(Duration::from_millis(200)).await;
            submit_command(&page, "look").await;

            assert!(
                wait_for_condition_async(
                    Duration::from_secs(10),
                    Duration::from_millis(100),
                    || async {
                        read_error_disclosure(&page, "#command-form [data-error-slot]")
                            .await
                            .0
                    },
                )
                .await,
                "a dead engine should still render the form's inline error"
            );
            let (visible, message, raw) =
                read_error_disclosure(&page, "#command-form [data-error-slot]").await;
            assert!(visible, "the inline slot should be shown");
            assert!(
                message.contains("unreachable"),
                "the inline message should say the engine is unreachable, got {message:?}"
            );
            assert!(
                raw.contains("No response"),
                "a dead engine's request has no response body, got {raw:?}"
            );

            let (banner_visible, role, _) = read_banner(&page).await;
            assert!(
                banner_visible,
                "the banner should be up while the engine is down"
            );
            assert_eq!(role, "alert", "an unreachable banner reports role alert");
        }
    })
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.21
#[tokio::test]
async fn test_generation_error_clamps_to_one_line_with_popover() {
    with_stub_page(StubActionOutcome::Pending, |page, stub| {
        let status = stub.status_handle();
        async move {
            let (input_width, area_height) = measure_action_area(&page).await;

            status.set(StubStatus::Error("narration failed".to_string()));
            assert!(
                wait_for_condition_async(
                    Duration::from_secs(10),
                    Duration::from_millis(100),
                    || async {
                        page.evaluate::<(), bool>(
                            "() => !!document.querySelector('#status-display .error-disclosure')",
                            None,
                        )
                        .await
                        .unwrap_or(false)
                    },
                )
                .await,
                "the status poll should swap in the clamped error disclosure"
            );

            let (message, raw) = page
                .evaluate::<(), (String, String)>(
                    r#"() => {
                        const message = document.querySelector('#status-display .error-disclosure-message');
                        const raw = document.querySelector('#status-display .error-detail-raw');
                        return [
                            message ? message.textContent.trim() : '',
                            raw ? raw.textContent.trim() : '',
                        ];
                    }"#,
                    None,
                )
                .await
                .unwrap();
            assert!(
                !message.contains("narration failed") && !message.is_empty(),
                "the visible line must be a short user-facing message, got {message:?}"
            );
            let raw_hidden: bool = page
                .evaluate::<(), bool>(
                    "() => { const p = document.querySelector('#status-error-popover'); return !!p && !p.classList.contains('open'); }",
                    None,
                )
                .await
                .unwrap();
            assert!(raw_hidden, "the raw text must not be visible until Details opens");

            let (input_after, area_after) = measure_action_area(&page).await;
            assert_eq!(
                input_after, input_width,
                "the command input's width must not change when the error appears"
            );
            assert_eq!(
                area_after, area_height,
                "the action area's height must not change when the error appears"
            );

            page.locator("#status-display .error-details-toggle")
                .await
                .click(None)
                .await
                .unwrap();
            let (open, raw_shown) = page
                .evaluate::<(), (bool, String)>(
                    r#"() => {
                        const popover = document.getElementById('status-error-popover');
                        const raw = popover ? popover.querySelector('.error-detail-raw') : null;
                        return [!!popover && popover.classList.contains('open'), raw ? raw.textContent.trim() : ''];
                    }"#,
                    None,
                )
                .await
                .unwrap();
            assert!(open, "Details should open the anchored disclosure");
            assert_eq!(raw, raw_shown, "the disclosure shows the raw text");

            page.keyboard().press("Escape", None).await.unwrap();
            let closed: bool = page
                .evaluate::<(), bool>(
                    "() => { const p = document.getElementById('status-error-popover'); return !!p && !p.classList.contains('open'); }",
                    None,
                )
                .await
                .unwrap();
            assert!(closed, "Escape should close the disclosure");

            status.set(StubStatus::Idle);
            wait_for_status_ready(&page).await;
            let gone: bool = page
                .evaluate::<(), bool>(
                    "() => !document.querySelector('#status-display .error-disclosure')",
                    None,
                )
                .await
                .unwrap();
            assert!(gone, "the error must clear when the status returns to Ready");
        }
    })
    .await;
}

async fn measure_action_area(page: &playwright_rs::Page) -> (f64, f64) {
    page.evaluate::<(), (f64, f64)>(
        r#"() => {
            const input = document.getElementById('command-input');
            const area = document.getElementById('action-area');
            return [
                input ? input.getBoundingClientRect().width : 0,
                area ? area.getBoundingClientRect().height : 0,
            ];
        }"#,
        None,
    )
    .await
    .unwrap()
}

async fn watch_live_region(page: &playwright_rs::Page, region_id: &str) {
    page.evaluate::<String, ()>(
        r#"(id) => {
            const region = document.getElementById(id);
            window.__liveCounts = window.__liveCounts || {};
            window.__liveCounts[id] = 0;
            new MutationObserver(() => {
                window.__liveCounts[id] += 1;
            }).observe(region, { childList: true, characterData: true, subtree: true });
        }"#,
        Some(&region_id.to_string()),
    )
    .await
    .unwrap();
}

async fn live_region_changes(page: &playwright_rs::Page, region_id: &str) -> i64 {
    page.evaluate::<String, i64>(
        r#"(id) => (window.__liveCounts && window.__liveCounts[id]) || 0"#,
        Some(&region_id.to_string()),
    )
    .await
    .unwrap()
}

async fn read_live_region(page: &playwright_rs::Page, region_id: &str) -> (String, String) {
    page.evaluate::<String, (String, String)>(
        r#"(id) => {
            const region = document.getElementById(id);
            if (!region) return ['', ''];
            return [region.getAttribute('role') || '', region.textContent.trim()];
        }"#,
        Some(&region_id.to_string()),
    )
    .await
    .unwrap()
}

async fn assert_live_region_role(page: &playwright_rs::Page, region_id: &str, role: AriaRole) {
    let by_role = page.get_by_role(role, None).await;
    let by_id = page.locator(&format!("#{region_id}")).await;
    let matches = by_role.and_(&by_id).count().await.unwrap_or(0);
    assert_eq!(
        matches, 1,
        "#{region_id} must compute to the {role:?} accessibility role"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.23
#[tokio::test]
async fn test_status_changes_are_announced_once() {
    with_stub_page(StubActionOutcome::Pending, |page, stub| {
        let status = stub.status_handle();
        async move {
            assert_live_region_role(&page, "status-announcer", AriaRole::Status).await;
            assert_live_region_role(&page, "status-error-announcer", AriaRole::Alert).await;
            let (_, phase_text) = read_live_region(&page, "status-announcer").await;
            assert_eq!(phase_text, "", "the phase announcer starts empty");

            watch_live_region(&page, "status-announcer").await;
            watch_live_region(&page, "status-error-announcer").await;
            watch_htmx_requests(&page).await;

            status.set(StubStatus::Phase("narrating".to_string()));
            assert!(
                wait_for_condition_async(
                    Duration::from_secs(8),
                    Duration::from_millis(100),
                    || async {
                        read_live_region(&page, "status-announcer")
                            .await
                            .1
                            .contains("Generating narration")
                    }
                )
                .await,
                "the phase transition should be announced"
            );
            assert_eq!(
                live_region_changes(&page, "status-announcer").await,
                1,
                "the phase change should be announced exactly once"
            );

            wait_for_htmx_requests(&page, "/status/generating", 1, Duration::from_secs(8)).await;
            assert_eq!(
                live_region_changes(&page, "status-announcer").await,
                1,
                "an unchanged phase must not be re-announced on a later poll"
            );

            status.set(StubStatus::Error("narration failed".to_string()));
            assert!(
                wait_for_condition_async(
                    Duration::from_secs(8),
                    Duration::from_millis(100),
                    || async {
                        !read_live_region(&page, "status-error-announcer")
                            .await
                            .1
                            .is_empty()
                    }
                )
                .await,
                "the generation error should be announced"
            );
            let (_, error_text) = read_live_region(&page, "status-error-announcer").await;
            assert_eq!(
                error_text, "The last turn failed to generate.",
                "the error announcer must carry the short user-facing line, not the raw text"
            );

            status.set(StubStatus::Idle);
            assert!(
                wait_for_condition_async(
                    Duration::from_secs(8),
                    Duration::from_millis(100),
                    || async { read_live_region(&page, "status-announcer").await.1 == "Ready" }
                )
                .await,
                "the return to Ready should be announced"
            );
        }
    })
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.24
#[tokio::test]
async fn test_new_narration_and_options_are_announced_once() {
    with_stub_page(StubActionOutcome::Pending, |page, stub| {
        let log = stub.story_log_handle();
        let options = stub.options_handle();
        async move {
            assert_live_region_role(&page, "narration-announcer", AriaRole::Status).await;
            assert_live_region_role(&page, "options-announcer", AriaRole::Status).await;

            watch_live_region(&page, "narration-announcer").await;
            watch_live_region(&page, "options-announcer").await;
            watch_htmx_requests(&page).await;

            assert_eq!(
                read_live_region(&page, "narration-announcer").await.1,
                "",
                "loading the page must not announce the whole log"
            );

            log.append_narration("A new dawn breaks over the courtyard.");
            assert!(
                wait_for_condition_async(
                    Duration::from_secs(8),
                    Duration::from_millis(100),
                    || async {
                        read_live_region(&page, "narration-announcer")
                            .await
                            .1
                            .contains("A new dawn breaks over the courtyard.")
                    }
                )
                .await,
                "the new narration should be announced"
            );
            let narration = read_live_region(&page, "narration-announcer").await.1;
            assert!(
                !narration.contains("Welcome to the Test World"),
                "the announcement must be scoped to the changed entry, got {narration:?}"
            );
            assert_eq!(
                live_region_changes(&page, "narration-announcer").await,
                1,
                "the new narration should be announced exactly once"
            );

            wait_for_element_children(&page, "#options-dock .option-item", 3).await;
            options.serve_alternate_set(true);
            assert!(
                wait_for_condition_async(
                    Duration::from_secs(8),
                    Duration::from_millis(100),
                    || async {
                        read_live_region(&page, "options-announcer")
                            .await
                            .1
                            .contains("Search the cellar")
                    }
                )
                .await,
                "the changed option set should be announced"
            );
            assert_eq!(
                live_region_changes(&page, "options-announcer").await,
                1,
                "the changed option set should be announced exactly once"
            );

            wait_for_htmx_requests(&page, "/fragment/story-log", 2, Duration::from_secs(9)).await;
            wait_for_htmx_requests(&page, "/fragment/options-dock", 2, Duration::from_secs(9))
                .await;
            assert_eq!(
                live_region_changes(&page, "narration-announcer").await,
                1,
                "a later poll must not re-announce the narration"
            );
            assert_eq!(
                live_region_changes(&page, "options-announcer").await,
                1,
                "a later poll must not re-announce the options"
            );
        }
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

async fn open_settings_panel(page: &playwright_rs::Page) {
    page.locator(r#"[data-tab="settings"]"#)
        .await
        .click(None)
        .await
        .unwrap();
    wait_until_visible(page, "#subtab-connections", Duration::from_secs(5)).await;
}

async fn connection_form_node_is_still_present(page: &playwright_rs::Page) -> bool {
    page.evaluate::<(), bool>(
        r#"() => {
            const now = document.querySelector('.connection-form-page');
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
            window.__connectionFormBefore = document.querySelector('.connection-form-page');
            return !!window.__connectionFormBefore;
        }"#,
        None,
    )
    .await
    .unwrap()
}

async fn submit_connection_form(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>(
        r#"() => document.querySelector('.connection-form-page form').requestSubmit()"#,
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
#[tokio::test]
async fn test_failed_connection_add_keeps_the_form_and_renders_inline() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        open_settings_panel(&page).await;

        click_and_settle(&page, ".connection-add button", ".settings-panel").await;
        wait_until_visible(&page, ".connection-form-page", Duration::from_secs(5)).await;
        assert!(
            stash_connection_form_node(&page).await,
            "the Add form page should be loaded"
        );

        submit_connection_form(&page).await;

        let slot = r#".connection-form-page [data-error-slot="connection-form"]"#;
        assert!(
            wait_for_inline_slot(&page, slot).await,
            "a refused connection add should render into the form's inline slot"
        );
        let (visible, message, raw) = read_error_disclosure(&page, slot).await;
        assert!(visible, "the inline slot should be shown");
        assert!(
            message.contains("action failed"),
            "the inline message should name the failed action, got {message:?}"
        );
        assert!(
            raw.contains("Unknown LLM backend"),
            "the raw refusal belongs in the disclosure, got {raw:?}"
        );
        assert!(
            connection_form_node_is_still_present(&page).await,
            "a refused connection add must leave the form page in place"
        );
    })
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.29
#[tokio::test]
async fn test_failed_connection_edit_keeps_the_form_and_renders_inline() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        open_settings_panel(&page).await;

        click_and_settle(
            &page,
            ".connection-list .connection-row:first-child button:has-text('Edit')",
            ".settings-panel",
        )
        .await;
        wait_until_visible(&page, ".connection-form-page", Duration::from_secs(5)).await;
        assert!(
            stash_connection_form_node(&page).await,
            "the Edit form page should be loaded"
        );

        submit_connection_form(&page).await;

        let slot = r#".connection-form-page [data-error-slot="connection-form"]"#;
        assert!(
            wait_for_inline_slot(&page, slot).await,
            "a refused connection edit should render into the form's inline slot"
        );
        assert!(
            connection_form_node_is_still_present(&page).await,
            "a refused connection edit must leave the form page in place"
        );
    })
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.30
#[tokio::test]
async fn test_failed_preset_add_keeps_the_panel_and_renders_inline() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        open_prompt_presets_tab(&page).await;

        let opened = page
            .evaluate::<(), bool>(
                r#"() => {
                    window.__presetPanelBefore = document.querySelector('.prompt-presets-panel');
                    const details = document.querySelector('.preset-add');
                    if (!details) return false;
                    details.open = true;
                    return true;
                }"#,
                None,
            )
            .await
            .unwrap();
        assert!(
            opened,
            "the Prompt Presets panel with an Add form should be loaded"
        );

        page.evaluate::<(), ()>(
            r#"() => {
                const form = document.querySelector('.preset-add form');
                form.querySelector('input[name="name"]').value = 'Bad Type';
                form.requestSubmit();
            }"#,
            None,
        )
        .await
        .unwrap();

        let slot = r#".preset-add form [data-error-slot="preset-add-system"]"#;
        assert!(
            wait_for_inline_slot(&page, slot).await,
            "a refused preset add should render into the form's inline slot"
        );
        let (visible, _, raw) = read_error_disclosure(&page, slot).await;
        assert!(visible, "the inline slot should be shown");
        assert!(
            raw.contains("Invalid preset type"),
            "the raw refusal belongs in the disclosure, got {raw:?}"
        );

        let kept = page
            .evaluate::<(), bool>(
                r#"() => {
                    const now = document.querySelector('.prompt-presets-panel');
                    return !!now
                        && now === window.__presetPanelBefore
                        && !!now.querySelector('.preset-card');
                }"#,
                None,
            )
            .await
            .unwrap();
        assert!(
            kept,
            "a refused preset add must leave the panel and its cards in place"
        );
    })
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.31
#[tokio::test]
async fn test_failed_preset_edit_keeps_the_card_and_renders_inline() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        open_prompt_presets_tab(&page).await;

        let edit_selector =
            r#".preset-card:has(.card-title:text-is("My Custom Prompt")) button:has-text('Edit')"#;
        click_and_settle(&page, edit_selector, ".preset-card").await;
        wait_until_visible(&page, ".preset-card.edit-form", Duration::from_secs(5)).await;

        let stashed = page
            .evaluate::<(), bool>(
                r#"() => {
                    window.__presetCardBefore = document.querySelector('.preset-card.edit-form');
                    return !!window.__presetCardBefore;
                }"#,
                None,
            )
            .await
            .unwrap();
        assert!(stashed, "the preset edit form should be loaded");

        page.evaluate::<(), ()>(
            r#"() => document.querySelector('.preset-card.edit-form form').requestSubmit()"#,
            None,
        )
        .await
        .unwrap();

        let slot = r#"[data-error-slot="preset-edit-custom_ref"]"#;
        assert!(
            wait_for_inline_slot(&page, slot).await,
            "a failed preset edit should render into the card's inline slot"
        );
        let (visible, _, raw) = read_error_disclosure(&page, slot).await;
        assert!(visible, "the inline slot should be shown");
        assert!(
            raw.contains("preset save failure"),
            "the raw failure belongs in the disclosure, got {raw:?}"
        );

        let kept = page
            .evaluate::<(), bool>(
                r#"() => {
                    const now = document.querySelector('.preset-card.edit-form');
                    return !!now && now === window.__presetCardBefore;
                }"#,
                None,
            )
            .await
            .unwrap();
        assert!(
            kept,
            "a failed preset edit must leave the card and its edited values in place"
        );
    })
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.32
#[tokio::test]
async fn test_refused_preset_delete_keeps_the_card_and_renders_inline() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
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

        let slot = r#"[data-error-slot="preset-custom_ref"]"#;
        assert!(
            wait_for_inline_slot(&page, slot).await,
            "a refused preset delete should render into the card's inline slot"
        );
        let (visible, _, raw) = read_error_disclosure(&page, slot).await;
        assert!(visible, "the inline slot should be shown");
        assert!(
            raw.contains("mode default"),
            "the raw refusal belongs in the disclosure, got {raw:?}"
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
            "a refused preset delete must leave the card in place"
        );
    })
    .await;
}

// The two load-only tab panels fetch once per page load, so this test arms the
// stub and reloads rather than waiting for a poll cycle.
// [docs/specs/browser_dashboard.md] SCENARIO: 16.33
#[tokio::test]
async fn test_failed_panel_load_reports_inside_the_panel() {
    with_stub_page(StubActionOutcome::Pending, |page, stub| {
        let url = stub.url();
        stub.failure_handle().set_panel_loads_failing(true);
        async move {
            page.goto(&url, None).await.unwrap();

            for (tab, panel) in [("worlds", ".worlds-panel"), ("games", ".games-panel")] {
                page.locator(&format!(r#"[data-tab="{tab}"]"#))
                    .await
                    .click(None)
                    .await
                    .unwrap_or_else(|e| panic!("click tab '{tab}' failed: {e}"));

                let slot = format!("{panel} [data-error-slot]");
                assert!(
                    wait_for_condition_async(
                        Duration::from_secs(5),
                        Duration::from_millis(50),
                        || async { read_error_disclosure(&page, &slot).await.0 },
                    )
                    .await,
                    "a failed {tab} panel load must report inside the panel"
                );
                let (_, message, raw) = read_error_disclosure(&page, &slot).await;
                assert!(
                    !message.is_empty() && !message.contains("panel load failed"),
                    "the panel shows a short line, got {message:?}"
                );
                assert!(
                    raw.contains("panel load failed"),
                    "the server's own text belongs in the disclosure, got {raw:?}"
                );
            }

            let (banner_visible, _, _) = read_banner(&page).await;
            assert!(
                !banner_visible,
                "a panel load failure on a reachable engine must not raise the banner"
            );
        }
    })
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.34
#[tokio::test]
async fn test_failed_row_action_reports_in_the_row() {
    with_stub_page(StubActionOutcome::Pending, |page, _stub| async move {
        page.on_dialog(|dialog| async move { dialog.accept(None).await })
            .await
            .unwrap();

        for (tab, row, control, failure) in [
            (
                "worlds",
                ".world-item",
                ".worlds-list .world-item:first-child .btn-danger",
                "Stub world delete failure",
            ),
            (
                "games",
                ".game-item",
                ".game-item.active .btn-reset-small",
                "Stub reset failure",
            ),
        ] {
            page.locator(&format!(r#"[data-tab="{tab}"]"#))
                .await
                .click(None)
                .await
                .unwrap_or_else(|e| panic!("click tab '{tab}' failed: {e}"));

            let rows_before = page.query_selector_all(row).await.unwrap_or_default().len();
            page.locator(control)
                .await
                .click(None)
                .await
                .unwrap_or_else(|e| panic!("click '{control}' failed: {e}"));

            let slot = format!("{row} > [data-error-slot]");
            assert!(
                wait_for_condition_async(
                    Duration::from_secs(5),
                    Duration::from_millis(50),
                    || async { read_error_disclosure(&page, &slot).await.0 },
                )
                .await,
                "a failed action in a {tab} row must report in that row"
            );
            let (_, message, raw) = read_error_disclosure(&page, &slot).await;
            assert!(
                !message.is_empty() && !message.contains(failure),
                "the row shows a short line, got {message:?}"
            );
            assert!(
                raw.contains(failure),
                "the server's own text belongs in the disclosure, got {raw:?}"
            );
            assert_eq!(
                page.query_selector_all(row).await.unwrap_or_default().len(),
                rows_before,
                "a failed row action must leave the row in place"
            );
        }

        assert!(
            !read_banner(&page).await.0,
            "a failed row action on a reachable engine must not raise the banner"
        );
    })
    .await;
}
