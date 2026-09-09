//! Browser behaviour tests: click→DOM change, htmx swap persistence, polling-pause, status wiring. Tagged against `docs/specs/browser.md`.

use std::time::Duration;

use playwright_rs::expect;

use super::*;

// [docs/specs/browser.md] SCENARIO: 16.1
#[tokio::test]
async fn test_edit_mode_activates_on_click() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            let clicked = page
                .evaluate::<(), bool>(
                    r#"(() => {
                    const btn = document.querySelector('.edit-btn');
                    if (btn) {
                        btn.click();
                        return true;
                    }
                    return false;
                })()"#,
                    None,
                )
                .await
                .unwrap();
            assert!(clicked, "Should find and click an edit button");

            wait_until_visible(&page, "#edit-textarea", Duration::from_millis(500)).await;
        },
    )
    .await;
}

// [docs/specs/browser.md] SCENARIO: 16.2
#[tokio::test]
async fn test_edit_cancel_restores_original() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            let original_text = page
                .locator(".log-entry .text")
                .await
                .inner_text()
                .await
                .unwrap_or_default();
            assert!(!original_text.is_empty(), "Should have original text");

            page.locator(".edit-btn").await.click(None).await.unwrap();
            wait_until_visible(&page, "#edit-textarea", Duration::from_millis(500)).await;

            let modified = "Modified text for testing";
            page.locator("#edit-textarea")
                .await
                .fill(modified, None)
                .await
                .unwrap();

            page.locator(".cancel-btn").await.click(None).await.unwrap();
            wait_until_hidden(&page, "#edit-textarea", Duration::from_millis(500)).await;

            let restored = page
                .locator(".log-entry .text")
                .await
                .inner_text()
                .await
                .unwrap_or_default();

            assert_eq!(
                restored, original_text,
                "Text should be restored to original after cancel"
            );
        },
    )
    .await;
}

// [docs/specs/browser.md] SCENARIO: 16.3
#[tokio::test]
async fn test_polling_pauses_during_edit() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            page.evaluate::<(), bool>(
                r#"(() => {
                const btn = document.querySelector('.edit-btn');
                if (btn) { btn.click(); return true; }
                return false;
            })()"#,
                None,
            )
            .await
            .unwrap();

            wait_until_visible(&page, "#edit-textarea", Duration::from_millis(500)).await;

            let persisted =
                wait_for_element_persist(&page, "#edit-textarea", Duration::from_secs(3)).await;
            assert!(
                persisted,
                "Edit textarea should persist during polling pause"
            );
        },
    )
    .await;
}

// [docs/specs/browser.md] SCENARIO: 16.4
#[tokio::test]
async fn test_delete_removes_message() {
    with_test_page(CONFIG_PATH, TEST_WORLD, TEST_PERSONA, |page, _port| async move {
        let initial = count_log_entries(&page).await;
        if initial < 2 {
            send_action(&page, "look").await;
            wait_for_status_ready(&page).await;
        }

        let count_before_delete = count_log_entries(&page).await;
        assert!(
            count_before_delete >= 2,
            "Need at least 2 entries for delete button, have {count_before_delete}"
        );

        page.evaluate::<(), ()>(
            r#"(() => {
                window.confirm = () => true;
            })()"#,
            None,
        )
        .await
        .unwrap();

        page.locator(".delete-btn").await.click(None).await.unwrap();

        let current_count = wait_for_log_entries_below(&page, count_before_delete).await;

        assert!(
            current_count < count_before_delete,
            "Delete should remove the message (expected < {count_before_delete}, got {current_count})"
        );
    })
    .await;
}

// [docs/specs/browser.md] SCENARIO: 16.5
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

// [docs/specs/browser.md] SCENARIO: 16.6
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

// [docs/specs/browser.md] SCENARIO: 16.7
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
            // without production-code changes (out of scope for this ticket).
            // ponytail: skip the 5s auto-hide setTimeout — asserting it would
            // burn 5s and add flake for no authority gain; the setTimeout is
            // trivial JS.
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

const COMMAND_INPUT_SELECTOR: &str = r##"#command-form input[name="command"]"##;

/// Type text into the command input one keystroke at a time so the `input`
/// event (which opens the menu) fires for every character.
async fn type_into_command(page: &playwright_rs::Page, text: &str) {
    let input = page.locator(COMMAND_INPUT_SELECTOR).await;
    input.focus().await.unwrap();
    input.press_sequentially(text, None).await.unwrap();
}

// [docs/specs/browser.md] SCENARIO: 17.1
#[tokio::test]
async fn test_slash_menu_opens_on_slash() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            type_into_command(&page, "/").await;

            wait_until_visible(&page, "#slash-menu", Duration::from_millis(1000)).await;

            let cmds: Vec<String> = page
                .locator("#slash-menu .slash-suggestion .slash-cmd")
                .await
                .all_inner_texts()
                .await
                .unwrap_or_default();
            assert_eq!(
                cmds,
                vec!["/impersonate".to_string(), "/guide".to_string()],
                "Menu should list the steering commands in canonical order"
            );
        },
    )
    .await;
}

// [docs/specs/browser.md] SCENARIO: 17.2
#[tokio::test]
async fn test_slash_menu_filters_by_prefix() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            type_into_command(&page, "/g").await;

            wait_until_visible(
                &page,
                "#slash-menu .slash-suggestion",
                Duration::from_millis(1000),
            )
            .await;

            let cmds: Vec<String> = page
                .locator("#slash-menu .slash-suggestion .slash-cmd")
                .await
                .all_inner_texts()
                .await
                .unwrap_or_default();
            assert_eq!(
                cmds,
                vec!["/guide".to_string()],
                "Only /guide should match the /g prefix"
            );
        },
    )
    .await;
}

// [docs/specs/browser.md] SCENARIO: 17.3
#[tokio::test]
async fn test_slash_menu_arrow_keys_move_active() {
    with_test_page(CONFIG_PATH, TEST_WORLD, TEST_PERSONA, |page, _port| async move {
        type_into_command(&page, "/").await;
        wait_until_visible(&page, "#slash-menu", Duration::from_millis(1000)).await;

        let input = page.locator(COMMAND_INPUT_SELECTOR).await;

        let first_active: bool = page
            .evaluate::<(), bool>(
                r#"(() => {
                    const items = document.querySelectorAll('#slash-menu .slash-suggestion');
                    return items.length > 0 && items[0].classList.contains('active');
                })()"#,
                None,
            )
            .await
            .unwrap();
        assert!(first_active, "First suggestion should start active");

        input.press("ArrowDown", None).await.unwrap();
        let second_active: bool = page
            .evaluate::<(), bool>(
                r#"(() => {
                    const items = document.querySelectorAll('#slash-menu .slash-suggestion');
                    return items.length > 1 && items[1].classList.contains('active') && !items[0].classList.contains('active');
                })()"#,
                None,
            )
            .await
            .unwrap();
        assert!(second_active, "ArrowDown should move active to the second suggestion");

        input.press("ArrowUp", None).await.unwrap();
        let first_active_again: bool = page
            .evaluate::<(), bool>(
                r#"(() => {
                    const items = document.querySelectorAll('#slash-menu .slash-suggestion');
                    return items.length > 0 && items[0].classList.contains('active');
                })()"#,
                None,
            )
            .await
            .unwrap();
        assert!(first_active_again, "ArrowUp should move active back to the first suggestion");
    })
    .await;
}

// [docs/specs/browser.md] SCENARIO: 17.4
#[tokio::test]
async fn test_slash_menu_enter_populates_input() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            type_into_command(&page, "/").await;
            wait_until_visible(&page, "#slash-menu", Duration::from_millis(1000)).await;

            let input = page.locator(COMMAND_INPUT_SELECTOR).await;
            // First suggestion (/impersonate) is active by default.
            input.press("Enter", None).await.unwrap();

            wait_until_hidden(&page, "#slash-menu", Duration::from_millis(1000)).await;

            let value: String = input.input_value(None).await.unwrap_or_default();
            assert_eq!(
                value, "/impersonate ",
                "Enter should populate the input with the highlighted command + trailing space"
            );
        },
    )
    .await;
}

// [docs/specs/browser.md] SCENARIO: 17.5
#[tokio::test]
async fn test_slash_menu_escape_closes() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            type_into_command(&page, "/g").await;
            wait_until_visible(&page, "#slash-menu", Duration::from_millis(1000)).await;

            let input = page.locator(COMMAND_INPUT_SELECTOR).await;
            input.press("Escape", None).await.unwrap();

            wait_until_hidden(&page, "#slash-menu", Duration::from_millis(1000)).await;

            let value: String = input.input_value(None).await.unwrap_or_default();
            assert_eq!(value, "/g", "Escape should leave the input value unchanged");
        },
    )
    .await;
}

// [docs/specs/browser.md] SCENARIO: 17.6
#[tokio::test]
async fn test_slash_menu_click_populates_input() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            type_into_command(&page, "/").await;
            wait_until_visible(&page, "#slash-menu", Duration::from_millis(1000)).await;

            // Click the /guide suggestion by its command text, independent of menu order.
            page.locator("#slash-menu .slash-suggestion:has(.slash-cmd:text-is('/guide'))")
                .await
                .click(None)
                .await
                .unwrap();

            wait_until_hidden(&page, "#slash-menu", Duration::from_millis(1000)).await;

            let value: String = page
                .locator(COMMAND_INPUT_SELECTOR)
                .await
                .input_value(None)
                .await
                .unwrap_or_default();
            assert_eq!(
                value, "/guide ",
                "Clicking a suggestion should populate the input with its command + trailing space"
            );
        },
    )
    .await;
}

// [docs/specs/browser.md] SCENARIO: 17.7
#[tokio::test]
async fn test_slash_menu_reopens_after_action_area_rerender() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            type_into_command(&page, "/").await;
            wait_until_visible(&page, "#slash-menu", Duration::from_millis(1000)).await;

            page.evaluate::<(), ()>(
                r##"(() => {
                const area = document.getElementById('action-area');
                const productionMarkup = area.innerHTML;
                area.innerHTML = productionMarkup;
            })()"##,
                None,
            )
            .await
            .unwrap();

            wait_until_hidden(&page, "#slash-menu", Duration::from_millis(1000)).await;

            type_into_command(&page, "/").await;
            wait_until_visible(&page, "#slash-menu", Duration::from_millis(1000)).await;

            let count: u32 = page
                .locator("#slash-menu .slash-suggestion")
                .await
                .count()
                .await
                .unwrap_or(0) as u32;
            assert_eq!(
                count, 2,
                "Menu should reopen with both commands after re-render"
            );
        },
    )
    .await;
}

// [docs/specs/browser.md] SCENARIO: 17.8
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

// [docs/specs/browser.md] SCENARIO: 17.9
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

// [docs/specs/games.md] SCENARIO: 20.1
#[tokio::test]
async fn test_games_panel_renders_posture_fragment() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            page.locator(r#".tab[data-tab="games"]"#)
                .await
                .click(None)
                .await
                .unwrap();
            wait_until_visible(&page, "#game-posture-controls", Duration::from_millis(1000)).await;

            let mode = page
                .locator(r#"#game-posture-controls select[name="narrator_mode"]"#)
                .await
                .input_value(None)
                .await
                .unwrap_or_default();
            assert_eq!(mode, "novel", "active game starts in Novel mode");

            let perspective = page
                .locator(r#"#game-posture-controls select[name="narrative_perspective"]"#)
                .await
                .input_value(None)
                .await
                .unwrap_or_default();
            assert_eq!(perspective, "third", "active game starts in Third person");

            let tense = page
                .locator(r#"#game-posture-controls select[name="narrative_tense"]"#)
                .await
                .input_value(None)
                .await
                .unwrap_or_default();
            assert_eq!(tense, "past", "active game starts in Past tense");

            for name in [
                "system_preset_id",
                "quantifier_preset_id",
                "impersonate_preset_id",
            ] {
                let select = page
                    .locator(&format!(r#"#game-posture-controls select[name="{name}"]"#))
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

// [docs/specs/games.md] SCENARIO: 20.2
#[tokio::test]
async fn test_games_tense_change_autosaves_and_rerenders() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            page.locator(r#".tab[data-tab="games"]"#)
                .await
                .click(None)
                .await
                .unwrap();
            wait_until_visible(&page, "#game-posture-controls", Duration::from_millis(1000)).await;

            page.locator(r#"#game-posture-controls select[name="narrative_tense"]"#)
                .await
                .select_option("present", None)
                .await
                .unwrap();

            // The POST /games/:id/posture response swaps the fragment
            // (outerHTML), so the locator re-resolves to the fresh select.
            let tense_select = page
                .locator(r#"#game-posture-controls select[name="narrative_tense"]"#)
                .await;
            if let Err(e) = expect(tense_select)
                .with_timeout(std::time::Duration::from_secs(5))
                .to_have_value("present")
                .await
            {
                panic!("fragment should re-render with tense present after auto-save: {e}");
            }
        },
    )
    .await;
}

// [docs/specs/games.md] SCENARIO: 20.3
#[tokio::test]
async fn test_games_mode_switch_retargets_and_nudges() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            page.locator(r#".tab[data-tab="games"]"#)
                .await
                .click(None)
                .await
                .unwrap();
            wait_until_visible(&page, "#game-posture-controls", Duration::from_millis(1000)).await;

            page.locator(r#"#game-posture-controls select[name="narrator_mode"]"#)
                .await
                .select_option("interactive_fiction", None)
                .await
                .unwrap();

            let perspective_select = page
                .locator(r#"#game-posture-controls select[name="narrative_perspective"]"#)
                .await;
            if let Err(e) = expect(perspective_select)
                .with_timeout(std::time::Duration::from_secs(5))
                .to_have_value("second")
                .await
            {
                panic!("mode switch should nudge perspective to the IF default: {e}");
            }

            let system_select = page
                .locator(r#"#game-posture-controls select[name="system_preset_id"]"#)
                .await;
            if let Err(e) = expect(system_select)
                .with_timeout(std::time::Duration::from_secs(5))
                .to_have_value("system_if_default")
                .await
            {
                panic!("mode switch should retarget the system preset to the IF bundle: {e}");
            }
        },
    )
    .await;
}

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

// [docs/specs/browser.md] SCENARIO: 18.1
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

// [docs/specs/browser.md] SCENARIO: 18.2
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

// [docs/specs/prompt_presets.md] SCENARIO: 21.27
#[tokio::test]
async fn test_preset_editor_mode_flags_roundtrip() {
    with_test_page(
        CONFIG_PATH,
        TEST_WORLD,
        TEST_PERSONA,
        |page, _port| async move {
            page.locator(r#".tab[data-tab="prompt-presets"]"#)
                .await
                .click(None)
                .await
                .unwrap();
            // All seeded presets are defaults (View-only), so the duplicated
            // copy is the only card with an Edit button.
            wait_for_element_children(
                &page,
                r#"#prompt-presets-tab button[hx-post$="/duplicate"]"#,
                1,
            )
            .await;
            page.locator(r#"#prompt-presets-tab button[hx-post$="/duplicate"]"#)
                .await
                .first()
                .click(None)
                .await
                .unwrap();
            wait_for_element_children(&page, r#"#prompt-presets-tab button[hx-get$="/edit"]"#, 1)
                .await;
            page.locator(r#"#prompt-presets-tab button[hx-get$="/edit"]"#)
                .await
                .first()
                .click(None)
                .await
                .unwrap();
            wait_until_visible(&page, ".preset-card.edit-form", Duration::from_millis(1000)).await;

            let name = page
                .locator(r#".preset-card.edit-form input[name="name"]"#)
                .await
                .input_value(None)
                .await
                .unwrap_or_default();
            assert!(!name.is_empty(), "edit form renders the copy's name");

            let novel_box = page
                .locator(r#".preset-card.edit-form input[name="allowed_mode_novel"]"#)
                .await;
            assert!(
                novel_box.is_checked().await.unwrap_or(false),
                "copy of the Novel-only default checks the novel flag"
            );
            let if_box = page
                .locator(r#".preset-card.edit-form input[name="allowed_mode_if"]"#)
                .await;
            assert!(
                !if_box.is_checked().await.unwrap_or(true),
                "copy of the Novel-only default leaves the IF flag unchecked"
            );

            if_box.check(None).await.unwrap();
            page.locator(r#".preset-card.edit-form button[type="submit"]"#)
                .await
                .click(None)
                .await
                .unwrap();

            // The update swaps the card in place; the edit form disappears.
            let edit_gone = page.locator(".preset-card.edit-form").await;
            if let Err(e) = expect(edit_gone)
                .with_timeout(std::time::Duration::from_secs(5))
                .to_be_hidden()
                .await
            {
                panic!("saving should swap the edit form away: {e}");
            }

            let saved = page
                .evaluate::<String, bool>(
                    r#"(name) => {
                    const cards = [...document.querySelectorAll('.preset-card')];
                    const card = cards.find((c) => {
                        const title = c.querySelector('.card-title');
                        return title && title.textContent.includes(name);
                    });
                    if (!card) return false;
                    return !!card.querySelector(
                        'button[hx-post$="activate?mode=interactive_fiction"]'
                    );
                }"#,
                    Some(&name),
                )
                .await
                .unwrap_or(false);
            assert!(
                saved,
                "saved card should offer Set Active (IF) after enabling the IF flag"
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
