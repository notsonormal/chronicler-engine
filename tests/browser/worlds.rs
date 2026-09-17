//! Browser worlds-panel tests: world posture editor render + auto-save. Tagged against `docs/specs/browser_worlds.md`.

use std::time::Duration;

use super::*;

/// Open the edit form for the seeded world "test" and wait for the posture
/// status target. Shared by the world posture scenarios.
///
/// The worlds fragment and its hx-loader div share the `.worlds-panel` class
/// (two matches before the first swap), so every wait scopes to an element
/// unique to the loaded fragment and htmx's `querySelector`-first target
/// resolution replaces the loader div with the form.
///
/// The Edit click's swap is awaited through the settle gate. That wait is
/// load-bearing, not decoration: htmx attaches the new form's `hx-trigger`
/// listeners at the end of the settle task, so a `change` fired on the posture
/// select before that settle is lost — measured as 6 failures in a 50-run loop
/// when the form's own swap was not awaited.
async fn open_world_edit(page: &playwright_rs::Page) {
    page.locator(r#".tab[data-tab="worlds"]"#)
        .await
        .click(None)
        .await
        .unwrap();
    wait_for_element_children(page, "#worlds-tab .btn-cyan", 1).await;
    let baseline = arm_settle_gate(page).await;
    page.locator("#worlds-tab .btn-cyan")
        .await
        .first()
        .click(None)
        .await
        .unwrap();
    // The Edit click's outerHTML swap replaces the loader div with the form;
    // htmx reports `detail.elt` as the `.worlds-panel` element it swapped.
    let outcome = settle_since_baseline(page, baseline, ".worlds-panel").await;
    outcome.expect_settled("open world edit form");
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
async fn test_world_posture_change_autosaves_server_state() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            open_world_edit(&page).await;

            // The `change` event fires inside htmx's 20 ms defaultSettleDelay
            // window; `select_option_and_settle` reads the afterSettle baseline
            // before the change and waits for the *posture status span* to
            // settle, which the pollers never touch. See
            // `.scratch/ui-verification-redesign/issues/03-root-cause-the-hx-post-no-fire.md`.
            select_option_and_settle(
                &page,
                r#"#worlds-tab select[name="narrative_tense"]"#,
                "present",
                "#world-posture-status",
            )
            .await;

            // Server state changed, observed through the browser: reload and
            // re-open the edit form so the select re-renders from the persisted
            // world. The `Saved` fragment contract lives in the HTTP tier
            // (`tests/http/worlds.rs`, SCENARIO 25.5); this test keeps the
            // browser-only question — does the change event reach the server at
            // all and does the re-render reflect it.
            page.reload(None)
                .await
                .expect("reload after the posture change");
            open_world_edit(&page).await;            let selected: String = page
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
