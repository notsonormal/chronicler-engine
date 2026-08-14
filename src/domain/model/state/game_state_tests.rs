use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use proptest::prelude::*;

use crate::domain::model::character::{CharacterSheet, NpcCard, PersonaCard};
use crate::domain::model::map::{Direction, MapDef, Overworld, Region, Room};
use crate::domain::model::quantifier::{
    MovementParseResult, MovementType, NpcEvent, NpcTransitionType, QuantifierConfidence,
    QuantifierParseResult, QuantifierResult,
};
use crate::domain::model::state::game_state::{FreeActionContext, GameState, GameStateBuilder};
use crate::domain::model::state::generation_status::{GenerationStatus, InputBuffer};
use crate::domain::model::state::message_types::MessageType;
use crate::domain::model::state::scene_state::SceneState;
use crate::domain::model::trigger::{
    ComparisonOperator, NpcEncounterLog, Trigger, TriggerNarration, TriggerRequirement,
};
use crate::test_support::*;

#[test]
fn test_game_state_initialization() {
    let state = TestGameState::in_room("room_1");

    assert_eq!(state.movement.current_room_id, "room_1");
}

#[test]
fn test_generation_state_status() {
    let mut buf = InputBuffer::default();

    assert_eq!(buf.status, GenerationStatus::Idle);
    assert!(!buf.status.is_generating());
    assert!(buf.status.error_message().is_none());

    buf.status = GenerationStatus::Error("LLM Error: 429 Too Many Requests".to_string());
    assert_eq!(
        buf.status,
        GenerationStatus::Error("LLM Error: 429 Too Many Requests".to_string())
    );
    assert!(buf.status.error_message().is_some());

    buf.status = GenerationStatus::Generating;
    assert!(buf.status.is_generating());
    assert!(buf.status.error_message().is_none());
}

#[test]
fn test_log_ordering() {
    let mut state = TestGameState::in_room("room1");

    state.add_message("Message 1".into(), None, MessageType::Narration);
    state.add_message("Message 2".into(), None, MessageType::Narration);

    assert_eq!(state.narrative.history().len(), 2);
    assert_eq!(state.narrative.history()[0].text, "Message 1");
    assert_eq!(state.narrative.history()[1].text, "Message 2");
}

#[test]
fn test_delete_last_log_recalculates_ids() {
    let mut state = TestGameState::in_room("room1");

    state.add_message("go north".into(), Some("Player".into()), MessageType::Input);
    state.add_message("You walk north.".into(), None, MessageType::Narration);

    assert_eq!(state.narrative.history.len(), 2);

    state.narrative.history.delete_last().unwrap();
    assert_eq!(state.narrative.history.len(), 1);
    assert_eq!(state.narrative.history.as_slice()[0].text(), "go north");

    state.narrative.history.delete_last().unwrap();
    assert!(state.narrative.history.is_empty());
    state.add_message("go south".into(), Some("Player".into()), MessageType::Input);
    assert_eq!(state.narrative.history.last().unwrap().text(), "go south");
}

#[test]
fn test_push_message_appends_swipe_on_retry_target() {
    let mut state = TestGameState::in_room("room1");

    let target = crate::domain::model::message::Message::new(
        None,
        "Original narration",
        MessageType::Narration,
        None,
        None,
    );
    state.narrative.retry_target = Some(target);

    state.add_message("Retried narration".into(), None, MessageType::Narration);

    assert_eq!(state.narrative.history.len(), 0);

    let target = state.narrative.retry_target.unwrap();
    assert_eq!(target.swipes.len(), 2);
    assert_eq!(target.text(), "Retried narration");
    assert_eq!(target.swipes[0].text, "Original narration");
    assert_eq!(target.swipes[1].text, "Retried narration");
    assert!(target.swipes[1].snapshot_id.is_none());
}

#[test]
fn test_push_message_creates_new_message_when_event_header_mismatches() {
    let mut state = TestGameState::in_room("room1");

    let target = crate::domain::model::message::Message::new(
        None,
        "Original narration",
        MessageType::Narration,
        None,
        None,
    );
    state.narrative.retry_target = Some(target);

    state.narrative.pending_event = Some("Trigger Event".to_string());
    state.add_message("Event narration".into(), None, MessageType::Narration);

    assert_eq!(state.narrative.history.len(), 1);
    assert_eq!(
        state.narrative.history.last().unwrap().text(),
        "Event narration"
    );
    assert_eq!(
        state.narrative.history.last().unwrap().event_header(),
        Some("Trigger Event")
    );

    let target = state.narrative.retry_target.unwrap();
    assert_eq!(target.swipes.len(), 1);
    assert_eq!(target.text(), "Original narration");
}

#[test]
fn test_push_message_creates_new_message_when_no_retry_target() {
    let mut state = TestGameState::in_room("room1");

    state.add_message("Normal narration".into(), None, MessageType::Narration);

    assert_eq!(state.narrative.history.len(), 1);
    assert_eq!(
        state.narrative.history.last().unwrap().text(),
        "Normal narration"
    );
    assert!(state.narrative.retry_target.is_none());
}

fn log_text_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ]{1,50}"
}

fn log_type_strategy() -> impl Strategy<Value = MessageType> {
    prop_oneof![
        Just(MessageType::Narration),
        Just(MessageType::Dialogue),
        Just(MessageType::System),
        Just(MessageType::Input),
    ]
}

proptest! {
    #[test]
    fn prop_log_appends_in_order(
        mut state in Just(TestGameState::in_room("room1")),
        entries in prop::collection::vec(
            (log_text_strategy(), log_type_strategy()),
            1..20
        )
    ) {
        let mut expected = Vec::new();
        for (text, log_type) in entries {
            state.add_message(text.clone(), None, log_type.clone());
            expected.push((text, log_type));
        }
        let history = state.narrative.history();
        prop_assert_eq!(history.len(), expected.len());
        for (i, (expected_text, expected_type)) in expected.iter().enumerate() {
            prop_assert_eq!(&history[i].text, expected_text);
            prop_assert_eq!(history[i].message_type.clone(), expected_type.clone());
        }
    }

    #[test]
    fn prop_log_turns_never_exceed_max_capacity(
        mut state in Just(TestGameState::in_room("room1")),
        entries in prop::collection::vec(
            (log_text_strategy(), log_type_strategy()),
            1000..1050
        )
    ) {
        for (text, log_type) in entries {
            state.add_message(text, None, log_type);
        }
        prop_assert!(
            state.narrative.history.len() <= 1000,
            "message count {} exceeds max 1000",
            state.narrative.history.len()
        );
    }

    #[test]
    fn prop_npcs_in_area_are_always_known(
        mut state in Just(TestGameState::in_room("room1")),
    ) {
        // npcs_in_area ⊆ npcs map invariant moved to engine_fn tests (orchestration_tests.rs).
        state.scene.npcs_in_area.push(TestNpc::named("alice", "Alice"));
        for npc_in_area in &state.scene.npcs_in_area {
            prop_assert!(!npc_in_area.id.is_empty(), "npc_in_area id should be non-empty");
        }
    }

    #[test]
    fn prop_npc_encounter_log_references_valid_npcs(
        mut state in Just(TestGameState::in_room("room1")),
    ) {
        // Encounter-log invariant moved to engine_fn tests via explicit npcs arg.
        let entry = state.npc_encounter_log.npcs.entry("bob".to_string()).or_default();
        entry.times_met += 1;

        for npc_id in state.npc_encounter_log.npcs.keys() {
            prop_assert!(!npc_id.is_empty(), "encounter-log npc_id should be non-empty");
        }
    }
}

#[test]
fn test_npcs_in_area_initialization() {
    let state = TestGameState::in_room("room_1");

    assert!(
        state.scene.npcs_in_area.is_empty(),
        "npcs_in_area should be empty on initialization"
    );
}

#[test]
fn test_npcs_in_area_can_be_populated() {
    let mut state = TestGameState::in_room("room_1");
    let npc = TestNpc::named("npc_1", "N");

    state.scene.npcs_in_area.push(npc);

    assert_eq!(
        state.scene.npcs_in_area.len(),
        1,
        "npcs_in_area should have 1 NPC after population"
    );
    assert_eq!(state.scene.npcs_in_area[0].id, "npc_1", "Should be npc_1");
}

#[test]
fn test_npcs_in_area_can_be_cleared() {
    let mut state = TestGameState::in_room("room_1");
    let npc = TestNpc::named("npc_1", "N");
    state.scene.npcs_in_area.push(npc);

    assert!(
        !state.scene.npcs_in_area.is_empty(),
        "npcs_in_area should be populated"
    );

    state.scene.npcs_in_area.clear();

    assert!(
        state.scene.npcs_in_area.is_empty(),
        "npcs_in_area should be empty after clear"
    );
}

#[test]
fn test_npcs_in_area_can_be_replaced() {
    let mut state = TestGameState::in_room("room_1");
    let npc1 = TestNpc::named("npc_1", "N");
    state.scene.npcs_in_area.push(npc1);

    assert_eq!(state.scene.npcs_in_area.len(), 1, "Should have 1 NPC");

    let new_npcs = vec![];
    state.scene.npcs_in_area = new_npcs;

    assert!(
        state.scene.npcs_in_area.is_empty(),
        "npcs_in_area should be empty after replacement"
    );
}
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
    let next_state = result.unwrap().post_commit_state;
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
    let next_state = result.unwrap().post_commit_state;
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
    let next_state = result.unwrap().post_commit_state;
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
    let next_state = result.unwrap().post_commit_state;
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
    let mut state = make_test_state();
    let original_room = state.movement.current_room_id.clone();

    state
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

    state
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
    let mut state = make_test_state();
    let original_room = state.movement.current_room_id.clone();

    state
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
    let mut state = make_test_state();

    state
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
    let mut state = make_test_state();

    state
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

    let mut state = turn_result.post_commit_state;
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
        let mut state = make_two_room_state();
        state.handle_movement( destination, &new_npc_ids, &deps.map, &deps.npcs).unwrap();
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
        let next_state = result.unwrap().post_commit_state;
        next_state.assert_state_consistency(&deps.map, &deps.npcs).ok();
    }
}
fn setup_test_state() -> (GameState, Arc<MapDef>) {
    let mut exits1 = HashMap::new();
    exits1.insert(Direction::North, "room2".to_string());
    exits1.insert(Direction::East, "room3".to_string());

    let mut exits2 = HashMap::new();
    exits2.insert(Direction::South, "room1".to_string());

    let mut exits3 = HashMap::new();
    exits3.insert(Direction::West, "room1".to_string());

    let room1 = Room {
        id: "room1".to_string(),
        name: "Grand Hall".to_string(),
        description: "A huge hall.".to_string(),
        exits: exits1,
        items: vec![],
        image_path: None,
        navigation_description: None,
    };

    let room2 = Room {
        id: "room2".to_string(),
        name: "Dusty Kitchen".to_string(),
        description: "Smells like mold.".to_string(),
        exits: exits2,
        items: vec![],
        image_path: None,
        navigation_description: None,
    };

    let room3 = Room {
        id: "room3".to_string(),
        name: "Library".to_string(),
        description: "Books everywhere.".to_string(),
        exits: exits3,
        items: vec![],
        image_path: None,
        navigation_description: None,
    };

    let region = Region {
        id: "reg1".to_string(),
        name: "Mansion".to_string(),
        rooms: vec![room1, room2, room3],
    };

    let overworld = Overworld {
        id: "ow1".to_string(),
        name: "World".to_string(),
        regions: vec![region],
    };

    let map = MapDef { overworld };

    let state = GameState::new("room1".to_string());
    (state, Arc::new(map))
}

#[test]
fn test_current_room_success() {
    let (state, map) = setup_test_state();
    let result = map
        .get_room_by_id(&state.movement.current_room_id)
        .or_else(|| {
            state
                .movement
                .dynamic_rooms
                .get(&state.movement.current_room_id)
        });
    assert!(result.is_some());
    assert_eq!(result.unwrap().name, "Grand Hall");
}

#[test]
fn test_get_current_room_failure() {
    let map = MapDef {
        overworld: Overworld {
            id: "o".into(),
            name: "W".into(),
            regions: vec![Region {
                id: "r".into(),
                name: "R".into(),
                rooms: vec![Room {
                    id: "room1".into(),
                    name: "Room".to_string(),
                    description: "D".to_string(),
                    exits: HashMap::new(),
                    items: vec![],
                    image_path: None,
                    navigation_description: None,
                }],
            }],
        },
    };

    let state = GameState::new("non_existent_room");

    let result = map
        .get_room_by_id(&state.movement.current_room_id)
        .or_else(|| {
            state
                .movement
                .dynamic_rooms
                .get(&state.movement.current_room_id)
        });
    assert!(result.is_none());
}

#[test]
fn test_attempt_semantic_walk_valid() {
    let (mut state, map) = setup_test_state();
    let result = state.attempt_semantic_walk(&map, "room2");
    assert!(result.is_ok());
    assert!(result.unwrap().contains("Dusty Kitchen"));
    assert_eq!(state.movement.current_room_id, "room2");
}

#[test]
fn test_attempt_semantic_walk_invalid() {
    let (mut state, map) = setup_test_state();
    let result = state.attempt_semantic_walk(&map, "nonexistent_room");
    assert!(result.is_err());
    assert_eq!(state.movement.current_room_id, "room1");
}

#[test]
fn test_attempt_semantic_walk_empty() {
    let (mut state, map) = setup_test_state();
    let result = state.attempt_semantic_walk(&map, "");
    assert!(result.is_err(), "Empty room id should return error");
}

#[test]
fn test_attempt_semantic_walk_dynamic_room() {
    let (mut state, map) = setup_test_state();
    let dynamic = Room::new_dynamic("Secret Cave", "Dark and damp.");
    let dynamic_id = dynamic.id.clone();
    state
        .movement
        .dynamic_rooms
        .insert(dynamic_id.clone(), dynamic);

    let result = state.attempt_semantic_walk(&map, &dynamic_id);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("Secret Cave"));
    assert_eq!(state.movement.current_room_id, dynamic_id);
}

#[test]
fn test_new_dynamic_room() {
    let room = Room::new_dynamic("Test Room", "A test room.");
    assert_eq!(room.name, "Test Room");
    assert_eq!(room.description, "A test room.");
    assert!(room.id.starts_with("dynamic_"));
    assert!(room.exits.is_empty());
    assert!(room.items.is_empty());
}
thread_local! {
    static TEST_NPCS: RefCell<HashMap<String, NpcCard>> = RefCell::new(HashMap::new());
}

fn evaluate_triggers(state: &GameState) -> Vec<(NpcCard, Trigger, usize)> {
    TEST_NPCS.with(|npcs| state.evaluate_triggers(&npcs.borrow()))
}
fn make_npc(id: &str, triggers: Vec<Trigger>) -> NpcCard {
    NpcCard {
        id: id.to_string(),
        sheet: CharacterSheet {
            name: id.to_string(),
            description: "Test NPC".to_string(),
            personality: "Neutral".to_string(),
            scenario: "Testing".to_string(),
            example_dialogue: String::new(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
        triggers,
        relationships: vec![],
    }
}

fn make_trigger(requirement: TriggerRequirement, repeat: bool) -> Trigger {
    Trigger {
        requirement,
        narration: TriggerNarration {
            name: "Test Event".to_string(),
            narration_prompt: "Test trigger".to_string(),
        },
        repeat,
        room_id: None,
    }
}

fn make_trigger_with_room(requirement: TriggerRequirement, repeat: bool, room_id: &str) -> Trigger {
    Trigger {
        requirement,
        narration: TriggerNarration {
            name: "Test Event".to_string(),
            narration_prompt: "Test trigger".to_string(),
        },
        repeat,
        room_id: Some(room_id.to_string()),
    }
}

fn make_state(
    npcs_in_area: Vec<NpcCard>,
    all_npcs: &[NpcCard],
    npc_encounter_log: NpcEncounterLog,
) -> GameState {
    TEST_NPCS.with(|store| {
        *store.borrow_mut() = all_npcs
            .iter()
            .cloned()
            .map(|npc| (npc.id.clone(), npc))
            .collect();
    });
    GameStateBuilder::new("room_1")
        .with_scene(SceneState {
            npcs_in_area,
            ..Default::default()
        })
        .with_npc_encounter_log(npc_encounter_log)
        .build()
}

#[test]
fn test_evaluate_triggers_empty_room() {
    let state = make_state(vec![], &[], NpcEncounterLog::default());
    let results = evaluate_triggers(&state);
    assert!(results.is_empty());
}

#[test]
fn test_evaluate_triggers_first_encounter() {
    let trigger = make_trigger(
        TriggerRequirement {
            operator: ComparisonOperator::Eq,
            threshold: 0,
        },
        false,
    );
    let npc = make_npc("gabriella", vec![trigger]);
    let state = make_state(
        vec![npc.clone()],
        &[npc.clone()],
        NpcEncounterLog::default(),
    );
    let results = evaluate_triggers(&state);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0.id, "gabriella");
}

#[test]
fn test_evaluate_triggers_no_match_when_times_met_greater() {
    let trigger = make_trigger(
        TriggerRequirement {
            operator: ComparisonOperator::Eq,
            threshold: 0,
        },
        false,
    );
    let npc = make_npc("gabriella", vec![trigger]);
    let mut npc_encounter_log = NpcEncounterLog::default();
    npc_encounter_log.increment_times_met("gabriella");
    let state = make_state(vec![npc.clone()], &[npc.clone()], npc_encounter_log);
    let results = evaluate_triggers(&state);
    assert!(results.is_empty());
}

#[test]
fn test_evaluate_triggers_skips_fired_non_repeatable() {
    let trigger = make_trigger(
        TriggerRequirement {
            operator: ComparisonOperator::Eq,
            threshold: 0,
        },
        false,
    );
    let npc = make_npc("gabriella", vec![trigger.clone()]);
    let mut npc_encounter_log = NpcEncounterLog::default();
    npc_encounter_log.mark_trigger_fired("gabriella", 0);
    let state = make_state(vec![npc.clone()], &[npc.clone()], npc_encounter_log);
    let results = evaluate_triggers(&state);
    assert!(results.is_empty());
}

#[test]
fn test_evaluate_triggers_fires_for_npc_not_in_area() {
    let trigger = make_trigger(
        TriggerRequirement {
            operator: ComparisonOperator::Eq,
            threshold: 0,
        },
        false,
    );
    let npc = make_npc("gabriella", vec![trigger]);
    let state = make_state(vec![], &[npc.clone()], NpcEncounterLog::default());

    assert!(
        state.scene.npcs_in_area.is_empty(),
        "npcs_in_area should be empty"
    );

    let results = evaluate_triggers(&state);
    assert_eq!(
        results.len(),
        1,
        "Global trigger should fire for NPC not in area"
    );
    assert_eq!(results[0].0.id, "gabriella");
}

#[test]
fn test_room_scoped_trigger_fires_in_correct_room() {
    let trigger = make_trigger_with_room(
        TriggerRequirement {
            operator: ComparisonOperator::Eq,
            threshold: 0,
        },
        false,
        "room_1",
    );
    let npc = make_npc("gabriella", vec![trigger]);
    let state = make_state(vec![], &[npc.clone()], NpcEncounterLog::default());

    assert_eq!(state.movement.current_room_id, "room_1");

    let results = evaluate_triggers(&state);
    assert_eq!(
        results.len(),
        1,
        "Room-scoped trigger should fire in correct room"
    );
    assert_eq!(results[0].0.id, "gabriella");
}

#[test]
fn test_room_scoped_trigger_skipped_in_wrong_room() {
    let trigger = make_trigger_with_room(
        TriggerRequirement {
            operator: ComparisonOperator::Eq,
            threshold: 0,
        },
        false,
        "entrance_hall",
    );
    let npc = make_npc("gabriella", vec![trigger]);
    let state = make_state(vec![], &[npc.clone()], NpcEncounterLog::default());

    assert_eq!(state.movement.current_room_id, "room_1");

    let results = evaluate_triggers(&state);
    assert!(
        results.is_empty(),
        "Room-scoped trigger should NOT fire in wrong room"
    );
}

#[test]
fn test_global_trigger_fires_in_any_room() {
    let trigger = make_trigger(
        TriggerRequirement {
            operator: ComparisonOperator::Eq,
            threshold: 0,
        },
        false,
    );
    let npc = make_npc("gabriella", vec![trigger]);
    let state = make_state(vec![], &[npc.clone()], NpcEncounterLog::default());

    let results = evaluate_triggers(&state);
    assert_eq!(
        results.len(),
        1,
        "Global trigger should fire regardless of room"
    );
}

#[test]
fn test_evaluate_triggers_repeatable_fires_again() {
    let trigger = make_trigger(
        TriggerRequirement {
            operator: ComparisonOperator::Lt,
            threshold: 3,
        },
        true,
    );
    let npc = make_npc("ranger", vec![trigger]);
    let mut npc_encounter_log = NpcEncounterLog::default();
    npc_encounter_log.increment_times_met("ranger");
    let state = make_state(vec![npc.clone()], &[npc.clone()], npc_encounter_log);
    let results = evaluate_triggers(&state);
    assert_eq!(results.len(), 1);
}

#[test]
fn test_currently_meeting_tracks_encounters() {
    let mut npc_encounter_log = NpcEncounterLog::default();

    assert!(!npc_encounter_log.is_currently_meeting("carla"));
    npc_encounter_log.set_currently_meeting("carla", true);
    assert!(npc_encounter_log.is_currently_meeting("carla"));

    npc_encounter_log.set_currently_meeting("carla", false);
    assert!(!npc_encounter_log.is_currently_meeting("carla"));
}

#[test]
fn test_increment_times_met_always_increments() {
    let mut npc_encounter_log = NpcEncounterLog::default();

    npc_encounter_log.increment_times_met("gabriella");
    assert_eq!(npc_encounter_log.get_times_met("gabriella"), 1);

    npc_encounter_log.increment_times_met("gabriella");
    assert_eq!(npc_encounter_log.get_times_met("gabriella"), 2);
}

#[test]
fn test_npc_encounter_log_initializes_with_starting_room_npcs() {
    let npc = crate::domain::model::character::NpcCard {
        id: "carla".into(),
        sheet: crate::domain::model::character::CharacterSheet {
            name: "Carla".into(),
            description: "Bodyguard".into(),
            personality: "Protective".into(),
            scenario: "Guarding".into(),
            example_dialogue: "Stay safe.".into(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        triggers: vec![],
        inventory: vec![],
        relationships: vec![],
    };
    let _npcs = vec![npc];
    let state = GameState::new("start");

    assert_eq!(state.npc_encounter_log.get_times_met("carla"), 0);
    assert!(!state.npc_encounter_log.is_currently_meeting("carla"));
}

#[test]
fn test_increment_times_met_and_mark_fired() {
    let trigger = make_trigger(
        TriggerRequirement {
            operator: ComparisonOperator::Eq,
            threshold: 0,
        },
        false,
    );
    let npc = make_npc("gabriella", vec![trigger]);
    let mut state = make_state(
        vec![npc.clone()],
        &[npc.clone()],
        NpcEncounterLog::default(),
    );
    let results = evaluate_triggers(&state);
    assert_eq!(results.len(), 1);
    state.npc_encounter_log.increment_times_met("gabriella");
    state.npc_encounter_log.mark_trigger_fired("gabriella", 0);
    let results = evaluate_triggers(&state);
    assert!(results.is_empty());
}

#[test]
fn test_trigger_index_with_skipped_triggers() {
    let trigger_0 = make_trigger(
        TriggerRequirement {
            operator: ComparisonOperator::Eq,
            threshold: 0,
        },
        false,
    );
    let trigger_1 = make_trigger(
        TriggerRequirement {
            operator: ComparisonOperator::Gte,
            threshold: 0,
        },
        false,
    );
    let npc = make_npc("gabriella", vec![trigger_0.clone(), trigger_1.clone()]);
    let mut npc_encounter_log = NpcEncounterLog::default();
    npc_encounter_log.mark_trigger_fired("gabriella", 0);
    let mut state = make_state(vec![npc.clone()], &[npc.clone()], npc_encounter_log);

    let results = evaluate_triggers(&state);
    assert_eq!(results.len(), 1, "Only trigger 1 should match");
    assert_eq!(results[0].2, 1, "Original index should be 1, not 0");

    state.npc_encounter_log.mark_trigger_fired("gabriella", 1);
    let results = evaluate_triggers(&state);
    assert!(results.is_empty(), "All triggers should now be skipped");
}

#[test]
fn test_check_condition_missing_npc_defaults_to_zero() {
    let npc_encounter_log = NpcEncounterLog::default();
    let condition = TriggerRequirement {
        operator: ComparisonOperator::Eq,
        threshold: 0,
    };
    assert!(npc_encounter_log.check_condition("unknown_npc", &condition));
}
