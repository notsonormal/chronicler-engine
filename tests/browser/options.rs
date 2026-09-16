//! Browser options-dock tests: Use/Edit interactions, reload persistence. Tagged against `docs/specs/browser_options.md`.

use super::*;

// [docs/specs/browser_options.md] SCENARIO: 26.1
#[tokio::test]
async fn test_options_use_click_submits_option() {
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

            let inputs_before = page
                .locator("#story-log .log-entry.input")
                .await
                .count()
                .await
                .unwrap_or(0);
            let narrations_before = page
                .locator("#story-log .log-entry.narration")
                .await
                .count()
                .await
                .unwrap_or(0);

            let clicked: bool = page
                .evaluate::<(), bool>(
                    r#"(() => {
                        const btn = document.querySelector('#options-dock .option-btn');
                        if (!btn) return false;
                        btn.click();
                        return true;
                    })()"#,
                    None,
                )
                .await
                .unwrap();
            assert!(clicked, "Use click must find the option button");

            wait_for_status_generating(&page).await;
            dismiss_text_check_if_present(&page).await;
            wait_for_status_ready(&page).await;

            let inputs_after = page
                .locator("#story-log .log-entry.input")
                .await
                .count()
                .await
                .unwrap_or(0);
            let narrations_after = page
                .locator("#story-log .log-entry.narration")
                .await
                .count()
                .await
                .unwrap_or(0);
            assert_eq!(
                inputs_after,
                inputs_before + 1,
                "Use must submit the option as one player Input"
            );
            assert!(
                narrations_after > narrations_before,
                "the submitted option must narrate"
            );

            let all_inputs: Vec<String> = page
                .evaluate::<(), Vec<String>>(
                    r#"(() => Array.from(document.querySelectorAll('#story-log .log-entry.input .text'))
                        .map(t => t.textContent))()"#,
                    None,
                )
                .await
                .unwrap();
            assert!(
                all_inputs.iter().any(|t| t.contains(option_text.as_str())),
                "the Input entry must carry the option text; inputs were: {all_inputs:?}"
            );
        },
    )
    .await;
}

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

// [docs/specs/browser_options.md] SCENARIO: 26.3
#[tokio::test]
async fn test_options_dock_edit_fills_without_submitting() {
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

            let entries_before = page
                .locator("#story-log .log-entry")
                .await
                .count()
                .await
                .unwrap_or(0);

            let clicked: bool = page
                .evaluate::<(), bool>(
                    r#"(() => {
                        const btn = document.querySelector('#options-dock .option-item .mini-btn');
                        if (!btn) return false;
                        btn.click();
                        return true;
                    })()"#,
                    None,
                )
                .await
                .unwrap();
            assert!(clicked, "Edit click must find the edit button");

            let focused: bool = page
                .evaluate::<(), bool>(
                    r#"(() => {
                        const input = document.querySelector('#command-form input[name="command"]');
                        return input.value.length > 0 && document.activeElement === input;
                    })()"#,
                    None,
                )
                .await
                .unwrap();
            assert!(
                focused,
                "Edit must fill the command input and focus it"
            );

            let input_value: String = page
                .evaluate::<(), String>(
                    r#"(() => document.querySelector('#command-form input[name="command"]').value)()"#,
                    None,
                )
                .await
                .unwrap();
            assert_eq!(input_value, option_text, "Edit must fill the option text");

            let entries_after = page
                .locator("#story-log .log-entry")
                .await
                .count()
                .await
                .unwrap_or(0);
            assert_eq!(
                entries_after, entries_before,
                "Edit must not submit or create a story-log entry"
            );
        },
    )
    .await;
}
