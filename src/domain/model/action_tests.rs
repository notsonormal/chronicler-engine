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

#[test]
fn test_parse_guide_command() {
    assert_eq!(
        Action::parse("/guide make the scene tense"),
        Action::Guide("make the scene tense".to_string())
    );
}

#[test]
fn test_parse_guide_command_preserves_internal_spacing() {
    assert_eq!(
        Action::parse("/guide make   it   scary"),
        Action::Guide("make   it   scary".to_string())
    );
}

#[test]
fn test_parse_guide_command_empty_argument() {
    assert_eq!(Action::parse("/guide"), Action::Guide(String::new()));
}

#[test]
fn test_parse_narrator_command() {
    assert_eq!(
        Action::parse("/narrator The door creaks open on its own"),
        Action::Narrator("The door creaks open on its own".to_string())
    );
}

#[test]
fn test_parse_impersonate_command_with_direction() {
    assert_eq!(
        Action::parse("/impersonate act with false confidence"),
        Action::Impersonate(Some("act with false confidence".to_string()))
    );
}

#[test]
fn test_parse_impersonate_command_without_direction() {
    assert_eq!(Action::parse("/impersonate"), Action::Impersonate(None));
}

#[test]
fn test_parse_impersonate_command_trailing_whitespace_is_empty() {
    assert_eq!(Action::parse("/impersonate   "), Action::Impersonate(None));
}

#[test]
fn test_parse_slash_commands_are_case_insensitive() {
    assert_eq!(
        Action::parse("/GUIDE be brief"),
        Action::Guide("be brief".to_string())
    );
    assert_eq!(
        Action::parse("/Narrator thunder rolls"),
        Action::Narrator("thunder rolls".to_string())
    );
    assert_eq!(Action::parse("/IMPERSONATE"), Action::Impersonate(None));
}

#[test]
fn test_parse_unknown_slash_falls_back_to_free_action_verbatim() {
    assert_eq!(
        Action::parse("/shrug"),
        Action::FreeAction("/shrug".to_string())
    );
    assert_eq!(
        Action::parse("/emote waves"),
        Action::FreeAction("/emote waves".to_string())
    );
}

#[test]
fn test_parse_slash_after_leading_whitespace_is_recognized() {
    assert_eq!(
        Action::parse("   /guide hurry"),
        Action::Guide("hurry".to_string())
    );
}
