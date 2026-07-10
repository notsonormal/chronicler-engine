use crate::domain::engine::action::Action;
use crate::domain::engine::parser::parse_action;

#[test]
fn test_parse_extra_whitespace() {
    assert_eq!(
        parse_action("  north  "),
        Action::FreeAction("  north  ".to_string())
    );
    assert_eq!(
        parse_action("  talk guard  "),
        Action::FreeAction("  talk guard  ".to_string())
    );
    assert_eq!(parse_action("   "), Action::FreeAction("   ".to_string()));
}

#[test]
fn test_parse_free_action() {
    assert_eq!(
        parse_action("Hello Carla, I'm the new heir."),
        Action::FreeAction("Hello Carla, I'm the new heir.".to_string())
    );
    assert_eq!(
        parse_action("I examine the iron gates closely"),
        Action::FreeAction("I examine the iron gates closely".to_string())
    );
    assert_eq!(parse_action(""), Action::FreeAction(String::new()));
}

#[test]
fn test_parse_quoted_dialogue_free_action() {
    assert_eq!(
        parse_action("\"Who is this lady?\" you ask Carla"),
        Action::FreeAction("\"Who is this lady?\" you ask Carla".to_string())
    );
}

#[test]
fn test_parse_mixed_case_commands() {
    assert_eq!(
        parse_action("Go North"),
        Action::FreeAction("Go North".to_string())
    );
    assert_eq!(
        parse_action("Walk to the kitchen"),
        Action::FreeAction("Walk to the kitchen".to_string())
    );
    assert_eq!(
        parse_action("Talk TO Carla"),
        Action::FreeAction("Talk TO Carla".to_string())
    );
}

#[test]
fn test_parse_north_as_free_action() {
    assert_eq!(
        parse_action("north"),
        Action::FreeAction("north".to_string())
    );
    assert_eq!(parse_action("n"), Action::FreeAction("n".to_string()));
    assert_eq!(
        parse_action("south"),
        Action::FreeAction("south".to_string())
    );
    assert_eq!(parse_action("s"), Action::FreeAction("s".to_string()));
}
