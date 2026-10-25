use std::collections::HashMap;
use std::sync::Arc;

use crate::application::llm_recorder::LlmCallRecorder;
use crate::application::ports::llm_provider::{LlmCallResult, LlmProvider};
use crate::domain::model::character::CharacterCard;
use crate::domain::model::map::{MapDef, Overworld, Region, Room};
use crate::domain::model::state::swipe_state::SwipeState;
use crate::domain::model::agent::WorldView;
use crate::domain::model::state::message_types::StoryRow;
use crate::error::EngineError;
use crate::test_support::fixtures::{TestSwipeState, TestMap, TestCharacter, TestPersona};
use crate::test_support::make_test_recorder;

use super::orchestration::{
    determine_characters_present, quantify_room_with_llm_call, static_character_result,
};
use crate::test_support::quantifier::{make_character, make_room};
use crate::application::agents::quantifier::types::{
    MovementParseResult, MovementType, QuantifierConfidence, QuantifierPromptContext,
    QuantifierReading,
};

fn recorder_with(backend: impl LlmProvider + 'static) -> Arc<LlmCallRecorder> {
    make_test_recorder(Arc::new(backend))
}

#[test]
fn test_quantifier_retry_on_low_confidence() {
    let room = make_room();
    let carla = make_character("carla", "Carla");
    let all_characters = vec![carla];
    let previous_characters: Vec<CharacterCard> = vec![];
    let history: Vec<StoryRow> = vec![];

    let context = QuantifierPromptContext {
        room: &room,
        previous_characters_present: &previous_characters,
        all_known_characters: &all_characters,
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &history,
        player_action: "I look around.",
        quantifier_prompt_override: None,
    };

    let backend = crate::adapters::driven::llm::providers::MockBackend::default()
        .with_prompt_responses(vec![
            "I am not sure what to say here.".to_string(),
            r#"{"characters_present": ["carla"], "movement": {"type": null}}"#.to_string(),
        ]);

    let recorder = recorder_with(backend);
    let result = quantify_room_with_llm_call(&context, recorder.as_ref(), None);

    assert_eq!(result.characters.character_ids, vec!["carla".to_string()]);
    assert_eq!(result.characters.confidence, QuantifierConfidence::High);
    assert_eq!(result.movement.movement_type, None);
}

#[test]
fn test_quantifier_no_retry_when_high_confidence() {
    let room = make_room();
    let carla = make_character("carla", "Carla");
    let all_characters = vec![carla];
    let previous_characters: Vec<CharacterCard> = vec![];
    let history: Vec<StoryRow> = vec![];

    let context = QuantifierPromptContext {
        room: &room,
        previous_characters_present: &previous_characters,
        all_known_characters: &all_characters,
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &history,
        player_action: "I look around.",
        quantifier_prompt_override: None,
    };

    let backend = crate::adapters::driven::llm::providers::MockBackend::default()
        .with_prompt_responses(vec![
            r#"{"characters_present": ["carla"], "movement": {"type": null}}"#.to_string(),
        ]);

    let recorder = recorder_with(backend);
    let result = quantify_room_with_llm_call(&context, recorder.as_ref(), None);

    assert_eq!(result.characters.confidence, QuantifierConfidence::High);
}

fn make_test_llm_result(text: &str) -> LlmCallResult {
    LlmCallResult {
        text: text.to_string(),
        raw_request_json: "".to_string(),
        raw_response_json: "".to_string(),
        backend_name: "Test".to_string(),
        model_name: "test".to_string(),
        agent_name: "quantifier".to_string(),
    }
}

struct HighConfidenceBackend {
    character_ids: Vec<String>,
}

struct MediumConfidenceBackend;

struct LowConfidenceBackend;

struct ErrBackend;

impl LlmProvider for HighConfidenceBackend {
    fn model(&self) -> &str {
        "test"
    }
    fn name(&self) -> &str {
        "Test"
    }
    fn complete(
        &self,
        _agent_name: &str,
        _system_prompt: &str,
        _user_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        let characters_json = serde_json::to_string(&self.character_ids).unwrap_or_default();
        Ok(make_test_llm_result(&format!(
            r#"{{"characters_present": {characters_json}, "movement": {{"type": "entering", "destination": "kitchen"}}}}"#
        )))
    }
}

impl LlmProvider for MediumConfidenceBackend {
    fn model(&self) -> &str {
        "test"
    }
    fn name(&self) -> &str {
        "Test"
    }
    fn complete(
        &self,
        _agent_name: &str,
        _system_prompt: &str,
        _user_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        Ok(make_test_llm_result("Carla is standing in the room."))
    }
}

impl LlmProvider for LowConfidenceBackend {
    fn model(&self) -> &str {
        "test"
    }
    fn name(&self) -> &str {
        "Test"
    }
    fn complete(
        &self,
        _agent_name: &str,
        _system_prompt: &str,
        _user_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        Ok(make_test_llm_result(""))
    }
}

impl LlmProvider for ErrBackend {
    fn model(&self) -> &str {
        "test"
    }
    fn name(&self) -> &str {
        "Test"
    }
    fn complete(
        &self,
        _agent_name: &str,
        _system_prompt: &str,
        _user_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        Err(EngineError::Llm(crate::error::LlmFailure::Network {
            url: "mock".to_string(),
            detail: "mock failure".to_string(),
        }))
    }
}

fn display_name_for_id(id: &str) -> String {
    match id {
        "carla" => "Carla".to_string(),
        "gabriella" => "Gabriella".to_string(),
        other => other.to_string(),
    }
}

fn character_map_from_ids(ids: &[String]) -> HashMap<String, CharacterCard> {
    ids.iter()
        .map(|id| TestCharacter::named(id, &display_name_for_id(id)))
        .map(|character| (character.id.clone(), character))
        .collect()
}

fn map_with_room(room: &Room) -> MapDef {
    MapDef {
        overworld: Overworld {
            id: "test_overworld".to_string(),
            name: "Test Overworld".to_string(),
            regions: vec![Region {
                id: "test_region".to_string(),
                name: "Test Region".to_string(),
                rooms: vec![room.clone()],
            }],
        },
    }
}

fn determine_characters_with_room(
    state: &mut SwipeState,
    current_room: &Room,
    character_pool_ids: &[String],
    scene_characters: &[CharacterCard],
    player_action: &str,
    recorder: &LlmCallRecorder,
) -> QuantifierReading {
    use crate::domain::model::agent::{HookInput, WorldView};
    use crate::domain::model::state::message_types::StoryRow;
    let map = map_with_room(current_room);
    let persona = TestPersona::standard();
    let characters = character_map_from_ids(character_pool_ids);
    state.scene.characters_present = scene_characters.to_vec();
    let story: Vec<StoryRow> = Vec::new();
    let world = WorldView {
        room: Some(current_room),
        map: &map,
        persona: &persona,
        characters: &characters,
        present: scene_characters,
    };
    let input = HookInput {
        text: Some(player_action),
        story: &story,
        world: &world,
    };
    determine_characters_present(
        &input,
        recorder,
        &crate::domain::model::agent::AgentSettings::default(),
    )
    .expect("test should not fail hook-input resolution")
}

#[test]
fn test_determine_characters_high_confidence() {
    let mut state = TestSwipeState::in_room("hall");
    let room = make_room();

    let backend = HighConfidenceBackend {
        character_ids: vec!["carla".to_string()],
    };
    let recorder = recorder_with(backend);
    let result = determine_characters_with_room(
        &mut state,
        &room,
        &["carla".to_string(), "gabriella".to_string()],
        &[],
        "test",
        recorder.as_ref(),
    );

    assert_eq!(result.characters.character_ids, vec!["carla"]);
    assert_eq!(result.characters.confidence, QuantifierConfidence::High);
    assert_eq!(result.movement.movement_type, Some(MovementType::Entering));
    assert_eq!(result.movement.destination, Some("kitchen".to_string()));
}

#[test]
fn test_determine_characters_medium_confidence() {
    let mut state = TestSwipeState::in_room("hall");
    let room = make_room();

    let backend = MediumConfidenceBackend;
    let recorder = recorder_with(backend);
    let result = determine_characters_with_room(
        &mut state,
        &room,
        &["carla".to_string()],
        &[],
        "test",
        recorder.as_ref(),
    );

    assert_eq!(result.characters.character_ids, vec!["carla"]);
    assert_eq!(result.characters.confidence, QuantifierConfidence::Medium);
}

#[test]
fn test_determine_characters_low_confidence_fallback() {
    let mut state = TestSwipeState::in_room("hall");
    let room = make_room();

    let backend = LowConfidenceBackend;
    let recorder = recorder_with(backend);
    let scene_characters = vec![
        make_character("carla", "Carla"),
        make_character("gabriella", "Gabriella"),
    ];
    let result = determine_characters_with_room(
        &mut state,
        &room,
        &["carla".to_string(), "gabriella".to_string()],
        &scene_characters,
        "test",
        recorder.as_ref(),
    );

    assert_eq!(result.characters.character_ids, vec!["carla", "gabriella"]);
    assert_eq!(result.characters.confidence, QuantifierConfidence::Low);
}

#[test]
fn test_determine_characters_backend_error_fallback() {
    let mut state = TestSwipeState::in_room("hall");
    let room = make_room();

    let backend = ErrBackend;
    let recorder = recorder_with(backend);
    let scene_characters = vec![make_character("carla", "Carla")];
    let result = determine_characters_with_room(
        &mut state,
        &room,
        &["carla".to_string()],
        &scene_characters,
        "test",
        recorder.as_ref(),
    );

    assert_eq!(result.characters.character_ids, vec!["carla"]);
    assert_eq!(result.characters.confidence, QuantifierConfidence::Low);
}

#[test]
fn test_determine_characters_filters_unknown_backend_ids() {
    let mut state = TestSwipeState::in_room("hall");
    let room = make_room();

    let backend = HighConfidenceBackend {
        character_ids: vec!["carla".to_string(), "unknown".to_string()],
    };
    let recorder = recorder_with(backend);
    let result = determine_characters_with_room(
        &mut state,
        &room,
        &["carla".to_string()],
        &[],
        "test",
        recorder.as_ref(),
    );

    assert_eq!(result.characters.character_ids, vec!["carla"]);
}

#[test]
fn test_quantifier_retry_on_llm_error() {
    let room = make_room();
    let carla = make_character("carla", "Carla");
    let all_characters = vec![carla];
    let previous_characters: Vec<CharacterCard> = vec![];
    let history: Vec<StoryRow> = vec![];

    let context = QuantifierPromptContext {
        room: &room,
        previous_characters_present: &previous_characters,
        all_known_characters: &all_characters,
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &history,
        player_action: "I look around.",
        quantifier_prompt_override: None,
    };

    struct RotatingBackend {
        responses: std::sync::Mutex<Vec<Result<String, EngineError>>>,
    }
    impl LlmProvider for RotatingBackend {
        fn model(&self) -> &str {
            "test"
        }
        fn name(&self) -> &str {
            "Test"
        }
        fn complete(
            &self,
            _agent_name: &str,
            _system_prompt: &str,
            _user_prompt: &str,
            _max_tokens: Option<u32>,
        ) -> Result<LlmCallResult, EngineError> {
            let mut responses = self.responses.lock().unwrap();
            let text = responses.remove(0)?;
            Ok(make_test_llm_result(&text))
        }
    }

    let backend = RotatingBackend {
        responses: std::sync::Mutex::new(vec![
            Err(EngineError::Llm(crate::error::LlmFailure::Network {
                url: "mock".to_string(),
                detail: "Network error".to_string(),
            })),
            Ok(r#"{"characters_present": ["carla"], "movement": {"type": null}}"#.to_string()),
        ]),
    };

    let recorder = recorder_with(backend);
    let result = quantify_room_with_llm_call(&context, recorder.as_ref(), None);

    assert_eq!(result.characters.character_ids, vec!["carla"]);
}

#[test]
fn test_quantifier_all_attempts_fail_fallback() {
    let room = make_room();
    let all_characters: Vec<CharacterCard> = vec![];
    let previous_characters: Vec<CharacterCard> = vec![];
    let history: Vec<StoryRow> = vec![];

    let context = QuantifierPromptContext {
        room: &room,
        previous_characters_present: &previous_characters,
        all_known_characters: &all_characters,
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &history,
        player_action: "I look around.",
        quantifier_prompt_override: None,
    };

    let backend = crate::adapters::driven::llm::providers::MockBackend::default()
        .with_trigger_narration_fail();

    let recorder = recorder_with(backend);
    let result = quantify_room_with_llm_call(&context, recorder.as_ref(), None);

    assert_eq!(result.characters.confidence, QuantifierConfidence::Low);
    assert_eq!(result.movement.movement_type, None);
}

#[test]
fn test_quantifier_low_confidence_then_error_fallback() {
    let room = make_room();
    let all_characters: Vec<CharacterCard> = vec![];
    let previous_characters: Vec<CharacterCard> = vec![];
    let history: Vec<StoryRow> = vec![];

    let context = QuantifierPromptContext {
        room: &room,
        previous_characters_present: &previous_characters,
        all_known_characters: &all_characters,
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &history,
        player_action: "I look around.",
        quantifier_prompt_override: None,
    };

    let backend = crate::adapters::driven::llm::providers::MockBackend::default()
        .with_prompt_responses(vec!["I am not sure what to say here.".to_string()])
        .with_trigger_narration_fail();

    let recorder = recorder_with(backend);
    let result = quantify_room_with_llm_call(&context, recorder.as_ref(), None);

    assert_eq!(result.characters.confidence, QuantifierConfidence::Low);
}

#[test]
fn test_static_character_result_preserves_movement() {
    let state = TestSwipeState::in_room("hall");
    let movement = MovementParseResult {
        movement_type: Some(MovementType::Leaving),
        destination: Some("garden".to_string()),
    };

    let characters = std::collections::HashMap::new();
    let map = TestMap::single_room("hall");
    let persona = TestPersona::standard();
    let world = WorldView {
        room: map.get_room_by_id("hall"),
        map: &map,
        persona: &persona,
        characters: &characters,
        present: &state.scene.characters_present,
    };
    let result = static_character_result(&world, movement.clone());

    assert!(result.characters.character_ids.is_empty());
    assert_eq!(result.movement.movement_type, Some(MovementType::Leaving));
    assert_eq!(result.movement.destination, Some("garden".to_string()));
}

#[test]
fn test_static_character_result_fallback_to_scene_characters() {
    let mut state = TestSwipeState::in_room("hall");
    state
        .scene
        .characters_present
        .push(TestCharacter::named("carla", "Carla"));

    let characters = std::collections::HashMap::new();
    let map = TestMap::single_room("hall");
    let persona = TestPersona::standard();
    let world = WorldView {
        room: map.get_room_by_id("hall"),
        map: &map,
        persona: &persona,
        characters: &characters,
        present: &state.scene.characters_present,
    };
    let result = static_character_result(&world, MovementParseResult::default());

    assert_eq!(result.characters.character_ids, vec!["carla"]);
}

#[test]
fn test_quantifier_routes_through_recorder_for_forensics() {
    use crate::adapters::driven::storage::Storage;
    use crate::test_support::make_test_recorder_with_storage;

    let room = make_room();
    let carla = make_character("carla", "Carla");
    let all_characters = vec![carla];
    let previous_characters: Vec<CharacterCard> = vec![];
    let history: Vec<StoryRow> = vec![];

    let context = QuantifierPromptContext {
        room: &room,
        previous_characters_present: &previous_characters,
        all_known_characters: &all_characters,
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &history,
        player_action: "I look around.",
        quantifier_prompt_override: None,
    };

    let backend = crate::adapters::driven::llm::providers::MockBackend::default()
        .with_prompt_responses(vec![
            r#"{"characters_present": ["carla"], "movement": {"type": null}}"#.to_string(),
        ]);

    let storage = Arc::new(Storage::new_in_memory());
    let recorder = make_test_recorder_with_storage(Arc::new(backend), Arc::clone(&storage));
    let result = quantify_room_with_llm_call(&context, recorder.as_ref(), None);

    assert_eq!(result.characters.confidence, QuantifierConfidence::High);
    assert_eq!(
        storage
            .list_latest_llm_messages(10)
            .expect("list should not error")
            .len(),
        1,
        "quantifier LLM call must route through LlmCallRecorder so forensics are saved exactly once"
    );
}
