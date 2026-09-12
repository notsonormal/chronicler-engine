//! Browser worlds-panel tests: world posture editor render + auto-save. Tagged against `docs/specs/browser_worlds.md`.

use std::time::Duration;

use playwright_rs::expect;

use super::*;

/// Open the edit form for the seeded world "test" and wait for the posture
/// status target. Shared by the world posture scenarios.
///
/// The worlds fragment and its hx-loader div share the `.worlds-panel` class
/// (two matches before the first swap), so every wait scopes to an element
/// unique to the loaded fragment and htmx's `querySelector`-first target
/// resolution replaces the loader div with the form.
async fn open_world_edit(page: &playwright_rs::Page) {
    page.locator(r#".tab[data-tab="worlds"]"#)
        .await
        .click(None)
        .await
        .unwrap();
    wait_for_element_children(page, "#worlds-tab .btn-cyan", 1).await;
    page.locator("#worlds-tab .btn-cyan")
        .await
        .first()
        .click(None)
        .await
        .unwrap();
    // The Edit click fetches the form over HTTP — allow 5s for fetch + swap.
    // Wait on the posture selects, not #world-posture-status: an empty
    // (zero-sized) span never becomes visible, see wait_until_visible docs.
    wait_until_visible(
        page,
        r#"#worlds-tab select[name="narrator_mode"]"#,
        Duration::from_millis(5000),
    )
    .await;
}

// [docs/specs/browser_worlds.md] SCENARIO: 29.1
#[tokio::test]
async fn test_world_edit_form_renders_posture_selects() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            open_world_edit(&page).await;

            for name in ["narrator_mode", "narrative_perspective", "narrative_tense"] {
                let select = page
                    .locator(&format!(r#"#worlds-tab select[name="{name}"]"#))
                    .await;
                assert!(
                    select.is_visible().await.unwrap_or(false),
                    "{name} select is rendered"
                );
            }
        },
    )
    .await;
}

// [docs/specs/browser_worlds.md] SCENARIO: 29.2
#[tokio::test]
async fn test_world_posture_change_autosaves_status() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            open_world_edit(&page).await;

            page.locator(r#"#worlds-tab select[name="narrative_tense"]"#)
                .await
                .select_option("present", None)
                .await
                .unwrap();

            let status = page.locator("#world-posture-status").await;
            if let Err(e) = expect(status)
                .with_timeout(std::time::Duration::from_secs(5))
                .to_contain_text("Saved")
                .await
            {
                panic!("world posture auto-save should report Saved: {e}");
            }
        },
    )
    .await;
}
