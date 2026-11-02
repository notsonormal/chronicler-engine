//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! The prompt part: one text with a role, an item of the list that a model call carries

/// [TRIVIAL_ENUM]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptRole {
    System,
    User,
    Assistant,
}

impl PromptRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
        }
    }

    pub fn from_wire_name(name: &str) -> Option<Self> {
        match name {
            "system" => Some(Self::System),
            "user" => Some(Self::User),
            "assistant" => Some(Self::Assistant),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptPart {
    pub role: PromptRole,
    pub text: String,
}

impl PromptPart {
    pub fn system(text: impl Into<String>) -> Self {
        Self {
            role: PromptRole::System,
            text: text.into(),
        }
    }

    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: PromptRole::User,
            text: text.into(),
        }
    }

    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: PromptRole::Assistant,
            text: text.into(),
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({ "role": self.role.as_str(), "content": self.text })
    }

    pub fn system_and_user(system: impl Into<String>, user: impl Into<String>) -> Vec<Self> {
        vec![Self::system(system), Self::user(user)]
    }
}
