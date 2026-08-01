use std::collections::HashMap;
use std::sync::Arc;

use proptest::prelude::*;

use crate::domain::model::state::game_state::{FreeActionContext, GameState};
use crate::domain::model::character::{NpcCard, PersonaCard};
use crate::domain::model::map::{MapDef, Overworld, Region};
use crate::domain::model::quantifier::{
    MovementParseResult, MovementType, NpcEvent, NpcTransitionType, QuantifierConfidence,
    QuantifierParseResult, QuantifierResult,
};
use crate::domain::model::state::message_types::MessageType;
use crate::test_support::{TestGameState, TestMap, TestNpc, TestPersona};

struct EngineDeps {
    map: Arc<MapDef>,
    persona: Arc<PersonaCard>,
    npcs: HashMap<String, NpcCard>,
}

fn engine_deps(map: MapDef, npcs: Vec<NpcCard>) -> EngineDeps {
    EngineDeps {
        map: Arc::new(map),
        persona: Arc::new(TestPersona::standard()),
        npcs: npcs.into_iter().map(|npc| (npc.id.clone(), npc)).collect(),
    }
}

fn single_room_named(room_id: &str, room_name: &str) -> MapDef {
    MapDef {
        overworld: Overworld {
            id: "test_overworld".to_string(),
            name: "Test Overworld".to_string(),
            regions: vec![Region {
                id: "test_region".to_string(),
                name: "Test Region".to_string(),
                rooms: vec![TestMap::room_named(room_id, room_name)],
            }],
        },
    }
}

fn deps_with_carla(room_id: &str) -> EngineDeps {
    engine_deps(
        single_room_named(room_id, "Test Room"),
        vec![TestNpc::named("carla", "Carla")],
    )
}

fn deps_for_npc_ids(map: MapDef, ids: &[String]) -> EngineDeps {
    let npcs = ids
        .iter()
        .map(|id| TestNpc::named(id, id))
        .collect::<Vec<_>>();
    engine_deps(map, npcs)
}

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

fn make_test_state() -> GameState {
    TestGameState::in_room("test_room")
}

#[test]
fn test_execute_freeaction_impl_no_movement() {
    let deps = deps_with_carla("room1");
    let mut state = TestGameState::in_room("room1");
    state.add_message(
        "You examine the room.".to_string(),
        None,
        MessageType::Narration,
    );

    let result = state.execute_freeaction_impl(
        &FreeActionContext {
            narration_text: "You examine the room.",
            quantifier_result: &make_quantifier_result_no_movement(),
        },
        &deps.map,
        &deps.persona,
        &deps.npcs,
    );

    assert!(
        result.is_ok(),
        "execute_freeaction_impl failed: {:?}",
        result.err()
    );
    let next_state = result.unwrap().next_state;
    assert_eq!(next_state.narrative.history().len(), 1);
    assert_eq!(
        next_state.narrative.history()[0].message_type,
        MessageType::Narration
    );
    assert_eq!(next_state.scene.npcs_in_area.len(), 1);
    assert_eq!(next_state.scene.npcs_in_area[0].id, "carla");
}

#[test]
fn test_execute_freeaction_impl_with_movement() {
    let deps = deps_with_carla("room1");
    let state = TestGameState::in_room("room1");

    let result = state.execute_freeaction_impl(
        &FreeActionContext {
            narration_text: "You walk to the tavern.",
            quantifier_result: &make_quantifier_result_with_movement("nonexistent_room"),
        },
        &deps.map,
        &deps.persona,
        &deps.npcs,
    );

    assert!(
        result.is_ok(),
        "execute_freeaction_impl failed: {:?}",
        result.err()
    );
    let next_state = result.unwrap().next_state;
    assert!(!next_state.narrative.history().is_empty());
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
    let deps = deps_with_carla("room1");
    let state = TestGameState::in_room("room1");

    assert!(state.scene.npcs_in_area.is_empty());

    let result = state.execute_freeaction_impl(
        &FreeActionContext {
            narration_text: "You look around.",
            quantifier_result: &make_quantifier_result_no_movement(),
        },
        &deps.map,
        &deps.persona,
        &deps.npcs,
    );

    assert!(result.is_ok());
    let next_state = result.unwrap().next_state;
    assert_eq!(next_state.scene.npcs_in_area.len(), 1);
    assert_eq!(next_state.scene.npcs_in_area[0].id, "carla");
}

#[test]
fn test_execute_freeaction_impl_npc_events_entered() {
    let deps = deps_with_carla("room1");
    let mut state = TestGameState::in_room("room1");
    state.scene.npcs_in_area = vec![];

    let result = state.execute_freeaction_impl(
        &FreeActionContext {
            narration_text: "You see Carla.",
            quantifier_result: &make_quantifier_result_no_movement(),
        },
        &deps.map,
        &deps.persona,
        &deps.npcs,
    );

    assert!(result.is_ok());
    let next_state = result.unwrap().next_state;
    let times_met = next_state
        .npc_encounter_log
        .npcs
        .get("carla")
        .map(|s| s.times_met)
        .unwrap_or(0);
    assert_eq!(times_met, 1);
}

#[test]
fn test_apply_npc_events_entered() {
    let deps = deps_with_carla("test_room");
    let mut state = make_test_state();
    let events = vec![NpcEvent {
        npc_id: "carla".to_string(),
        event_type: NpcTransitionType::Entered,
    }];

    state
        .apply_npc_events(&events, &deps.map, &deps.npcs)
        .unwrap();

    assert!(state.npc_encounter_log.is_currently_meeting("carla"));
}

#[test]
fn test_apply_npc_events_left() {
    let deps = deps_with_carla("test_room");
    let mut state = make_test_state();
    state.npc_encounter_log.set_currently_meeting("carla", true);
    let events = vec![NpcEvent {
        npc_id: "carla".to_string(),
        event_type: NpcTransitionType::Left,
    }];

    state
        .apply_npc_events(&events, &deps.map, &deps.npcs)
        .unwrap();

    assert!(!state.npc_encounter_log.is_currently_meeting("carla"));
}

#[test]
fn test_apply_npc_events_increments_times_met() {
    let deps = deps_with_carla("test_room");
    let mut state = make_test_state();
    let initial_times = state.npc_encounter_log.get_times_met("carla");
    let events = vec![NpcEvent {
        npc_id: "carla".to_string(),
        event_type: NpcTransitionType::Entered,
    }];

    state
        .apply_npc_events(&events, &deps.map, &deps.npcs)
        .unwrap();

    assert_eq!(
        state.npc_encounter_log.get_times_met("carla"),
        initial_times + 1
    );
}

#[test]
fn test_handle_movement_no_destination() {
    let deps = deps_with_carla("test_room");
    let state = make_test_state();
    let original_room = state.movement.current_room_id.clone();

    let state = state
        .handle_movement(None, &["carla".to_string()], &deps.map, &deps.npcs)
        .unwrap();

    assert_eq!(state.movement.current_room_id, original_room);
}

#[test]
fn test_handle_movement_same_room_no_increment() {
    let deps = deps_with_carla("test_room");
    let mut state = make_test_state();
    state.movement.current_room_id = "test_room".to_string();
    let initial_times = state.npc_encounter_log.get_times_met("carla");

    let state = state
        .handle_movement(
            Some("test_room"),
            &["carla".to_string()],
            &deps.map,
            &deps.npcs,
        )
        .unwrap();

    assert_eq!(
        state.npc_encounter_log.get_times_met("carla"),
        initial_times
    );
}

#[test]
fn test_handle_movement_creates_dynamic_room() {
    let deps = deps_with_carla("test_room");
    let state = make_test_state();
    let original_room = state.movement.current_room_id.clone();

    let state = state
        .handle_movement(Some("nonexistent_room"), &[], &deps.map, &deps.npcs)
        .unwrap();

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
    let deps = deps_with_carla("test_room");
    let state = make_test_state();

    let state = state
        .handle_movement(
            Some("test_room"),
            &["carla".to_string()],
            &deps.map,
            &deps.npcs,
        )
        .unwrap();

    assert!(state.narrative.history().is_empty());
    assert_eq!(
        state.narrative.pending_location,
        Some("Test Room".to_string())
    );
}

#[test]
fn test_handle_movement_sets_currently_meeting() {
    let deps = deps_with_carla("test_room");
    let state = make_test_state();

    let state = state
        .handle_movement(
            Some("new_room"),
            &["carla".to_string()],
            &deps.map,
            &deps.npcs,
        )
        .unwrap();

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
    let deps = engine_deps(single_room_named("test_room", "Test Room"), vec![npc]);
    let mut state = TestGameState::in_room("test_room");
    state.add_message(
        "You enter the room.".to_string(),
        None,
        MessageType::Narration,
    );

    let turn_result = state
        .execute_freeaction_impl(
            &FreeActionContext {
                narration_text: "You enter the room.",
                quantifier_result: &make_quantifier_result_no_movement(),
            },
            &deps.map,
            &deps.persona,
            &deps.npcs,
        )
        .unwrap();

    let request = crate::test_support::TestStoredTriggerContext::for_npc(
        "carla",
        "Carla Introduction",
        "Carla appears",
    );

    let mut state = turn_result.next_state;
    state
        .commit_trigger_narration(
            &request,
            "Carla emerges from the shadows.",
            &deps.map,
            &deps.npcs,
        )
        .unwrap();

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
    let deps = deps_with_carla("test_room");
    let mut state = make_test_state();

    let request = crate::test_support::TestStoredTriggerContext::for_npc(
        "carla",
        "Carla Introduction",
        "Carla appears",
    );

    state
        .commit_trigger_narration(
            &request,
            "Gabriella emerges from the shadows.",
            &deps.map,
            &deps.npcs,
        )
        .unwrap();

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
    let deps = deps_with_carla("test_room");
    let mut state = make_test_state();

    let request = crate::test_support::TestStoredTriggerContext::for_npc(
        "carla",
        "Carla Introduction",
        "Carla appears",
    );

    state
        .commit_trigger_narration(&request, "Some text.", &deps.map, &deps.npcs)
        .unwrap();

    assert!(
        state.npc_encounter_log.is_trigger_fired("carla", 0),
        "Non-repeating trigger should be marked as fired"
    );
}

#[test]
fn test_commit_trigger_narration_does_not_mark_repeat_trigger_fired() {
    let deps = deps_with_carla("test_room");
    let mut state = make_test_state();

    let mut stored = crate::test_support::TestStoredTriggerContext::for_npc(
        "carla",
        "Carla Greeting",
        "Carla greets",
    );
    stored.trigger_repeat = true;
    let request = stored;

    state
        .commit_trigger_narration(&request, "Some text.", &deps.map, &deps.npcs)
        .unwrap();

    assert!(
        !state.npc_encounter_log.is_trigger_fired("carla", 0),
        "Repeating trigger should NOT be marked as fired"
    );
}

#[test]
fn test_commit_trigger_narration_empty_text_is_noop() {
    let deps = deps_with_carla("test_room");
    let mut state = make_test_state();

    let request = crate::test_support::TestStoredTriggerContext::for_npc(
        "carla",
        "Carla Introduction",
        "Carla appears",
    );

    state
        .commit_trigger_narration(&request, "", &deps.map, &deps.npcs)
        .unwrap();
    assert!(state.narrative.history().is_empty());

    state
        .commit_trigger_narration(&request, "   ", &deps.map, &deps.npcs)
        .unwrap();
    assert!(state.narrative.history().is_empty());
}

#[test]
fn test_commit_trigger_narration_stores_trigger_context() {
    let deps = deps_with_carla("test_room");
    let mut state = make_test_state();

    let request = crate::test_support::TestStoredTriggerContext::with_max_tokens(
        "carla",
        "Carla Introduction",
        "Carla appears from shadows",
        512,
    );

    state
        .commit_trigger_narration(&request, "Carla emerges.", &deps.map, &deps.npcs)
        .unwrap();

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

fn make_two_room_state() -> GameState {
    GameState::new("room1")
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
        let deps = deps_for_npc_ids(TestMap::two_rooms("room1", "room2"), &new_npc_ids);
        let state = make_two_room_state();
        let state = state.handle_movement( destination, &new_npc_ids, &deps.map, &deps.npcs).unwrap();
        state.assert_state_consistency(&deps.map, &deps.npcs).ok();
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
        let npc_ids = events.iter().map(|(npc_id, _)| npc_id.clone()).collect::<Vec<_>>();
        let deps = deps_for_npc_ids(TestMap::single_room("room1"), &npc_ids);
        let mut state = TestGameState::in_room("room1");
        let events: Vec<NpcEvent> = events
            .into_iter()
            .map(|(npc_id, event_type)| NpcEvent {
                npc_id,
                event_type,
            })
            .collect();
        state.apply_npc_events(&events, &deps.map, &deps.npcs).unwrap();
        state.assert_state_consistency(&deps.map, &deps.npcs).ok();
    }

    #[test]
    fn prop_execute_freeaction_impl_preserves_state_consistency(
        has_movement in prop::bool::ANY,
        destination in "[a-z]{1,15}",
    ) {
        let deps = deps_with_carla("room1");
        let state = TestGameState::in_room("room1");

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

        let result = state.execute_freeaction_impl(
            &FreeActionContext {
                narration_text: "You do something.",
                quantifier_result: &quantifier_result,
            },
            &deps.map,
            &deps.persona,
            &deps.npcs,
        );

        prop_assert!(
            result.is_ok(),
            "execute_freeaction_impl failed: {:?}",
            result.err()
        );
        let next_state = result.unwrap().next_state;
        next_state.assert_state_consistency(&deps.map, &deps.npcs).ok();
    }
}
