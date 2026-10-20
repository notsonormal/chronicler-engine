//! The rules of a starting state.

use std::collections::HashMap;

use crate::domain::model::character::NpcCard;
use crate::domain::model::scenario::StartingScenario;
use crate::test_support::TestNpc;

use super::swipe_state::SwipeState;

fn scenario_with(npcs: &[&str]) -> StartingScenario {
    StartingScenario {
        id: "start".to_string(),
        name: "Start".to_string(),
        description: String::new(),
        starting_room_id: "room1".to_string(),
        text: "The gate opens.".to_string(),
        npcs: npcs.iter().map(|id| id.to_string()).collect(),
    }
}

fn characters(ids: &[&str]) -> HashMap<String, NpcCard> {
    ids.iter()
        .map(|id| (id.to_string(), TestNpc::named(id, "Carla")))
        .collect()
}

#[test]
fn starting_state_holds_the_starting_room_and_no_added_rooms() {
    let state = SwipeState::starting(Some(&scenario_with(&[])), &characters(&[]), "room1");

    assert_eq!(state.movement.current_room_id, "room1");
    assert!(state.movement.dynamic_rooms.is_empty());
    assert!(state.trigger_context.is_none());
}

#[test]
fn starting_state_marks_each_starting_character_met_once_and_met_now() {
    let scenario = scenario_with(&["npc_1", "npc_2"]);

    let state = SwipeState::starting(Some(&scenario), &characters(&["npc_1", "npc_2"]), "room1");

    assert_eq!(state.scene.npcs_in_area.len(), 2);
    for id in ["npc_1", "npc_2"] {
        assert_eq!(state.npc_encounter_log.get_times_met(id), 1, "{id}");
        assert!(
            state.npc_encounter_log.npcs[id].currently_meeting,
            "{id} is met now"
        );
    }
}

#[test]
fn starting_state_without_a_scenario_holds_no_character() {
    let state = SwipeState::starting(None, &characters(&["npc_1"]), "room1");

    assert!(state.scene.npcs_in_area.is_empty());
    assert_eq!(state.npc_encounter_log.get_times_met("npc_1"), 0);
}

fn event_record(repeat: bool) -> crate::domain::model::state::trigger_context::EventRecord {
    crate::domain::model::state::trigger_context::EventRecord {
        trigger: crate::domain::model::state::trigger_context::TriggerRef {
            npc_id: "npc_1".to_string(),
            trigger_idx: 0,
            trigger_name: "The gate opens".to_string(),
            trigger_repeat: repeat,
        },
        header_name: "The gate opens".to_string(),
        direction: "the gate swings open".to_string(),
    }
}

#[test]
fn pre_analysis_clears_the_trigger_context() {
    let mut state = SwipeState::starting(Some(&scenario_with(&[])), &characters(&[]), "room1");
    state.trigger_context = Some(crate::domain::model::state::trigger_context::TriggerRef {
        npc_id: "npc_1".to_string(),
        trigger_idx: 0,
        trigger_name: "The gate opens".to_string(),
        trigger_repeat: false,
    });

    let pre = state.pre_analysis(None);

    assert!(pre.trigger_context.is_none());
    assert_eq!(pre.movement.current_room_id, "room1");
}

#[test]
fn pre_analysis_of_an_event_marks_a_one_shot_trigger_fired() {
    let state = SwipeState::starting(Some(&scenario_with(&[])), &characters(&[]), "room1");

    let pre = state.pre_analysis(Some(&event_record(false)));

    assert!(pre.npc_encounter_log.is_trigger_fired("npc_1", 0));
}

#[test]
fn pre_analysis_of_a_repeatable_trigger_marks_nothing() {
    let state = SwipeState::starting(Some(&scenario_with(&[])), &characters(&[]), "room1");

    let pre = state.pre_analysis(Some(&event_record(true)));

    assert!(!pre.npc_encounter_log.is_trigger_fired("npc_1", 0));
}
