use crate::engine::action::Action;

pub fn parse_command(input: &str) -> Action {
    // Handle quoted string for messages (e.g. talk carla "hello")
    let (base_input, message) = if let Some(start_quote) = input.find('"') {
        if let Some(end_quote) = input.rfind('"') {
            if end_quote > start_quote {
                let msg = input[start_quote + 1..end_quote].to_string();
                let base = input[..start_quote].trim();
                (base, Some(msg))
            } else {
                (input.trim(), None)
            }
        } else {
            (input.trim(), None)
        }
    } else {
        (input.trim(), None)
    };

    let lower = base_input.to_lowercase();
    let tokens: Vec<&str> = lower.split_whitespace().collect();

    if tokens.is_empty() {
        return Action::FreeAction(input.to_string());
    }

    match tokens[0] {
        "l" | "look" if tokens.len() == 1 => Action::Look,
        "i" | "inv" | "inventory" if tokens.len() == 1 => Action::Inventory,
        "t" | "talk" => {
            if tokens.len() >= 2 {
                // If the command is "talk to Gary", ignore the "to"
                if tokens[1] == "to" && tokens.len() >= 3 {
                    Action::Talk(tokens[2..].join(" "), message)
                } else {
                    Action::Talk(tokens[1..].join(" "), message)
                }
            } else {
                Action::FreeAction(input.to_string())
            }
        }
        "q" | "quit" | "exit" => Action::Quit,
        _ => Action::FreeAction(input.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_extra_whitespace() {
        // Extra whitespace handling - original input is preserved in FreeAction
        // "north" is now a FreeAction (quantifier-driven movement)
        assert_eq!(
            parse_command("  north  "),
            Action::FreeAction("  north  ".to_string())
        );
        assert_eq!(
            parse_command("  talk guard  "),
            Action::Talk("guard".to_string(), None)
        );
        // Whitespace-only becomes FreeAction with the whitespace
        assert_eq!(parse_command("   "), Action::FreeAction("   ".to_string()));
    }

    #[test]
    fn test_parse_talk() {
        assert_eq!(
            parse_command("talk gary"),
            Action::Talk("gary".to_string(), None)
        );
        assert_eq!(
            parse_command("talk to gary"),
            Action::Talk("gary".to_string(), None)
        );
    }

    #[test]
    fn test_parse_talk_with_message() {
        assert_eq!(
            parse_command("talk carla \"Who are you?\""),
            Action::Talk("carla".to_string(), Some("Who are you?".to_string()))
        );
        assert_eq!(
            parse_command("talk to carla \"Hello there!\""),
            Action::Talk("carla".to_string(), Some("Hello there!".to_string()))
        );
    }

    #[test]
    fn test_parse_free_action() {
        // Anything not matching a command should become FreeAction
        assert_eq!(
            parse_command("Hello Carla, I'm the new heir."),
            Action::FreeAction("Hello Carla, I'm the new heir.".to_string())
        );
        assert_eq!(
            parse_command("I examine the iron gates closely"),
            Action::FreeAction("I examine the iron gates closely".to_string())
        );
        // Empty input should also become FreeAction (handled silently in the REPL)
        assert_eq!(parse_command(""), Action::FreeAction(String::new()));
    }

    #[test]
    fn test_parse_quoted_dialogue_free_action() {
        // This was previously failing because it extracted the quote and left the base empty
        assert_eq!(
            parse_command("\"Who is this lady?\" you ask Carla"),
            Action::FreeAction("\"Who is this lady?\" you ask Carla".to_string())
        );
    }

    #[test]
    fn test_parse_look() {
        // Look command variants
        assert_eq!(parse_command("look"), Action::Look);
        assert_eq!(parse_command("l"), Action::Look);
        assert_eq!(parse_command("LOOK"), Action::Look);
        assert_eq!(
            parse_command("Look around"),
            Action::FreeAction("Look around".to_string())
        );
    }

    #[test]
    fn test_parse_inventory() {
        // Inventory command variants
        assert_eq!(parse_command("inventory"), Action::Inventory);
        assert_eq!(parse_command("inv"), Action::Inventory);
        assert_eq!(parse_command("i"), Action::Inventory);
        assert_eq!(parse_command("INVENTORY"), Action::Inventory);
        // "inventory" with arguments should be free action
        assert_eq!(
            parse_command("inventory check"),
            Action::FreeAction("inventory check".to_string())
        );
    }

    #[test]
    fn test_parse_quit() {
        // Quit command variants
        assert_eq!(parse_command("quit"), Action::Quit);
        assert_eq!(parse_command("q"), Action::Quit);
        assert_eq!(parse_command("exit"), Action::Quit);
        assert_eq!(parse_command("QUIT"), Action::Quit);
    }

    #[test]
    fn test_parse_talk_variants() {
        // Various talk command formats
        assert_eq!(
            parse_command("talk guard"),
            Action::Talk("guard".to_string(), None)
        );
        assert_eq!(
            parse_command("speak to innkeeper"),
            Action::FreeAction("speak to innkeeper".to_string())
        );
        assert_eq!(
            parse_command("say hello"),
            Action::FreeAction("say hello".to_string())
        );
    }

    #[test]
    fn test_parse_mixed_case_commands() {
        // Mixed case handling - explicit commands become FreeAction (quantifier interprets)
        // FreeAction preserves original case
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
            Action::Talk("carla".to_string(), None)
        );
        assert_eq!(parse_command("InVeNtOrY"), Action::Inventory);
    }

    #[test]
    fn test_parse_north_as_free_action() {
        // Cardinal directions now become FreeAction (preserving original case)
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
}
