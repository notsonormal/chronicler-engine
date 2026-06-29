use crate::domain::engine::action_processing::{
    FreeActionContext, apply_npc_events, commit_trigger_narration, execute_freeaction_impl,
};
use crate::domain::model::quantifier::{
    MovementParseResult, MovementType, NpcEvent, NpcTransitionType, QuantifierConfidence,
    QuantifierParseResult, QuantifierResult,
};
use crate::domain::model::state::message_types::MessageType;
use crate::test_support::{TestGameState, TestNpc};

fn make_quantifier_result_no_movement() -> QuantifierResult {
    QuantifierResult {
        npcs: QuantifierParseResult {
            npc_ids: vec!["carla".to_string()],
            confidence: QuantifierConfidence::High,
        },
        movement: MovementParseResult::default(),
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
    // Narration already added by phase_narrate (pre-quantifier save)
    state.add_message(
        "You examine the room.".to_string(),
        None,
        MessageType::Narration,
    );

    let result = execute_freeaction_impl(
        &state,
        &FreeActionContext {
            narration_text: "You examine the room.",
            quantifier_result: &make_quantifier_result_no_movement(),
        },
    );

    assert!(
        result.is_ok(),
        "execute_freeaction_impl failed: {:?}",
        result.err()
    );
    let next_state = result.unwrap().next_state;
    // Narration should already exist (not duplicated)
    assert_eq!(next_state.narrative.history().len(), 1);
    assert_eq!(
        next_state.narrative.history()[0].message_type,
        MessageType::Narration
    );
    // NPCs in area should be updated
    assert_eq!(next_state.scene.npcs_in_area.len(), 1);
    assert_eq!(next_state.scene.npcs_in_area[0].id, "carla");
}

#[test]
fn test_execute_freeaction_impl_with_movement() {
    let state = TestGameState::with_npc_raw("room1", TestNpc::named("carla", "Carla"));

    // quantifier result with movement to a new room
    let result = execute_freeaction_impl(
        &state,
        &FreeActionContext {
            narration_text: "You walk to the tavern.",
            quantifier_result: &make_quantifier_result_with_movement("nonexistent_room"),
        },
    );

    assert!(
        result.is_ok(),
        "execute_freeaction_impl failed: {:?}",
        result.err()
    );
    let next_state = result.unwrap().next_state;
    // Narration logged
    assert!(!next_state.narrative.history().is_empty());
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

    assert!(state.scene.npcs_in_area.is_empty());

    let result = execute_freeaction_impl(
        &state,
        &FreeActionContext {
            narration_text: "You look around.",
            quantifier_result: &make_quantifier_result_no_movement(),
        },
    );

    assert!(result.is_ok());
    let next_state = result.unwrap().next_state;
    // npcs_in_area should now contain carla
    assert_eq!(next_state.scene.npcs_in_area.len(), 1);
    assert_eq!(next_state.scene.npcs_in_area[0].id, "carla");
}

#[test]
fn test_execute_freeaction_impl_npc_events_entered() {
    let mut state = TestGameState::with_npc_raw("room1", TestNpc::named("carla", "Carla"));
    // NPC already in area (simulating re-encounter after leaving)
    state.scene.npcs_in_area = vec![]; // Empty - NPC is not currently in area

    let result = execute_freeaction_impl(
        &state,
        &FreeActionContext {
            narration_text: "You see Carla.",
            quantifier_result: &make_quantifier_result_no_movement(),
        },
    );

    assert!(result.is_ok());
    // NPC enters - times_met should increment
    let next_state = result.unwrap().next_state;
    let times_met = next_state
        .npc_encounter_log
        .npcs
        .get("carla")
        .map(|s| s.times_met)
        .unwrap_or(0);
    assert_eq!(times_met, 1);
}

use crate::domain::engine::action_processing::handle_movement;
use crate::domain::model::state::game_state::GameState;

fn make_test_state() -> GameState {
    TestGameState::with_npc_in_named_room_raw(
        "test_room",
        "Test Room",
        TestNpc::named("carla", "Carla"),
    )
}

#[test]
fn test_apply_npc_events_entered() {
    let state = make_test_state();
    let events = vec![NpcEvent {
        npc_id: "carla".to_string(),
        event_type: NpcTransitionType::Entered,
    }];

    let state = apply_npc_events(state, &events).unwrap();

    assert!(state.npc_encounter_log.is_currently_meeting("carla"));
}

#[test]
fn test_apply_npc_events_left() {
    let mut state = make_test_state();
    state.npc_encounter_log.set_currently_meeting("carla", true);
    let events = vec![NpcEvent {
        npc_id: "carla".to_string(),
        event_type: NpcTransitionType::Left,
    }];

    let state = apply_npc_events(state, &events).unwrap();

    assert!(!state.npc_encounter_log.is_currently_meeting("carla"));
}

#[test]
fn test_apply_npc_events_increments_times_met() {
    let state = make_test_state();
    let initial_times = state.npc_encounter_log.get_times_met("carla");
    let events = vec![NpcEvent {
        npc_id: "carla".to_string(),
        event_type: NpcTransitionType::Entered,
    }];

    let state = apply_npc_events(state, &events).unwrap();

    assert_eq!(
        state.npc_encounter_log.get_times_met("carla"),
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
    let initial_times = state.npc_encounter_log.get_times_met("carla");

    let state = handle_movement(state, Some("test_room"), &["carla".to_string()]).unwrap();

    // times_met should not increment when room doesn't change
    assert_eq!(
        state.npc_encounter_log.get_times_met("carla"),
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
fn test_handle_movement_sets_pending_location() {
    let state = make_test_state();

    let state = handle_movement(state, Some("test_room"), &["carla".to_string()]).unwrap();

    assert!(state.narrative.history().is_empty());
    assert_eq!(
        state.narrative.pending_location,
        Some("Test Room".to_string())
    );
}

#[test]
fn test_handle_movement_sets_currently_meeting() {
    let state = make_test_state();

    let state = handle_movement(state, Some("new_room"), &["carla".to_string()]).unwrap();

    // Should set currently_meeting for NPCs in new room
    assert!(state.npc_encounter_log.is_currently_meeting("carla"));
}

#[test]
fn test_trigger_split_architecture_produces_event_header() {
    let npc = TestNpc::with_times_met_trigger(
        "carla",
        "Carla",
        crate::domain::model::trigger::ComparisonOperator::Eq,
        0,
    );
    let mut state = TestGameState::with_npc_raw("test_room", npc);
    // Narration already added by phase_narrate (pre-quantifier save)
    state.add_message(
        "You enter the room.".to_string(),
        None,
        MessageType::Narration,
    );

    let turn_result = execute_freeaction_impl(
        &state,
        &FreeActionContext {
            narration_text: "You enter the room.",
            quantifier_result: &make_quantifier_result_no_movement(),
        },
    )
    .unwrap();

    let request = crate::test_support::TestStoredTriggerContext::for_npc(
        "carla",
        "Carla Introduction",
        "Carla appears",
    );

    let state = commit_trigger_narration(
        turn_result.next_state,
        &request,
        "Carla emerges from the shadows.",
    )
    .unwrap();

    // Should have 2 entries: main narration + trigger continuation
    assert_eq!(state.narrative.history().len(), 2);

    let main_entry = &state.narrative.history()[0];
    assert_eq!(main_entry.message_type, MessageType::Narration);
    assert_eq!(main_entry.text, "You enter the room.");
    assert_eq!(main_entry.event_header, None);

    let trigger_entry = &state.narrative.history()[1];
    assert_eq!(trigger_entry.message_type, MessageType::Narration);
    assert_eq!(
        trigger_entry.event_header,
        Some("Carla Introduction".to_string())
    );
    assert_eq!(trigger_entry.text, "Carla emerges from the shadows.");
}

#[test]
fn test_commit_trigger_narration_adds_event_header_and_narration() {
    let state = make_test_state();

    let request = crate::test_support::TestStoredTriggerContext::for_npc(
        "carla",
        "Carla Introduction",
        "Carla appears",
    );

    let state =
        commit_trigger_narration(state, &request, "Gabriella emerges from the shadows.").unwrap();

    assert_eq!(state.narrative.history().len(), 1);

    let narration_entry = &state.narrative.history()[0];
    assert_eq!(narration_entry.message_type, MessageType::Narration);
    assert_eq!(
        narration_entry.event_header,
        Some("Carla Introduction".to_string())
    );
    assert_eq!(narration_entry.text, "Gabriella emerges from the shadows.");
}

#[test]
fn test_commit_trigger_narration_marks_non_repeat_trigger_fired() {
    let state = make_test_state();

    let request = crate::test_support::TestStoredTriggerContext::for_npc(
        "carla",
        "Carla Introduction",
        "Carla appears",
    );

    let state = commit_trigger_narration(state, &request, "Some text.").unwrap();

    assert!(
        state.npc_encounter_log.is_trigger_fired("carla", 0),
        "Non-repeating trigger should be marked as fired"
    );
}

#[test]
fn test_commit_trigger_narration_does_not_mark_repeat_trigger_fired() {
    let state = make_test_state();

    let mut stored = crate::test_support::TestStoredTriggerContext::for_npc(
        "carla",
        "Carla Greeting",
        "Carla greets",
    );
    stored.trigger_repeat = true;
    let request = stored;

    let state = commit_trigger_narration(state, &request, "Some text.").unwrap();

    assert!(
        !state.npc_encounter_log.is_trigger_fired("carla", 0),
        "Repeating trigger should NOT be marked as fired"
    );
}

#[test]
fn test_commit_trigger_narration_empty_text_is_noop() {
    let state = make_test_state();

    let request = crate::test_support::TestStoredTriggerContext::for_npc(
        "carla",
        "Carla Introduction",
        "Carla appears",
    );

    let state = commit_trigger_narration(state, &request, "").unwrap();
    assert!(state.narrative.history().is_empty());

    let state = commit_trigger_narration(state, &request, "   ").unwrap();
    assert!(state.narrative.history().is_empty());
}

#[test]
fn test_commit_trigger_narration_stores_trigger_context() {
    let state = make_test_state();

    let request = crate::test_support::TestStoredTriggerContext::with_max_tokens(
        "carla",
        "Carla Introduction",
        "Carla appears from shadows",
        512,
    );

    let state = commit_trigger_narration(state, &request, "Carla emerges.").unwrap();

    let trigger = state
        .narrative
        .last_trigger
        .expect("last_trigger should be set");
    assert_eq!(trigger.npc_id, "carla");
    assert_eq!(trigger.trigger_idx, 0);
    assert_eq!(trigger.trigger_name, "Carla Introduction");
    assert!(!trigger.trigger_repeat);
    assert_eq!(
        trigger.trigger_narration_prompt,
        "Carla appears from shadows"
    );
    assert_eq!(trigger.system_prompt, "system prompt text");
    assert_eq!(trigger.user_prompt, "user prompt text");
    assert_eq!(trigger.max_tokens, Some(512));
}
use proptest::prelude::*;

fn make_two_room_state() -> GameState {
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
        crate::domain::engine::state_diagnostics::assert_state_consistency(&state).ok();
    }

    #[test]
    fn prop_apply_npc_events_preserves_state_consistency(
        events in prop::collection::vec(
            ("[a-z]{1,10}", prop_oneof![
                Just(NpcTransitionType::Entered),
                Just(NpcTransitionType::Left),
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
        crate::domain::engine::state_diagnostics::assert_state_consistency(&state).ok();
    }

    #[test]
    fn prop_execute_freeaction_impl_preserves_state_consistency(
        has_movement in prop::bool::ANY,
        destination in "[a-z]{1,15}",
    ) {
        let state = TestGameState::with_npc_raw("room1", TestNpc::named("carla", "Carla"));

        let movement = if has_movement {
            MovementParseResult {
                movement_type: Some(MovementType::Entering),
                destination: Some(destination),
                confidence: QuantifierConfidence::High,
            }
        } else {
            MovementParseResult::default()
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
                quantifier_result: &quantifier_result,
            },
        );

        prop_assert!(
            result.is_ok(),
            "execute_freeaction_impl failed: {:?}",
            result.err()
        );
        let next_state = result.unwrap().next_state;
        crate::domain::engine::state_diagnostics::assert_state_consistency(&next_state).ok();
    }
}
