use crate::error::EngineError;
use crate::model::character::NpcCard;
use crate::model::settings::{AppSettings, Connection};
use crate::narrative::prompt::PromptContext;

pub trait LlmBackend: Send + Sync {
    fn generate_dialogue(
        &self,
        context: &PromptContext,
        npc: &NpcCard,
    ) -> Result<String, EngineError>;

    fn narrate_action(&self, context: &PromptContext) -> Result<String, EngineError>;

    fn narrate_arrival(&self, context: &PromptContext) -> Result<String, EngineError>;

    fn narrate_continuation(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        trigger_prompt: &str,
        max_tokens: Option<u32>,
    ) -> Result<String, EngineError>;

    fn narrate_action_from_prompt(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        max_tokens: Option<u32>,
    ) -> Result<String, EngineError>;

    fn name(&self) -> &str;
}

// [DOC: docs/system/llm_processing.md]
pub use crate::model::llm_backend::LlmBackendType;

/// Create an LLM backend for a specific connection.
/// [DOC: docs/system/llm_processing.md]
pub fn get_llm_backend_for(connection: &Connection) -> Box<dyn LlmBackend> {
    match connection.provider {
        LlmBackendType::Mock => Box::new(super::mock::MockBackend::default()),
        LlmBackendType::DeepSeek => Box::new(super::deepseek::DeepSeekBackend::from_connection(
            connection,
        )),
        LlmBackendType::OpenRouter => Box::new(
            super::openrouter::OpenRouterBackend::from_connection(connection),
        ),
        LlmBackendType::Ollama => {
            Box::new(super::ollama::OllamaBackend::from_connection(connection))
        }
    }
}

/// Get the LLM backend for the current narration connection.
/// Respects test overrides for backward compatibility with tests.
/// [DOC: docs/system/llm_processing.md]
pub fn get_llm_backend() -> Box<dyn LlmBackend> {
    let settings = crate::settings::load_settings().unwrap_or_default();
    let connection = settings
        .get_narration_connection()
        .cloned()
        .unwrap_or_else(|| Connection::new("default", "Default", LlmBackendType::Mock));
    get_llm_backend_for(&connection)
}

/// Backward-compatible helper used by some tests.
pub fn get_llm_backend_with_settings(settings: &AppSettings) -> Box<dyn LlmBackend> {
    let connection = settings
        .get_narration_connection()
        .cloned()
        .unwrap_or_else(|| Connection::new("default", "Default", LlmBackendType::Mock));
    get_llm_backend_for(&connection)
}

/// Merge system and user prompts into a single user message.
/// Used for models that ignore the system role.
pub fn merge_single_user_message(system_prompt: &str, user_text: &str) -> String {
    format!("[SYSTEM]\n{system_prompt}\n\n{user_text}")
}

#[cfg(test)]
mod tests {
    use super::merge_single_user_message;

    #[test]
    fn test_merge_single_user_message_format() {
        let merged = merge_single_user_message("system content", "user content");
        assert!(merged.starts_with("[SYSTEM]\n"));
        assert!(merged.contains("system content"));
        assert!(merged.contains("user content"));
        // System content should come before user content
        let system_pos = merged.find("system content").unwrap();
        let user_pos = merged.find("user content").unwrap();
        assert!(system_pos < user_pos);
    }

    #[test]
    fn test_merge_single_user_message_preserves_multiline() {
        let system = "Line 1\nLine 2";
        let user = "User Line 1\nUser Line 2";
        let merged = merge_single_user_message(system, user);
        assert!(merged.contains("Line 1\nLine 2"));
        assert!(merged.contains("User Line 1\nUser Line 2"));
    }

    #[test]
    fn test_merge_single_user_message_empty_system() {
        let merged = merge_single_user_message("", "user content");
        assert_eq!(merged, "[SYSTEM]\n\n\nuser content");
    }

    #[test]
    fn test_merge_single_user_message_empty_user() {
        let merged = merge_single_user_message("system content", "");
        assert_eq!(merged, "[SYSTEM]\nsystem content\n\n");
    }

    #[test]
    fn test_merge_single_user_message_both_empty() {
        let merged = merge_single_user_message("", "");
        assert_eq!(merged, "[SYSTEM]\n\n\n");
    }
}
