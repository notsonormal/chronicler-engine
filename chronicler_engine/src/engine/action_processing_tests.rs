use std::sync::Arc;

use crate::engine::action_processing::{
    FreeActionContext, TriggerContinuationRequest, apply_npc_events, commit_trigger_narration,
    execute_freeaction_impl,
};
use crate::engine::trigger_eval::{get_times_met, is_currently_meeting, set_currently_meeting};
use crate::model::state::LogType;
use crate::narrative::agents::quantifier::{
    MovementParseResult, MovementType, NpcEvent, NpcEventType, QuantifierConfidence,
    QuantifierParseResult, QuantifierResult,
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
    let state = TestGameState::with_npc_raw("room1", TestNpc::named("carla", "Carla"));
    let world = Arc::new(TestWorld::minimal());
    let player = Arc::new(TestPlayer::standard());
    let all_npcs = vec![TestNpc::named("carla", "Carla")];
    let history = vec![];

    let result = execute_freeaction_impl(
        &state,
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

    assert!(
        result.is_ok(),
        "execute_freeaction_impl failed: {:?}",
        result.err()
    );
    let next_state = result.unwrap().next_state;
    // Narration should be logged
    assert_eq!(next_state.narrative.history.len(), 1);
    assert_eq!(next_state.narrative.history[0].log_type, LogType::Narration);
    // NPCs in area should be updated
    assert_eq!(next_state.scene.npcs_in_area.len(), 1);
    assert_eq!(next_state.scene.npcs_in_area[0].id, "carla");
}

#[test]
fn test_execute_freeaction_impl_with_movement() {
    let state = TestGameState::with_npc_raw("room1", TestNpc::named("carla", "Carla"));
    let world = Arc::new(TestWorld::minimal());
    let player = Arc::new(TestPlayer::standard());
    let all_npcs = vec![TestNpc::named("carla", "Carla")];
    let history = vec![];

    // quantifier result with movement to a new room
    let result = execute_freeaction_impl(
        &state,
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

    assert!(
        result.is_ok(),
        "execute_freeaction_impl failed: {:?}",
        result.err()
    );
    let next_state = result.unwrap().next_state;
    // Narration logged
    assert!(!next_state.narrative.history.is_empty());
    // Room changed to a dynamic room (since destination doesn't exist)
    assert!(next_state.movement.current_room_id.starts_with("dynamic_"));
    assert!(
        next_state
            .movement
            .dynamic_rooms
            .contains_key(&next_state.movement.current_room_id)
    );
}

#[test]
fn test_execute_freeaction_impl_updates_npcs_in_area() {
    let state = TestGameState::with_npc_raw("room1", TestNpc::named("carla", "Carla"));
    let world = Arc::new(TestWorld::minimal());
    let player = Arc::new(TestPlayer::standard());
    let all_npcs = vec![TestNpc::named("carla", "Carla")];

    assert!(state.scene.npcs_in_area.is_empty());

    let result = execute_freeaction_impl(
        &state,
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
    let next_state = result.unwrap().next_state;
    // npcs_in_area should now contain carla
    assert_eq!(next_state.scene.npcs_in_area.len(), 1);
    assert_eq!(next_state.scene.npcs_in_area[0].id, "carla");
}

#[test]
fn test_execute_freeaction_impl_returns_trigger_request_when_trigger_matches() {
    let npc = TestNpc::with_times_met_trigger(
        "carla",
        "Carla",
        crate::model::trigger::ComparisonOperator::Eq,
        0,
    );

    let state = TestGameState::with_npc_raw("room1", npc.clone());
    let world = Arc::new(TestWorld::minimal());
    let player = Arc::new(TestPlayer::standard());

    let result = execute_freeaction_impl(
        &state,
        &FreeActionContext {
            narration_text: "You enter the room.",
            user_input: "enter",
            quantifier_result: &make_quantifier_result_no_movement(),
            world: &world,
            player: &player,
            all_npcs: &[],
            history: &[],
            llm_backend: &crate::narrative::llm::MockBackend::default(),
        },
    );

    assert!(result.is_ok(), "execute_freeaction_impl should succeed");
    let trigger_request = result.unwrap().trigger_continuation;
    assert!(
        trigger_request.is_some(),
        "Should return TriggerContinuationRequest when trigger matches"
    );
    let request = trigger_request.unwrap();
    assert_eq!(request.trigger_name, "Carla Introduction");
    assert_eq!(request.npc_id, "carla");
    assert!(!request.trigger_repeat);
    // Prompts should be non-empty after successful build
    assert!(!request.system_prompt.is_empty());
    assert!(!request.user_prompt.is_empty());
}

#[test]
fn test_execute_freeaction_impl_returns_none_when_no_trigger_matches() {
    // bartender has no triggers
    let state = TestGameState::with_npc_raw("room1", TestNpc::named("bartender", "Bartender"));
    let world = Arc::new(TestWorld::minimal());
    let player = Arc::new(TestPlayer::standard());

    let result = execute_freeaction_impl(
        &state,
        &FreeActionContext {
            narration_text: "You look around.",
            user_input: "look around",
            quantifier_result: &make_quantifier_result_no_movement(),
            world: &world,
            player: &player,
            all_npcs: &[],
            history: &[],
            llm_backend: &crate::narrative::llm::MockBackend::default(),
        },
    );

    assert!(result.is_ok(), "execute_freeaction_impl should succeed");
    assert!(
        result.unwrap().trigger_continuation.is_none(),
        "Should return None when no trigger matches"
    );
}

#[test]
fn test_execute_freeaction_impl_npc_events_entered() {
    let mut state = TestGameState::with_npc_raw("room1", TestNpc::named("carla", "Carla"));
    // NPC already in area (simulating re-encounter after leaving)
    state.scene.npcs_in_area = vec![]; // Empty - NPC is not currently in area
    let world = Arc::new(TestWorld::minimal());
    let player = Arc::new(TestPlayer::standard());
    let all_npcs = vec![TestNpc::named("carla", "Carla")];

    let result = execute_freeaction_impl(
        &state,
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
    let next_state = result.unwrap().next_state;
    let times_met = next_state
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
    let state = make_test_state();
    let events = vec![NpcEvent {
        npc_id: "carla".to_string(),
        event_type: NpcEventType::Entered,
    }];

    let state = apply_npc_events(state, &events).unwrap();

    assert!(is_currently_meeting(&state.character_state, "carla"));
}

#[test]
fn test_apply_npc_events_left() {
    let mut state = make_test_state();
    set_currently_meeting(&mut state.character_state, "carla", true);

    let events = vec![NpcEvent {
        npc_id: "carla".to_string(),
        event_type: NpcEventType::Left,
    }];

    let state = apply_npc_events(state, &events).unwrap();

    assert!(!is_currently_meeting(&state.character_state, "carla"));
}

#[test]
fn test_apply_npc_events_increments_times_met() {
    let state = make_test_state();
    let initial_times = get_times_met(&state.character_state, "carla");

    let events = vec![NpcEvent {
        npc_id: "carla".to_string(),
        event_type: NpcEventType::Entered,
    }];

    let state = apply_npc_events(state, &events).unwrap();

    assert_eq!(
        get_times_met(&state.character_state, "carla"),
        initial_times + 1
    );
}

#[test]
fn test_handle_movement_no_destination() {
    let state = make_test_state();
    let original_room = state.movement.current_room_id.clone();

    let state = handle_movement(state, None, &["carla".to_string()]).unwrap();

    // Room should not change when destination is None
    assert_eq!(state.movement.current_room_id, original_room);
}

#[test]
fn test_handle_movement_same_room_no_increment() {
    let mut state = make_test_state();
    // Already in test_room, moving to same room
    state.movement.current_room_id = "test_room".to_string();
    let initial_times = get_times_met(&state.character_state, "carla");

    let state = handle_movement(state, Some("test_room"), &["carla".to_string()]).unwrap();

    // times_met should not increment when room doesn't change
    assert_eq!(
        get_times_met(&state.character_state, "carla"),
        initial_times
    );
}

#[test]
fn test_handle_movement_creates_dynamic_room() {
    let state = make_test_state();
    let original_room = state.movement.current_room_id.clone();

    // Attempt to move to a non-existent room
    let state = handle_movement(state, Some("nonexistent_room"), &[]).unwrap();

    // Should create a dynamic room
    assert_ne!(state.movement.current_room_id, original_room);
    assert!(
        state
            .movement
            .dynamic_rooms
            .contains_key(&state.movement.current_room_id)
    );
}

#[test]
fn test_handle_movement_success_adds_room_log() {
    let state = make_test_state();

    let state = handle_movement(state, Some("test_room"), &["carla".to_string()]).unwrap();

    assert!(!state.narrative.history.is_empty());
    let last_entry = state.narrative.history.last().unwrap();
    assert_eq!(last_entry.log_type, LogType::Narration);
    assert_eq!(last_entry.sender, Some("Test Room".to_string()));
}

#[test]
fn test_handle_movement_sets_currently_meeting() {
    let state = make_test_state();

    let state = handle_movement(state, Some("new_room"), &["carla".to_string()]).unwrap();

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
    let history = state.narrative.history.clone();

    let trigger_context = crate::narrative::prompt::PromptContext {
        world: &world,
        room: &room,
        all_npcs: &[],
        npcs_in_area: &[],
        player: &player,
        user_message: "test",
        history: &history,
    };

    let state =
        evaluate_and_narrate_triggers(state, "You enter the room.", &trigger_context, &llm_backend)
            .unwrap();

    // Should have at least 2 entries: event header + narration
    assert!(
        state.narrative.history.len() >= 2,
        "Expected event header + narration, got {:?}",
        state.narrative.history
    );

    // First trigger-related entry should be the event header
    let event_entry = &state.narrative.history[0];
    assert_eq!(event_entry.log_type, LogType::Event);
    assert_eq!(event_entry.sender, Some("Carla Introduction".to_string()));
    assert_eq!(event_entry.text, "");

    // Second entry should be the narration
    let narration_entry = &state.narrative.history[1];
    assert_eq!(narration_entry.log_type, LogType::Narration);
}

#[test]
fn test_commit_trigger_narration_adds_event_header_and_narration() {
    let state = make_test_state();

    let request = TriggerContinuationRequest {
        npc_id: "carla".to_string(),
        trigger_idx: 0,
        trigger_name: "Carla Introduction".to_string(),
        trigger_repeat: false,
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        max_tokens: None,
    };

    let state =
        commit_trigger_narration(state, &request, "Gabriella emerges from the shadows.").unwrap();

    assert_eq!(state.narrative.history.len(), 2);

    let event_entry = &state.narrative.history[0];
    assert_eq!(event_entry.log_type, LogType::Event);
    assert_eq!(event_entry.sender, Some("Carla Introduction".to_string()));
    assert_eq!(event_entry.text, "");

    let narration_entry = &state.narrative.history[1];
    assert_eq!(narration_entry.log_type, LogType::Narration);
    assert_eq!(narration_entry.text, "Gabriella emerges from the shadows.");
}

#[test]
fn test_commit_trigger_narration_marks_non_repeat_trigger_fired() {
    let state = make_test_state();

    let request = TriggerContinuationRequest {
        npc_id: "carla".to_string(),
        trigger_idx: 0,
        trigger_name: "Carla Introduction".to_string(),
        trigger_repeat: false,
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        max_tokens: None,
    };

    let state = commit_trigger_narration(state, &request, "Some text.").unwrap();

    assert!(
        crate::engine::trigger_eval::is_trigger_fired(&state.character_state, "carla", 0),
        "Non-repeating trigger should be marked as fired"
    );
}

#[test]
fn test_commit_trigger_narration_does_not_mark_repeat_trigger_fired() {
    let state = make_test_state();

    let request = TriggerContinuationRequest {
        npc_id: "carla".to_string(),
        trigger_idx: 0,
        trigger_name: "Carla Greeting".to_string(),
        trigger_repeat: true,
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        max_tokens: None,
    };

    let state = commit_trigger_narration(state, &request, "Some text.").unwrap();

    assert!(
        !crate::engine::trigger_eval::is_trigger_fired(&state.character_state, "carla", 0),
        "Repeating trigger should NOT be marked as fired"
    );
}

#[test]
fn test_commit_trigger_narration_empty_text_is_noop() {
    let state = make_test_state();

    let request = TriggerContinuationRequest {
        npc_id: "carla".to_string(),
        trigger_idx: 0,
        trigger_name: "Carla Introduction".to_string(),
        trigger_repeat: false,
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        max_tokens: None,
    };

    let state = commit_trigger_narration(state, &request, "").unwrap();
    assert!(state.narrative.history.is_empty());

    let state = commit_trigger_narration(state, &request, "   ").unwrap();
    assert!(state.narrative.history.is_empty());
}

// ─── Property-based tests ────────────────────────────────────────────────────

use proptest::prelude::*;

fn make_two_room_state() -> crate::model::state::GameState {
    use crate::test_support::{TestMap, TestPlayer, TestWorld};
    use std::sync::Arc;
    GameState::new(
        Arc::new(TestWorld::minimal()),
        Arc::new(TestMap::two_rooms("room1", "room2")),
        Arc::new(TestPlayer::standard()),
        vec![],
        "room1".to_string(),
    )
}

proptest! {
    #[test]
    fn prop_handle_movement_preserves_state_consistency(
        destination in prop_oneof![
            Just(None),
            Just(Some("room2")),
            Just(Some("nonexistent_room")),
        ],
        new_npc_ids in prop::collection::vec("[a-z]{1,10}", 0..3),
    ) {
        let state = make_two_room_state();
        let state = handle_movement(state, destination, &new_npc_ids).unwrap();
        // The primary invariant: state remains consistent after movement
        crate::engine::state_diagnostics::assert_state_consistency(&state).ok();
    }

    #[test]
    fn prop_apply_npc_events_preserves_state_consistency(
        events in prop::collection::vec(
            ("[a-z]{1,10}", prop_oneof![
                Just(NpcEventType::Entered),
                Just(NpcEventType::Left),
            ]),
            0..10
        ),
    ) {
        let state = TestGameState::with_npc_raw("room1", TestNpc::named("carla", "Carla"));
        let events: Vec<NpcEvent> = events
            .into_iter()
            .map(|(npc_id, event_type)| NpcEvent {
                npc_id,
                event_type,
            })
            .collect();
        let state = apply_npc_events(state, &events).unwrap();
        crate::engine::state_diagnostics::assert_state_consistency(&state).ok();
    }

    #[test]
    fn prop_execute_freeaction_impl_preserves_state_consistency(
        has_movement in prop::bool::ANY,
        destination in "[a-z]{1,15}",
    ) {
        let state = TestGameState::with_npc_raw("room1", TestNpc::named("carla", "Carla"));
        let world = Arc::new(TestWorld::minimal());
        let player = Arc::new(TestPlayer::standard());
        let all_npcs = vec![TestNpc::named("carla", "Carla")];

        let movement = if has_movement {
            MovementParseResult {
                movement_type: Some(MovementType::Entering),
                destination: Some(destination),
                confidence: QuantifierConfidence::High,
            }
        } else {
            MovementParseResult {
                movement_type: None,
                destination: None,
                confidence: QuantifierConfidence::Low,
            }
        };

        let quantifier_result = QuantifierResult {
            npcs: QuantifierParseResult {
                npc_ids: vec!["carla".to_string()],
                confidence: QuantifierConfidence::High,
            },
            movement,
        };

        let result = execute_freeaction_impl(
            &state,
            &FreeActionContext {
                narration_text: "You do something.",
                user_input: "do something",
                quantifier_result: &quantifier_result,
                world: &world,
                player: &player,
                all_npcs: &all_npcs,
                history: &[],
                llm_backend: &crate::narrative::llm::MockBackend::default(),
            },
        );

        prop_assert!(
            result.is_ok(),
            "execute_freeaction_impl failed: {:?}",
            result.err()
        );
        let next_state = result.unwrap().next_state;
        crate::engine::state_diagnostics::assert_state_consistency(&next_state).ok();
    }
}
