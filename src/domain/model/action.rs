//! [DOC: docs/diataxis/reference/narrative/ai_steering.md]
//! Action enum and semantic command types (slash-command steering entry)

/// [TRIVIAL_ENUM]
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    FreeAction(String),
    Guide(String),
    Impersonate(Option<String>),
}

impl Action {
    pub fn parse(input: &str) -> Self {
        let Some(after_slash) = input.trim_start().strip_prefix('/') else {
            return Self::FreeAction(input.to_string());
        };
        let (command, argument) = match after_slash.find(char::is_whitespace) {
            Some(index) => (&after_slash[..index], after_slash[index..].trim()),
            None => (after_slash, ""),
        };
        match command.to_ascii_lowercase().as_str() {
            "guide" => Self::Guide(argument.to_string()),
            "impersonate" => Self::Impersonate(if argument.is_empty() {
                None
            } else {
                Some(argument.to_string())
            }),
            _ => Self::FreeAction(input.to_string()),
        }
    }

    pub fn is_steering(&self) -> bool {
        matches!(self, Self::Guide(_) | Self::Impersonate(_))
    }
}
