//! [DOC: docs/reference/testing.md]

use super::*;

#[tokio::test]
async fn test_form_submission() {
    with_test_page(CONFIG_PATH, TEST_WORLD, |page, _port| async move {
        let initial_entries = wait_for_element_children(&page, "#story-log .log-entry", 1).await;
        page.evaluate::<(), ()>(
            "(() => { 
            const input = document.querySelector('#command-form input');
            if (input) input.value = 'hello';
            const form = document.querySelector('#command-form');
            if (form) form.requestSubmit();
        })()",
            None,
        )
        .await
        .unwrap();
        wait_for_status_ready(&page).await;
        let after_entries =
            wait_for_element_children(&page, "#story-log .log-entry", initial_entries + 1).await;
        assert!(
            after_entries > initial_entries,
            "Form submission should add log entries: {initial_entries} -> {after_entries}"
        );
    })
    .await;
}
