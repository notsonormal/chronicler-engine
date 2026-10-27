//! Stub-browser tests for the failure display: the banner, the in-place slots and the clamped disclosures. Tagged against `docs/specs/browser_dashboard.md`.

// A failing action route answers 500, so htmx fires `htmx:responseError` with no
// dispatched event.

use std::time::Duration;

use chronicler_engine::domain::model::change::ChangeKind;
use chronicler_engine::domain::model::state::generation_status::GenerationFailureKind;
use playwright_rs::Viewport;

use super::*;
use super::support::{
    StubRunner, active_element_is, ask_once_without_waiting, read_banner, read_command_input,
    start_current,
};

/// A failed action leaves Send locked until the next idle status load and swaps nothing
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
            return !!popover && !popover.hidden;
        }"#,
        Some(&popover_id.to_string()),
    )
    .await
    .unwrap_or(false)
}

#[derive(serde::Deserialize)]
struct InlineErrorSizes {
    #[serde(rename = "sendHeight")]
    send_height: f64,
    #[serde(rename = "toggleHeight")]
    toggle_height: f64,
    #[serde(rename = "sendFontSize")]
    send_font_size: f64,
    #[serde(rename = "toggleFontSize")]
    toggle_font_size: f64,
    #[serde(rename = "messageFontSize")]
    message_font_size: f64,
}

async fn read_inline_error_sizes(page: &playwright_rs::Page) -> InlineErrorSizes {
    page.evaluate::<(), InlineErrorSizes>(
        r#"() => {
            const submit = document.getElementById('submit-btn');
            const toggle = document.querySelector(
                '#command-form .inline-error-slot .error-details-toggle');
            const message = document.querySelector(
                '#command-form .inline-error-slot .error-disclosure-message');
            const size = (el) => (el ? parseFloat(getComputedStyle(el).fontSize) : 0);
            return {
                sendHeight: submit ? submit.getBoundingClientRect().height : 0,
                toggleHeight: toggle ? toggle.getBoundingClientRect().height : 0,
                sendFontSize: size(submit),
                toggleFontSize: size(toggle),
                messageFontSize: size(message),
            };
        }"#,
        None,
    )
    .await
    .unwrap()
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
    load_screen_now(&page).await;
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
                        return [!!popover && !popover.hidden, popover ? popover.textContent : ''];
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
async fn check_failed_screen_load_leaves_the_parts_unchanged_and_shows_the_screen_error(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let changes = stub.changes_handle();
    start_current(&page).await;
    let before: String = page
        .evaluate::<(), String>("() => document.getElementById('story-log').innerHTML", None)
        .await
        .unwrap();

    changes.set_loads_failing(true);
    stub.story_log_handle()
        .append_narration("A line that arrives late.");
    changes.bump(ChangeKind::STORY);
    ask_once_without_waiting(&page).await;
    let shown = || async { read_error_disclosure(&page, "#screen-error").await };
    assert!(
        wait_for_condition_async(
            Duration::from_secs(10),
            Duration::from_millis(50),
            || async { shown().await.0 }
        )
        .await,
        "a failed screen load should show the screen error"
    );
    let (_, line, raw) = shown().await;
    assert!(line.contains("GET /screen answered 500"), "got {line:?}");
    assert!(
        raw.contains("read failed"),
        "the detail names the failure, got {raw:?}"
    );
    let (banner_visible, _, _) = read_banner(&page).await;
    assert!(
        !banner_visible,
        "the engine answered, so the banner must not report it unreachable"
    );
    let after: String = page
        .evaluate::<(), String>("() => document.getElementById('story-log').innerHTML", None)
        .await
        .unwrap();
    assert_eq!(
        after, before,
        "a failed load must leave the story log's last good content in place"
    );

    changes.set_loads_failing(false);
    ask_once(&page).await;
    assert!(
        !read_error_disclosure(&page, "#screen-error").await.0,
        "a later successful load should clear the screen error"
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

    let sizes = read_inline_error_sizes(&page).await;
    assert!(
        sizes.toggle_height > 0.0 && sizes.toggle_height < sizes.send_height,
        "the inline error's Details control must stay a small inline button, not a second \
         Send button ({}px against Send's {}px)",
        sizes.toggle_height,
        sizes.send_height
    );
    assert!(
        sizes.message_font_size > 0.0
            && sizes.message_font_size < sizes.send_font_size
            && (sizes.message_font_size - sizes.toggle_font_size).abs() < 0.01,
        "the inline error's message and its Details control share the failure size \
         ({}px and {}px), below the form's own {}px",
        sizes.message_font_size,
        sizes.toggle_font_size,
        sizes.send_font_size
    );
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.25
async fn check_failed_action_does_not_raise_unreachable_banner(
    page: playwright_rs::Page,
    _stub: StubServer,
) {
    open_games_tab(&page).await;
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
        "a failed action must report on its own surface"
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

    let part_kept = page
        .evaluate::<(), bool>(
            r#"() => {
                    const now = document.getElementById('game-posture-controls');
                    return !!now && now === window.__postureBefore;
                }"#,
            None,
        )
        .await
        .unwrap();
    assert!(part_kept, "a failed action must leave its part in place");
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.26
async fn check_status_details_popover_survives_a_load(page: playwright_rs::Page, stub: StubServer) {
    let status = stub.status_handle();
    status.set(StubStatus::Error(
        GenerationFailureKind::Other,
        "narration failed".to_string(),
    ));
    load_screen_now(&page).await;
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
        "the status load should swap in the clamped error disclosure"
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

    load_screen_now(&page).await;

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
                                    && now === window.__statusToggleBefore
                                    && !!popover
                                    && !popover.hidden;
                            }"#,
                            None,
                        )
                        .await
                        .unwrap_or(false)
                    },
                )
                .await,
                "a load must keep the disclosure and not collapse it"
            );
    assert!(
        active_element_is(&page, "#status-display .error-details-toggle").await,
        "focus should return to the disclosure's toggle after the load"
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
async fn check_banner_details_popover_survives_a_banner_load(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let failures = stub.failure_handle();
    failures.set_header_degraded(true);
    load_screen_now(&page).await;
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

    load_screen_now(&page).await;

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
                                    && now === window.__bannerToggleBefore
                                    && !!popover
                                    && !popover.hidden;
                            }"#,
                            None,
                        )
                        .await
                        .unwrap_or(false)
                    },
                )
                .await,
                "a load of the banner must keep its open disclosure, on the same node"
            );
    assert!(
        active_element_is(&page, "#failure-banner-degraded .error-details-toggle").await,
        "focus should return to the banner disclosure's toggle after the load"
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
    load_screen_now(&page).await;
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
        "the status load should swap in the clamped error disclosure"
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
                    "() => { const p = document.querySelector('#status-error-popover'); return !!p && p.hidden; }",
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
                        return [!!popover && !popover.hidden, raw ? raw.textContent.trim() : ''];
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
                    "() => { const p = document.getElementById('status-error-popover'); return !!p && p.hidden; }",
                    None,
                )
                .await
                .unwrap();
    assert!(closed, "Escape should close the disclosure");

    status.set(StubStatus::Idle);
    load_screen_now(&page).await;
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

#[derive(serde::Deserialize)]
struct StatusState {
    #[serde(rename = "textColour")]
    text_colour: String,
    #[serde(rename = "paletteError")]
    palette_error: String,
    fits: bool,
}

async fn read_status_state(page: &playwright_rs::Page) -> StatusState {
    let script = r#"() => {
            const display = document.getElementById('status-display');
            const text = display.querySelector('.error-disclosure-message, .status');
            const probe = document.createElement('span');
            probe.style.color = 'var(--color-accent-red)';
            document.body.appendChild(probe);
            const paletteError = getComputedStyle(probe).color;
            probe.remove();
            __MEASURE_CLAMPED_TEXT__
            const measured = clampedText(display);
            let readable = false;
            if (measured) {
                readable =
                    measured.overhang <= 1 &&
                    measured.scrollWidth <= measured.clientWidth + 1 &&
                    measured.neededHeight <= measured.boxHeight + 1;
            }
            return {
                textColour: text ? getComputedStyle(text).color : '',
                paletteError,
                fits: readable,
            };
        }"#
    .replace("__MEASURE_CLAMPED_TEXT__", MEASURE_CLAMPED_TEXT);
    page.evaluate::<(), StatusState>(script.as_str(), None)
        .await
        .unwrap()
}

/// Measuring the clamp's own box misses the lines it cuts, so the height the
/// sentence needs has to come off that same element — it keeps the width and font
/// an offscreen copy would only approximate.
const MEASURE_CLAMPED_TEXT: &str = r#"
            const clampedText = (display) => {
                const text = display.querySelector('.error-disclosure-text');
                if (!text) return null;
                const style = getComputedStyle(text);
                const box = text.getBoundingClientRect();
                const measured = {
                    text: text.textContent.trim(),
                    boxHeight: box.height,
                    lineHeight: parseFloat(style.lineHeight),
                    scrollWidth: text.scrollWidth,
                    clientWidth: text.clientWidth,
                    overhang: box.right - display.getBoundingClientRect().right,
                };
                text.setAttribute(
                    'style',
                    'display: block; overflow: visible; max-height: none; -webkit-line-clamp: none;',
                );
                measured.neededHeight = text.getBoundingClientRect().height;
                text.removeAttribute('style');
                return measured;
            };
"#;

// [docs/specs/browser_dashboard.md] SCENARIO: 16.41
async fn check_generation_error_is_drawn_in_the_error_colour_and_reads_without_details(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let status = stub.status_handle();
    let ready = read_status_state(&page).await;
    assert_ne!(
        ready.text_colour, ready.palette_error,
        "Ready is not drawn in the error colour"
    );

    status.set(StubStatus::Error(
        GenerationFailureKind::PromptTooLong,
        "narration failed".to_string(),
    ));
    load_screen_now(&page).await;
    assert!(
        wait_for_condition_async(
            Duration::from_secs(10),
            Duration::from_millis(100),
            || async {
                let state = read_status_state(&page).await;
                state.text_colour == state.palette_error
            },
        )
        .await,
        "a generation error must be drawn in the palette's error colour"
    );

    let error = read_status_state(&page).await;
    assert_ne!(
        error.text_colour, ready.text_colour,
        "the error must not be drawn in the Ready colour"
    );
    assert!(
        error.fits,
        "the longest clamped line must be drawn inside the status column, unclipped, without opening Details"
    );

    status.set(StubStatus::Idle);
    load_screen_now(&page).await;
    wait_for_status_ready(&page).await;
    let recovered = read_status_state(&page).await;
    assert_ne!(
        recovered.text_colour, error.palette_error,
        "Ready must not keep the error's colour"
    );
}

/// `needed_height` is the height the sentence needs once the clamp is lifted off it.
#[derive(serde::Deserialize)]
struct ClampedLine {
    text: String,
    #[serde(rename = "boxHeight")]
    box_height: f64,
    #[serde(rename = "neededHeight")]
    needed_height: f64,
    #[serde(rename = "lineHeight")]
    line_height: f64,
    #[serde(rename = "scrollWidth")]
    scroll_width: f64,
    #[serde(rename = "clientWidth")]
    client_width: f64,
    overhang: f64,
}

async fn read_clamped_line(page: &playwright_rs::Page) -> ClampedLine {
    let script = r#"() => {
            const display = document.getElementById('status-display');
            __MEASURE_CLAMPED_TEXT__
            const measured = clampedText(display);
            if (!measured) {
                return {
                    text: '',
                    boxHeight: 0,
                    neededHeight: 0,
                    lineHeight: 0,
                    scrollWidth: 0,
                    clientWidth: 0,
                    overhang: 0,
                };
            }
            return {
                text: measured.text,
                boxHeight: measured.boxHeight,
                neededHeight: measured.neededHeight,
                lineHeight: measured.lineHeight,
                scrollWidth: measured.scrollWidth,
                clientWidth: measured.clientWidth,
                overhang: measured.overhang,
            };
        }"#
    .replace("__MEASURE_CLAMPED_TEXT__", MEASURE_CLAMPED_TEXT);
    page.evaluate::<(), ClampedLine>(script.as_str(), None)
        .await
        .unwrap()
}

/// A viewport change reflows asynchronously, so the clamp check waits for the
/// new width before it measures anything.
async fn wait_for_viewport_width(page: &playwright_rs::Page, width: u32) {
    assert!(
        wait_for_condition_async(
            Duration::from_secs(5),
            Duration::from_millis(50),
            || async { (read_viewport_width(page).await - f64::from(width)).abs() < 1.0 },
        )
        .await,
        "the viewport should reflow to {width}px before the check measures"
    );
}

async fn read_viewport_width(page: &playwright_rs::Page) -> f64 {
    page.evaluate::<(), f64>("() => window.innerWidth", None)
        .await
        .unwrap_or_default()
}

/// The status column clamps its sentence to two lines inside fixed widths, so
/// every failure sentence has to fit at every supported width.
// [docs/specs/browser_dashboard.md] SCENARIO: 16.43
async fn check_every_failure_sentence_reads_in_full(page: playwright_rs::Page, stub: StubServer) {
    let status = stub.status_handle();
    for (width, height) in [(1280u32, 800u32), (1024, 700)] {
        page.set_viewport_size(Viewport { width, height })
            .await
            .unwrap();
        wait_for_viewport_width(&page, width).await;

        let (baseline_input, baseline_area) = measure_action_area(&page).await;
        for &kind in GenerationFailureKind::ALL {
            // The missing-data line carries what is missing, so this check uses a
            // name a real failure carries; the other kinds ignore the raw text.
            let raw = match kind {
                GenerationFailureKind::MissingData => "the story".to_string(),
                _ => format!("stub raw failure for {kind:?}"),
            };
            let sentence = kind.line(&raw);
            status.set(StubStatus::Error(kind, raw));
            load_screen_now(&page).await;
            assert!(
                wait_for_condition_async(
                    Duration::from_secs(10),
                    Duration::from_millis(50),
                    || async { read_clamped_line(&page).await.text == sentence },
                )
                .await,
                "the {kind:?} line should reach the status display at {width}px"
            );

            let line = read_clamped_line(&page).await;
            assert_eq!(
                line.text, sentence,
                "the status display must draw the {kind:?} sentence at {width}px"
            );
            assert!(
                line.needed_height <= line.box_height + 1.0,
                "the {kind:?} sentence is clipped at {width}px: it needs {}px in {}px",
                line.needed_height,
                line.box_height
            );
            assert!(
                line.box_height <= 2.0 * line.line_height + 1.0,
                "the {kind:?} sentence must be drawn inside the clamp's two lines at {width}px: \
                 {}px against {}px a line",
                line.box_height,
                line.line_height
            );
            assert!(
                line.scroll_width <= line.client_width + 1.0,
                "the {kind:?} sentence overflows its column at {width}px: {}px in {}px",
                line.scroll_width,
                line.client_width
            );
            assert!(
                line.overhang <= 1.0,
                "the {kind:?} sentence is drawn {}px past the status column at {width}px",
                line.overhang
            );

            let (input_after, area_after) = measure_action_area(&page).await;
            assert_eq!(
                input_after, baseline_input,
                "the command input must not resize for the {kind:?} sentence at {width}px"
            );
            assert_eq!(
                area_after, baseline_area,
                "the action area must not resize for the {kind:?} sentence at {width}px"
            );
        }
    }
}

// A panel loads when a number of its kinds moves, and a panel in a hidden section
// waits until its section opens, so this test arms the stub and reloads rather
// than waiting for a load that no number would start.
// [docs/specs/browser_dashboard.md] SCENARIO: 16.33
async fn check_failed_panel_load_reports_in_the_screen_error(
    page: playwright_rs::Page,
    stub: StubServer,
) {
    let url = stub.url();
    stub.failure_handle().set_panel_loads_failing(true);
    page.goto(&url, None).await.unwrap();
    wait_for_screen_current(&page).await;

    for (tab, panel) in [
        ("worlds", ".worlds-panel"),
        ("games", ".games-panel"),
        ("settings", ".settings-panel"),
        ("prompt-presets", ".prompt-presets-panel"),
    ] {
        page.locator(&format!(r#"[data-tab="{tab}"]"#))
            .await
            .click(None)
            .await
            .unwrap_or_else(|e| panic!("click tab '{tab}' failed: {e}"));

        wait_for_error_disclosure(
            &page,
            "#screen-error",
            "a failed {tab} panel load must report in the screen error",
        )
        .await;
        let (_, message, raw) = read_error_disclosure(&page, "#screen-error").await;
        assert!(
            message.contains("GET /screen answered 500") && !message.contains("panel load failed"),
            "the slot shows a short line, got {message:?}"
        );
        assert!(
            raw.contains("panel load failed"),
            "the server's own text belongs in the disclosure, got {raw:?}"
        );
        assert!(
            page.evaluate::<String, bool>(
                "(selector) => document.querySelector(selector).childNodes.length === 0",
                Some(&panel.to_string()),
            )
            .await
            .unwrap(),
            "the {tab} part never loaded, so it holds nothing"
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

        // A section loads its panel when it opens, so the rows exist once the
        // control does.
        wait_until_visible(&page, control, Duration::from_secs(5)).await;
        let rows_before = page.query_selector_all(row).await.unwrap_or_default().len();
        page.locator(control)
            .await
            .click(None)
            .await
            .unwrap_or_else(|e| panic!("click '{control}' failed: {e}"));

        let slot = format!("{row} > [data-error-slot]");
        wait_for_error_disclosure(
            &page,
            &slot,
            "a failed action in a {tab} row must report in that row",
        )
        .await;
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
        assert!(
            active_element_is(&page, control).await,
            "a failed {tab} action must leave focus on the control that asked for it"
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
    load_screen_now(&page).await;
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
            check_failed_screen_load_leaves_the_parts_unchanged_and_shows_the_screen_error,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_status_details_popover_survives_a_load,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_banner_details_popover_survives_a_banner_load,
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
            check_failed_action_does_not_raise_unreachable_banner,
        )
        .await;
    runner
        .run_without_convergence(
            "the engine stays stopped",
            StubActionOutcome::Pending,
            check_dead_engine_renders_inline_error_for_pending_action,
        )
        .await;
    runner
        .run_without_convergence(
            "the engine stays stopped",
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
            check_generation_error_is_drawn_in_the_error_colour_and_reads_without_details,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_every_failure_sentence_reads_in_full,
        )
        .await;
    runner
        .run_without_convergence(
            "the panel loads keep failing",
            StubActionOutcome::Pending,
            check_failed_panel_load_reports_in_the_screen_error,
        )
        .await;
    runner
        .run(
            StubActionOutcome::Pending,
            check_failed_row_action_reports_in_the_row,
        )
        .await;
    runner
        .run_without_convergence(
            "the engine stays stopped",
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
