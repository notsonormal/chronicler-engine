//! Browser options-dock tests: Use/Edit interactions, reload persistence. Tagged against `docs/specs/browser_options.md`.

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
