use crate::domain::engine::action::Action;
use crate::domain::engine::parser::parse_command;

#[test]
fn test_parse_extra_whitespace() {
    assert_eq!(
        parse_command("  north  "),
        Action::FreeAction("  north  ".to_string())
    );
    assert_eq!(
        parse_command("  talk guard  "),
        Action::FreeAction("  talk guard  ".to_string())
    );
    assert_eq!(parse_command("   "), Action::FreeAction("   ".to_string()));
}

#[test]
fn test_parse_free_action() {
    assert_eq!(
        parse_command("Hello Carla, I'm the new heir."),
        Action::FreeAction("Hello Carla, I'm the new heir.".to_string())
    );
    assert_eq!(
        parse_command("I examine the iron gates closely"),
        Action::FreeAction("I examine the iron gates closely".to_string())
    );
    assert_eq!(parse_command(""), Action::FreeAction(String::new()));
}

#[test]
fn test_parse_quoted_dialogue_free_action() {
    assert_eq!(
        parse_command("\"Who is this lady?\" you ask Carla"),
        Action::FreeAction("\"Who is this lady?\" you ask Carla".to_string())
    );
}

#[test]
fn test_parse_mixed_case_commands() {
    assert_eq!(
        parse_command("Go North"),
        Action::FreeAction("Go North".to_string())
    );
    assert_eq!(
        parse_command("Walk to the kitchen"),
        Action::FreeAction("Walk to the kitchen".to_string())
    );
    assert_eq!(
        parse_command("Talk TO Carla"),
        Action::FreeAction("Talk TO Carla".to_string())
    );
}

#[test]
fn test_parse_north_as_free_action() {
    assert_eq!(
        parse_command("north"),
        Action::FreeAction("north".to_string())
    );
    assert_eq!(parse_command("n"), Action::FreeAction("n".to_string()));
    assert_eq!(
        parse_command("south"),
        Action::FreeAction("south".to_string())
    );
    assert_eq!(parse_command("s"), Action::FreeAction("s".to_string()));
}
