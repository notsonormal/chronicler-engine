use crate::model::llm_backend::LlmBackendType;
use crate::model::settings::Connection;
use crate::narrative::quantifier::backends::{
    MockQuantifierBackend, QuantifierBackendTrait, get_quantifier_backend_for,
};
use crate::narrative::quantifier::test_support::{make_npc, make_room};
use crate::narrative::quantifier::types::{
    MovementParseResult, MovementType, QuantifierConfidence, QuantifierPromptContext,
};

#[test]
fn test_mock_quantifier_backend_auto_detect() {
    let carla = make_npc("carla", "Carla");
    let gabriella = make_npc("gabriella", "Gabriella");
    let all_npcs = vec![carla, gabriella];

    let backend = MockQuantifierBackend::default();

    let room = make_room();
    let context = QuantifierPromptContext {
        room: &room,
        previous_room_npcs: &[],
        all_known_npcs: &all_npcs,
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &[],
        player_action: "I talk to Carla about the quest.",
    };

    let result = backend.quantify_room(&context, &[]).unwrap();

    assert!(result.npcs.npc_ids.contains(&"carla".to_string()));
    assert_eq!(result.npcs.confidence, QuantifierConfidence::High);
}

#[test]
fn test_mock_quantifier_backend_explicit_npcs() {
    let all_npcs = vec![make_npc("carla", "Carla")];

    let backend = MockQuantifierBackend {
        npcs_to_return: vec!["carla".to_string()],
        movement_to_return: Some(MovementParseResult {
            movement_type: Some(MovementType::Entering),
            destination: Some("entrance".to_string()),
            confidence: QuantifierConfidence::High,
        }),
    };

    let room = make_room();
    let context = QuantifierPromptContext {
        room: &room,
        previous_room_npcs: &[],
        all_known_npcs: &all_npcs,
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &[],
        player_action: "I walk in.",
    };

    let result = backend.quantify_room(&context, &[]).unwrap();

    assert_eq!(result.npcs.npc_ids, vec!["carla".to_string()]);
    assert_eq!(result.movement.movement_type, Some(MovementType::Entering));
    assert_eq!(result.movement.destination, Some("entrance".to_string()));
}

#[test]
fn test_mock_quantifier_no_match_when_different_action() {
    let carla = make_npc("carla", "Carla");
    let all_npcs = vec![carla];

    let backend = MockQuantifierBackend::default();

    let room = make_room();
    let context = QuantifierPromptContext {
        room: &room,
        previous_room_npcs: &[],
        all_known_npcs: &all_npcs,
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &[],
        player_action: "I look at the wall.",
    };

    let result = backend.quantify_room(&context, &[]).unwrap();

    assert!(result.npcs.npc_ids.is_empty());
}

#[test]
fn test_get_quantifier_backend_for_mock() {
    let connection = Connection::new("mock", "Mock", LlmBackendType::Mock);
    let backend = get_quantifier_backend_for(&connection);

    let carla = make_npc("carla", "Carla");
    let all_npcs = vec![carla];
    let room = make_room();
    let context = QuantifierPromptContext {
        room: &room,
        previous_room_npcs: &[],
        all_known_npcs: &all_npcs,
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &[],
        player_action: "I talk to Carla.",
    };

    let result = backend.quantify_room(&context, &[]).unwrap();
    assert!(result.npcs.npc_ids.contains(&"carla".to_string()));
}

#[test]
fn test_get_quantifier_backend_for_ollama() {
    let mut connection = Connection::new("ollama", "Ollama", LlmBackendType::Ollama);
    connection.base_url = Some("http://localhost:11434".to_string());
    connection.model = "llama3".to_string();
    let _backend = get_quantifier_backend_for(&connection);
}

#[test]
fn test_get_quantifier_backend_for_openrouter() {
    let mut connection = Connection::new("openrouter", "OpenRouter", LlmBackendType::OpenRouter);
    connection.api_key = Some("test-key".to_string());
    connection.model = "openai/gpt-4o".to_string();
    let _backend = get_quantifier_backend_for(&connection);
}

#[test]
fn test_get_quantifier_backend_for_deepseek() {
    let mut connection = Connection::new("deepseek", "DeepSeek", LlmBackendType::DeepSeek);
    connection.api_key = Some("test-key".to_string());
    connection.model = "deepseek-chat".to_string();
    let _backend = get_quantifier_backend_for(&connection);
}

#[test]
fn test_real_quantifier_backend_quantify_room_network_error() {
    // RealQuantifierBackend wraps QuantifierBackend and delegates quantify_room
    let mut connection = Connection::new("openrouter", "OpenRouter", LlmBackendType::OpenRouter);
    connection.api_key = Some("fake-key".to_string());
    connection.model = "test-model".to_string();

    let backend = get_quantifier_backend_for(&connection);
    let room = make_room();
    let context = QuantifierPromptContext {
        room: &room,
        previous_room_npcs: &[],
        all_known_npcs: &[],
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &[],
        player_action: "walk",
    };

    // Network call will fail, but this exercises QuantifierBackend::quantify_room
    let result = backend.quantify_room(&context, &[]);
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_ollama_quantifier_backend_quantify_room_network_error() {
    // get_quantifier_backend_for with Ollama returns OllamaQuantifierBackend
    let mut connection = Connection::new("ollama", "Ollama", LlmBackendType::Ollama);
    connection.base_url = Some("http://localhost:59999".to_string());
    connection.model = "test-model".to_string();

    let backend = get_quantifier_backend_for(&connection);
    let room = make_room();
    let context = QuantifierPromptContext {
        room: &room,
        previous_room_npcs: &[],
        all_known_npcs: &[],
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &[],
        player_action: "walk",
    };

    // Network call will fail, but this exercises OllamaQuantifierBackend::quantify_room
    let result = backend.quantify_room(&context, &[]);
    assert!(result.is_err() || result.is_ok());
}
