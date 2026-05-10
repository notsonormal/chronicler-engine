use askama::Template;
use chronicler_engine::server::templates::HeaderTemplate;

#[test]
fn test_header_template_renders_room_name() {
    let template = HeaderTemplate {
        room_name: "Test Room".to_string(),
    };
    let rendered = template.render().unwrap();
    assert!(
        rendered.contains("Chronicler Engine"),
        "Expected rendered output to contain 'Chronicler Engine': {rendered}"
    );
    assert!(
        rendered.contains(r#"class="header""#),
        "Expected header class: {rendered}"
    );
    assert!(
        rendered.contains(r#"class="game-title""#),
        "Expected game-title class: {rendered}"
    );
    assert!(
        rendered.contains("connection-status"),
        "Expected connection-status in: {rendered}"
    );
}

#[test]
fn test_header_template_ignores_room_name() {
    let template = HeaderTemplate {
        room_name: "<script>alert('xss')</script>".to_string(),
    };
    let rendered = template.render().unwrap();
    assert!(
        rendered.contains("Chronicler Engine"),
        "Should contain Chronicler Engine: {rendered}"
    );
    assert!(
        !rendered.contains("<script>"),
        "Template should not contain raw script tag: {rendered}"
    );
}

#[test]
fn test_header_template_connection_status() {
    let template = HeaderTemplate {
        room_name: "Any Room".to_string(),
    };
    let rendered = template.render().unwrap();
    assert!(
        rendered.contains(r#"id="connection-status""#),
        "Expected connection-status id: {rendered}"
    );
    assert!(
        rendered.contains("Connected"),
        "Expected Connected text: {rendered}"
    );
}
