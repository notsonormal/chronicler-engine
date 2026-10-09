//! Shared stub-browser helpers: page probes and node reads used by more than one stub test module.

pub(super) async fn active_element_is(page: &playwright_rs::Page, selector: &str) -> bool {
    page.evaluate::<String, bool>(
        r#"(selector) => {
            const el = document.querySelector(selector);
            return !!el && document.activeElement === el;
        }"#,
        Some(&selector.to_string()),
    )
    .await
    .unwrap()
}

pub(super) async fn read_banner(page: &playwright_rs::Page) -> (bool, String, String) {
    page.evaluate::<(), (bool, String, String)>(
        r#"() => {
            const banner = document.getElementById('failure-banner');
            if (!banner) return [false, '', ''];
            return [
                !banner.hidden,
                banner.getAttribute('role') || '',
                banner.textContent.trim(),
            ];
        }"#,
        None,
    )
    .await
    .unwrap()
}

pub(super) async fn story_poll_count(page: &playwright_rs::Page) -> f64 {
    page.evaluate::<(), f64>("(() => window.__storyPolls || 0)()", None)
        .await
        .unwrap_or(0.0)
}

pub(super) async fn install_story_poll_counter(page: &playwright_rs::Page) {
    page.evaluate::<(), ()>(
        r#"(() => {
            window.__storyPolls = 0;
            document.body.addEventListener('htmx:afterRequest', (evt) => {
                const elt = evt.detail && evt.detail.elt;
                if (elt && elt.id === 'story-log') window.__storyPolls += 1;
            });
        })()"#,
        None,
    )
    .await
    .unwrap();
}
