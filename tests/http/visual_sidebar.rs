//! HTTP E2E tests for the visual sidebar fragment (`GET /fragment/visual-sidebar`) — the portrait labels.

use chronicler_engine::test_support::{TestAppBuilder, TestDataBuilder, TestNpc};

use crate::support::http_requests::fetch_body;

// [docs/specs/visual_sidebar.md] SCENARIO: 32.1
#[tokio::test]
async fn test_visual_sidebar_shows_character_name_label_http() {
    let app = TestAppBuilder::default_test().build();

    let body = fetch_body(&app, "/fragment/visual-sidebar").await;

    assert!(
        body.contains(r#"<div class="image-label" title="Test NPC">Test NPC</div>"#),
        "each portrait must show the Character's name as a visible label: {body}"
    );
}

// [docs/specs/visual_sidebar.md] SCENARIO: 32.2
#[tokio::test]
async fn test_visual_sidebar_escapes_character_name_label_http() {
    let mut npc = TestNpc::named("escapist", "Ben & Jerry <Script>");
    npc.sheet.headshot_image = Some("data/images/escapist.png".to_string());
    let data = TestDataBuilder::default_test()
        .npc(npc)
        .room_npc("escapist")
        .build();
    let app = TestAppBuilder::default_test().data(data).build();

    let body = fetch_body(&app, "/fragment/visual-sidebar").await;

    assert!(
        !body.contains("<Script>"),
        "the Character's name must not render as markup: {body}"
    );
    assert!(
        body.contains("Ben &#38; Jerry &#60;Script&#62;"),
        "the Character's name must render escaped in the label: {body}"
    );
}
