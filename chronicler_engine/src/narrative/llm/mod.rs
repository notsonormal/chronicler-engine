pub mod backend;
pub mod deepseek;
pub mod mock;
pub mod ollama;
pub mod openrouter;

pub use backend::{
    LlmBackend, LlmBackendType, get_llm_backend, get_llm_backend_for,
    get_llm_backend_with_settings, merge_single_user_message,
};
pub use deepseek::DeepSeekBackend;
pub use mock::MockBackend;
pub use ollama::OllamaBackend;
pub use openrouter::OpenRouterBackend;

#[cfg(test)]
pub(crate) mod test_support {
    use std::collections::HashMap;

    use crate::model::character::{CharacterSheet, PlayerCard};
    use crate::model::map::Room;
    use crate::model::world::WorldCard;
    use crate::narrative::prompt::PromptContext;

    pub fn make_test_room() -> Room {
        // [DOC: docs/reference/testing.md]
        Room {
            id: "room1".to_string(),
            name: "Test Room".to_string(),
            description: "A plain room.".to_string(),
            exits: HashMap::new(),
            items: vec![],
            npcs: vec![],
            image_path: None,
            navigation_description: None,
        }
    }

    pub fn make_test_world() -> WorldCard {
        // [DOC: docs/reference/testing.md]
        WorldCard {
            name: "Test World".to_string(),
            description: "Testing.".to_string(),
            global_rules: vec!["Rule 1".to_string()],
            ..Default::default()
        }
    }

    pub fn make_test_player() -> PlayerCard {
        // [DOC: docs/reference/testing.md]
        PlayerCard {
            sheet: CharacterSheet {
                name: "Hero".to_string(),
                description: "The protagonist.".to_string(),
                personality: "Brave".to_string(),
                scenario: "Generic Quest".to_string(),
                example_dialogue: "".to_string(),
                summary: None,
                profile_image: None,
                headshot_image: None,
            },
            inventory: vec![],
        }
    }

    pub fn make_test_context(user_message: &str) -> PromptContext<'static> {
        // [DOC: docs/reference/testing.md]
        let world = make_test_world();
        let room = make_test_room();
        let player = make_test_player();
        let npcs: Vec<crate::model::character::NpcCard> = vec![];
        let user_msg = user_message.to_string();
        PromptContext {
            world: Box::leak(Box::new(world)),
            room: Box::leak(Box::new(room)),
            all_npcs: Box::leak(Box::new(npcs)),
            npcs_in_area: &[],
            player: Box::leak(Box::new(player)),
            user_message: Box::leak(Box::new(user_msg)),
            history: &[],
        }
    }

    pub fn make_test_context_with_npc(
        npc: &crate::model::character::NpcCard,
        user_message: &str,
    ) -> PromptContext<'static> {
        // [DOC: docs/reference/testing.md]
        let world = make_test_world();
        let room = make_test_room();
        let player = make_test_player();
        let npcs = vec![npc.clone()];
        let npcs_in_area = vec![npc.clone()];
        let user_msg = user_message.to_string();
        PromptContext {
            world: Box::leak(Box::new(world)),
            room: Box::leak(Box::new(room)),
            all_npcs: Box::leak(Box::new(npcs)),
            npcs_in_area: Box::leak(Box::new(npcs_in_area)),
            player: Box::leak(Box::new(player)),
            user_message: Box::leak(Box::new(user_msg)),
            history: &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AppSettings;
    use crate::EngineError;
    use crate::model::settings::Connection;

    fn make_settings_with_provider(provider: LlmBackendType) -> AppSettings {
        let conn = Connection::new("test-conn", "Test", provider);
        AppSettings {
            connections: vec![conn],
            narration_connection_id: "test-conn".into(),
            quantifier_connection_id: "test-conn".into(),
            response_length: "flexible".into(),
        }
    }

    #[test]
    fn test_get_llm_backend_with_settings_all_types() {
        let openrouter_settings = make_settings_with_provider(LlmBackendType::OpenRouter);
        assert_eq!(
            get_llm_backend_with_settings(&openrouter_settings).name(),
            "OpenRouter"
        );

        let mock_settings = make_settings_with_provider(LlmBackendType::Mock);
        assert_eq!(get_llm_backend_with_settings(&mock_settings).name(), "Mock");

        let deepseek_settings = make_settings_with_provider(LlmBackendType::DeepSeek);
        assert_eq!(
            get_llm_backend_with_settings(&deepseek_settings).name(),
            "DeepSeek"
        );

        let ollama_settings = make_settings_with_provider(LlmBackendType::Ollama);
        assert_eq!(
            get_llm_backend_with_settings(&ollama_settings).name(),
            "Ollama"
        );
    }

    #[test]
    fn test_llm_backend_type_from_env_default() {
        assert_eq!(LlmBackendType::from_env(), LlmBackendType::OpenRouter);
    }

    #[test]
    fn test_llm_empty_response_error_variant() {
        let err = EngineError::LlmEmptyResponse;
        assert_eq!(err.to_string(), "LLM returned an empty response");
    }
}
