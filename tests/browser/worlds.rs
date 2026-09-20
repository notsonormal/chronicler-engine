//! Browser worlds-panel tests: world posture auto-save wiring guard. Tagged against `docs/specs/browser_worlds.md`.

use super::*;

/// The browser-only question: does the `change` event on a swap-registered
/// select reach the server, and does the re-rendered form reflect the persisted
/// value. The edit-form render is covered at HTTP (`worlds.md` 25.6, 25.5).
// [docs/specs/browser_worlds.md] SCENARIO: 29.2
#[tokio::test]
async fn test_world_posture_change_autosaves_server_state() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            open_world_edit(&page).await;

            // The posture status span is the swap target: the pollers never
            // touch it, so the wait cannot satisfy itself on poller noise.
            select_option_and_settle(
                &page,
                r#"#worlds-tab select[name="narrative_tense"]"#,
                "present",
                "#world-posture-status",
            )
            .await;

            // Reload and re-open the edit form so the select re-renders from the
            // persisted world.
            page.reload(None)
                .await
                .expect("reload after the posture change");
            open_world_edit(&page).await;
            let selected: String = page
                .evaluate::<(), String>(
                    r#"(() => {
                        const sel = document.querySelector('#worlds-tab select[name="narrative_tense"]');
                        return sel ? sel.value : '';
                    })()"#,
                    None,
                )
                .await
                .unwrap_or_default();
            assert_eq!(
                selected, "present",
                "the posture change must reach the server: the re-rendered form should show the persisted tense"
            );
        },
    )
    .await;
}
