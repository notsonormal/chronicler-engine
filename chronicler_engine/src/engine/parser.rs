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
        "n" if tokens.len() == 1 => Action::WalkTo("north".to_string()),
        "s" if tokens.len() == 1 => Action::WalkTo("south".to_string()),
        "e" if tokens.len() == 1 => Action::WalkTo("east".to_string()),
        "w" if tokens.len() == 1 => Action::WalkTo("west".to_string()),
        "u" if tokens.len() == 1 => Action::WalkTo("up".to_string()),
        "d" if tokens.len() == 1 => Action::WalkTo("down".to_string()),
        "go" | "walk" | "move" if tokens.len() > 1 => {
            // Check if they typed "go to X" or "walk to X"
            if tokens[1] == "to" && tokens.len() >= 3 {
                Action::WalkTo(tokens[2..].join(" "))
            } else {
                Action::WalkTo(tokens[1..].join(" "))
            }
        }
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
    fn test_parse_look() {
        assert_eq!(parse_command("look"), Action::Look);
        assert_eq!(parse_command(" L "), Action::Look);
    }

    #[test]
    fn test_parse_navigate() {
        assert_eq!(parse_command("n"), Action::WalkTo("north".to_string()));
        assert_eq!(
            parse_command("go south"),
            Action::WalkTo("south".to_string())
        );
        assert_eq!(
            parse_command("walk to kitchen"),
            Action::WalkTo("kitchen".to_string())
        );
        assert_eq!(
            parse_command("go to front gates"),
            Action::WalkTo("front gates".to_string())
        );
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
}
