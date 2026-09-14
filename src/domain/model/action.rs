//! [DOC: docs/diataxis/reference/narrative/ai_steering.md]
//! Action enum and semantic command types (slash-command steering entry)

/// How the pipeline treats one submitted player input.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Ordinary player text — typed input or an unrecognized slash command —
    /// narrated as the player's action. Passes through the player-input text
    /// check like any free text.
    FreeAction(String),
    /// A steering instruction shaping how the narrator writes the next
    /// scene; never treated as an in-story player action.
    Guide(String),
    /// The player dictates their own character's action. The optional
    /// payload steers how the narrator renders it.
    Impersonate(Option<String>),
    /// Regenerate the pickable next-action set for the current scene on
    /// demand; narrates nothing and requires existing scene history.
    Options,
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
            // No-argument only: `/options something` falls through to the
            // narrator as player text, like any unknown slash command.
            "options" if argument.is_empty() => Self::Options,
            _ => Self::FreeAction(input.to_string()),
        }
    }

    pub fn is_steering(&self) -> bool {
        matches!(self, Self::Guide(_) | Self::Impersonate(_))
    }

    /// True for slash commands the engine itself handles. These bypass the
    /// player-input text check; unknown slash commands stay `FreeAction` and
    /// are checked like ordinary player text.
    pub fn is_engine_command(&self) -> bool {
        !matches!(self, Self::FreeAction(_))
    }
}
