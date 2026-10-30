//! HTTP E2E tests for the visual sidebar part of the screen — the portrait labels.

use chronicler_engine::test_support::{TestAppBuilder, TestDataBuilder, TestCharacter};

use crate::support::http_requests::fetch_part;

// [docs/specs/visual_sidebar.md] SCENARIO: 32.1
#[tokio::test]
async fn test_visual_sidebar_shows_character_name_label_http() {
    let mut second = TestCharacter::named("character_2", "Second Character");
    second.sheet.headshot_image = Some("data/images/character_2.png".to_string());
    let data = TestDataBuilder::default_test()
        .character(second)
        .room_character("character_2")
        .build();
    let app = TestAppBuilder::default_test().data(data).build();

    let body = fetch_part(&app, "visual-sidebar").await;

    assert_eq!(
        body.matches(r#"class="image-container character-portrait"#)
            .count(),
        2,
        "the Room's two Characters must render two portraits: {body}"
    );
    assert!(
        body.contains(r#"title="Test Character""#) && body.contains(">Test Character</div>"),
        "each portrait must show the Character's name as visible label text: {body}"
    );
    assert!(
        body.contains(r#"title="Second Character""#) && body.contains(">Second Character</div>"),
        "each portrait must show the Character's name as visible label text: {body}"
    );
}

// [docs/specs/visual_sidebar.md] SCENARIO: 32.2
#[tokio::test]
async fn test_visual_sidebar_escapes_character_name_label_http() {
    let mut character = TestCharacter::named("escapist", "Ben & Jerry <Script>");
    character.sheet.headshot_image = Some("data/images/escapist.png".to_string());
    let data = TestDataBuilder::default_test()
        .character(character)
        .room_character("escapist")
        .build();
    let app = TestAppBuilder::default_test().data(data).build();

    let body = fetch_part(&app, "visual-sidebar").await;

    assert!(
        !body.contains("<Script>"),
        "the Character's name must not render as markup: {body}"
    );
    assert!(
        !body.contains("Ben & Jerry"),
        "the ampersand in the Character's name must be escaped: {body}"
    );
    assert!(
        body.contains("Ben &#") || body.contains("Ben &amp;"),
        "the Character's name must still render in its escaped form: {body}"
    );
    assert!(
        body.contains("Jerry"),
        "the Character's name must still render in the label: {body}"
    );
}
