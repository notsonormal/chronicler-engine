use crate::engine::action::Action;

pub fn parse_command(input: &str) -> Action {
    // [DOC: docs/architecture/system.md]
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
        "t" | "talk" => {
            if tokens.len() >= 2 {
                if tokens[1] == "to" && tokens.len() >= 3 {
                    Action::Talk(tokens[2..].join(" "), message)
                } else {
                    Action::Talk(tokens[1..].join(" "), message)
                }
            } else {
                Action::FreeAction(input.to_string())
            }
        }
        _ => Action::FreeAction(input.to_string()),
    }
}
