//! The rules of a starting state.

use std::collections::HashMap;

use crate::domain::model::character::NpcCard;
use crate::domain::model::map::Room;
use crate::domain::model::scenario::StartingScenario;
use crate::test_support::{TestMap, TestNpc};

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

#[test]
fn room_added_room_finds_a_map_room_else_an_added_room_else_none() {
    let map = TestMap::single_room("room1");
    let mut state = SwipeState::starting(None, &characters(&[]), "room1");
    assert_eq!(state.room(&map).map(|room| room.id.as_str()), Some("room1"));

    let added = Room::new_dynamic("Misty Glade", "A glade.");
    state.movement.current_room_id = added.id.clone();
    assert!(state.room(&map).is_none(), "an unknown room is none");

    state
        .movement
        .dynamic_rooms
        .insert(added.id.clone(), added.clone());
    assert_eq!(
        state.room(&map).map(|room| room.name.as_str()),
        Some("Misty Glade")
    );
}
