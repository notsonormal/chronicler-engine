//! Browser slash-command palette tests: open, filter, navigate, submit flows. Tagged against `docs/specs/browser_slash_menu.md`.

use super::*;

// [docs/specs/browser_slash_menu.md] SCENARIO: 31.8
#[tokio::test]
async fn test_slash_impersonate_produces_input_entry() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            let before = page.locator("#story-log .log-entry.input").await.count().await.unwrap_or(0);

            send_action(&page, "/impersonate hello").await;
            wait_for_status_ready(&page).await;

            let after = page.locator("#story-log .log-entry.input").await.count().await.unwrap_or(0);
            assert_eq!(
                after,
                before + 1,
                "Submitting /impersonate should add one Input entry"
            );

            let input_text: String = page
                .evaluate::<(), String>(
                    r#"(() => {
                        const el = Array.from(document.querySelectorAll('#story-log .log-entry.input .text'))
                            .find(t => t.textContent.includes('/impersonate hello'));
                        return el ? el.textContent : '';
                    })()"#,
                    None,
                )
                .await
                .unwrap();
            assert!(
                input_text.is_empty(),
                "/impersonate should not be persisted as a plain Input entry"
            );
        },
    )
    .await;
}

// [docs/specs/browser_slash_menu.md] SCENARIO: 31.9
#[tokio::test]
async fn test_slash_guide_does_not_persist_input_entry() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
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

            send_action(&page, "/guide look around").await;
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
                inputs_after, inputs_before,
                "/guide should not add an Input entry"
            );
            assert!(
                narrations_after > narrations_before,
                "/guide should produce at least one Narration entry"
            );
        },
    )
    .await;
}
