use crate::error::EngineError;
use crate::model::character::NpcCard;
use crate::model::settings::Connection;

use super::backend::LlmBackend;

#[derive(Clone, Default)]
#[allow(dead_code)]
pub struct DeepSeekBackend {
    api_key: String,
    model: String,
    max_context_tokens: u32,
}

impl DeepSeekBackend {
    pub fn from_connection(connection: &Connection) -> Self {
        let api_key = connection.resolve_api_key().unwrap_or_default();
        Self {
            api_key,
            model: connection.model.clone(),
            max_context_tokens: connection.resolve_max_context_tokens(),
        }
    }
}

impl LlmBackend for DeepSeekBackend {
    fn generate_dialogue(
        &self,
        _context: &crate::narrative::prompt::PromptContext,
        _npc: &NpcCard,
    ) -> Result<String, EngineError> {
        Err(EngineError::Config(
            "DeepSeek backend is not yet implemented. Configure an OpenRouter or Ollama connection."
                .into(),
        ))
    }

    fn narrate_action(
        &self,
        _context: &crate::narrative::prompt::PromptContext,
    ) -> Result<String, EngineError> {
        Err(EngineError::Config(
            "DeepSeek backend is not yet implemented. Configure an OpenRouter or Ollama connection."
                .into(),
        ))
    }

    fn narrate_arrival(
        &self,
        _context: &crate::narrative::prompt::PromptContext,
    ) -> Result<String, EngineError> {
        Err(EngineError::Config(
            "DeepSeek backend is not yet implemented. Configure an OpenRouter or Ollama connection."
                .into(),
        ))
    }

    fn narrate_continuation(
        &self,
        _system_prompt: &str,
        _user_prompt: &str,
        _trigger_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<String, EngineError> {
        Err(EngineError::Config(
            "DeepSeek backend is not yet implemented. Configure an OpenRouter or Ollama connection."
                .into(),
        ))
    }

    fn narrate_action_from_prompt(
        &self,
        _system_prompt: &str,
        _user_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<String, EngineError> {
        Err(EngineError::Config(
            "DeepSeek backend is not yet implemented. Configure an OpenRouter or Ollama connection."
                .into(),
        ))
    }

    fn name(&self) -> &str {
        "DeepSeek"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::character::{CharacterSheet, NpcCard};
    use crate::narrative::llm::test_support::{make_test_context, make_test_context_with_npc};

    #[test]
    fn test_deepseek_generate_dialogue() {
        let backend = DeepSeekBackend::default();
        let npc = NpcCard {
            id: "npc1".to_string(),
            sheet: CharacterSheet {
                name: "Test".to_string(),
                description: "Test".to_string(),
                personality: "Test".to_string(),
                scenario: "Test".to_string(),
                example_dialogue: "".to_string(),
                summary: None,
                profile_image: None,
                headshot_image: None,
            },
            inventory: vec![],
            triggers: vec![],
        };
        let result = backend.generate_dialogue(&make_test_context_with_npc(&npc, ""), &npc);
        assert!(
            result.is_err(),
            "DeepSeek generate_dialogue should return Err (not yet implemented)"
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("not yet implemented")
        );
    }

    #[test]
    fn test_deepseek_narrate_action() {
        let backend = DeepSeekBackend::default();
        let result = backend.narrate_action(&make_test_context("test"));
        assert!(
            result.is_err(),
            "DeepSeek narrate_action should return Err (not yet implemented)"
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("not yet implemented")
        );
    }

    #[test]
    fn test_deepseek_narrate_arrival() {
        let backend = DeepSeekBackend::default();
        let result = backend.narrate_arrival(&make_test_context(""));
        assert!(
            result.is_err(),
            "DeepSeek narrate_arrival should return Err (not yet implemented)"
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("not yet implemented")
        );
    }

    #[test]
    fn test_deepseek_name() {
        let backend = DeepSeekBackend::default();
        assert_eq!(backend.name(), "DeepSeek");
    }

    #[test]
    fn test_deepseek_narrate_continuation() {
        let backend = DeepSeekBackend::default();
        let result = backend.narrate_continuation("system", "user", "trigger", None);
        assert!(
            result.is_err(),
            "DeepSeek narrate_continuation should return Err (not yet implemented)"
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("not yet implemented")
        );
    }

    #[test]
    fn test_deepseek_narrate_action_from_prompt() {
        let backend = DeepSeekBackend::default();
        let result = backend.narrate_action_from_prompt("system", "user", None);
        assert!(
            result.is_err(),
            "DeepSeek narrate_action_from_prompt should return Err (not yet implemented)"
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("not yet implemented")
        );
    }

    #[test]
    fn test_deepseek_all_methods_return_not_implemented() {
        let backend = DeepSeekBackend::default();
        let npc = NpcCard {
            id: "npc1".to_string(),
            sheet: CharacterSheet {
                name: "Test".to_string(),
                description: "Test".to_string(),
                personality: "Test".to_string(),
                scenario: "Test".to_string(),
                example_dialogue: "".to_string(),
                summary: None,
                profile_image: None,
                headshot_image: None,
            },
            inventory: vec![],
            triggers: vec![],
        };

        let dialogue_result =
            backend.generate_dialogue(&make_test_context_with_npc(&npc, "test"), &npc);
        assert!(dialogue_result.is_err());

        let action_result = backend.narrate_action(&make_test_context("test"));
        assert!(action_result.is_err());

        let arrival_result = backend.narrate_arrival(&make_test_context("test"));
        assert!(arrival_result.is_err());

        let continuation_result = backend.narrate_continuation("sys", "user", "trigger", None);
        assert!(continuation_result.is_err());

        let prompt_result = backend.narrate_action_from_prompt("sys", "user", None);
        assert!(prompt_result.is_err());
    }

    #[test]
    fn test_deepseek_error_message_descriptive() {
        let backend = DeepSeekBackend::default();
        let result = backend.narrate_action(&make_test_context("test"));
        assert!(
            result.is_err(),
            "DeepSeek should return Err, not a placeholder string"
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("not yet implemented"),
            "Error message should explain the backend is unimplemented, got: {msg}"
        );
    }

    #[test]
    fn test_deepseek_backend_name() {
        let backend = DeepSeekBackend::default();
        assert_eq!(backend.name(), "DeepSeek");
    }
}
