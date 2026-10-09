//! Browser dashboard-chrome tests: static command form, status display. Tagged against `docs/specs/browser_dashboard.md`.

use std::time::Duration;

use super::*;

// [docs/specs/browser_dashboard.md] SCENARIO: 16.5
#[tokio::test]
async fn test_form_stays_static_after_submission() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            // Stash the node, not its id: a replaced form carries the same id,
            // so an id-string comparison cannot see the regression.
            page.evaluate::<(), ()>(
                "(() => { window.__formBefore = document.querySelector('#command-form'); })()",
                None,
            )
            .await
            .unwrap();

            // A command submit retargets to #status-display, so the form is
            // never re-registered and has no registering settle to race.
            send_action(&page, "look").await;

            wait_for_element_children(&page, "#story-log .log-entry", 2).await;

            let form_same = page
                .evaluate::<(), bool>(
                    r#"(() => {
                        const form = document.querySelector('#command-form');
                        return !!form && form === window.__formBefore;
                    })()"#,
                    None,
                )
                .await
                .unwrap();

            assert!(
                form_same,
                "#command-form was replaced by the submission \
                 (a fresh form would carry the same id)"
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

// [docs/specs/browser_dashboard.md] SCENARIO: 16.12
#[tokio::test]
async fn test_confirm_preview_leaves_form_ready_and_usable() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, port| async move {
            // Enable the real text check so a misspelling is intercepted.
            let client = reqwest::Client::new();
            let resp = client
                .post(format!("http://127.0.0.1:{port}/settings/text-check"))
                .form(&[("check_mode", "spell"), ("enable_auto_check", "true")])
                .send()
                .await
                .expect("enable text check request");
            assert!(
                resp.status().is_success(),
                "enabling text check failed: {}",
                resp.status()
            );

            // Submit a misspelled command; the settle waits for the preview's
            // swap into #action-preview.
            fill_command_input(&page, "i recieve a letter").await;
            click_and_settle(
                &page,
                "#command-form button[type=\"submit\"]",
                "#action-preview",
            )
            .await
            .expect_settled("submit intercepted command");
            wait_until_visible(&page, ".text-check-preview", Duration::from_secs(5)).await;

            click_and_settle(
                &page,
                ".text-check-preview .btn-original",
                "#status-display",
            )
            .await
            .expect_settled("confirm text-check preview");
            wait_until_hidden(&page, ".text-check-preview", Duration::from_secs(5)).await;
            wait_for_status_ready(&page).await;

            let (form_present, input_disabled) = page
                .evaluate::<(), (bool, bool)>(
                    r#"(() => {
                        const form = document.getElementById('command-form');
                        const input = form && form.querySelector('input[name="command"]');
                        return [!!form, !!input && input.disabled];
                    })()"#,
                    None,
                )
                .await
                .unwrap();
            assert!(form_present, "the command form was lost after the confirm");
            assert!(
                !input_disabled,
                "the command input must accept input once the status is Ready"
            );

            let before = count_log_entries(&page).await as u32;
            send_action(&page, "look around").await;
            wait_for_status_ready(&page).await;
            wait_for_element_children(&page, "#story-log .log-entry", before + 1).await;
        },
    )
    .await;
}
