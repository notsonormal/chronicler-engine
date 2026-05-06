use std::sync::Arc;

use crate::engine::action_processing::{
    FreeActionContext, apply_npc_events, execute_freeaction_impl,
};
use crate::engine::trigger_eval::{get_times_met, is_currently_meeting, set_currently_meeting};
use crate::model::state::LogType;
use crate::narrative::quantifier::{
    MovementParseResult, MovementType, QuantifierConfidence, QuantifierParseResult,
    QuantifierResult,
};
use crate::test_support::{TestGameState, TestNpc, TestPlayer, TestWorld};

fn make_quantifier_result_no_movement() -> QuantifierResult {
    QuantifierResult {
        npcs: QuantifierParseResult {
            npc_ids: vec!["carla".to_string()],
            confidence: QuantifierConfidence::High,
        },
        movement: MovementParseResult {
            movement_type: None,
            destination: None,
            confidence: QuantifierConfidence::Low,
        },
    }
}

fn make_quantifier_result_with_movement(destination: &str) -> QuantifierResult {
    QuantifierResult {
        npcs: QuantifierParseResult {
            npc_ids: vec!["carla".to_string()],
            confidence: QuantifierConfidence::High,
        },
        movement: MovementParseResult {
            movement_type: Some(MovementType::Entering),
            destination: Some(destination.to_string()),
            confidence: QuantifierConfidence::High,
        },
    }
}

#[test]
fn test_execute_freeaction_impl_no_movement() {
    let mut state = TestGameState::with_npc_raw("room1", TestNpc::named("carla", "Carla"));
    let world = Arc::new(TestWorld::minimal());
    let player = Arc::new(TestPlayer::standard());
    let all_npcs = vec![TestNpc::named("carla", "Carla")];
    let history = vec![];

    let result = execute_freeaction_impl(
        &mut state,
        &FreeActionContext {
            narration_text: "You examine the room.",
            user_input: "examine the room",
            quantifier_result: &make_quantifier_result_no_movement(),
            world: &world,
            player: &player,
            all_npcs: &all_npcs,
            history: &history,
            llm_backend: &crate::narrative::llm::MockBackend::default(),
        },
    );

    assert!(result.is_ok());
    // Narration should be logged
    assert_eq!(state.narration_history.len(), 1);
    assert_eq!(state.narration_history[0].log_type, LogType::Narration);
    // NPCs in area should be updated
    assert_eq!(state.npcs_in_area.len(), 1);
    assert_eq!(state.npcs_in_area[0].id, "carla");
}

#[test]
fn test_execute_freeaction_impl_with_movement() {
    let mut state = TestGameState::with_npc_raw("room1", TestNpc::named("carla", "Carla"));
    let world = Arc::new(TestWorld::minimal());
    let player = Arc::new(TestPlayer::standard());
    let all_npcs = vec![TestNpc::named("carla", "Carla")];
    let history = vec![];

    // quantifier result with movement to a new room
    let result = execute_freeaction_impl(
        &mut state,
        &FreeActionContext {
            narration_text: "You walk to the tavern.",
            user_input: "walk to the tavern",
            quantifier_result: &make_quantifier_result_with_movement("nonexistent_room"),
            world: &world,
            player: &player,
            all_npcs: &all_npcs,
            history: &history,
            llm_backend: &crate::narrative::llm::MockBackend::default(),
        },
    );

    assert!(result.is_ok());
    // Narration logged
    assert!(!state.narration_history.is_empty());
    // Room changed to a dynamic room (since destination doesn't exist)
    assert!(state.current_room_id.starts_with("dynamic_"));
    assert!(state.dynamic_rooms.contains_key(&state.current_room_id));
}

#[test]
fn test_execute_freeaction_impl_updates_npcs_in_area() {
    let mut state = TestGameState::with_npc_raw("room1", TestNpc::named("carla", "Carla"));
    let world = Arc::new(TestWorld::minimal());
    let player = Arc::new(TestPlayer::standard());
    let all_npcs = vec![TestNpc::named("carla", "Carla")];

    assert!(state.npcs_in_area.is_empty());

    let result = execute_freeaction_impl(
        &mut state,
        &FreeActionContext {
            narration_text: "You look around.",
            user_input: "look around",
            quantifier_result: &make_quantifier_result_no_movement(),
            world: &world,
            player: &player,
            all_npcs: &all_npcs,
            history: &[],
            llm_backend: &crate::narrative::llm::MockBackend::default(),
        },
    );

    assert!(result.is_ok());
    // npcs_in_area should now contain carla
    assert_eq!(state.npcs_in_area.len(), 1);
    assert_eq!(state.npcs_in_area[0].id, "carla");
}

#[test]
fn test_execute_freeaction_impl_triggers_evaluated() {
    let npc = TestNpc::with_times_met_trigger(
        "carla",
        "Carla",
        crate::model::trigger::ComparisonOperator::Eq,
        0,
    );

    let mut state = TestGameState::with_npc_raw("room1", npc.clone());
    let world = Arc::new(TestWorld::minimal());
    let player = Arc::new(TestPlayer::standard());

    // NPC has TimesMet Eq 0 trigger - should fire because times_met starts at 0
    // Note: evaluate_and_narrate_triggers calls LLM internally, so this test
    // will use whatever backend is configured (mock in test env)
    let result = execute_freeaction_impl(
        &mut state,
        &FreeActionContext {
            narration_text: "You enter the room.",
            user_input: "enter",
            quantifier_result: &make_quantifier_result_no_movement(),
            world: &world,
            player: &player,
            all_npcs: &[], // empty won't match triggers
            history: &[],
            llm_backend: &crate::narrative::llm::MockBackend::default(),
        },
    );

    // Result should be ok (even if trigger LLM call fails, fn handles gracefully)
    assert!(result.is_ok());
}

#[test]
fn test_execute_freeaction_impl_npc_events_entered() {
    let mut state = TestGameState::with_npc_raw("room1", TestNpc::named("carla", "Carla"));
    // NPC already in area (simulating re-encounter after leaving)
    state.npcs_in_area = vec![]; // Empty - NPC is not currently in area
    let world = Arc::new(TestWorld::minimal());
    let player = Arc::new(TestPlayer::standard());
    let all_npcs = vec![TestNpc::named("carla", "Carla")];

    let result = execute_freeaction_impl(
        &mut state,
        &FreeActionContext {
            narration_text: "You see Carla.",
            user_input: "look around",
            quantifier_result: &make_quantifier_result_no_movement(),
            world: &world,
            player: &player,
            all_npcs: &all_npcs,
            history: &[],
            llm_backend: &crate::narrative::llm::MockBackend::default(),
        },
    );

    assert!(result.is_ok());
    // NPC enters - times_met should increment
    let times_met = state
        .character_state
        .npcs
        .get("carla")
        .map(|s| s.times_met)
        .unwrap_or(0);
    assert_eq!(times_met, 1);
}

use crate::engine::action_processing::{
    evaluate_and_narrate_triggers, get_static_npcs, handle_movement,
};
use crate::test_support::TestMap;

use crate::model::state::GameState;

fn make_test_state() -> GameState {
    TestGameState::with_npc_in_named_room_raw(
        "test_room",
        "Test Room",
        TestNpc::named("carla", "Carla"),
    )
}

#[test]
fn test_get_static_npcs_returns_npcs() {
    let state = make_test_state();
    let room_npcs = vec!["carla".to_string()];
    let result = get_static_npcs(&state, &room_npcs);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, "carla");
}

#[test]
fn test_get_static_npcs_empty_for_unknown() {
    let state = make_test_state();
    let room_npcs = vec!["unknown".to_string()];
    let result = get_static_npcs(&state, &room_npcs);
    assert!(result.is_empty());
}

#[test]
fn test_apply_npc_events_entered() {
    let mut state = make_test_state();
    let events = vec![crate::narrative::quantifier::NpcEvent {
        npc_id: "carla".to_string(),
        event_type: crate::narrative::quantifier::NpcEventType::Entered,
    }];

    apply_npc_events(&mut state, &events);

    assert!(is_currently_meeting(&state.character_state, "carla"));
}

#[test]
fn test_apply_npc_events_left() {
    let mut state = make_test_state();
    set_currently_meeting(&mut state.character_state, "carla", true);

    let events = vec![crate::narrative::quantifier::NpcEvent {
        npc_id: "carla".to_string(),
        event_type: crate::narrative::quantifier::NpcEventType::Left,
    }];

    apply_npc_events(&mut state, &events);

    assert!(!is_currently_meeting(&state.character_state, "carla"));
}

#[test]
fn test_apply_npc_events_increments_times_met() {
    let mut state = make_test_state();
    let initial_times = get_times_met(&state.character_state, "carla");

    let events = vec![crate::narrative::quantifier::NpcEvent {
        npc_id: "carla".to_string(),
        event_type: crate::narrative::quantifier::NpcEventType::Entered,
    }];

    apply_npc_events(&mut state, &events);

    assert_eq!(
        get_times_met(&state.character_state, "carla"),
        initial_times + 1
    );
}

#[test]
fn test_handle_movement_no_destination() {
    let mut state = make_test_state();
    let original_room = state.current_room_id.clone();

    handle_movement(&mut state, None, &["carla".to_string()]);

    // Room should not change when destination is None
    assert_eq!(state.current_room_id, original_room);
}

#[test]
fn test_handle_movement_same_room_no_increment() {
    let mut state = make_test_state();
    // Already in test_room, moving to same room
    state.current_room_id = "test_room".to_string();
    let initial_times = get_times_met(&state.character_state, "carla");

    handle_movement(&mut state, Some("test_room"), &["carla".to_string()]);

    // times_met should not increment when room doesn't change
    assert_eq!(
        get_times_met(&state.character_state, "carla"),
        initial_times
    );
}

#[test]
fn test_handle_movement_creates_dynamic_room() {
    let mut state = make_test_state();
    let original_room = state.current_room_id.clone();

    // Attempt to move to a non-existent room
    handle_movement(&mut state, Some("nonexistent_room"), &[]);

    // Should create a dynamic room
    assert_ne!(state.current_room_id, original_room);
    assert!(state.dynamic_rooms.contains_key(&state.current_room_id));
}

#[test]
fn test_handle_movement_success_adds_room_log() {
    let mut state = make_test_state();

    handle_movement(&mut state, Some("test_room"), &["carla".to_string()]);

    assert!(!state.narration_history.is_empty());
    let last_entry = state.narration_history.last().unwrap();
    assert_eq!(last_entry.log_type, LogType::Narration);
    assert_eq!(last_entry.sender, Some("Test Room".to_string()));
}

#[test]
fn test_handle_movement_sets_currently_meeting() {
    let mut state = make_test_state();

    handle_movement(&mut state, Some("new_room"), &["carla".to_string()]);

    // Should set currently_meeting for NPCs in new room
    assert!(is_currently_meeting(&state.character_state, "carla"));
}

#[test]
fn test_evaluate_and_narrate_triggers_adds_event_header() {
    let llm_backend = crate::narrative::llm::MockBackend::default();

    let mut state = make_test_state();
    let npc_with_trigger = TestNpc::with_times_met_trigger(
        "carla",
        "Carla",
        crate::model::trigger::ComparisonOperator::Eq,
        0,
    );
    state
        .npcs
        .insert("carla".to_string(), npc_with_trigger.clone());

    let mut room = TestMap::room_named("test_room", "Test Room");
    room.npcs.push("carla".to_string());

    let world = state.world.clone();
    let player = state.player.clone();
    let history = state.narration_history.clone();

    let trigger_context = crate::narrative::prompt::PromptContext {
        world: &world,
        room: &room,
        all_npcs: &[],
        npcs_in_area: &[],
        player: &player,
        user_message: "test",
        history: &history,
    };

    evaluate_and_narrate_triggers(
        &mut state,
        "You enter the room.",
        &trigger_context,
        &llm_backend,
    );

    // Should have at least 2 entries: event header + narration
    assert!(
        state.narration_history.len() >= 2,
        "Expected event header + narration, got {:?}",
        state.narration_history
    );

    // First trigger-related entry should be the event header
    let event_entry = &state.narration_history[0];
    assert_eq!(event_entry.log_type, LogType::Event);
    assert_eq!(event_entry.sender, Some("Carla Introduction".to_string()));
    assert_eq!(event_entry.text, "");

    // Second entry should be the narration
    let narration_entry = &state.narration_history[1];
    assert_eq!(narration_entry.log_type, LogType::Narration);
}
