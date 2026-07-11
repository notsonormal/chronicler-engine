use std::collections::HashMap;
use std::sync::Arc;

use crate::application::llm_recorder::LlmCallRecorder;
use crate::application::ports::llm_provider::{LlmCallResult, LlmProvider};
use crate::domain::model::character::NpcCard;
use crate::domain::model::map::{MapDef, Overworld, Region, Room};
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::message_types::MessageEntry;
use crate::error::EngineError;
use crate::test_support::fixtures::{TestGameState, TestNpc, TestPersona};
use crate::test_support::noop_forensics::make_test_recorder;

use super::orchestration::{determine_npcs_in_room, quantify_room_with_llm_call, static_npc_result};
use super::test_support::{make_npc, make_room};
use super::types::{MovementParseResult, MovementType, QuantifierConfidence, QuantifierPromptContext};

fn recorder_with(backend: impl LlmProvider + 'static) -> Arc<LlmCallRecorder> {
    make_test_recorder(Arc::new(backend))
}

#[test]
fn test_quantifier_retry_on_low_confidence() {
    let room = make_room();
    let carla = make_npc("carla", "Carla");
    let all_npcs = vec![carla];
    let previous_npcs: Vec<NpcCard> = vec![];
    let history: Vec<MessageEntry> = vec![];

    let context = QuantifierPromptContext {
        room: &room,
        previous_room_npcs: &previous_npcs,
        all_known_npcs: &all_npcs,
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &history,
        player_action: "I look around.",
        quantifier_prompt_override: None,
    };

    let backend = crate::adapters::driven::llm::providers::MockBackend::default()
        .with_prompt_responses(vec![
            "I am not sure what to say here.".to_string(),
            r#"{"npcs_in_room": ["carla"], "movement": {"type": null}}"#.to_string(),
        ]);

    let recorder = recorder_with(backend);
    let result = quantify_room_with_llm_call(&context, &["carla".to_string()], recorder.as_ref());

    assert_eq!(result.npcs.npc_ids, vec!["carla".to_string()]);
    assert_eq!(result.npcs.confidence, QuantifierConfidence::High);
    assert_eq!(result.movement.movement_type, None);
}

#[test]
fn test_quantifier_no_retry_when_high_confidence() {
    let room = make_room();
    let carla = make_npc("carla", "Carla");
    let all_npcs = vec![carla];
    let previous_npcs: Vec<NpcCard> = vec![];
    let history: Vec<MessageEntry> = vec![];

    let context = QuantifierPromptContext {
        room: &room,
        previous_room_npcs: &previous_npcs,
        all_known_npcs: &all_npcs,
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &history,
        player_action: "I look around.",
        quantifier_prompt_override: None,
    };

    let backend = crate::adapters::driven::llm::providers::MockBackend::default()
        .with_prompt_responses(vec![
            r#"{"npcs_in_room": ["carla"], "movement": {"type": null}}"#.to_string(),
        ]);

    let recorder = recorder_with(backend);
    let result = quantify_room_with_llm_call(&context, &["carla".to_string()], recorder.as_ref());

    assert_eq!(result.npcs.confidence, QuantifierConfidence::High);
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
    npc_ids: Vec<String>,
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
        let npcs_json = serde_json::to_string(&self.npc_ids).unwrap_or_default();
        Ok(make_test_llm_result(&format!(
            r#"{{"npcs_in_room": {npcs_json}, "movement": {{"type": "entering", "destination": "kitchen"}}}}"#
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

fn npc_map_from_ids(ids: &[String]) -> HashMap<String, NpcCard> {
    ids.iter()
        .map(|id| TestNpc::named(id, &display_name_for_id(id)))
        .map(|npc| (npc.id.clone(), npc))
        .collect()
}

fn npc_map(npcs: Vec<NpcCard>) -> HashMap<String, NpcCard> {
    npcs.into_iter().map(|npc| (npc.id.clone(), npc)).collect()
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

fn determine_npcs_with_room(
    state: &GameState,
    current_room: &Room,
    room_npc_ids: &[String],
    previous_room_npcs: &[crate::domain::model::character::NpcCard],
    player_action: &str,
    recorder: &LlmCallRecorder,
) -> crate::domain::model::quantifier::QuantifierResult {
    let map = map_with_room(current_room);
    let persona = TestPersona::standard();
    let mut npcs = npc_map_from_ids(room_npc_ids);
    for npc in previous_room_npcs {
        npcs.insert(npc.id.clone(), npc.clone());
    }
    determine_npcs_in_room(
        state,
        current_room,
        room_npc_ids,
        previous_room_npcs,
        player_action,
        recorder,
        None,
        &map,
        &persona,
        &npcs,
    )
}

#[test]
fn test_determine_npcs_high_confidence() {
    let state = TestGameState::in_room("hall");
    let room = make_room();

    let backend = HighConfidenceBackend {
        npc_ids: vec!["carla".to_string()],
    };
    let recorder = recorder_with(backend);
    let result = determine_npcs_with_room(
        &state,
        &room,
        &["carla".to_string(), "gabriella".to_string()],
        &[],
        "test",
        recorder.as_ref(),
    );

    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
    assert_eq!(result.npcs.confidence, QuantifierConfidence::High);
    assert_eq!(result.movement.movement_type, Some(MovementType::Entering));
    assert_eq!(result.movement.destination, Some("kitchen".to_string()));
}

#[test]
fn test_determine_npcs_medium_confidence() {
    let state = TestGameState::in_room("hall");
    let room = make_room();

    let backend = MediumConfidenceBackend;
    let recorder = recorder_with(backend);
    let result = determine_npcs_with_room(
        &state,
        &room,
        &["carla".to_string()],
        &[],
        "test",
        recorder.as_ref(),
    );

    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
    assert_eq!(result.npcs.confidence, QuantifierConfidence::Medium);
}

#[test]
fn test_determine_npcs_low_confidence_fallback() {
    let state = TestGameState::in_room("hall");
    let room = make_room();

    let backend = LowConfidenceBackend;
    let recorder = recorder_with(backend);
    let result = determine_npcs_with_room(
        &state,
        &room,
        &["carla".to_string(), "gabriella".to_string()],
        &[],
        "test",
        recorder.as_ref(),
    );

    assert_eq!(result.npcs.npc_ids, vec!["carla", "gabriella"]);
    assert_eq!(result.npcs.confidence, QuantifierConfidence::Low);
}

#[test]
fn test_determine_npcs_backend_error_fallback() {
    let state = TestGameState::in_room("hall");
    let room = make_room();

    let backend = ErrBackend;
    let recorder = recorder_with(backend);
    let result = determine_npcs_with_room(
        &state,
        &room,
        &["carla".to_string()],
        &[],
        "test",
        recorder.as_ref(),
    );

    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
    assert_eq!(result.npcs.confidence, QuantifierConfidence::Low);
}

#[test]
fn test_determine_npcs_filters_unknown_backend_ids() {
    let state = TestGameState::in_room("hall");
    let room = make_room();

    let backend = HighConfidenceBackend {
        npc_ids: vec!["carla".to_string(), "unknown".to_string()],
    };
    let recorder = recorder_with(backend);
    let result = determine_npcs_with_room(
        &state,
        &room,
        &["carla".to_string()],
        &[],
        "test",
        recorder.as_ref(),
    );

    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
}

#[test]
fn test_quantifier_retry_on_llm_error() {
    let room = make_room();
    let carla = make_npc("carla", "Carla");
    let all_npcs = vec![carla];
    let previous_npcs: Vec<NpcCard> = vec![];
    let history: Vec<MessageEntry> = vec![];

    let context = QuantifierPromptContext {
        room: &room,
        previous_room_npcs: &previous_npcs,
        all_known_npcs: &all_npcs,
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
            Ok(r#"{"npcs_in_room": ["carla"], "movement": {"type": null}}"#.to_string()),
        ]),
    };

    let recorder = recorder_with(backend);
    let result = quantify_room_with_llm_call(&context, &["carla".to_string()], recorder.as_ref());

    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
}

#[test]
fn test_quantifier_all_attempts_fail_fallback() {
    let room = make_room();
    let all_npcs: Vec<NpcCard> = vec![];
    let previous_npcs: Vec<NpcCard> = vec![];
    let history: Vec<MessageEntry> = vec![];

    let context = QuantifierPromptContext {
        room: &room,
        previous_room_npcs: &previous_npcs,
        all_known_npcs: &all_npcs,
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &history,
        player_action: "I look around.",
        quantifier_prompt_override: None,
    };

    let backend = crate::adapters::driven::llm::providers::MockBackend::default()
        .with_trigger_narration_fail();

    let recorder = recorder_with(backend);
    let result = quantify_room_with_llm_call(&context, &["carla".to_string()], recorder.as_ref());

    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
    assert_eq!(result.npcs.confidence, QuantifierConfidence::Low);
    assert_eq!(result.movement.movement_type, None);
}

#[test]
fn test_quantifier_low_confidence_then_error_fallback() {
    let room = make_room();
    let all_npcs: Vec<NpcCard> = vec![];
    let previous_npcs: Vec<NpcCard> = vec![];
    let history: Vec<MessageEntry> = vec![];

    let context = QuantifierPromptContext {
        room: &room,
        previous_room_npcs: &previous_npcs,
        all_known_npcs: &all_npcs,
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
    let result = quantify_room_with_llm_call(&context, &["carla".to_string()], recorder.as_ref());

    assert_eq!(result.npcs.npc_ids, vec!["carla".to_string()]);
    assert_eq!(result.npcs.confidence, QuantifierConfidence::Low);
}

#[test]
fn test_static_npc_result_valid_ids() {
    let carla = TestNpc::named("carla", "Carla");
    let npcs = npc_map(vec![carla]);
    let state = TestGameState::in_room("hall");
    let movement = MovementParseResult {
        movement_type: Some(MovementType::Entering),
        destination: Some("kitchen".to_string()),
        confidence: QuantifierConfidence::High,
    };

    let result = static_npc_result(&state, &["carla".to_string()], movement.clone(), &npcs);

    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
    assert_eq!(result.movement.movement_type, Some(MovementType::Entering));
    assert_eq!(result.movement.destination, Some("kitchen".to_string()));
}

#[test]
fn test_static_npc_result_filters_unknown_ids() {
    let state = TestGameState::in_room("hall");
    let npcs = npc_map(vec![TestNpc::named("carla", "Carla")]);
    let result = static_npc_result(
        &state,
        &["carla".to_string(), "unknown".to_string()],
        MovementParseResult::default(),
        &npcs,
    );

    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
}

#[test]
fn test_static_npc_result_preserves_movement() {
    let state = TestGameState::in_room("hall");
    let movement = MovementParseResult {
        movement_type: Some(MovementType::Leaving),
        destination: Some("garden".to_string()),
        confidence: QuantifierConfidence::Medium,
    };

    let result = static_npc_result(&state, &[], movement.clone(), &HashMap::new());

    assert!(result.npcs.npc_ids.is_empty());
    assert_eq!(result.movement.movement_type, Some(MovementType::Leaving));
    assert_eq!(result.movement.destination, Some("garden".to_string()));
}

#[test]
fn test_static_npc_result_fallback_to_scene_npcs() {
    let carla = TestNpc::named("carla", "Carla");
    let npcs = npc_map(vec![carla.clone()]);
    let mut state = TestGameState::in_room("hall");
    state.scene.npcs_in_area.push(carla.clone());

    let result = static_npc_result(&state, &[], MovementParseResult::default(), &npcs);

    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
}

#[test]
fn test_quantifier_routes_through_recorder_for_forensics() {
    // Regression guard: bypassing the recorder (e.g. `recorder.provider().clone()`) makes `SpyForensics::save_count` stay at 0.
    use crate::test_support::noop_forensics::make_spy_recorder;

    let room = make_room();
    let carla = make_npc("carla", "Carla");
    let all_npcs = vec![carla];
    let previous_npcs: Vec<NpcCard> = vec![];
    let history: Vec<MessageEntry> = vec![];

    let context = QuantifierPromptContext {
        room: &room,
        previous_room_npcs: &previous_npcs,
        all_known_npcs: &all_npcs,
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &history,
        player_action: "I look around.",
        quantifier_prompt_override: None,
    };

    let backend = crate::adapters::driven::llm::providers::MockBackend::default()
        .with_prompt_responses(vec![
            r#"{"npcs_in_room": ["carla"], "movement": {"type": null}}"#.to_string(),
        ]);

    let (recorder, spy) = make_spy_recorder(Arc::new(backend));
    let result = quantify_room_with_llm_call(&context, &["carla".to_string()], recorder.as_ref());

    assert_eq!(result.npcs.confidence, QuantifierConfidence::High);
    assert_eq!(
        spy.save_count(),
        1,
        "quantifier LLM call must route through LlmCallRecorder so forensics are saved exactly once"
    );
}
