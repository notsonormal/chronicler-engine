//! Browser dashboard-chrome tests: static command form, status display, error toast. Tagged against `docs/specs/browser_dashboard.md`; the stdout-tee health check is a named exemption.

use std::time::Duration;

use playwright_rs::expect;

use super::*;

// [docs/specs/browser_dashboard.md] SCENARIO: 16.5
#[tokio::test]
async fn test_form_stays_static_after_submission() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            let form_id_before: String = page
                .evaluate::<(), String>("document.querySelector('#command-form')?.id || ''", None)
                .await
                .unwrap();

            page.evaluate::<(), ()>(
                "(() => {
                const input = document.querySelector('#command-form input');
                if (input) input.value = 'look';
                const form = document.querySelector('#command-form');
                if (form) form.requestSubmit();
            })()",
                None,
            )
            .await
            .unwrap();

            let _ = wait_for_element_children(&page, "#story-log .log-entry", 2).await;

            let form_id_after: String = page
                .evaluate::<(), String>("document.querySelector('#command-form')?.id || ''", None)
                .await
                .unwrap();

            assert_eq!(
                form_id_before, form_id_after,
                "Form should stay in DOM (static shell)"
            );
        },
    )
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.6
#[tokio::test]
async fn test_status_updates_during_generation() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            send_action(&page, "wait").await;

            let status_locator = page.locator("#status-display").await;
            let _ = expect(status_locator)
                .with_timeout(Duration::from_millis(500))
                .not()
                .to_contain_text("Ready")
                .await;

            let status_text = page
                .locator("#status-display")
                .await
                .inner_text()
                .await
                .unwrap_or_default();
            assert!(
                status_text.contains("Thinking")
                    || status_text.contains("Narrating")
                    || status_text.contains("Generating")
                    || status_text.contains("Quantifying"),
                "Status should show generating state, got: {status_text}"
            );
        },
    )
    .await;
}

// [docs/specs/browser_dashboard.md] SCENARIO: 16.7
#[tokio::test]
async fn test_error_toast_on_action_failure() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            // Dispatch the `htmx:beforeSwap` event that a 500 from POST /action
            // produces. The app code under test is the body-level listener that
            // calls showError on isError — htmx's 500→isError=true mapping is
            // htmx's contract, not ours.
            //
            // serverResponse carries HTML tags so the assertion exercises the
            // handler's tag-stripping path (`response.replace(/<[^>]*>/g, "")`)
            // and proves the toast text is *derived from* the response body,
            // not just non-empty.
            //
            // Why synthetic instead of route.fulfill: this playwright-rs
            // version's route.fulfill is empirically broken for BOTH status and
            // body (a fulfill with status 500 + non-empty body arrives at the
            // page as status 200 with empty body — verified via a fetch probe),
            // so we cannot produce a real 500 through route interception, and
            // the real server has no path that returns 500 from /action/check
            // without production-code changes. Skip the 5s auto-hide
            // setTimeout — asserting it would burn 5s and add flake; the
            // setTimeout is trivial JS.
            page.evaluate::<(), ()>(
                r#"(() => {
                    const evt = new CustomEvent('htmx:beforeSwap', {
                        bubbles: true,
                        cancelable: true,
                        detail: {
                            isError: true,
                            serverResponse: '<p>Internal server error</p>',
                            target: document.getElementById('action-area'),
                        },
                    });
                    document.body.dispatchEvent(evt);
                })()"#,
                None,
            )
            .await
            .unwrap();

            // Toast update is synchronous in showError (no animation frame).
            let toast_state: (bool, String) = page
                .evaluate::<(), (bool, String)>(
                    r#"(() => {
                        const el = document.getElementById('error-notification');
                        if (!el) return [false, ''];
                        return [el.classList.contains('visible'), el.textContent || ''];
                    })()"#,
                    None,
                )
                .await
                .unwrap();

            assert!(
                toast_state.0,
                "#error-notification should gain .visible class on a 500 htmx:beforeSwap event"
            );
            assert_eq!(
                toast_state.1, "Internal server error",
                "#error-notification should display the response body with tags stripped, got {:?}",
                toast_state.1
            );
        },
    )
    .await;
}

// Infrastructure health check, not a spec scenario (no tag). The engine's
// stdout tee is the artifact every failure dump reads; if it goes missing,
// every diagnostic in this tier goes dark with it.
#[tokio::test]
async fn test_engine_output_teed_to_file() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |_page, port| async move {
            let tee_path = format!("tmp/test_server_logs/{port}_stdout.log");
            let content = std::fs::read_to_string(&tee_path)
                .unwrap_or_else(|e| panic!("engine stdout tee missing at {tee_path}: {e}"));
            assert!(
                content.contains("HTMX Dashboard running"),
                "tee should contain the engine boot line, tail: {}",
                content.lines().last().unwrap_or("")
            );
        },
    )
    .await;
}
