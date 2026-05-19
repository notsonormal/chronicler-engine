use std::collections::HashMap;
use std::sync::Arc;

use crate::engine::trigger_eval::{
    check_condition, evaluate_triggers, get_times_met, increment_times_met, is_currently_meeting,
    mark_trigger_fired, set_currently_meeting,
};
use crate::model::character::{CharacterSheet, NpcCard};
use crate::model::map::{MapDef, Overworld};
use crate::model::state::GameState;
use crate::model::trigger::{
    ComparisonOperator, NpcEncounterLog, Trigger, TriggerCondition, TriggerEffect,
};
use crate::model::world::WorldCard;

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

fn make_trigger(condition: TriggerCondition, repeat: bool) -> Trigger {
    Trigger {
        condition,
        effect: TriggerEffect {
            name: "Test Event".to_string(),
            narration_prompt: "Test trigger".to_string(),
        },
        repeat,
        room_id: None,
    }
}

fn make_trigger_with_room(condition: TriggerCondition, repeat: bool, room_id: &str) -> Trigger {
    Trigger {
        condition,
        effect: TriggerEffect {
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
    let world = Arc::new(WorldCard {
        name: "Test".into(),
        description: "Test world".into(),
        global_rules: vec![],
        ..Default::default()
    });
    let map = Arc::new(MapDef {
        overworld: Overworld {
            id: "ow".into(),
            name: "ow".into(),
            regions: vec![],
        },
    });
    let player = Arc::new(crate::model::character::PlayerCard {
        sheet: CharacterSheet {
            name: "Player".into(),
            description: "Test player".into(),
            personality: "Brave".to_string(),
            scenario: "Testing".to_string(),
            example_dialogue: String::new(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
    });
    let npcs: Vec<NpcCard> = all_npcs.to_vec();
    crate::model::state::GameStateBuilder::new(world, map, player, "room_1")
        .with_npcs(npcs)
        .with_scene(crate::model::state::SceneState {
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
    let trigger = make_trigger(TriggerCondition::TimesMet(ComparisonOperator::Eq, 0), false);
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
    let trigger = make_trigger(TriggerCondition::TimesMet(ComparisonOperator::Eq, 0), false);
    let npc = make_npc("gabriella", vec![trigger]);
    let mut npc_encounter_log = NpcEncounterLog::default();
    increment_times_met(&mut npc_encounter_log, "gabriella");
    let state = make_state(vec![npc.clone()], &[npc.clone()], npc_encounter_log);
    let results = evaluate_triggers(&state);
    assert!(results.is_empty());
}

#[test]
fn test_evaluate_triggers_skips_fired_non_repeatable() {
    let trigger = make_trigger(TriggerCondition::TimesMet(ComparisonOperator::Eq, 0), false);
    let npc = make_npc("gabriella", vec![trigger.clone()]);
    let mut npc_encounter_log = NpcEncounterLog::default();
    mark_trigger_fired(&mut npc_encounter_log, "gabriella", 0);
    let state = make_state(vec![npc.clone()], &[npc.clone()], npc_encounter_log);
    let results = evaluate_triggers(&state);
    assert!(results.is_empty());
}

#[test]
fn test_evaluate_triggers_fires_for_npc_not_in_area() {
    // [DOC: docs/architecture/system.md]
    let trigger = make_trigger(TriggerCondition::TimesMet(ComparisonOperator::Eq, 0), false);
    let npc = make_npc("gabriella", vec![trigger]);
    // npcs_in_area is empty, but Gabriella IS in state.npcs
    let state = make_state(vec![], &[npc.clone()], NpcEncounterLog::default());

    // Verify Gabriella IS in state.npcs but NOT in npcs_in_area
    assert!(
        state.npcs.contains_key("gabriella"),
        "Gabriella should be in state.npcs"
    );
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
        TriggerCondition::TimesMet(ComparisonOperator::Eq, 0),
        false,
        "room_1",
    );
    let npc = make_npc("gabriella", vec![trigger]);
    let state = make_state(vec![], &[npc.clone()], NpcEncounterLog::default());

    // State is in room_1 by default
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
        TriggerCondition::TimesMet(ComparisonOperator::Eq, 0),
        false,
        "entrance_hall",
    );
    let npc = make_npc("gabriella", vec![trigger]);
    let state = make_state(vec![], &[npc.clone()], NpcEncounterLog::default());

    // State is in room_1, but trigger is scoped to entrance_hall
    assert_eq!(state.movement.current_room_id, "room_1");

    let results = evaluate_triggers(&state);
    assert!(
        results.is_empty(),
        "Room-scoped trigger should NOT fire in wrong room"
    );
}

#[test]
fn test_global_trigger_fires_in_any_room() {
    let trigger = make_trigger(TriggerCondition::TimesMet(ComparisonOperator::Eq, 0), false);
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
    let trigger = make_trigger(TriggerCondition::TimesMet(ComparisonOperator::Lt, 3), true);
    let npc = make_npc("ranger", vec![trigger]);
    let mut npc_encounter_log = NpcEncounterLog::default();
    increment_times_met(&mut npc_encounter_log, "ranger");
    let state = make_state(vec![npc.clone()], &[npc.clone()], npc_encounter_log);
    let results = evaluate_triggers(&state);
    assert_eq!(results.len(), 1);
}

#[test]
fn test_currently_meeting_tracks_encounters() {
    let mut npc_encounter_log = NpcEncounterLog::default();

    assert!(!is_currently_meeting(&npc_encounter_log, "carla"));
    set_currently_meeting(&mut npc_encounter_log, "carla", true);
    assert!(is_currently_meeting(&npc_encounter_log, "carla"));

    // End meeting (player leaves room)
    set_currently_meeting(&mut npc_encounter_log, "carla", false);
    assert!(!is_currently_meeting(&npc_encounter_log, "carla"));
}

#[test]
fn test_increment_times_met_always_increments() {
    // increment_times_met always increments - the guard is in handle_movement
    let mut npc_encounter_log = NpcEncounterLog::default();

    increment_times_met(&mut npc_encounter_log, "gabriella");
    assert_eq!(get_times_met(&npc_encounter_log, "gabriella"), 1);

    increment_times_met(&mut npc_encounter_log, "gabriella");
    assert_eq!(get_times_met(&npc_encounter_log, "gabriella"), 2);
}

#[test]
fn test_npc_encounter_log_initializes_with_starting_room_npcs() {
    use crate::model::character::NpcCard;
    use crate::model::map::{MapDef, Overworld, Region, Room};
    use crate::model::world::WorldCard;
    use std::sync::Arc;

    let world = Arc::new(WorldCard {
        name: "Test".into(),
        description: "Test".into(),
        ..Default::default()
    });

    let room = Room {
        id: "start".into(),
        name: "Start".into(),
        description: "A room".into(),
        exits: HashMap::new(),
        items: vec![],
        image_path: None,
        navigation_description: None,
    };
    let region = Region {
        id: "reg".into(),
        name: "Region".into(),
        rooms: vec![room],
    };
    let map = Arc::new(MapDef {
        overworld: Overworld {
            id: "ow".into(),
            name: "Overworld".into(),
            regions: vec![region],
        },
    });

    let npc = NpcCard {
        id: "carla".into(),
        sheet: crate::model::character::CharacterSheet {
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
    let npcs = vec![npc];

    let state = crate::model::state::GameState::new(
        world,
        map,
        Arc::new(crate::model::character::PlayerCard {
            sheet: crate::model::character::CharacterSheet {
                name: "Player".into(),
                description: "".into(),
                personality: "Brave".into(),
                scenario: "Test".into(),
                example_dialogue: "".into(),
                summary: None,
                profile_image: None,
                headshot_image: None,
            },
            inventory: vec![],
        }),
        npcs,
        "start".into(),
    );

    // Carla should NOT be auto-initialised by GameState::new anymore
    // (room.npcs was removed; scenario-driven init happens in bootstrap/run.rs)
    assert_eq!(get_times_met(&state.npc_encounter_log, "carla"), 0);
    assert!(!is_currently_meeting(&state.npc_encounter_log, "carla"));
}

#[test]
fn test_increment_times_met_and_mark_fired() {
    let trigger = make_trigger(TriggerCondition::TimesMet(ComparisonOperator::Eq, 0), false);
    let npc = make_npc("gabriella", vec![trigger]);
    let mut state = make_state(
        vec![npc.clone()],
        &[npc.clone()],
        NpcEncounterLog::default(),
    );
    let results = evaluate_triggers(&state);
    assert_eq!(results.len(), 1);
    increment_times_met(&mut state.npc_encounter_log, "gabriella");
    mark_trigger_fired(&mut state.npc_encounter_log, "gabriella", 0);
    let results = evaluate_triggers(&state);
    assert!(results.is_empty());
}

#[test]
fn test_trigger_index_with_skipped_triggers() {
    // NPC with 2 triggers: trigger 0 (already fired), trigger 1 (not fired).
    // evaluate_triggers should return trigger 1 with original index 1.
    let trigger_0 = make_trigger(TriggerCondition::TimesMet(ComparisonOperator::Eq, 0), false);
    let trigger_1 = make_trigger(
        TriggerCondition::TimesMet(ComparisonOperator::Gte, 0),
        false,
    );
    let npc = make_npc("gabriella", vec![trigger_0.clone(), trigger_1.clone()]);
    let mut npc_encounter_log = NpcEncounterLog::default();
    mark_trigger_fired(&mut npc_encounter_log, "gabriella", 0);
    let mut state = make_state(vec![npc.clone()], &[npc.clone()], npc_encounter_log);

    let results = evaluate_triggers(&state);
    assert_eq!(results.len(), 1, "Only trigger 1 should match");
    assert_eq!(results[0].2, 1, "Original index should be 1, not 0");

    // After marking trigger 1 as fired, no triggers should match
    mark_trigger_fired(&mut state.npc_encounter_log, "gabriella", 1);
    let results = evaluate_triggers(&state);
    assert!(results.is_empty(), "All triggers should now be skipped");
}

#[test]
fn test_check_condition_missing_npc_defaults_to_zero() {
    let npc_encounter_log = NpcEncounterLog::default();
    let condition = TriggerCondition::TimesMet(ComparisonOperator::Eq, 0);
    assert!(check_condition(
        &npc_encounter_log,
        "unknown_npc",
        &condition
    ));
}
