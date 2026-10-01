//! HTTP E2E tests for the visual sidebar fragment (`GET /fragment/visual-sidebar`) — the portrait labels.

use chronicler_engine::test_support::{TestAppBuilder, TestDataBuilder, TestNpc};

use crate::support::http_requests::fetch_body;

// [docs/specs/visual_sidebar.md] SCENARIO: 32.1
#[tokio::test]
async fn test_visual_sidebar_shows_character_name_label_http() {
    let mut second = TestNpc::named("npc_2", "Second NPC");
    second.sheet.headshot_image = Some("data/images/npc_2.png".to_string());
    let data = TestDataBuilder::default_test()
        .npc(second)
        .room_npc("npc_2")
        .build();
    let app = TestAppBuilder::default_test().data(data).build();

    let body = fetch_body(&app, "/fragment/visual-sidebar").await;

    assert_eq!(
        body.matches(r#"class="image-label""#).count(),
        2,
        "the Room's two Characters must render two portrait labels: {body}"
    );
    assert!(
        body.contains(r#"title="Test NPC""#) && body.contains(">Test NPC</div>"),
        "each portrait must show the Character's name as visible label text: {body}"
    );
    assert!(
        body.contains(r#"title="Second NPC""#) && body.contains(">Second NPC</div>"),
        "each portrait must show the Character's name as visible label text: {body}"
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
        !body.contains("Ben & Jerry"),
        "the ampersand in the Character's name must be escaped: {body}"
    );
    assert!(
        body.contains("Jerry"),
        "the Character's name must still render in the label: {body}"
    );
}
