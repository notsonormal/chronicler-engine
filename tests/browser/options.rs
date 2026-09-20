//! Browser options-dock tests: reload persistence and the Use-click wiring guard. Tagged against `docs/specs/browser_options.md`.

use super::*;

// [docs/specs/browser_options.md] SCENARIO: 26.2
#[tokio::test]
async fn test_options_dock_survives_reload() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            send_action(&page, "look").await;
            wait_for_status_ready(&page).await;
            send_action(&page, "/options").await;
            wait_for_status_ready(&page).await;

            wait_for_element_children(&page, "#options-dock .option-item", 3).await;
            let option_text: String = page
                .evaluate::<(), String>(
                    r#"(() => document.querySelector('#options-dock .option-btn').textContent)()"#,
                    None,
                )
                .await
                .unwrap();
            assert!(!option_text.is_empty(), "the dock must show an option");

            page.reload(None).await.unwrap();
            wait_for_element_children(&page, "#options-dock .option-item", 3).await;
            let reloaded_text: String = page
                .evaluate::<(), String>(
                    r#"(() => document.querySelector('#options-dock .option-btn').textContent)()"#,
                    None,
                )
                .await
                .unwrap();
            assert_eq!(
                reloaded_text, option_text,
                "the offered set must survive a page reload"
            );
        },
    )
    .await;
}

// [docs/specs/browser_options.md] SCENARIO: 26.4
#[tokio::test]
async fn test_options_use_click_submits_rendered_option_text() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            let entries_before = count_log_entries(&page).await;

            send_action(&page, "/options").await;
            wait_for_status_ready(&page).await;
            wait_for_element_children(&page, "#options-dock .option-item", 3).await;

            // Read the text from the *rendered* button, not a hardcoded seed
            // string: HTTP-tier assertions cannot observe this linkage.
            let first_option_btn = "#options-dock .option-btn >> nth=0";
            let option_text: String = page
                .evaluate::<(), String>(
                    r#"(() => {
                        const btn = document.querySelector('#options-dock .option-btn');
                        return btn ? btn.textContent : '';
                    })()"#,
                    None,
                )
                .await
                .unwrap();
            assert!(!option_text.is_empty(), "the dock must show an option");

            // `useOption` submits `/action/check`, whose response is retargeted
            // to `#status-display`, so that span is the POST's own swap.
            click_and_settle(&page, first_option_btn, "#status-display").await;
            wait_for_status_ready(&page).await;

            // `#story-log` polls, so the readiness gates are the settle gate
            // above plus `wait_for_status_ready`.
            wait_for_element_children(&page, "#story-log .log-entry.input", 1).await;
            let newest_input: String = page
                .evaluate::<(), String>(
                    r#"(() => {
                        const entries = document.querySelectorAll('#story-log .log-entry.input .text');
                        const last = entries[entries.length - 1];
                        return last ? last.textContent : '';
                    })()"#,
                    None,
                )
                .await
                .unwrap();
            assert_eq!(
                newest_input.trim(),
                option_text.trim(),
                "the submitted command must be the option text read from the rendered button"
            );
            assert!(
                count_log_entries(&page).await > entries_before,
                "the submitted option must add a log entry"
            );
        },
    )
    .await;
}
