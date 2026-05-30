use std::sync::Arc;

use chronicler_engine::application::game_service::DefaultGameService;
use chronicler_engine::model::character::{CharacterSheet, NpcCard};
use chronicler_engine::model::state::GenerationStatus;
use chronicler_engine::model::state::MessageType;

use chronicler_engine::test_support::make_test_context;

use crate::backends::*;
use crate::{BenchmarkResult, DiagnosticScores, print_benchmark_result, run_scenario};

// Scenario 1: LLM HTTP 401 Unauthorized

#[test]
fn benchmark_llm_http_401() {
    let (error_msg, phase, ctx) = run_scenario(
        Arc::new(HttpErrorBackend::unauthorized()),
        Arc::new(chronicler_engine::narrative::llm::MockBackend::default()),
        "llm_http_401",
        "LLM",
        "HTTP 401 Unauthorized from LLM provider",
    );

    let snapshot = ctx.storage.load_latest_snapshot().unwrap().unwrap();
    let _has_dynamic_room = !snapshot.movement.dynamic_rooms.is_empty();
    let _current_room = snapshot.movement.current_room_id.clone();

    let result = BenchmarkResult {
        scenario: "llm_http_401".to_string(),
        category: "LLM".to_string(),
        injected_failure: "HTTP 401 Unauthorized from LLM provider".to_string(),
        error_message: error_msg.clone(),
        generation_phase: phase,
        scores: DiagnosticScores {
            error_specificity: if error_msg.contains("401") { 10 } else { 2 },
            state_visibility: if error_msg.contains("401") { 6 } else { 2 },
            log_independence: if error_msg.contains("401") { 8 } else { 1 },
        },
        root_cause_discoverable_from_ui: error_msg.contains("401"),
        root_cause_discoverable_from_debug_endpoint: error_msg.contains("401"),
        root_cause_discoverable_without_logs: error_msg.contains("401"),
        notes: format!(
            "HTTP 401 is mapped to '{}'. The status code and body are preserved via map_llm_error. The debug endpoint also exposes last_error, making the root cause discoverable without logs.",
            error_msg
        ),
    };

    print_benchmark_result(&result);

    assert!(
        error_msg.contains("401"),
        "map_llm_error should preserve HTTP status codes"
    );
}

// Scenario 2: LLM HTTP 429 Rate Limited

#[test]
fn benchmark_llm_http_429() {
    let (error_msg, phase, _state) = run_scenario(
        Arc::new(HttpErrorBackend::rate_limited()),
        Arc::new(chronicler_engine::narrative::llm::MockBackend::default()),
        "llm_http_429",
        "LLM",
        "HTTP 429 Rate Limited from LLM provider",
    );

    let result = BenchmarkResult {
        scenario: "llm_http_429".to_string(),
        category: "LLM".to_string(),
        injected_failure: "HTTP 429 Rate Limited from LLM provider".to_string(),
        error_message: error_msg.clone(),
        generation_phase: phase,
        scores: DiagnosticScores {
            error_specificity: if error_msg.contains("429") { 10 } else { 2 },
            state_visibility: if error_msg.contains("429") { 6 } else { 2 },
            log_independence: if error_msg.contains("429") { 8 } else { 1 },
        },
        root_cause_discoverable_from_ui: error_msg.contains("429"),
        root_cause_discoverable_from_debug_endpoint: error_msg.contains("429"),
        root_cause_discoverable_without_logs: error_msg.contains("429"),
        notes: format!(
            "HTTP 429 is mapped to '{}'. The status code and body are preserved via map_llm_error. The debug endpoint also exposes last_error.",
            error_msg
        ),
    };

    print_benchmark_result(&result);
}

// Scenario 3: LLM Network Error (Ollama down)

#[test]
fn benchmark_llm_network_error() {
    let (error_msg, phase, _state) = run_scenario(
        Arc::new(NetworkErrorBackend::connection_refused()),
        Arc::new(chronicler_engine::narrative::llm::MockBackend::default()),
        "llm_network_error",
        "LLM",
        "Network error: Ollama connection refused",
    );

    let result = BenchmarkResult {
        scenario: "llm_network_error".to_string(),
        category: "LLM".to_string(),
        injected_failure: "Network error: Ollama connection refused".to_string(),
        error_message: error_msg.clone(),
        generation_phase: phase,
        scores: DiagnosticScores {
            error_specificity: if error_msg.contains("refused")
                || error_msg.contains("localhost:11434")
            {
                8
            } else {
                3
            },
            state_visibility: if error_msg.contains("refused") { 6 } else { 2 },
            log_independence: if error_msg.contains("refused") { 6 } else { 2 },
        },
        root_cause_discoverable_from_ui: error_msg.contains("refused")
            || error_msg.contains("incomplete"),
        root_cause_discoverable_from_debug_endpoint: error_msg.contains("refused")
            || error_msg.contains("incomplete"),
        root_cause_discoverable_without_logs: error_msg.contains("refused"),
        notes: format!(
            "Network error is mapped to '{}'. The URL and detail are preserved via map_llm_error. The debug endpoint also exposes last_error.",
            error_msg
        ),
    };

    print_benchmark_result(&result);
}

// Scenario 4: LLM Parse Error (non-JSON response)

#[test]
fn benchmark_llm_parse_error() {
    let (error_msg, phase, _state) = run_scenario(
        Arc::new(ParseErrorBackend {
            raw_response: "This is not JSON, just raw text from the model.".to_string(),
        }),
        Arc::new(chronicler_engine::narrative::llm::MockBackend::default()),
        "llm_parse_error",
        "LLM",
        "LLM returned non-JSON response",
    );

    let result = BenchmarkResult {
        scenario: "llm_parse_error".to_string(),
        category: "LLM".to_string(),
        injected_failure: "LLM returned non-JSON response".to_string(),
        error_message: error_msg.clone(),
        generation_phase: phase,
        scores: DiagnosticScores {
            error_specificity: if error_msg.contains("parse") || error_msg.contains("format") {
                6
            } else {
                3
            },
            state_visibility: if error_msg.contains("parse") || error_msg.contains("format") {
                5
            } else {
                2
            },
            log_independence: if error_msg.contains("parse") || error_msg.contains("format") {
                5
            } else {
                2
            },
        },
        root_cause_discoverable_from_ui: error_msg.contains("parse")
            || error_msg.contains("format"),
        root_cause_discoverable_from_debug_endpoint: error_msg.contains("parse")
            || error_msg.contains("format"),
        root_cause_discoverable_without_logs: error_msg.contains("parse")
            || error_msg.contains("format"),
        notes: format!(
            "Parse error is mapped to '{}'. The expected format is preserved in the message. The debug endpoint also exposes last_error.",
            error_msg
        ),
    };

    print_benchmark_result(&result);
}

// Scenario 5: LLM Timeout

#[test]
fn benchmark_llm_timeout() {
    let (error_msg, phase, _state) = run_scenario(
        Arc::new(TimeoutBackend),
        Arc::new(chronicler_engine::narrative::llm::MockBackend::default()),
        "llm_timeout",
        "LLM",
        "LLM request timed out after 180s",
    );

    let result = BenchmarkResult {
        scenario: "llm_timeout".to_string(),
        category: "LLM".to_string(),
        injected_failure: "LLM request timed out after 180s".to_string(),
        error_message: error_msg.clone(),
        generation_phase: phase,
        scores: DiagnosticScores {
            error_specificity: if error_msg.contains("timed out") {
                8
            } else {
                3
            },
            state_visibility: if error_msg.contains("timed out") {
                5
            } else {
                2
            },
            log_independence: if error_msg.contains("timed out") {
                7
            } else {
                2
            },
        },
        root_cause_discoverable_from_ui: error_msg.contains("timed out"),
        root_cause_discoverable_from_debug_endpoint: error_msg.contains("timed out"),
        root_cause_discoverable_without_logs: error_msg.contains("timed out"),
        notes: format!(
            "Timeout is mapped to '{}'. 'timed out' is reasonably specific. The debug endpoint exposes last_error. The exact timeout threshold (e.g. 180s) is not included in the message.",
            error_msg
        ),
    };

    print_benchmark_result(&result);
}

// Scenario 6: Empty LLM Response

#[test]
fn benchmark_llm_empty_response() {
    let (error_msg, phase, _state) = run_scenario(
        Arc::new(chronicler_engine::narrative::llm::MockBackend::with_empty_response()),
        Arc::new(chronicler_engine::narrative::llm::MockBackend::default()),
        "llm_empty_response",
        "LLM",
        "LLM returned empty content field",
    );

    let result = BenchmarkResult {
        scenario: "llm_empty_response".to_string(),
        category: "LLM".to_string(),
        injected_failure: "LLM returned empty content field".to_string(),
        error_message: error_msg.clone(),
        generation_phase: phase,
        scores: DiagnosticScores {
            error_specificity: if error_msg.contains("empty") { 8 } else { 3 },
            state_visibility: if error_msg.contains("empty") { 5 } else { 2 },
            log_independence: if error_msg.contains("empty") { 7 } else { 2 },
        },
        root_cause_discoverable_from_ui: error_msg.contains("empty"),
        root_cause_discoverable_from_debug_endpoint: error_msg.contains("empty"),
        root_cause_discoverable_without_logs: error_msg.contains("empty"),
        notes: format!(
            "Empty response is mapped to '{}'. 'empty response' is specific enough to know the model returned nothing. The debug endpoint also exposes last_error.",
            error_msg
        ),
    };

    print_benchmark_result(&result);
}

// Scenario 7: Quantifier Complete Failure

#[test]
fn benchmark_quantifier_complete_failure() {
    let (error_msg, phase, ctx) = run_scenario(
        Arc::new(chronicler_engine::narrative::llm::MockBackend::default()),
        Arc::new(FailingQuantifierBackend),
        "quantifier_complete_failure",
        "Quantifier",
        "Quantifier LLM call fails completely (connection refused)",
    );

    let snapshot = ctx.storage.load_latest_snapshot().unwrap().unwrap();
    let messages = ctx.load_messages().unwrap();
    let npc_count = snapshot.scene.npcs_in_area.len();
    let has_system_log = messages.iter().any(|m| {
        m.message_type == MessageType::System && m.text().contains("NPC detection uncertain")
    });

    let result = BenchmarkResult {
        scenario: "quantifier_complete_failure".to_string(),
        category: "Quantifier".to_string(),
        injected_failure: "Quantifier LLM call fails completely (connection refused)".to_string(),
        error_message: error_msg.clone(),
        generation_phase: phase,
        scores: DiagnosticScores {
            error_specificity: 1,
            state_visibility: if has_system_log { 5 } else { 3 },
            log_independence: if has_system_log { 4 } else { 1 },
        },
        root_cause_discoverable_from_ui: has_system_log,
        root_cause_discoverable_from_debug_endpoint: has_system_log,
        root_cause_discoverable_without_logs: has_system_log,
        notes: format!(
            "Quantifier failure falls back to static room NPCs ({} NPCs in area). No error is shown, but a System log entry '{}' is now added to the story log. The game continues with a visible signal.",
            npc_count,
            if has_system_log {
                "was added"
            } else {
                "was NOT added — fix may be incomplete"
            }
        ),
    };

    print_benchmark_result(&result);

    assert_eq!(
        error_msg, "(no error, idle)",
        "Quantifier failures are silent"
    );
}

// Scenario 8: Quantifier Low Confidence

#[test]
fn benchmark_quantifier_low_confidence() {
    let (error_msg, phase, ctx) = run_scenario(
        Arc::new(chronicler_engine::narrative::llm::MockBackend::default()),
        Arc::new(LowConfidenceQuantifierBackend),
        "quantifier_low_confidence",
        "Quantifier",
        "Quantifier returns Low confidence NPC detection",
    );

    let snapshot = ctx.storage.load_latest_snapshot().unwrap().unwrap();
    let messages = ctx.load_messages().unwrap();
    let npc_count = snapshot.scene.npcs_in_area.len();
    let has_narration = messages
        .iter()
        .any(|m| m.message_type == MessageType::Narration);
    let has_system_log = messages.iter().any(|m| {
        m.message_type == MessageType::System && m.text().contains("NPC detection uncertain")
    });

    let result = BenchmarkResult {
        scenario: "quantifier_low_confidence".to_string(),
        category: "Quantifier".to_string(),
        injected_failure: "Quantifier returns Low confidence NPC detection".to_string(),
        error_message: error_msg.clone(),
        generation_phase: phase,
        scores: DiagnosticScores {
            error_specificity: 1,
            state_visibility: if has_system_log { 5 } else { 3 },
            log_independence: if has_system_log { 4 } else { 1 },
        },
        root_cause_discoverable_from_ui: has_system_log,
        root_cause_discoverable_from_debug_endpoint: has_system_log,
        root_cause_discoverable_without_logs: has_system_log,
        notes: format!(
            "Low confidence quantifier result falls back to static NPCs ({} NPCs in area). A System log entry '{}' is now added. Narration was generated: {}. User can see that NPC detection was uncertain.",
            npc_count,
            if has_system_log {
                "was added"
            } else {
                "was NOT added — fix may be incomplete"
            },
            has_narration
        ),
    };

    print_benchmark_result(&result);
}

// Scenario 9: Dynamic Room Creation (navigation bug)

#[test]
fn benchmark_dynamic_room_creation() {
    let (error_msg, phase, ctx) = run_scenario(
        Arc::new(chronicler_engine::narrative::llm::MockBackend::default()),
        Arc::new(MisleadingMovementQuantifierBackend),
        "dynamic_room_creation",
        "Navigation",
        "Quantifier returns movement to non-existent room 'nonexistent_room'",
    );

    let snapshot = ctx.storage.load_latest_snapshot().unwrap().unwrap();
    let messages = ctx.load_messages().unwrap();
    let current_room = snapshot.movement.current_room_id.clone();
    let is_dynamic = current_room.starts_with("dynamic_");
    let dynamic_room_count = snapshot.movement.dynamic_rooms.len();
    let has_system_log = messages.iter().any(|m| {
        m.message_type == MessageType::System && m.text().contains("Entered unknown location")
    });

    let result = BenchmarkResult {
        scenario: "dynamic_room_creation".to_string(),
        category: "Navigation".to_string(),
        injected_failure: "Quantifier returns movement to non-existent room 'nonexistent_room'"
            .to_string(),
        error_message: error_msg.clone(),
        generation_phase: phase,
        scores: DiagnosticScores {
            error_specificity: if is_dynamic { 6 } else { 1 },
            state_visibility: if is_dynamic { 7 } else { 2 },
            log_independence: if has_system_log { 6 } else { 4 },
        },
        root_cause_discoverable_from_ui: has_system_log,
        root_cause_discoverable_from_debug_endpoint: is_dynamic,
        root_cause_discoverable_without_logs: has_system_log,
        notes: format!(
            "Player ended up in room '{}'. Dynamic room created: {} (count: {}). A System log '{}' is now added. The debug endpoint shows dynamic_rooms list.",
            current_room,
            is_dynamic,
            dynamic_room_count,
            if has_system_log {
                "was added"
            } else {
                "was NOT added — fix may be incomplete"
            }
        ),
    };

    print_benchmark_result(&result);

    assert!(is_dynamic, "Failed room resolution creates a dynamic room");
}

// Scenario 10: Narrative Generation Failure (MockBackend failing)

#[test]
fn benchmark_narrative_generation_failure() {
    let (error_msg, phase, _state) = run_scenario(
        Arc::new(chronicler_engine::narrative::llm::MockBackend::failing()),
        Arc::new(chronicler_engine::narrative::llm::MockBackend::default()),
        "narrative_generation_failure",
        "Narrative",
        "MockBackend configured to fail all narration calls",
    );

    let result = BenchmarkResult {
        scenario: "narrative_generation_failure".to_string(),
        category: "Narrative".to_string(),
        injected_failure: "MockBackend configured to fail all narration calls".to_string(),
        error_message: error_msg.clone(),
        generation_phase: phase,
        scores: DiagnosticScores {
            error_specificity: if error_msg.contains("mock") { 5 } else { 4 },
            state_visibility: 3,
            log_independence: if error_msg.contains("mock") { 5 } else { 4 },
        },
        root_cause_discoverable_from_ui: error_msg.contains("mock")
            || error_msg.contains("Generation"),
        root_cause_discoverable_from_debug_endpoint: error_msg.contains("mock")
            || error_msg.contains("Generation"),
        root_cause_discoverable_without_logs: error_msg.contains("mock")
            || error_msg.contains("Generation"),
        notes: format!(
            "Narrative failure is mapped to '{}'. The structured error 'Generation {{ stage: \"mock\", reason: \"configured_failure\" }}' propagates through map_llm_error. Decent specificity — you know it's a generation failure, though not why.",
            error_msg
        ),
    };

    print_benchmark_result(&result);
}

// Scenario 11: Trigger Not Firing (wrong room_id)

#[test]
fn benchmark_trigger_wrong_room_id() {
    let npc_with_trigger = NpcCard {
        id: "trigger_npc".into(),
        sheet: CharacterSheet {
            name: "Mysterious Stranger".into(),
            description: "A cloaked figure".into(),
            personality: "Secretive".into(),
            scenario: "Appears in the garden".into(),
            example_dialogue: "Psst...".into(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
        triggers: vec![chronicler_engine::model::trigger::Trigger {
            requirement: chronicler_engine::model::trigger::TriggerRequirement::TimesMet(
                chronicler_engine::model::trigger::ComparisonOperator::Eq,
                0,
            ),
            narration: chronicler_engine::model::trigger::TriggerNarration {
                name: "Greeting".into(),
                narration_prompt: "The stranger nods at you.".into(),
            },
            repeat: true,
            room_id: Some("wrong_room".into()),
        }],
        relationships: vec![],
    };

    let state = crate::test_data::create_test_state_with_npcs(
        vec!["trigger_npc".to_string()],
        vec![npc_with_trigger],
    );
    let ctx = make_test_context(state);

    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(chronicler_engine::narrative::llm::MockBackend::default()),
        Arc::new(chronicler_engine::narrative::llm::MockBackend {
            per_call_prompt_responses: vec![r#"{"npcs_in_room": ["trigger_npc"]}"#.to_string()],
            ..Default::default()
        }),
    );

    service.execute_action(
        ctx.clone(),
        "look around".to_string(),
        "Test Player".to_string(),
    );

    let snapshot = ctx.storage.load_latest_snapshot().unwrap().unwrap();
    let messages = ctx.load_messages().unwrap();
    let trigger_fired = messages.iter().any(|m| m.text().contains("stranger nods"));
    let error_msg = match &snapshot.narrative.input_buffer.status {
        GenerationStatus::Error(msg) => msg.clone(),
        _ => "(no error)".to_string(),
    };

    let result = BenchmarkResult {
        scenario: "trigger_wrong_room_id".to_string(),
        category: "Triggers".to_string(),
        injected_failure: "Trigger scoped to wrong room_id ('wrong_room' instead of 'room1')"
            .to_string(),
        error_message: error_msg.clone(),
        generation_phase: format!("{:?}", snapshot.narrative.input_buffer.phase),
        scores: DiagnosticScores {
            error_specificity: 1,
            state_visibility: 4,
            log_independence: 2,
        },
        root_cause_discoverable_from_ui: false,
        root_cause_discoverable_from_debug_endpoint: false,
        root_cause_discoverable_without_logs: false,
        notes: format!(
            "Trigger did NOT fire (fired={}). No error is shown. To diagnose, you must check /debug/state → npc_encounter_log.trigger_npc.triggers_fired, then compare trigger.room_id to current_room_id. This requires reading trigger definitions in data files. Silent failure.",
            trigger_fired
        ),
    };

    print_benchmark_result(&result);

    assert!(!trigger_fired, "Trigger with wrong room_id should not fire");
}

// Scenario 12: State Stuck in Generating (mid-pipeline failure)

#[test]
fn benchmark_state_stuck_generating() {
    let npc_with_trigger = NpcCard {
        id: "test_npc".into(),
        sheet: CharacterSheet {
            name: "Innkeeper".into(),
            description: "A friendly innkeeper".into(),
            personality: "Helpful".into(),
            scenario: "Runs the tavern".into(),
            example_dialogue: "Welcome!".into(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
        triggers: vec![chronicler_engine::model::trigger::Trigger {
            requirement: chronicler_engine::model::trigger::TriggerRequirement::TimesMet(
                chronicler_engine::model::trigger::ComparisonOperator::Eq,
                0,
            ),
            narration: chronicler_engine::model::trigger::TriggerNarration {
                name: "Greeting".into(),
                narration_prompt: "The innkeeper waves at you.".into(),
            },
            repeat: true,
            room_id: Some("room1".into()),
        }],
        relationships: vec![],
    };

    let mut state = crate::test_data::create_test_state_with_npcs(
        vec!["test_npc".to_string()],
        vec![npc_with_trigger],
    );

    if let Some(encounter) = state.npc_encounter_log.npcs.get_mut("test_npc") {
        encounter.times_met = 0;
    }

    let ctx = make_test_context(state);

    let service = DefaultGameService::with_mock_quantifier(
        Arc::new(chronicler_engine::narrative::llm::MockBackend::with_failing_trigger_narration()),
        Arc::new(chronicler_engine::narrative::llm::MockBackend {
            per_call_prompt_responses: vec![r#"{"npcs_in_room": ["test_npc"]}"#.to_string()],
            ..Default::default()
        }),
    );

    service.execute_action(
        ctx.clone(),
        "look around".to_string(),
        "Test Player".to_string(),
    );

    let snapshot = ctx.storage.load_latest_snapshot().unwrap().unwrap();
    let is_generating = snapshot.narrative.input_buffer.status.is_generating();
    let is_idle = matches!(
        snapshot.narrative.input_buffer.status,
        GenerationStatus::Idle
    );
    let has_error = matches!(
        snapshot.narrative.input_buffer.status,
        GenerationStatus::Error(_)
    );
    let error_msg = match &snapshot.narrative.input_buffer.status {
        GenerationStatus::Error(msg) => msg.clone(),
        _ => "(no error)".to_string(),
    };

    let result = BenchmarkResult {
        scenario: "state_stuck_generating".to_string(),
        category: "State Management".to_string(),
        injected_failure: "Trigger narration fails after main narration succeeds".to_string(),
        error_message: error_msg.clone(),
        generation_phase: format!("{:?}", snapshot.narrative.input_buffer.phase),
        scores: DiagnosticScores {
            error_specificity: if has_error { 7 } else { 1 },
            state_visibility: if is_idle || has_error { 7 } else { 2 },
            log_independence: if is_idle || has_error { 6 } else { 1 },
        },
        root_cause_discoverable_from_ui: is_idle || has_error,
        root_cause_discoverable_from_debug_endpoint: is_idle || has_error,
        root_cause_discoverable_without_logs: is_idle || has_error,
        notes: format!(
            "After trigger narration failure: status={:?}, idle={}, error={}, generating={}. The error message '{}' is set and phase preserved at failure point ({:?}). A system log contains the detailed trigger failure.",
            snapshot.narrative.input_buffer.status,
            is_idle,
            has_error,
            is_generating,
            error_msg,
            snapshot.narrative.input_buffer.phase
        ),
    };

    print_benchmark_result(&result);

    assert!(
        !is_generating,
        "Status should not be Generating after trigger narration failure"
    );
}
