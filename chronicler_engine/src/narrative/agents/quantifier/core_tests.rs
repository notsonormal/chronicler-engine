use crate::error::EngineError;
use crate::model::character::NpcCard;
use crate::model::state::LogEntry;
use crate::narrative::llm::backend::{LlmBackend, LlmCallResult};
use crate::test_support::{TestGameState, TestNpc};

use super::core::{
    action_boundary_contains, determine_npcs_in_room, quantify_room_with_llm_call,
    static_npc_result,
};
use super::test_support::{make_boundary_chars, make_npc, make_room};
use super::types::{
    MovementParseResult, MovementType, QuantifierConfidence, QuantifierPromptContext,
};

#[test]
fn test_action_boundary_exact_match_at_start() {
    let boundary_chars = make_boundary_chars();
    let result = action_boundary_contains("hello world", "hello", &boundary_chars);
    assert!(result, "Should match at start of text");
}

#[test]
fn test_action_boundary_exact_match_at_end() {
    let boundary_chars = make_boundary_chars();
    let result = action_boundary_contains("hello world", "world", &boundary_chars);
    assert!(result, "Should match at end of text");
}

#[test]
fn test_action_boundary_match_in_middle() {
    let boundary_chars = make_boundary_chars();
    let result = action_boundary_contains("hello world here", "world", &boundary_chars);
    assert!(result, "Should match in middle with spaces on both sides");
}

#[test]
fn test_action_boundary_no_match_prefix() {
    let boundary_chars = make_boundary_chars();
    let result = action_boundary_contains("announcement", "ann", &boundary_chars);
    assert!(
        !result,
        "Should not match when substring is prefix of longer word"
    );
}

#[test]
fn test_action_boundary_no_match_suffix() {
    let boundary_chars = make_boundary_chars();
    let result = action_boundary_contains("maryann", "ann", &boundary_chars);
    assert!(
        !result,
        "Should not match when substring is suffix of longer word"
    );
}

#[test]
fn test_action_boundary_no_match_mid_word() {
    let boundary_chars = make_boundary_chars();
    let result = action_boundary_contains("canny", "ann", &boundary_chars);
    assert!(
        !result,
        "Should not match when substring appears but surrounded by non-boundary chars"
    );
}

#[test]
fn test_action_boundary_match_with_comma() {
    let boundary_chars = make_boundary_chars();
    let result = action_boundary_contains("hello,world", "world", &boundary_chars);
    assert!(result, "Should match with comma as boundary");
}

#[test]
fn test_action_boundary_match_with_period() {
    let boundary_chars = make_boundary_chars();
    let result = action_boundary_contains("hello.world", "world", &boundary_chars);
    assert!(result, "Should match with period as boundary");
}

#[test]
fn test_action_boundary_match_with_exclamation() {
    let boundary_chars = make_boundary_chars();
    let result = action_boundary_contains("hello!world", "world", &boundary_chars);
    assert!(result, "Should match with exclamation as boundary");
}

#[test]
fn test_action_boundary_match_with_question_mark() {
    let boundary_chars = make_boundary_chars();
    let result = action_boundary_contains("hello?world", "world", &boundary_chars);
    assert!(result, "Should match with question mark as boundary");
}

#[test]
fn test_action_boundary_empty_text() {
    let boundary_chars = make_boundary_chars();
    let result = action_boundary_contains("", "hello", &boundary_chars);
    assert!(!result, "Should not match empty text");
}

#[test]
fn test_action_boundary_empty_substring() {
    let boundary_chars = make_boundary_chars();
    let result = action_boundary_contains("hello world", "", &boundary_chars);
    assert!(
        !result,
        "Empty substring should not match when followed by non-boundary char"
    );
}

#[test]
fn test_action_boundary_both_empty() {
    let boundary_chars = make_boundary_chars();
    let result = action_boundary_contains("", "", &boundary_chars);
    assert!(result, "Empty text and substring should match");
}

#[test]
fn test_action_boundary_no_match_not_found() {
    let boundary_chars = make_boundary_chars();
    let result = action_boundary_contains("hello world", "xyz", &boundary_chars);
    assert!(!result, "Should not match when substring not found");
}

#[test]
fn test_action_boundary_substring_at_start_no_boundary_after() {
    let boundary_chars = make_boundary_chars();
    let result = action_boundary_contains("carlax", "carla", &boundary_chars);
    assert!(
        !result,
        "Should not match when followed by non-boundary char"
    );
}

#[test]
fn test_action_boundary_substring_at_end_no_boundary_before() {
    let boundary_chars = make_boundary_chars();
    let result = action_boundary_contains("xcarla", "carla", &boundary_chars);
    assert!(
        !result,
        "Should not match when preceded by non-boundary char"
    );
}

#[test]
fn test_quantifier_retry_on_low_confidence() {
    let room = make_room();
    let carla = make_npc("carla", "Carla");
    let all_npcs = vec![carla];
    let previous_npcs: Vec<NpcCard> = vec![];
    let history: Vec<LogEntry> = vec![];

    let context = QuantifierPromptContext {
        room: &room,
        previous_room_npcs: &previous_npcs,
        all_known_npcs: &all_npcs,
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &history,
        player_action: "I look around.",
    };

    let backend = crate::narrative::llm::MockBackend {
        per_call_prompt_responses: vec![
            "I am not sure what to say here.".to_string(),
            r#"{"npcs_in_room": ["carla"], "movement": {"type": null}}"#.to_string(),
        ],
        ..Default::default()
    };

    let result = quantify_room_with_llm_call(&context, &["carla".to_string()], &backend);

    assert!(result.is_ok());
    let result = result.unwrap();

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
    let history: Vec<LogEntry> = vec![];

    let context = QuantifierPromptContext {
        room: &room,
        previous_room_npcs: &previous_npcs,
        all_known_npcs: &all_npcs,
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &history,
        player_action: "I look around.",
    };

    let backend = crate::narrative::llm::MockBackend {
        per_call_prompt_responses: vec![
            r#"{"npcs_in_room": ["carla"], "movement": {"type": null}}"#.to_string(),
        ],
        ..Default::default()
    };

    let result = quantify_room_with_llm_call(&context, &["carla".to_string()], &backend);

    assert!(result.is_ok());
    let result = result.unwrap();

    assert_eq!(result.npcs.confidence, QuantifierConfidence::High);
}

// ─── Custom backends for determine_npcs_in_room tests ────────────────────────

fn make_test_llm_result(text: &str) -> LlmCallResult {
    LlmCallResult {
        text: text.to_string(),
        system_prompt: "".to_string(),
        user_prompt: "".to_string(),
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

impl LlmBackend for HighConfidenceBackend {
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
    fn generate_dialogue(
        &self,
        _agent_name: &str,
        _context: &crate::narrative::prompt::PromptContext,
        _npc: &NpcCard,
    ) -> Result<LlmCallResult, EngineError> {
        unreachable!()
    }
    fn narrate_action(
        &self,
        _agent_name: &str,
        _context: &crate::narrative::prompt::PromptContext,
    ) -> Result<LlmCallResult, EngineError> {
        unreachable!()
    }
    fn narrate_arrival(
        &self,
        _agent_name: &str,
        _context: &crate::narrative::prompt::PromptContext,
    ) -> Result<LlmCallResult, EngineError> {
        unreachable!()
    }
    fn narrate_continuation(
        &self,
        _agent_name: &str,
        _system_prompt: &str,
        _user_prompt: &str,
        _trigger_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        unreachable!()
    }
}

impl LlmBackend for MediumConfidenceBackend {
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
    fn generate_dialogue(
        &self,
        _agent_name: &str,
        _context: &crate::narrative::prompt::PromptContext,
        _npc: &NpcCard,
    ) -> Result<LlmCallResult, EngineError> {
        unreachable!()
    }
    fn narrate_action(
        &self,
        _agent_name: &str,
        _context: &crate::narrative::prompt::PromptContext,
    ) -> Result<LlmCallResult, EngineError> {
        unreachable!()
    }
    fn narrate_arrival(
        &self,
        _agent_name: &str,
        _context: &crate::narrative::prompt::PromptContext,
    ) -> Result<LlmCallResult, EngineError> {
        unreachable!()
    }
    fn narrate_continuation(
        &self,
        _agent_name: &str,
        _system_prompt: &str,
        _user_prompt: &str,
        _trigger_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        unreachable!()
    }
}

impl LlmBackend for LowConfidenceBackend {
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
    fn generate_dialogue(
        &self,
        _agent_name: &str,
        _context: &crate::narrative::prompt::PromptContext,
        _npc: &NpcCard,
    ) -> Result<LlmCallResult, EngineError> {
        unreachable!()
    }
    fn narrate_action(
        &self,
        _agent_name: &str,
        _context: &crate::narrative::prompt::PromptContext,
    ) -> Result<LlmCallResult, EngineError> {
        unreachable!()
    }
    fn narrate_arrival(
        &self,
        _agent_name: &str,
        _context: &crate::narrative::prompt::PromptContext,
    ) -> Result<LlmCallResult, EngineError> {
        unreachable!()
    }
    fn narrate_continuation(
        &self,
        _agent_name: &str,
        _system_prompt: &str,
        _user_prompt: &str,
        _trigger_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        unreachable!()
    }
}

impl LlmBackend for ErrBackend {
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
    fn generate_dialogue(
        &self,
        _agent_name: &str,
        _context: &crate::narrative::prompt::PromptContext,
        _npc: &NpcCard,
    ) -> Result<LlmCallResult, EngineError> {
        unreachable!()
    }
    fn narrate_action(
        &self,
        _agent_name: &str,
        _context: &crate::narrative::prompt::PromptContext,
    ) -> Result<LlmCallResult, EngineError> {
        unreachable!()
    }
    fn narrate_arrival(
        &self,
        _agent_name: &str,
        _context: &crate::narrative::prompt::PromptContext,
    ) -> Result<LlmCallResult, EngineError> {
        unreachable!()
    }
    fn narrate_continuation(
        &self,
        _agent_name: &str,
        _system_prompt: &str,
        _user_prompt: &str,
        _trigger_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        unreachable!()
    }
}

// ─── determine_npcs_in_room tests ────────────────────────────────────────────

#[test]
fn test_determine_npcs_high_confidence() {
    let carla = TestNpc::named("carla", "Carla");
    let gabriella = TestNpc::named("gabriella", "Gabriella");
    let state = TestGameState::with_npcs("hall", vec![carla.clone(), gabriella.clone()]);

    let backend = HighConfidenceBackend {
        npc_ids: vec!["carla".to_string()],
    };
    let result = determine_npcs_in_room(
        &state,
        &["carla".to_string(), "gabriella".to_string()],
        &[],
        "test",
        &backend,
    );

    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
    assert_eq!(result.npcs.confidence, QuantifierConfidence::High);
    assert_eq!(result.movement.movement_type, Some(MovementType::Entering));
    assert_eq!(result.movement.destination, Some("kitchen".to_string()));
}

#[test]
fn test_determine_npcs_medium_confidence() {
    let carla = TestNpc::named("carla", "Carla");
    let state = TestGameState::with_npcs("hall", vec![carla.clone()]);

    let backend = MediumConfidenceBackend;
    let result = determine_npcs_in_room(&state, &["carla".to_string()], &[], "test", &backend);

    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
    assert_eq!(result.npcs.confidence, QuantifierConfidence::Medium);
}

#[test]
fn test_determine_npcs_low_confidence_fallback() {
    let carla = TestNpc::named("carla", "Carla");
    let gabriella = TestNpc::named("gabriella", "Gabriella");
    let state = TestGameState::with_npcs("hall", vec![carla.clone(), gabriella.clone()]);

    let backend = LowConfidenceBackend;
    let result = determine_npcs_in_room(
        &state,
        &["carla".to_string(), "gabriella".to_string()],
        &[],
        "test",
        &backend,
    );

    assert_eq!(result.npcs.npc_ids, vec!["carla", "gabriella"]);
    assert_eq!(result.npcs.confidence, QuantifierConfidence::Low);
}

#[test]
fn test_determine_npcs_backend_error_fallback() {
    let carla = TestNpc::named("carla", "Carla");
    let state = TestGameState::with_npcs("hall", vec![carla.clone()]);

    let backend = ErrBackend;
    let result = determine_npcs_in_room(&state, &["carla".to_string()], &[], "test", &backend);

    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
    assert_eq!(result.npcs.confidence, QuantifierConfidence::Low);
}

#[test]
fn test_determine_npcs_invalid_room_fallback() {
    let carla = TestNpc::named("carla", "Carla");
    let mut state = TestGameState::with_npcs("hall", vec![carla.clone()]);
    state.movement.current_room_id = "nonexistent".to_string();

    let backend = HighConfidenceBackend {
        npc_ids: vec!["carla".to_string()],
    };
    let result = determine_npcs_in_room(&state, &["carla".to_string()], &[], "test", &backend);

    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
    assert_eq!(result.npcs.confidence, QuantifierConfidence::Low);
}

#[test]
fn test_determine_npcs_filters_unknown_backend_ids() {
    let carla = TestNpc::named("carla", "Carla");
    let state = TestGameState::with_npcs("hall", vec![carla.clone()]);

    let backend = HighConfidenceBackend {
        npc_ids: vec!["carla".to_string(), "unknown".to_string()],
    };
    let result = determine_npcs_in_room(&state, &["carla".to_string()], &[], "test", &backend);

    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
}

// ─── quantify_room_with_llm_call failure path tests ──────────────────────────

#[test]
fn test_quantifier_retry_on_llm_error() {
    let room = make_room();
    let carla = make_npc("carla", "Carla");
    let all_npcs = vec![carla];
    let previous_npcs: Vec<NpcCard> = vec![];
    let history: Vec<LogEntry> = vec![];

    let context = QuantifierPromptContext {
        room: &room,
        previous_room_npcs: &previous_npcs,
        all_known_npcs: &all_npcs,
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &history,
        player_action: "I look around.",
    };

    struct RotatingBackend {
        responses: std::sync::Mutex<Vec<Result<String, EngineError>>>,
    }
    impl LlmBackend for RotatingBackend {
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
        fn generate_dialogue(
            &self,
            _agent_name: &str,
            _context: &crate::narrative::prompt::PromptContext,
            _npc: &NpcCard,
        ) -> Result<LlmCallResult, EngineError> {
            unreachable!()
        }
        fn narrate_action(
            &self,
            _agent_name: &str,
            _context: &crate::narrative::prompt::PromptContext,
        ) -> Result<LlmCallResult, EngineError> {
            unreachable!()
        }
        fn narrate_arrival(
            &self,
            _agent_name: &str,
            _context: &crate::narrative::prompt::PromptContext,
        ) -> Result<LlmCallResult, EngineError> {
            unreachable!()
        }
        fn narrate_continuation(
            &self,
            _agent_name: &str,
            _system_prompt: &str,
            _user_prompt: &str,
            _trigger_prompt: &str,
            _max_tokens: Option<u32>,
        ) -> Result<LlmCallResult, EngineError> {
            unreachable!()
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

    let result = quantify_room_with_llm_call(&context, &["carla".to_string()], &backend);

    assert!(result.is_ok());
    assert_eq!(result.unwrap().npcs.npc_ids, vec!["carla"]);
}

#[test]
fn test_quantifier_all_attempts_fail_fallback() {
    let room = make_room();
    let all_npcs: Vec<NpcCard> = vec![];
    let previous_npcs: Vec<NpcCard> = vec![];
    let history: Vec<LogEntry> = vec![];

    let context = QuantifierPromptContext {
        room: &room,
        previous_room_npcs: &previous_npcs,
        all_known_npcs: &all_npcs,
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &history,
        player_action: "I look around.",
    };

    let backend = crate::narrative::llm::MockBackend {
        trigger_narration_should_fail: true.into(),
        ..Default::default()
    };

    let result = quantify_room_with_llm_call(&context, &["carla".to_string()], &backend);

    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
    assert_eq!(result.npcs.confidence, QuantifierConfidence::Low);
    assert_eq!(result.movement.movement_type, None);
}

#[test]
fn test_quantifier_low_confidence_then_error_fallback() {
    let room = make_room();
    let all_npcs: Vec<NpcCard> = vec![];
    let previous_npcs: Vec<NpcCard> = vec![];
    let history: Vec<LogEntry> = vec![];

    let context = QuantifierPromptContext {
        room: &room,
        previous_room_npcs: &previous_npcs,
        all_known_npcs: &all_npcs,
        all_rooms: &[],
        player_name: "Hero",
        recent_history: &history,
        player_action: "I look around.",
    };

    let backend = crate::narrative::llm::MockBackend {
        per_call_prompt_responses: vec!["I am not sure what to say here.".to_string()],
        trigger_narration_should_fail: true.into(),
        ..Default::default()
    };

    let result = quantify_room_with_llm_call(&context, &["carla".to_string()], &backend);

    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.npcs.npc_ids, vec!["carla".to_string()]);
    assert_eq!(result.npcs.confidence, QuantifierConfidence::Low);
}

// ─── static_npc_result tests ─────────────────────────────────────────────────

#[test]
fn test_static_npc_result_valid_ids() {
    let carla = TestNpc::named("carla", "Carla");
    let state = TestGameState::with_npcs("hall", vec![carla.clone()]);
    let movement = MovementParseResult {
        movement_type: Some(MovementType::Entering),
        destination: Some("kitchen".to_string()),
        confidence: QuantifierConfidence::High,
    };

    let result = static_npc_result(&state, &["carla".to_string()], movement.clone());

    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
    assert_eq!(result.movement.movement_type, Some(MovementType::Entering));
    assert_eq!(result.movement.destination, Some("kitchen".to_string()));
}

#[test]
fn test_static_npc_result_filters_unknown_ids() {
    let carla = TestNpc::named("carla", "Carla");
    let state = TestGameState::with_npcs("hall", vec![carla.clone()]);
    let movement = MovementParseResult {
        movement_type: None,
        destination: None,
        confidence: QuantifierConfidence::Low,
    };

    let result = static_npc_result(
        &state,
        &["carla".to_string(), "unknown".to_string()],
        movement,
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

    let result = static_npc_result(&state, &[], movement.clone());

    assert!(result.npcs.npc_ids.is_empty());
    assert_eq!(result.movement.movement_type, Some(MovementType::Leaving));
    assert_eq!(result.movement.destination, Some("garden".to_string()));
}

#[test]
fn test_static_npc_result_fallback_to_scene_npcs() {
    let carla = TestNpc::named("carla", "Carla");
    let mut state = TestGameState::with_npcs("hall", vec![carla.clone()]);
    state.scene.npcs_in_area.push(carla.clone());

    let movement = MovementParseResult {
        movement_type: None,
        destination: None,
        confidence: QuantifierConfidence::Low,
    };

    let result = static_npc_result(&state, &[], movement);

    assert_eq!(result.npcs.npc_ids, vec!["carla"]);
}
