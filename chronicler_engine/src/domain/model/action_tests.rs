use crate::domain::model::action::Action;

#[test]
fn test_parse_extra_whitespace() {
    assert_eq!(
        Action::parse("  north  "),
        Action::FreeAction("  north  ".to_string())
    );
    assert_eq!(
        Action::parse("  talk guard  "),
        Action::FreeAction("  talk guard  ".to_string())
    );
    assert_eq!(Action::parse("   "), Action::FreeAction("   ".to_string()));
}

#[test]
fn test_parse_free_action() {
    assert_eq!(
        Action::parse("Hello Carla, I'm the new heir."),
        Action::FreeAction("Hello Carla, I'm the new heir.".to_string())
    );
    assert_eq!(
        Action::parse("I examine the iron gates closely"),
        Action::FreeAction("I examine the iron gates closely".to_string())
    );
    assert_eq!(Action::parse(""), Action::FreeAction(String::new()));
}

#[test]
fn test_parse_quoted_dialogue_free_action() {
    assert_eq!(
        Action::parse("\"Who is this lady?\" you ask Carla"),
        Action::FreeAction("\"Who is this lady?\" you ask Carla".to_string())
    );
}

#[test]
fn test_parse_mixed_case_commands() {
    assert_eq!(
        Action::parse("Go North"),
        Action::FreeAction("Go North".to_string())
    );
    assert_eq!(
        Action::parse("Walk to the kitchen"),
        Action::FreeAction("Walk to the kitchen".to_string())
    );
    assert_eq!(
        Action::parse("Talk TO Carla"),
        Action::FreeAction("Talk TO Carla".to_string())
    );
}

#[test]
fn test_parse_north_as_free_action() {
    assert_eq!(
        Action::parse("north"),
        Action::FreeAction("north".to_string())
    );
    assert_eq!(Action::parse("n"), Action::FreeAction("n".to_string()));
    assert_eq!(
        Action::parse("south"),
        Action::FreeAction("south".to_string())
    );
    assert_eq!(Action::parse("s"), Action::FreeAction("s".to_string()));
}
