//! Tests for `utils/game_rules.rs` game rules.

use std::collections::HashMap;
use std::sync::Arc;

use proptest::prelude::*;

use crate::domain::model::state::trigger_context::TriggerRef;
use crate::domain::model::character::{CharacterSheet, NpcCard, PersonaCard};
use crate::domain::model::map::{MapDef, Room};
use crate::domain::model::quantifier::{
    MovementParseResult, MovementType, NpcEvent, NpcTransitionType, QuantifierConfidence,
    QuantifierParseResult, QuantifierResult,
};
use crate::domain::model::state::movement::MovementState;
use crate::domain::model::state::scene_state::SceneState;
use crate::domain::model::state::swipe_state::SwipeState;
use crate::domain::model::trigger::{
    ComparisonOperator, NpcEncounterLog, Trigger, TriggerNarration, TriggerRequirement,
};
use crate::domain::model::utils::game_rules::{
    apply_game_rules, GameRuleRequest, GameRulesOutcome, TriggerMatch,
};
use crate::test_support::*;

struct RulesDeps {
    map: Arc<MapDef>,
    persona: Arc<PersonaCard>,
    npcs: HashMap<String, NpcCard>,
}

fn rules_deps(map: MapDef, npcs: Vec<NpcCard>) -> RulesDeps {
    RulesDeps {
        map: Arc::new(map),
        persona: Arc::new(TestPersona::standard()),
        npcs: npcs.into_iter().map(|npc| (npc.id.clone(), npc)).collect(),
    }
}

fn deps_with_carla(room_id: &str) -> RulesDeps {
    rules_deps(
        TestMap::single_room(room_id),
        vec![TestNpc::named("carla", "Carla")],
    )
}

fn swipe_in(room_id: &str) -> SwipeState {
    swipe_with(room_id, vec![], NpcEncounterLog::default())
}

fn swipe_with(room_id: &str, present: Vec<NpcCard>, log: NpcEncounterLog) -> SwipeState {
    SwipeState {
        trigger_context: None,
        movement: MovementState {
            current_room_id: room_id.to_string(),
            dynamic_rooms: HashMap::new(),
        },
        scene: SceneState {
            npcs_in_area: present,
            ..Default::default()
        },
        npc_encounter_log: log,
    }
}

fn log_of(npc_id: &str, times_met: u32, currently_meeting: bool) -> NpcEncounterLog {
    let mut log = NpcEncounterLog::default();
    if times_met > 0 || currently_meeting {
        let entry = log.npcs.entry(npc_id.to_string()).or_default();
        entry.times_met = times_met;
        entry.currently_meeting = currently_meeting;
    }
    log
}

fn quantifier(npc_ids: &[&str], destination: Option<&str>) -> QuantifierResult {
    QuantifierResult {
        npcs: QuantifierParseResult {
            npc_ids: npc_ids.iter().map(|id| id.to_string()).collect(),
            confidence: QuantifierConfidence::High,
        },
        movement: match destination {
            Some(destination) => MovementParseResult {
                movement_type: Some(MovementType::Entering),
                destination: Some(destination.to_string()),
                confidence: QuantifierConfidence::High,
            },
            None => MovementParseResult::default(),
        },
    }
}

fn rule_step(
    swipe: &mut SwipeState,
    quantifier_result: &QuantifierResult,
    deps: &RulesDeps,
) -> Result<GameRulesOutcome, crate::error::EngineError> {
    apply_game_rules(
        swipe,
        GameRuleRequest::Analysis {
            quantifier_result,
            persona_name: &deps.persona.sheet.name,
        },
        &deps.map,
        &deps.npcs,
    )
}

fn trigger_step(
    swipe: &mut SwipeState,
    trigger: &TriggerRef,
    text: &str,
    deps: &RulesDeps,
) -> GameRulesOutcome {
    apply_game_rules(
        swipe,
        GameRuleRequest::TriggerNarration { trigger, text },
        &deps.map,
        &deps.npcs,
    )
    .expect("a Trigger narration rule step cannot fail")
}

fn trigger_ref(npc_id: &str, trigger_name: &str, repeat: bool) -> TriggerRef {
    TriggerRef {
        npc_id: npc_id.to_string(),
        trigger_idx: 0,
        trigger_name: trigger_name.to_string(),
        trigger_repeat: repeat,
    }
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

fn requirement(operator: ComparisonOperator, threshold: u32) -> TriggerRequirement {
    TriggerRequirement {
        operator,
        threshold,
    }
}

/// One generated Trigger, and whether the encounter log already holds it fired.
fn trigger_shapes() -> impl Strategy<Value = (Trigger, bool)> {
    (
        any::<bool>(),
        any::<bool>(),
        prop_oneof![Just(None), Just(Some("room_1")), Just(Some("room_2"))],
        prop_oneof![
            Just((ComparisonOperator::Eq, 0u32)),
            Just((ComparisonOperator::Gte, 0u32)),
            Just((ComparisonOperator::Lt, 3u32)),
        ],
    )
        .prop_map(|(repeat, fired, room_id, (operator, threshold))| {
            (
                Trigger {
                    requirement: requirement(operator, threshold),
                    narration: TriggerNarration {
                        name: "Test Event".to_string(),
                        narration_prompt: "Test trigger".to_string(),
                    },
                    repeat,
                    room_id: room_id.map(|room_id| room_id.to_string()),
                },
                fired,
            )
        })
}

struct TriggerCase {
    deps: RulesDeps,
    swipe: SwipeState,
}

fn trigger_case(
    trigger: Trigger,
    fired: bool,
    present_before: bool,
    times_met: u32,
) -> TriggerCase {
    let npc = make_npc("gabriella", vec![trigger]);
    let deps = rules_deps(TestMap::two_rooms("room_1", "room_2"), vec![npc.clone()]);

    let mut log = log_of("gabriella", times_met, present_before);
    if fired {
        log.mark_trigger_fired("gabriella", 0);
    }
    let present = if present_before { vec![npc] } else { vec![] };

    TriggerCase {
        deps,
        swipe: swipe_with("room_1", present, log),
    }
}

fn matched(swipe: &mut SwipeState, deps: &RulesDeps, named: &[&str]) -> Option<TriggerMatch> {
    rule_step(swipe, &quantifier(named, None), deps)
        .expect("the rule step keeps the state consistent")
        .trigger_match
}

#[test]
fn test_the_step_without_a_movement_keeps_the_room_and_reports_no_arrival() {
    let deps = deps_with_carla("room1");
    let mut swipe = swipe_in("room1");

    let outcome = rule_step(&mut swipe, &quantifier(&["carla"], None), &deps).unwrap();

    assert_eq!(swipe.movement.current_room_id, "room1");
    assert!(
        outcome.arrived_room_name.is_none(),
        "no arrival reports no room"
    );
    assert!(outcome.unknown_location.is_none());
}

#[test]
fn test_the_step_sets_the_characters_it_names() {
    let deps = deps_with_carla("room1");
    let mut swipe = swipe_in("room1");

    rule_step(&mut swipe, &quantifier(&["carla"], None), &deps).unwrap();

    assert_eq!(swipe.scene.npcs_in_area.len(), 1);
    assert_eq!(swipe.scene.npcs_in_area[0].id, "carla");
    assert!(swipe.npc_encounter_log.is_currently_meeting("carla"));
    assert_eq!(swipe.npc_encounter_log.get_times_met("carla"), 1);
}

#[test]
fn test_the_step_walks_into_a_room_of_the_map() {
    let deps = rules_deps(
        TestMap::two_rooms("room1", "room2"),
        vec![TestNpc::named("carla", "Carla")],
    );
    let mut swipe = swipe_in("room1");

    let outcome = rule_step(&mut swipe, &quantifier(&["carla"], Some("room2")), &deps).unwrap();

    assert_eq!(swipe.movement.current_room_id, "room2");
    assert_eq!(outcome.arrived_room_name.as_deref(), Some("Room room2"));
    assert!(outcome.unknown_location.is_none());
}

#[test]
fn test_the_step_adds_a_room_the_map_does_not_know() {
    let deps = deps_with_carla("room1");

    for destination in ["nonexistent_room", ""] {
        let mut swipe = swipe_in("room1");

        let outcome = rule_step(&mut swipe, &quantifier(&[], Some(destination)), &deps).unwrap();

        assert!(
            swipe.movement.current_room_id.starts_with("dynamic_"),
            "an unknown destination becomes a room of its own"
        );
        let added = swipe
            .movement
            .dynamic_rooms
            .get(&swipe.movement.current_room_id)
            .expect("the rules added the room they walked into");
        assert_eq!(
            outcome
                .unknown_location
                .as_ref()
                .map(|room| room.id.clone()),
            Some(swipe.movement.current_room_id.clone()),
            "the notice names the room the rules added"
        );
        assert_eq!(
            outcome.arrived_room_name.as_deref(),
            Some(added.name.as_str())
        );
    }
}

#[test]
fn test_the_step_uses_a_room_the_rules_added() {
    let deps = deps_with_carla("room1");
    let mut swipe = swipe_in("room1");
    let added = Room::new_dynamic("Secret Cave", "Dark and damp.");
    let added_id = added.id.clone();
    swipe.movement.dynamic_rooms.insert(added_id.clone(), added);

    let outcome = rule_step(&mut swipe, &quantifier(&[], Some(&added_id)), &deps).unwrap();

    assert_eq!(swipe.movement.current_room_id, added_id);
    assert!(outcome.unknown_location.is_none());
    assert_eq!(outcome.arrived_room_name.as_deref(), Some("Secret Cave"));
}

#[test]
fn test_the_step_reports_the_room_it_arrived_in() {
    let deps = deps_with_carla("test_room");
    let mut swipe = swipe_in("test_room");

    let outcome = rule_step(
        &mut swipe,
        &quantifier(&["carla"], Some("test_room")),
        &deps,
    )
    .unwrap();

    assert_eq!(outcome.arrived_room_name.as_deref(), Some("Room test_room"));
}

#[test]
fn test_the_step_does_not_meet_a_character_again() {
    let deps = deps_with_carla("room1");
    let carla = TestNpc::named("carla", "Carla");
    let mut swipe = swipe_with("room1", vec![carla], log_of("carla", 1, true));

    rule_step(&mut swipe, &quantifier(&["carla"], Some("room1")), &deps).unwrap();

    assert_eq!(
        swipe.npc_encounter_log.get_times_met("carla"),
        1,
        "a Character already in the scene does not enter again"
    );
}

#[test]
fn test_the_step_removes_a_character_that_left() {
    let deps = deps_with_carla("room1");
    let carla = TestNpc::named("carla", "Carla");
    let mut swipe = swipe_with("room1", vec![carla], log_of("carla", 1, true));

    rule_step(&mut swipe, &quantifier(&[], Some("room1")), &deps).unwrap();

    assert!(!swipe.npc_encounter_log.is_currently_meeting("carla"));
    assert!(swipe.scene.npcs_in_area.is_empty());
    assert_eq!(swipe.npc_encounter_log.get_times_met("carla"), 1);
}

#[test]
fn test_the_step_matches_no_trigger_without_one() {
    let deps = rules_deps(
        TestMap::single_room("room_1"),
        vec![TestNpc::named("gabriella", "Gabriella")],
    );
    let mut swipe = swipe_in("room_1");

    assert!(matched(&mut swipe, &deps, &[]).is_none());
}

#[test]
fn test_the_step_matches_a_first_encounter_trigger() {
    let mut case = trigger_case(
        make_trigger(requirement(ComparisonOperator::Eq, 0), false),
        false,
        false,
        0,
    );

    let matched =
        matched(&mut case.swipe, &case.deps, &["gabriella"]).expect("the first meeting matches");

    assert_eq!(matched.npc_id, "gabriella");
    assert_eq!(matched.trigger_idx, 0);
    assert!(!matched.trigger_repeat);
}

#[test]
fn test_the_step_matches_no_trigger_when_the_requirement_is_past() {
    let mut case = trigger_case(
        make_trigger(requirement(ComparisonOperator::Eq, 0), false),
        false,
        false,
        1,
    );

    assert!(matched(&mut case.swipe, &case.deps, &["gabriella"]).is_none());
}

#[test]
fn test_the_step_skips_a_one_shot_trigger_that_fired() {
    let mut case = trigger_case(
        make_trigger(requirement(ComparisonOperator::Eq, 0), false),
        true,
        false,
        0,
    );

    assert!(matched(&mut case.swipe, &case.deps, &["gabriella"]).is_none());
}

#[test]
fn test_a_global_trigger_matches_for_a_character_outside_the_scene() {
    let mut case = trigger_case(
        make_trigger(requirement(ComparisonOperator::Eq, 0), false),
        false,
        false,
        0,
    );

    assert!(
        case.swipe.scene.npcs_in_area.is_empty(),
        "the scene is empty"
    );

    let matched = matched(&mut case.swipe, &case.deps, &[])
        .expect("a global Trigger fires for a Character not in the scene");

    assert_eq!(matched.npc_id, "gabriella");
}

#[test]
fn test_a_global_trigger_matches_in_any_room() {
    let mut case = trigger_case(
        make_trigger(requirement(ComparisonOperator::Eq, 0), false),
        false,
        false,
        0,
    );
    case.swipe.movement.current_room_id = "room_2".to_string();

    assert!(matched(&mut case.swipe, &case.deps, &[]).is_some());
}

#[test]
fn test_a_room_scoped_trigger_matches_in_its_room() {
    let mut case = trigger_case(
        make_trigger_with_room(requirement(ComparisonOperator::Eq, 0), false, "room_1"),
        false,
        false,
        0,
    );

    assert_eq!(case.swipe.movement.current_room_id, "room_1");

    assert!(matched(&mut case.swipe, &case.deps, &[]).is_some());
}

#[test]
fn test_a_room_scoped_trigger_is_skipped_in_another_room() {
    let mut case = trigger_case(
        make_trigger_with_room(
            requirement(ComparisonOperator::Eq, 0),
            false,
            "entrance_hall",
        ),
        false,
        false,
        0,
    );

    assert_eq!(case.swipe.movement.current_room_id, "room_1");

    assert!(
        matched(&mut case.swipe, &case.deps, &[]).is_none(),
        "a room-scoped Trigger does not fire in another room"
    );
}

#[test]
fn test_a_repeatable_trigger_matches_again() {
    let mut case = trigger_case(
        make_trigger(requirement(ComparisonOperator::Lt, 3), true),
        false,
        false,
        1,
    );

    assert!(matched(&mut case.swipe, &case.deps, &[]).is_some());
}

#[test]
fn test_the_trigger_index_skips_the_triggers_before_it() {
    let npc = make_npc(
        "gabriella",
        vec![
            make_trigger(requirement(ComparisonOperator::Eq, 0), false),
            make_trigger(requirement(ComparisonOperator::Gte, 0), false),
        ],
    );
    let deps = rules_deps(TestMap::single_room("room_1"), vec![npc]);
    let mut swipe = swipe_with("room_1", vec![], {
        let mut log = NpcEncounterLog::default();
        log.mark_trigger_fired("gabriella", 0);
        log
    });

    let matched = rule_step(&mut swipe, &quantifier(&[], None), &deps)
        .expect("the rule step keeps the state consistent")
        .trigger_match
        .expect("the second Trigger matches");
    assert_eq!(matched.trigger_idx, 1, "the index is the Trigger's own");

    swipe.npc_encounter_log.mark_trigger_fired("gabriella", 1);
    assert!(
        rule_step(&mut swipe, &quantifier(&[], None), &deps)
            .expect("the rule step keeps the state consistent")
            .trigger_match
            .is_none(),
        "both Triggers are now skipped"
    );
}

#[test]
fn test_a_landed_one_shot_trigger_does_not_match_again() {
    let mut case = trigger_case(
        make_trigger(requirement(ComparisonOperator::Gte, 0), false),
        false,
        false,
        0,
    );
    let trigger = trigger_ref("gabriella", "Test Event", false);

    assert!(matched(&mut case.swipe, &case.deps, &["gabriella"]).is_some());

    let outcome = trigger_step(&mut case.swipe, &trigger, "Gabriella appears.", &case.deps);
    assert!(outcome.trigger_narration_landed);
    assert!(
        matched(&mut case.swipe, &case.deps, &["gabriella"]).is_none(),
        "a one-shot Trigger fires once"
    );
}

#[test]
fn test_the_step_reads_times_met_before_the_character_events_advance_it() {
    let npc = TestNpc::with_times_met_trigger("carla", "Carla", ComparisonOperator::Eq, 0);
    let deps = rules_deps(TestMap::single_room("test_room"), vec![npc]);
    let mut swipe = swipe_in("test_room");

    let outcome = rule_step(&mut swipe, &quantifier(&["carla"], None), &deps).unwrap();

    assert!(
        outcome.trigger_match.is_some(),
        "trigger should have fired (evaluated before times_met increment)"
    );
    assert_eq!(swipe.npc_encounter_log.get_times_met("carla"), 1);
}

#[test]
fn test_apply_npc_events_entered() {
    let deps = deps_with_carla("test_room");
    let mut swipe = swipe_in("test_room");
    let events = vec![NpcEvent {
        npc_id: "carla".to_string(),
        event_type: NpcTransitionType::Entered,
    }];

    apply_game_rules(
        &mut swipe,
        GameRuleRequest::NpcEvents(&events),
        &deps.map,
        &deps.npcs,
    )
    .unwrap();

    assert!(swipe.npc_encounter_log.is_currently_meeting("carla"));
}

#[test]
fn test_apply_npc_events_left() {
    let deps = deps_with_carla("test_room");
    let mut swipe = swipe_in("test_room");
    swipe.npc_encounter_log.set_currently_meeting("carla", true);
    let events = vec![NpcEvent {
        npc_id: "carla".to_string(),
        event_type: NpcTransitionType::Left,
    }];

    apply_game_rules(
        &mut swipe,
        GameRuleRequest::NpcEvents(&events),
        &deps.map,
        &deps.npcs,
    )
    .unwrap();

    assert!(!swipe.npc_encounter_log.is_currently_meeting("carla"));
}

#[test]
fn test_apply_npc_events_increments_times_met() {
    let deps = deps_with_carla("test_room");
    let mut swipe = swipe_in("test_room");
    let initial_times = swipe.npc_encounter_log.get_times_met("carla");
    let events = vec![NpcEvent {
        npc_id: "carla".to_string(),
        event_type: NpcTransitionType::Entered,
    }];

    apply_game_rules(
        &mut swipe,
        GameRuleRequest::NpcEvents(&events),
        &deps.map,
        &deps.npcs,
    )
    .unwrap();

    assert_eq!(
        swipe.npc_encounter_log.get_times_met("carla"),
        initial_times + 1
    );
}

#[test]
fn test_a_trigger_narration_marks_a_one_shot_trigger_fired() {
    let deps = deps_with_carla("test_room");
    let mut swipe = swipe_in("test_room");
    let trigger = trigger_ref("carla", "Carla Introduction", false);

    let outcome = trigger_step(&mut swipe, &trigger, "Some text.", &deps);

    assert!(outcome.trigger_narration_landed);
    assert!(
        swipe.npc_encounter_log.is_trigger_fired("carla", 0),
        "a one-shot Trigger is marked fired"
    );
}

#[test]
fn test_a_trigger_narration_does_not_mark_a_repeating_trigger_fired() {
    let deps = deps_with_carla("test_room");
    let mut swipe = swipe_in("test_room");
    let trigger = trigger_ref("carla", "Carla Greeting", true);

    trigger_step(&mut swipe, &trigger, "Some text.", &deps);

    assert!(
        !swipe.npc_encounter_log.is_trigger_fired("carla", 0),
        "a repeating Trigger is never marked fired"
    );
}

#[test]
fn test_an_empty_trigger_narration_lands_nothing() {
    let deps = deps_with_carla("test_room");
    let mut swipe = swipe_in("test_room");
    let trigger = trigger_ref("carla", "Carla Introduction", false);

    for text in ["", "   "] {
        let outcome = trigger_step(&mut swipe, &trigger, text, &deps);

        assert!(!outcome.trigger_narration_landed);
    }

    assert!(
        !swipe.npc_encounter_log.is_trigger_fired("carla", 0),
        "an empty narration fires nothing"
    );
}

#[test]
fn test_state_mutation_order() {
    let npc_id = "shopkeeper";
    let deps = rules_deps(TestMap::single_room("room1"), vec![shopkeeper_npc()]);
    let mut swipe = swipe_in("room1");

    assert_eq!(
        swipe.npc_encounter_log.get_times_met(npc_id),
        0,
        "pre-condition: times_met should be 0"
    );

    let outcome = rule_step(&mut swipe, &quantifier(&[npc_id], None), &deps).unwrap();

    assert!(
        outcome.trigger_match.is_some(),
        "trigger should have fired (evaluated before times_met increment)"
    );
    assert_eq!(
        swipe.npc_encounter_log.get_times_met(npc_id),
        1,
        "times_met should be 1 after the Character events are applied"
    );
}

proptest! {
    #[test]
    fn prop_the_step_keeps_the_state_consistent(
        destination in prop_oneof![
            Just(None),
            Just(Some("room1")),
            Just(Some("room2")),
            Just(Some("nowhere")),
        ],
        present_before in any::<bool>(),
        times_met in 0u32..3,
        named_now in any::<bool>(),
    ) {
        let carla = TestNpc::named("carla", "Carla");
        let deps = rules_deps(TestMap::two_rooms("room1", "room2"), vec![carla.clone()]);
        let present = if present_before { vec![carla] } else { vec![] };
        let mut swipe = swipe_with("room1", present, log_of("carla", times_met, present_before));
        let named: Vec<&str> = if named_now { vec!["carla"] } else { vec![] };

        let outcome = rule_step(&mut swipe, &quantifier(&named, destination), &deps)
            .expect("the rule step leaves a consistent state");

        let scene: Vec<&str> = swipe.scene.npcs_in_area.iter().map(|npc| npc.id.as_str()).collect();
        prop_assert_eq!(scene, named.clone(), "the scene is what the analysis named");
        let known = matches!(destination, None | Some("room1") | Some("room2"));
        prop_assert_eq!(
            outcome.unknown_location.is_none(),
            known,
            "only a destination the map does not know adds a room"
        );
        prop_assert_eq!(
            swipe.movement.current_room_id.starts_with("dynamic_"),
            !known,
            "the player stands in the destination, added when unknown"
        );
    }

    #[test]
    fn prop_npc_events_keep_the_state_consistent(
        npc_id in "[a-z]{1,10}",
        transition in prop_oneof![
            Just(NpcTransitionType::Entered),
            Just(NpcTransitionType::Left),
        ],
    ) {
        let deps = rules_deps(TestMap::single_room("room1"), vec![TestNpc::named(&npc_id, &npc_id)]);
        let mut swipe = swipe_in("room1");
        let events = vec![NpcEvent { npc_id, event_type: transition }];

        apply_game_rules(&mut swipe, GameRuleRequest::NpcEvents(&events), &deps.map, &deps.npcs)
            .expect("the Character events leave a consistent state");
    }

    #[test]
    fn prop_analysis_preserves_the_mutation_order(
        times_met in 0u32..3,
    ) {
        let npc = TestNpc::with_times_met_trigger(
            "carla",
            "Carla",
            ComparisonOperator::Eq,
            times_met,
        );
        let deps = rules_deps(TestMap::single_room("room1"), vec![npc]);
        let mut swipe = swipe_with("room1", vec![], log_of("carla", times_met, false));

        let outcome = rule_step(&mut swipe, &quantifier(&["carla"], None), &deps)
            .expect("the rule step keeps the state consistent");

        prop_assert!(
            outcome.trigger_match.is_some(),
            "a Trigger that meets its requirement before the events fires"
        );
        prop_assert_eq!(
            swipe.npc_encounter_log.get_times_met("carla"),
            times_met + 1,
            "the events advance times_met after the Triggers read it"
        );
    }

    #[test]
    fn prop_a_trigger_never_breaks_the_state(
        (trigger, fired) in trigger_shapes(),
        present_before in any::<bool>(),
        times_met in 0u32..3,
    ) {
        let repeat = trigger.repeat;
        let mut case = trigger_case(trigger, fired, present_before, times_met);

        let matched = matched(&mut case.swipe, &case.deps, &["gabriella"]);

        prop_assert!(
            !fired || repeat || matched.is_none(),
            "a fired one-shot Trigger does not match"
        );
        if let Some(matched) = matched {
            prop_assert_eq!(matched.npc_id, "gabriella");
            prop_assert_eq!(matched.trigger_idx, 0);
            prop_assert_eq!(matched.trigger_repeat, repeat);
        }
    }

    #[test]
    fn prop_a_one_shot_trigger_fires_at_most_once(
        repeat in any::<bool>(),
        fired_before in any::<bool>(),
    ) {
        let trigger = make_trigger(requirement(ComparisonOperator::Gte, 0), repeat);
        let mut case = trigger_case(trigger, fired_before, false, 0);
        let trigger = trigger_ref("gabriella", "Test Event", repeat);

        let outcome = trigger_step(&mut case.swipe, &trigger, "Gabriella appears.", &case.deps);
        prop_assert!(outcome.trigger_narration_landed);

        prop_assert_eq!(
            matched(&mut case.swipe, &case.deps, &["gabriella"]).is_some(),
            repeat,
            "only a repeating Trigger matches after its narration landed"
        );
    }

    #[test]
    fn prop_a_room_limited_trigger_fires_only_in_its_room(
        destination in prop_oneof![Just("room_1"), Just("room_2")],
        room_id in prop_oneof![Just(None), Just(Some("room_1")), Just(Some("room_2"))],
    ) {
        let trigger = match room_id {
            Some(room_id) => make_trigger_with_room(
                requirement(ComparisonOperator::Gte, 0),
                true,
                room_id,
            ),
            None => make_trigger(requirement(ComparisonOperator::Gte, 0), true),
        };
        let mut case = trigger_case(trigger, false, false, 0);

        let outcome = rule_step(&mut case.swipe, &quantifier(&["gabriella"], Some(destination)), &case.deps)
            .expect("the rule step keeps the state consistent");
        let arrived = case.swipe.movement.current_room_id.clone();

        prop_assert_eq!(arrived.clone(), destination.to_string());
        prop_assert_eq!(
            outcome.trigger_match.is_some(),
            room_id.is_none_or(|room_id| room_id == arrived),
            "a room-limited Trigger fires only in its room"
        );
    }

    #[test]
    fn prop_characters_that_leave_are_removed(
        present_before in any::<bool>(),
        named_now in any::<bool>(),
    ) {
        let carla = TestNpc::named("carla", "Carla");
        let deps = deps_with_carla("room1");
        let present = if present_before { vec![carla] } else { vec![] };
        let mut swipe = swipe_with("room1", present, log_of("carla", 1, present_before));
        let named: Vec<&str> = if named_now { vec!["carla"] } else { vec![] };

        rule_step(&mut swipe, &quantifier(&named, None), &deps)
            .expect("the rule step keeps the state consistent");

        let in_scene = swipe.scene.npcs_in_area.iter().any(|npc| npc.id == "carla");
        prop_assert_eq!(in_scene, named_now, "the scene is what the analysis named");
        prop_assert_eq!(
            swipe.npc_encounter_log.is_currently_meeting("carla"),
            named_now,
            "a Character that left is not meeting"
        );
    }
}
