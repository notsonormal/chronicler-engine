//! Stub-browser tests for the failure display: the banner, the in-place slots and the clamped disclosures. Tagged against `docs/specs/browser_dashboard.md`.

// The engine's failing action route answers 500, so htmx fires
// `htmx:responseError` without a dispatched event.

use std::time::Duration;

use chronicler_engine::domain::model::state::generation_status::GenerationFailureKind;

use super::*;
use super::support::{StubRunner, active_element_is, poll_now, read_banner, read_command_input};

/// A failed action leaves Send locked until the next idle poll and swaps nothing
/// on its 500, so this submits the form directly rather than clicking Send.
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

// [docs/specs/browser_dashboard.md] SCENARIO: 16.22
async fn check_degraded_role_raises_status_banner(page: playwright_rs::Page, stub: StubServer) {
    let failures = stub.failure_handle();
    failures.set_header_degraded(true);
    poll_now(&page, "#header").await;
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
    assert!(
        text.contains("(engine-wide role health)"),
        "the banner should state the scope of role health, got {text:?}"
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

// [docs/specs/browser_dashboard.md] SCENARIO: 16.18
async fn check_failed_poll_keeps_region_and_marks_banner(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let failures = stub.failure_handle();
    let before: String = page
        .evaluate::<(), String>("() => document.getElementById('story-log').innerHTML", None)
        .await
        .unwrap();

    failures.set_polls_failing(true);
    poll_now(&page, "#story-log").await;
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
        .evaluate::<(), String>("() => document.getElementById('story-log').innerHTML", None)
        .await
        .unwrap();
    assert_eq!(
        after, before,
        "a failed poll must leave the story log's last good content in place"
    );

    failures.set_polls_failing(false);
    poll_now(&page, "#story-log").await;
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

// [docs/specs/browser_dashboard.md] SCENARIO: 16.19
async fn check_failed_action_renders_inline_slot(page: playwright_rs::Page, _stub: StubServer) {
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
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.25
async fn check_failed_non_poll_action_does_not_raise_unreachable_banner(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
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
                read_error_disclosure(&page, "#game-posture-controls [data-error-slot]")
                    .await
                    .0
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
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.26
async fn check_status_details_popover_survives_a_poll(page: playwright_rs::Page, stub: StubServer) {
    let status = stub.status_handle();
    status.set(StubStatus::Error(
        GenerationFailureKind::Other,
        "narration failed".to_string(),
    ));
    poll_now(&page, "#status-display").await;
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

    poll_now(&page, "#status-display").await;

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

// [docs/specs/browser_dashboard.md] SCENARIO: 16.27
async fn check_banner_details_popover_survives_a_header_poll(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let failures = stub.failure_handle();
    failures.set_header_degraded(true);
    poll_now(&page, "#header").await;
    assert!(
        wait_for_condition_async(
            Duration::from_secs(10),
            Duration::from_millis(100),
            || async { read_banner(&page).await.0 },
        )
        .await,
        "the degraded-role banner should become visible"
    );

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

    poll_now(&page, "#header").await;

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

// [docs/specs/browser_dashboard.md] SCENARIO: 16.20
async fn check_dead_engine_renders_inline_error_for_pending_action(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let addr = stub.addr();
    stub.stop();
    wait_until_port_closed(addr).await;
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

// [docs/specs/browser_dashboard.md] SCENARIO: 16.40
async fn check_failed_confirm_reports_on_the_command_form_slot(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let lifecycle = stub.lifecycle_handle();
    let addr = stub.addr();
    submit_command(&page, "look at the casle").await;
    wait_until_visible(&page, ".text-check-preview", Duration::from_secs(5)).await;

    lifecycle.stop();
    wait_until_port_closed(addr).await;

    page.locator(".text-check-preview form .btn-cyan")
        .await
        .click(None)
        .await
        .unwrap();
    wait_until_hidden(&page, ".text-check-preview", Duration::from_secs(5)).await;

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
        "a confirm that cannot reach the engine must report on the command form's slot"
    );
    let (visible, message, raw) =
        read_error_disclosure(&page, "#command-form [data-error-slot]").await;
    assert!(visible, "the command form's slot should be shown");
    assert!(
        message.contains("unreachable"),
        "the slot should say the engine is unreachable, got {message:?}"
    );
    assert!(
        raw.contains("No response"),
        "a dead engine's request has no response body, got {raw:?}"
    );
    assert_eq!(
        read_command_input(&page).await,
        "look at the casle",
        "a confirm the engine never took must keep the typed command"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.40
async fn check_refused_confirm_reports_on_the_command_form_slot(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    submit_command(&page, "look at the casle").await;
    wait_until_visible(&page, ".text-check-preview", Duration::from_secs(5)).await;

    page.locator(".text-check-preview form .btn-cyan")
        .await
        .click(None)
        .await
        .unwrap();
    wait_until_hidden(&page, ".text-check-preview", Duration::from_secs(5)).await;

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
        "a refused confirm must report on the command form's slot"
    );
    let (visible, message, _) =
        read_error_disclosure(&page, "#command-form [data-error-slot]").await;
    assert!(visible, "the command form's slot should be shown");
    assert!(
        message.contains("action failed"),
        "the short line names the failed action, got {message:?}"
    );
    assert!(
        !message.contains("Failed to process action"),
        "a 500's server text stays behind the disclosure, got {message:?}"
    );
    let (_, _, raw) = read_error_disclosure(&page, "#command-form [data-error-slot]").await;
    assert!(
        raw.contains("Failed to process action"),
        "the server's own text belongs in the disclosure, got {raw:?}"
    );
    assert_eq!(
        read_command_input(&page).await,
        "look at the casle",
        "a refused confirm must keep the typed command"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.21
async fn check_generation_error_clamps_to_one_line_with_popover(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let status = stub.status_handle();
    let (input_width, area_height) = measure_action_area(&page).await;

    status.set(StubStatus::Error(
        GenerationFailureKind::Other,
        "narration failed".to_string(),
    ));
    poll_now(&page, "#status-display").await;
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
    assert!(
        raw_hidden,
        "the raw text must not be visible until Details opens"
    );

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
    poll_now(&page, "#status-display").await;
    wait_for_status_ready(&page).await;
    let gone: bool = page
        .evaluate::<(), bool>(
            "() => !document.querySelector('#status-display .error-disclosure')",
            None,
        )
        .await
        .unwrap();
    assert!(
        gone,
        "the error must clear when the status returns to Ready"
    );
}

fn carries_state(class_name: &str, state: &str) -> bool {
    class_name.split_whitespace().any(|class| class == state)
}

// The returned tuple: the container's class, its inherited text colour, the
// palette's error colour, and whether the short line fits the column unclipped.
async fn read_status_state(page: &playwright_rs::Page) -> (String, String, String, bool) {
    page.evaluate::<(), (String, String, String, bool)>(
        r#"() => {
            const display = document.getElementById('status-display');
            const text = display.querySelector('.error-disclosure-message, .status');
            const probe = document.createElement('span');
            probe.style.color = 'var(--color-accent-red)';
            document.body.appendChild(probe);
            const paletteError = getComputedStyle(probe).color;
            probe.remove();
            const message = display.querySelector('.error-disclosure-message');
            let readable = false;
            if (message) {
                const box = message.getBoundingClientRect();
                readable =
                    box.right <= display.getBoundingClientRect().right + 1 &&
                    message.scrollWidth <= message.clientWidth + 1 &&
                    message.scrollHeight <= message.clientHeight + 1;
            }
            return [
                display.className,
                text ? getComputedStyle(text).color : '',
                paletteError,
                readable,
            ];
        }"#,
        None,
    )
    .await
    .unwrap()
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.41
async fn check_generation_error_takes_the_error_class_and_reads_without_details(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let status = stub.status_handle();
    let (ready_class, ready_colour, _, _) = read_status_state(&page).await;
    assert!(
        carries_state(&ready_class, "ready") && !carries_state(&ready_class, "error"),
        "an idle dashboard must carry the ready state, got {ready_class:?}"
    );

    status.set(StubStatus::Error(
        GenerationFailureKind::PromptTooLong,
        "narration failed".to_string(),
    ));
    poll_now(&page, "#status-display").await;
    assert!(
        wait_for_condition_async(
            Duration::from_secs(10),
            Duration::from_millis(100),
            || async { read_status_state(&page).await.0.contains("error") },
        )
        .await,
        "a generation error must put the status display into its error state"
    );

    let (error_class, error_colour, palette_error, fits) = read_status_state(&page).await;
    assert!(
        carries_state(&error_class, "error") && !carries_state(&error_class, "ready"),
        "the error state must replace the ready state, not sit under it, got {error_class:?}"
    );
    assert_ne!(
        error_colour, ready_colour,
        "the error must not be drawn in the Ready colour"
    );
    assert_eq!(
        error_colour, palette_error,
        "the error must be drawn in the palette's error colour"
    );
    assert!(
        fits,
        "the longest clamped line must be drawn inside the status column, unclipped, without opening Details"
    );

    status.set(StubStatus::Idle);
    poll_now(&page, "#status-display").await;
    wait_for_status_ready(&page).await;
    let (recovered_class, recovered_colour, _, _) = read_status_state(&page).await;
    assert!(
        carries_state(&recovered_class, "ready") && !carries_state(&recovered_class, "error"),
        "returning to Ready must restore the ready state, got {recovered_class:?}"
    );
    assert_ne!(
        recovered_colour, palette_error,
        "Ready must not keep the error's colour"
    );
}

// The two load-only tab panels fetch once per page load, so this test arms the
// stub and reloads rather than waiting for a poll cycle.
// [docs/specs/browser_dashboard.md] SCENARIO: 16.33
async fn check_failed_panel_load_reports_inside_the_panel(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let url = stub.url();
    stub.failure_handle().set_panel_loads_failing(true);
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

// [docs/specs/browser_dashboard.md] SCENARIO: 16.34
async fn check_failed_row_action_reports_in_the_row(page: playwright_rs::Page, _stub: StubServer) {
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
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.37
async fn check_dead_engine_keeps_the_typed_command(page: playwright_rs::Page, stub: StubServer) {
    let lifecycle = stub.lifecycle_handle();
    lifecycle.stop();
    wait_until_port_closed(lifecycle.addr()).await;
    submit_command(&page, "look around").await;

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
    assert_eq!(
        read_command_input(&page).await,
        "look around",
        "a send that cannot reach the engine must keep the typed command"
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.38
async fn check_engine_recovery_clears_the_inline_error_and_banner(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let lifecycle = stub.lifecycle_handle();
    lifecycle.stop();
    wait_until_port_closed(lifecycle.addr()).await;
    submit_command(&page, "look around").await;

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
        "a dead engine must render the form's inline error before the recovery"
    );

    lifecycle.restart().await;
    poll_now(&page, "#story-log").await;
    assert!(
        wait_for_condition_async(
            Duration::from_secs(10),
            Duration::from_millis(100),
            || async {
                !read_error_disclosure(&page, "#command-form [data-error-slot]")
                    .await
                    .0
            },
        )
        .await,
        "the engine's recovery must clear the inline error"
    );
    assert!(
        !read_banner(&page).await.0,
        "the inline error and the banner must clear together"
    );
    assert_eq!(
        read_command_input(&page).await,
        "look around",
        "the recovery must not drop the typed command"
    );
}

#[tokio::test]
async fn run_failure_display_banner_checks() {
    let mut runner = StubRunner::launch().await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_degraded_role_raises_status_banner,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_failed_poll_keeps_region_and_marks_banner,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_status_details_popover_survives_a_poll,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_banner_details_popover_survives_a_header_poll,
        )
        .await;
    runner.finish().await;
}

#[tokio::test]
async fn run_failure_display_slot_checks() {
    let mut runner = StubRunner::launch().await;
    runner
        .run(
            StubActionOutcome::Error,
            check_failed_action_renders_inline_slot,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_failed_non_poll_action_does_not_raise_unreachable_banner,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_dead_engine_renders_inline_error_for_pending_action,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_failed_confirm_reports_on_the_command_form_slot,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Error,
            check_refused_confirm_reports_on_the_command_form_slot,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_generation_error_clamps_to_one_line_with_popover,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_generation_error_takes_the_error_class_and_reads_without_details,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_failed_panel_load_reports_inside_the_panel,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_failed_row_action_reports_in_the_row,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_dead_engine_keeps_the_typed_command,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_engine_recovery_clears_the_inline_error_and_banner,
        )
        .await;
    runner.finish().await;
}
