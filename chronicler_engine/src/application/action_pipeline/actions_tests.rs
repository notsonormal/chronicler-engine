use crate::application::action_pipeline::execute_action_impl;
use crate::application::action_pipeline::pipeline::ActionPipelineBackend;
use crate::error::EngineError;
use crate::domain::model::quantifier::QuantifierResult;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::MessageType;
use crate::application::ports::llm_provider::{AGENT_NARRATOR, LlmCallResult};
use crate::application::narrative_prompt::LayeredPromptAssembler;
use crate::test_support::fixtures::{TestGameState, TestNpc};
use crate::test_support::make_test_context;
struct MockBackend {
    narrate_result: Result<String, EngineError>,
    complete_result: Result<String, EngineError>,
    quantifier_result: QuantifierResult,
}
impl Default for MockBackend {
    fn default() -> Self {
        Self {
            narrate_result: Ok("You look around the room.".to_string()),
            complete_result: Ok("The orb glows brighter.".to_string()),
            quantifier_result: QuantifierResult::default(),
        }
    }
}
impl ActionPipelineBackend for MockBackend {
    fn assembler(&self) -> &LayeredPromptAssembler {
        static ASSEMBLER: std::sync::OnceLock<LayeredPromptAssembler> = std::sync::OnceLock::new();
        ASSEMBLER.get_or_init(|| {
            LayeredPromptAssembler::new(
                crate::application::narrative_prompt::budget::MAX_CONTEXT_TOKENS,
            )
        })
    }
    fn complete(
        &self,
        agent_name: &str,
        _system_prompt: &str,
        _user_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        let result = if agent_name == AGENT_NARRATOR {
            &self.narrate_result
        } else {
            &self.complete_result
        };
        match result {
            Ok(text) => Ok(LlmCallResult {
                text: text.clone(),
                system_prompt: String::new(),
                user_prompt: String::new(),
                raw_request_json: String::new(),
                raw_response_json: String::new(),
                backend_name: "mock".to_string(),
                model_name: "mock".to_string(),
                agent_name: agent_name.to_string(),
            }),
            Err(_) => Err(EngineError::Llm(crate::error::LlmFailure::EmptyResponse)),
        }
    }
    fn run_post_generation_agents(
        &self,
        _state: &GameState,
        _player_input: &str,
        _main_response: &str,
        result: &mut QuantifierResult,
    ) {
        *result = self.quantifier_result.clone();
    }
}
fn make_test_state() -> GameState {
    TestGameState::with_npc("start", TestNpc::named("npc1", "Test NPC"))
}
#[test]
fn test_execute_action_impl_completes_and_persists_state() {
    let state = make_test_state();
    let ctx = make_test_context(state.clone());
    let backend = MockBackend::default();
    execute_action_impl(&backend, ctx.clone(), "look".to_string());
    let final_state = ctx.load_state_for_test();
    assert_eq!(
        final_state.narrative.input_buffer.status,
        GenerationStatus::Idle
    );
    assert_eq!(
        final_state.narrative.input_buffer.phase,
        GenerationPhase::default()
    );
    let has_narration = final_state
        .narrative
        .history()
        .iter()
        .any(|e| e.message_type == MessageType::Narration);
    assert!(
        has_narration,
        "execute_action_impl should persist narration"
    );
}
#[test]
fn test_execute_action_impl_clears_last_trigger() {
    let mut state = make_test_state();
    state.narrative.last_trigger = Some(crate::test_support::TestStoredTriggerContext::named(
        "Old Trigger",
        "npc1",
    ));
    let ctx = make_test_context(state);
    let backend = MockBackend::default();
    execute_action_impl(&backend, ctx.clone(), "look".to_string());
    let final_state = ctx.load_state_for_test();
    assert!(
        final_state.narrative.last_trigger.is_none(),
        "last_trigger should be cleared before pipeline runs"
    );
}
#[test]
fn test_execute_action_impl_handles_narration_error() {
    let state = make_test_state();
    let ctx = make_test_context(state.clone());
    let backend = MockBackend {
        narrate_result: Err(EngineError::Llm(crate::error::LlmFailure::EmptyResponse)),
        ..Default::default()
    };
    execute_action_impl(&backend, ctx.clone(), "look".to_string());
    let final_state = ctx.load_state_for_test();
    assert!(
        matches!(
            final_state.narrative.input_buffer.status,
            GenerationStatus::Error(_)
        ),
        "State should reflect error status after failed narration"
    );
}
#[test]
fn test_execute_action_impl_handles_cancellation() {
    let state = make_test_state();
    let ctx = make_test_context(state.clone());
    ctx.cancel_token.cancel();
    let backend = MockBackend::default();
    execute_action_impl(&backend, ctx.clone(), "look".to_string());
    let final_state = ctx.load_state_for_test();
    assert_eq!(
        final_state.narrative.input_buffer.status,
        GenerationStatus::Idle,
        "Cancellation should reset status to Idle"
    );
}
#[test]
fn test_execute_action_impl_preserves_existing_input_log() {
    let mut state = make_test_state();
    state.add_message(
        "examine room".to_string(),
        Some("Player".to_string()),
        MessageType::Input,
    );
    let ctx = make_test_context(state);
    let backend = MockBackend::default();
    execute_action_impl(&backend, ctx.clone(), "examine room".to_string());
    let final_state = ctx.load_state_for_test();
    let entries: Vec<_> = final_state.narrative.history().into_iter().collect();
    let input_idx = entries
        .iter()
        .position(|e| e.message_type == MessageType::Input);
    let narration_idx = entries
        .iter()
        .position(|e| e.message_type == MessageType::Narration);
    assert!(input_idx.is_some(), "Existing input should be preserved");
    assert!(narration_idx.is_some(), "Narration should be added");
    assert!(
        input_idx.unwrap() < narration_idx.unwrap(),
        "Input should appear before narration"
    );
}
#[test]
fn test_phase_transitions_to_quantifying_during_post_generation() {
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;
    struct SlowQuantifierBackend {
        narrate_result: Result<String, EngineError>,
        quantifier_result: QuantifierResult,
    }
    impl ActionPipelineBackend for SlowQuantifierBackend {
        fn assembler(&self) -> &LayeredPromptAssembler {
            static ASSEMBLER: std::sync::LazyLock<LayeredPromptAssembler> =
                std::sync::LazyLock::new(|| {
                    LayeredPromptAssembler::new(
                        crate::application::narrative_prompt::budget::MAX_CONTEXT_TOKENS,
                    )
                });
            &ASSEMBLER
        }
        fn complete(
            &self,
            agent_name: &str,
            _system_prompt: &str,
            _user_prompt: &str,
            _max_tokens: Option<u32>,
        ) -> Result<LlmCallResult, EngineError> {
            match &self.narrate_result {
                Ok(text) => Ok(LlmCallResult {
                    text: text.clone(),
                    system_prompt: String::new(),
                    user_prompt: String::new(),
                    raw_request_json: String::new(),
                    raw_response_json: String::new(),
                    backend_name: "mock".to_string(),
                    model_name: "mock".to_string(),
                    agent_name: agent_name.to_string(),
                }),
                Err(_) => Err(EngineError::Llm(crate::error::LlmFailure::EmptyResponse)),
            }
        }
        fn run_post_generation_agents(
            &self,
            _state: &GameState,
            _player_input: &str,
            _main_response: &str,
            result: &mut QuantifierResult,
        ) {
            // Simulate slow quantifier
            thread::sleep(Duration::from_millis(200));
            *result = self.quantifier_result.clone();
        }
    }
    let state = make_test_state();
    let ctx = make_test_context(state);
    let backend = Arc::new(SlowQuantifierBackend {
        narrate_result: Ok("Narration text".to_string()),
        quantifier_result: QuantifierResult::default(),
    });
    let ctx_clone = ctx.clone();
    let backend_clone = backend.clone();
    let handle = thread::spawn(move || {
        execute_action_impl(&*backend_clone, ctx_clone, "look".to_string());
    });
    // Wait for narration to complete and quantifier to start
    thread::sleep(Duration::from_millis(100));
    // Check phase is Quantifying mid-flight
    let mid_state = ctx.load_state_for_test();
    assert_eq!(
        mid_state.narrative.input_buffer.phase,
        GenerationPhase::Quantifying,
        "Phase should be Quantifying during post-generation, not stuck on Narrating"
    );
    handle.join().expect("Action thread should complete");
    // Verify final phase is reset
    let final_state = ctx.load_state_for_test();
    assert_eq!(
        final_state.narrative.input_buffer.phase,
        GenerationPhase::default(),
        "Phase should reset to default after completion"
    );
}
#[test]
fn test_narration_saved_before_quantifying_phase() {
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;
    struct SlowQuantifierBackend {
        narrate_result: Result<String, EngineError>,
        quantifier_result: QuantifierResult,
    }
    impl ActionPipelineBackend for SlowQuantifierBackend {
        fn assembler(&self) -> &LayeredPromptAssembler {
            static ASSEMBLER: std::sync::LazyLock<LayeredPromptAssembler> =
                std::sync::LazyLock::new(|| {
                    LayeredPromptAssembler::new(
                        crate::application::narrative_prompt::budget::MAX_CONTEXT_TOKENS,
                    )
                });
            &ASSEMBLER
        }
        fn complete(
            &self,
            agent_name: &str,
            _system_prompt: &str,
            _user_prompt: &str,
            _max_tokens: Option<u32>,
        ) -> Result<LlmCallResult, EngineError> {
            match &self.narrate_result {
                Ok(text) => Ok(LlmCallResult {
                    text: text.clone(),
                    system_prompt: String::new(),
                    user_prompt: String::new(),
                    raw_request_json: String::new(),
                    raw_response_json: String::new(),
                    backend_name: "mock".to_string(),
                    model_name: "mock".to_string(),
                    agent_name: agent_name.to_string(),
                }),
                Err(_) => Err(EngineError::Llm(crate::error::LlmFailure::EmptyResponse)),
            }
        }
        fn run_post_generation_agents(
            &self,
            _state: &GameState,
            _player_input: &str,
            _main_response: &str,
            result: &mut QuantifierResult,
        ) {
            thread::sleep(Duration::from_millis(200));
            *result = self.quantifier_result.clone();
        }
    }
    let state = make_test_state();
    let ctx = make_test_context(state);
    let backend = Arc::new(SlowQuantifierBackend {
        narrate_result: Ok("Narration text".to_string()),
        quantifier_result: QuantifierResult::default(),
    });
    let ctx_clone = ctx.clone();
    let backend_clone = backend.clone();
    let handle = thread::spawn(move || {
        execute_action_impl(&*backend_clone, ctx_clone, "test".to_string());
    });
    // Wait for narration save (before quantifier completes)
    thread::sleep(Duration::from_millis(100));
    // Verify narration is saved
    let messages = ctx.load_messages().unwrap();
    let narration_count = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .count();
    assert!(
        narration_count >= 1,
        "Narration should be saved before quantifier completes"
    );
    // Verify phase is Quantifying
    let mid_state = ctx.load_state_for_test();
    assert_eq!(
        mid_state.narrative.input_buffer.phase,
        GenerationPhase::Quantifying,
        "Phase should be Quantifying while quantifier runs"
    );
    handle.join().expect("Action thread should complete");
}
