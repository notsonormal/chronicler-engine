use std::cell::RefCell;
use std::collections::HashMap;

use crate::domain::engine::trigger_eval::{
    check_condition, evaluate_triggers as evaluate_triggers_with_npcs,
};
use crate::domain::model::character::{CharacterSheet, NpcCard};
use crate::domain::model::state::game_state::{GameState, GameStateBuilder};
use crate::domain::model::state::scene_state::SceneState;
use crate::domain::model::trigger::{
    ComparisonOperator, NpcEncounterLog, Trigger, TriggerNarration, TriggerRequirement,
};

thread_local! {
    static TEST_NPCS: RefCell<HashMap<String, NpcCard>> = RefCell::new(HashMap::new());
}

fn evaluate_triggers(state: &GameState) -> Vec<(NpcCard, Trigger, usize)> {
    TEST_NPCS.with(|npcs| evaluate_triggers_with_npcs(state, &npcs.borrow()))
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
    assert!(check_condition(
        &npc_encounter_log,
        "unknown_npc",
        &condition
    ));
}
