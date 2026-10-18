//! Tests for `state/game_state.rs` game state, story and builder.

use std::collections::HashMap;
use std::sync::Arc;

use proptest::prelude::*;

use crate::domain::model::character::{CharacterSheet, NpcCard, PersonaCard};
use crate::domain::model::map::{Direction, MapDef, Overworld, Region, Room};
use crate::domain::model::message::Message;
use crate::domain::model::quantifier::{
    MovementParseResult, MovementType, QuantifierConfidence, QuantifierParseResult,
    QuantifierResult,
};
use crate::domain::model::state::game_state::{FreeActionContext, GameState};
use crate::domain::model::state::generation_status::{
    GenerationFailure, GenerationFailureKind, GenerationStatus, InputBuffer,
};
use crate::domain::model::state::message_types::MessageType;
use crate::domain::model::state::narrative_state::ArrivingLocationWrite;
use crate::domain::model::trigger::{ComparisonOperator, NpcEncounterLog, TriggerRequirement};
use crate::test_support::*;

#[test]
fn test_game_state_initialization() {
    let state = TestGameState::in_room("room_1");

    assert_eq!(state.swipe.movement.current_room_id, "room_1");
}

#[test]
fn test_generation_state_status() {
    let mut buf = InputBuffer::default();

    assert_eq!(buf.status, GenerationStatus::Idle);
    assert!(!buf.status.is_generating());
    assert!(buf.status.failure().is_none());

    let failure = GenerationFailure::new(
        GenerationFailureKind::Other,
        "LLM Error: 429 Too Many Requests",
    );
    buf.status = GenerationStatus::Error(failure.clone());
    assert_eq!(buf.status, GenerationStatus::Error(failure));
    assert!(buf.status.failure().is_some());

    buf.status = GenerationStatus::Generating;
    assert!(buf.status.is_generating());
    assert!(buf.status.failure().is_none());
}

#[test]
fn test_log_ordering() {
    let mut state = TestGameState::in_room("room1");

    state.add_message("Message 1".into(), MessageType::Narration);
    state.add_message("Message 2".into(), MessageType::Narration);

    assert_eq!(state.narrative.history().len(), 2);
    assert_eq!(state.narrative.history()[0].text, "Message 1");
    assert_eq!(state.narrative.history()[1].text, "Message 2");
}

#[test]
fn test_delete_last_log_recalculates_ids() {
    let mut state = TestGameState::in_room("room1");

    state.add_message("go north".into(), MessageType::Input);
    state.add_message("You walk north.".into(), MessageType::Narration);

    assert_eq!(state.narrative.history.len(), 2);

    state.narrative.history.delete_last().unwrap();
    assert_eq!(state.narrative.history.len(), 1);
    assert_eq!(state.narrative.history.as_slice()[0].text(), "go north");

    state.narrative.history.delete_last().unwrap();
    assert!(state.narrative.history.is_empty());
    state.add_message("go south".into(), MessageType::Input);
    assert_eq!(state.narrative.history.last().unwrap().text(), "go south");
}

#[test]
fn test_push_message_appends_swipe_on_retry_target() {
    let mut state = TestGameState::in_room("room1");

    let target = crate::domain::model::message::Message::new(
        "Original narration",
        MessageType::Narration,
        None,
        None,
    );
    state.narrative.retry_target = Some(target);

    state.add_message("Retried narration".into(), MessageType::Narration);

    assert_eq!(state.narrative.history.len(), 0);

    let target = state.narrative.retry_target.unwrap();
    assert_eq!(target.swipes.len(), 2);
    assert_eq!(target.text(), "Retried narration");
    assert_eq!(target.swipes[0].text, "Original narration");
    assert_eq!(target.swipes[1].text, "Retried narration");
    assert!(target.swipes[1].snapshot_id.is_none());
}

#[test]
fn test_push_message_inherits_stored_inputs_on_retry_swipe() {
    let mut state = TestGameState::in_room("room1");

    let mut target = crate::domain::model::message::Message::new(
        "Original narration",
        MessageType::Narration,
        None,
        None,
    );
    target.swipes[0].impersonated = true;
    target.swipes[0].impersonate_direction = Some("as the player".to_string());
    state.narrative.retry_target = Some(target);

    state.add_message("Retried narration".into(), MessageType::Narration);

    let target = state.narrative.retry_target.unwrap();
    assert_eq!(target.swipes.len(), 2);
    assert!(target.swipes[1].impersonated);
    assert_eq!(
        target.swipes[1].impersonate_direction.as_deref(),
        Some("as the player")
    );
}

#[test]
fn test_push_message_creates_new_message_when_event_header_mismatches() {
    let mut state = TestGameState::in_room("room1");

    let target = crate::domain::model::message::Message::new(
        "Original narration",
        MessageType::Narration,
        None,
        None,
    );
    state.narrative.retry_target = Some(target);

    state.narrative.pending_event = Some("Trigger Event".to_string());
    state.add_message("Event narration".into(), MessageType::Narration);

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

    state.add_message("Normal narration".into(), MessageType::Narration);

    assert_eq!(state.narrative.history.len(), 1);
    assert_eq!(
        state.narrative.history.last().unwrap().text(),
        "Normal narration"
    );
    assert!(state.narrative.retry_target.is_none());
}

#[test]
fn test_push_message_writes_stored_inputs_on_new_narration_message() {
    let mut state = TestGameState::in_room("room1");

    state.add_message_with_inputs(
        "Guided narration".into(),
        MessageType::Narration,
        false,
        Some("steer toward the cellar".to_string()),
        None,
    );

    let message = state.narrative.history.last().unwrap();
    assert_eq!(message.text(), "Guided narration");
    assert!(message.is_guided());
    assert_eq!(message.guide_text(), Some("steer toward the cellar"));
}

#[test]
fn test_push_message_no_stored_inputs_leaves_swipe_inputs_empty() {
    let mut state = TestGameState::in_room("room1");

    state.add_message("Normal narration".into(), MessageType::Narration);

    let message = state.narrative.history.last().unwrap();
    assert!(
        !message.impersonated()
            && message.guide_text().is_none()
            && message.impersonate_direction().is_none(),
        "swipe stays plain without stored inputs"
    );
}

#[test]
fn test_push_message_writes_impersonate_inputs_on_player_voiced_input() {
    let mut state = TestGameState::in_room("room1");

    state.add_message_with_inputs(
        "I ask about the artifact.".into(),
        MessageType::Input,
        true,
        None,
        Some("ask about the artifact".to_string()),
    );

    let message = state.narrative.history.last().unwrap();
    assert_eq!(message.text(), "I ask about the artifact.");
    assert_eq!(message.message_type, MessageType::Input);
    assert!(message.impersonated());
    assert_eq!(
        message.impersonate_direction(),
        Some("ask about the artifact")
    );
}

fn log_text_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ]{1,50}"
}

fn log_type_strategy() -> impl Strategy<Value = MessageType> {
    prop_oneof![
        Just(MessageType::Narration),
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
            state.add_message(text.clone(), log_type.clone());
            expected.push((text, log_type));
        }
        let history = state.narrative.history();
        prop_assert_eq!(history.len(), expected.len());
        for (i, (expected_text, expected_type)) in expected.iter().enumerate() {
            prop_assert_eq!(&history[i].text, expected_text);
            prop_assert_eq!(history[i].message_type.clone(), expected_type.clone());
        }
    }
}

// The cap is one boundary at 1000 entries, so a handful of cases covers it.
// `MessageHistory::append` drops the oldest via `Vec::remove(0)` (O(n)), which
// makes each oversized case costly; the 1000..1050 overshoot still straddles
// the cap.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(4))]

    #[test]
    fn prop_log_turns_never_exceed_max_capacity(
        mut state in Just(TestGameState::in_room("room1")),
        entries in prop::collection::vec(
            (log_text_strategy(), log_type_strategy()),
            1000..1050
        )
    ) {
        for (text, log_type) in entries {
            state.add_message(text, log_type);
        }
        prop_assert!(
            state.narrative.history.len() <= 1000,
            "message count {} exceeds max 1000",
            state.narrative.history.len()
        );
    }
}

#[test]
fn test_npcs_in_area_initialization() {
    let state = TestGameState::in_room("room_1");

    assert!(
        state.swipe.scene.npcs_in_area.is_empty(),
        "npcs_in_area should be empty on initialization"
    );
}

#[test]
fn test_npcs_in_area_can_be_populated() {
    let mut state = TestGameState::in_room("room_1");
    let npc = TestNpc::named("npc_1", "N");

    state.swipe.scene.npcs_in_area.push(npc);

    assert_eq!(
        state.swipe.scene.npcs_in_area.len(),
        1,
        "npcs_in_area should have 1 NPC after population"
    );
    assert_eq!(
        state.swipe.scene.npcs_in_area[0].id, "npc_1",
        "Should be npc_1"
    );
}

#[test]
fn test_npcs_in_area_can_be_cleared() {
    let mut state = TestGameState::in_room("room_1");
    let npc = TestNpc::named("npc_1", "N");
    state.swipe.scene.npcs_in_area.push(npc);

    assert!(
        !state.swipe.scene.npcs_in_area.is_empty(),
        "npcs_in_area should be populated"
    );

    state.swipe.scene.npcs_in_area.clear();

    assert!(
        state.swipe.scene.npcs_in_area.is_empty(),
        "npcs_in_area should be empty after clear"
    );
}

#[test]
fn test_npcs_in_area_can_be_replaced() {
    let mut state = TestGameState::in_room("room_1");
    let npc1 = TestNpc::named("npc_1", "N");
    state.swipe.scene.npcs_in_area.push(npc1);

    assert_eq!(state.swipe.scene.npcs_in_area.len(), 1, "Should have 1 NPC");

    let new_npcs = vec![];
    state.swipe.scene.npcs_in_area = new_npcs;

    assert!(
        state.swipe.scene.npcs_in_area.is_empty(),
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

fn make_quantifier_result_moving_to(destination: &str, npc_ids: &[&str]) -> QuantifierResult {
    QuantifierResult {
        npcs: QuantifierParseResult {
            npc_ids: npc_ids.iter().map(|id| id.to_string()).collect(),
            confidence: QuantifierConfidence::High,
        },
        movement: MovementParseResult {
            movement_type: Some(MovementType::Entering),
            destination: Some(destination.to_string()),
            confidence: QuantifierConfidence::High,
        },
    }
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

fn make_test_state() -> GameState {
    TestGameState::in_room("test_room")
}

#[test]
fn test_execute_freeaction_impl_puts_the_room_name_on_the_arriving_narration() {
    let deps = deps_with_carla("test_room");
    let mut state = make_test_state();
    state.add_message("You step inside.".to_string(), MessageType::Narration);

    let result = state
        .execute_freeaction_impl(
            &FreeActionContext {
                narration_text: "You step inside.",
                quantifier_result: &make_quantifier_result_moving_to("test_room", &["carla"]),
            },
            &deps.map,
            &deps.persona,
            &deps.npcs,
        )
        .unwrap();
    let mut state = result.post_commit_state;

    assert_eq!(
        state
            .narrative
            .history
            .last()
            .and_then(Message::room_report),
        Some("Test Room"),
        "the Narration that arrived in the room must report it"
    );

    state.add_message("look".to_string(), MessageType::Input);
    assert_eq!(
        state
            .narrative
            .history
            .last()
            .and_then(Message::room_report),
        None,
        "the Message after the movement must not take the room name"
    );
}

#[test]
fn test_execute_freeaction_impl_puts_the_room_name_on_the_redo_swipe() {
    let deps = deps_with_carla("test_room");
    let mut state = make_test_state();
    state.add_message("An older narration.".to_string(), MessageType::Narration);

    let mut retried = Message::new(
        "The narration it retries.",
        MessageType::Narration,
        None,
        None,
    );
    retried.swipes.push(dummy_swipe("The redo."));
    retried.set_active_swipe(1);
    state.narrative.retry_target = Some(retried);

    let result = state
        .execute_freeaction_impl(
            &FreeActionContext {
                narration_text: "The redo.",
                quantifier_result: &make_quantifier_result_moving_to("test_room", &["carla"]),
            },
            &deps.map,
            &deps.persona,
            &deps.npcs,
        )
        .unwrap();
    let state = result.post_commit_state;

    let retried = state
        .narrative
        .retry_target
        .as_ref()
        .expect("the redo keeps the Message it retries");
    assert_eq!(
        retried.room_report(),
        Some("Test Room"),
        "the redo's Swipe carries this generation's Narration"
    );
    assert_eq!(
        state
            .narrative
            .history
            .last()
            .and_then(Message::room_report),
        None,
        "an older Message must not be relabelled"
    );
}

#[test]
fn test_execute_freeaction_impl_queues_the_arriving_location_write_for_a_saved_message() {
    let deps = deps_with_carla("test_room");
    let mut state = make_test_state();
    let mut saved = Message::new("You step inside.", MessageType::Narration, None, None);
    saved.id = 7;
    saved.swipes.push(dummy_swipe("A second swipe."));
    saved.set_active_swipe(1);
    state.narrative.history.append(saved);
    state.narrative.pending_location = Some("The room before".to_string());

    let result = state
        .execute_freeaction_impl(
            &FreeActionContext {
                narration_text: "You step inside.",
                quantifier_result: &make_quantifier_result_moving_to("test_room", &["carla"]),
            },
            &deps.map,
            &deps.persona,
            &deps.npcs,
        )
        .unwrap();
    let state = result.post_commit_state;

    assert_eq!(
        state
            .narrative
            .history
            .last()
            .and_then(Message::room_report),
        Some("Test Room"),
        "the saved Narration reports the room it arrives in"
    );
    assert_eq!(
        state.narrative.arriving_location_write,
        Some(ArrivingLocationWrite {
            message_id: 7,
            swipe_index: 1,
            room_name: "Test Room".to_string(),
        }),
        "the Swipe row already exists, so the next save writes it as an update"
    );
    assert_eq!(
        state.narrative.pending_location, None,
        "the arriving Narration owns the room name, not the next Message"
    );
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
    state.add_message("You enter the room.".to_string(), MessageType::Narration);

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
fn test_a_failed_trigger_consistency_check_keeps_the_event_narration() {
    let deps = deps_with_carla("test_room");
    let mut state = make_test_state();
    // The Trigger names a Character the state does not hold, so the check fails.
    let request = crate::test_support::TestStoredTriggerContext::for_npc(
        "unknown_npc",
        "Carla Introduction",
        "Carla appears",
    );

    let result = state.commit_trigger_narration(&request, "Carla emerges.", &deps.map, &deps.npcs);

    assert!(result.is_err());
    assert_eq!(
        state.narrative.history().len(),
        1,
        "the event Narration landed before the check ran"
    );
    assert_eq!(state.narrative.history()[0].text, "Carla emerges.");
    assert_eq!(
        state
            .narrative
            .last_trigger
            .as_ref()
            .map(|t| t.npc_id.as_str()),
        Some("unknown_npc")
    );
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
        .get_room_by_id(&state.swipe.movement.current_room_id)
        .or_else(|| {
            state
                .swipe
                .movement
                .dynamic_rooms
                .get(&state.swipe.movement.current_room_id)
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
        .get_room_by_id(&state.swipe.movement.current_room_id)
        .or_else(|| {
            state
                .swipe
                .movement
                .dynamic_rooms
                .get(&state.swipe.movement.current_room_id)
        });
    assert!(result.is_none());
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

    assert_eq!(state.swipe.npc_encounter_log.get_times_met("carla"), 0);
    assert!(!state.swipe.npc_encounter_log.is_currently_meeting("carla"));
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

#[test]
fn test_execute_freeaction_impl_adds_a_room_the_map_does_not_know() {
    let state = create_test_state();
    let invalid_destination = "nonexistent_place_xyz";

    let map = Arc::new(create_test_map());
    let player = Arc::new(create_test_player());
    let test_npc = NpcCard {
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
        triggers: vec![],
        relationships: vec![],
    };
    let npcs = npc_map(vec![test_npc]);

    assert!(
        !map.overworld
            .regions
            .iter()
            .flat_map(|r| r.rooms.iter())
            .any(|r| r.id == invalid_destination),
        "precondition: invalid_destination should not exist in map"
    );

    let result = state
        .execute_freeaction_impl(
            &FreeActionContext {
                narration_text: "I walk there.",
                quantifier_result: &make_quantifier_result_moving_to(invalid_destination, &[]),
            },
            &map,
            &player,
            &npcs,
        )
        .unwrap();
    let state = result.post_commit_state;
    assert!(
        !state.swipe.movement.dynamic_rooms.is_empty(),
        "dynamic room should be created for invalid destination"
    );
    assert!(
        state.swipe.movement.current_room_id.starts_with("dynamic_"),
        "current_room_id should be set to a dynamic room"
    );
    let history = state.narrative.history();
    let system_messages: Vec<_> = history
        .iter()
        .filter(|m| m.message_type == MessageType::System)
        .collect();
    assert!(
        !system_messages.is_empty(),
        "system message should be logged for dynamic room creation"
    );
    assert!(
        system_messages
            .iter()
            .any(|m| m.text.contains("Entered unknown location")),
        "system message should mention 'Entered unknown location'"
    );
}
