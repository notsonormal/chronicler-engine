//! Tests for `utils/game_rules.rs` game rules.

use std::collections::HashMap;
use std::sync::Arc;

use proptest::prelude::*;

use crate::domain::model::agent::Kind;
use crate::domain::model::character::{CharacterSheet, CharacterCard, PersonaCard};
use crate::domain::model::generation::GenerationOrder;
use crate::domain::model::map::{MapDef, Room};
use crate::domain::model::message::SwipeAt;
use crate::domain::model::quantifier::QuantifierResult;
use crate::domain::model::state::movement::MovementState;
use crate::domain::model::state::scene_state::SceneState;
use crate::domain::model::state::swipe_state::SwipeState;
use crate::domain::model::state::event_record::{EventRecord, TriggerRef};
use crate::domain::model::trigger::{
    ComparisonOperator, Encounters, Trigger, TriggerNarration, TriggerRequirement,
};
use crate::domain::model::utils::game_rules::{apply, GameWorld, RulesOutcome};
use crate::test_support::*;

struct RulesDeps {
    map: Arc<MapDef>,
    persona: Arc<PersonaCard>,
    characters: HashMap<String, CharacterCard>,
}

fn rules_deps(map: MapDef, characters: Vec<CharacterCard>) -> RulesDeps {
    RulesDeps {
        map: Arc::new(map),
        persona: Arc::new(TestPersona::standard()),
        characters: characters
            .into_iter()
            .map(|character| (character.id.clone(), character))
            .collect(),
    }
}

fn deps_with_carla(room_id: &str) -> RulesDeps {
    rules_deps(
        TestMap::single_room(room_id),
        vec![TestCharacter::named("carla", "Carla")],
    )
}

fn swipe_in(room_id: &str) -> SwipeState {
    swipe_with(room_id, vec![], Encounters::default())
}

fn swipe_with(room_id: &str, present: Vec<CharacterCard>, log: Encounters) -> SwipeState {
    SwipeState {
        event_record: None,
        movement: MovementState {
            current_room_id: room_id.to_string(),
            dynamic_rooms: HashMap::new(),
        },
        scene: SceneState {
            characters_present: present,
        },
        encounters: log,
    }
}

fn log_of(character_id: &str, times_met: u32, currently_meeting: bool) -> Encounters {
    let mut log = Encounters::default();
    if times_met > 0 || currently_meeting {
        let entry = log.characters.entry(character_id.to_string()).or_default();
        entry.times_met = times_met;
        entry.currently_meeting = currently_meeting;
    }
    log
}

fn quantifier(character_ids: &[&str], destination: Option<&str>) -> QuantifierResult {
    QuantifierResult {
        present: character_ids.iter().map(|id| id.to_string()).collect(),
        destination: destination.map(str::to_string),
    }
}

fn rule_step_of(
    kind: Kind,
    swipe: &mut SwipeState,
    result: &QuantifierResult,
    deps: &RulesDeps,
) -> Result<RulesOutcome, crate::error::EngineError> {
    let world = GameWorld {
        map: &deps.map,
        characters: &deps.characters,
        persona_name: &deps.persona.sheet.name,
    };
    let outcome = apply(swipe.clone(), result, &world, kind)?;
    *swipe = outcome.state.clone();
    Ok(outcome)
}

fn rule_step(
    swipe: &mut SwipeState,
    result: &QuantifierResult,
    deps: &RulesDeps,
) -> Result<RulesOutcome, crate::error::EngineError> {
    rule_step_of(Kind::Narration, swipe, result, deps)
}

fn event_record_of(character_id: &str, trigger_name: &str, repeat: bool) -> EventRecord {
    EventRecord {
        trigger: TriggerRef {
            character_id: character_id.to_string(),
            trigger_idx: 0,
            trigger_name: trigger_name.to_string(),
            trigger_repeat: repeat,
        },
        header_name: trigger_name.to_string(),
        direction: "A direction.".to_string(),
    }
}

fn make_character(id: &str, triggers: Vec<Trigger>) -> CharacterCard {
    CharacterCard {
        id: id.to_string(),
        sheet: CharacterSheet {
            name: id.to_string(),
            description: "Test Character".to_string(),
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

/// One generated Trigger, and whether the encounters already hold it fired.
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
    let character = make_character("gabriella", vec![trigger]);
    let deps = rules_deps(
        TestMap::two_rooms("room_1", "room_2"),
        vec![character.clone()],
    );

    let mut log = log_of("gabriella", times_met, present_before);
    if fired {
        log.mark_trigger_fired("gabriella", 0);
    }
    let present = if present_before {
        vec![character]
    } else {
        vec![]
    };

    TriggerCase {
        deps,
        swipe: swipe_with("room_1", present, log),
    }
}

fn matched(swipe: &mut SwipeState, deps: &RulesDeps, named: &[&str]) -> Option<EventRecord> {
    rule_step(swipe, &quantifier(named, None), deps)
        .expect("the rule step keeps the state consistent")
        .fired
}

/// The state the text save of an event Narration writes: the Swipe state the rules
/// read for the generation after it.
fn event_state_of(start: &SwipeState, record: &EventRecord) -> SwipeState {
    GenerationOrder::event_after(
        SwipeAt {
            message: 1,
            swipe: 0,
        },
        record.clone(),
    )
    .write(String::new(), start)
    .swipe
    .state
}

#[test]
fn test_the_step_without_a_movement_keeps_the_room() {
    let deps = deps_with_carla("room1");
    let mut swipe = swipe_in("room1");

    rule_step(&mut swipe, &quantifier(&["carla"], None), &deps).unwrap();

    assert_eq!(swipe.movement.current_room_id, "room1");
}

#[test]
fn test_the_step_sets_the_characters_it_names() {
    let deps = deps_with_carla("room1");
    let mut swipe = swipe_in("room1");

    rule_step(&mut swipe, &quantifier(&["carla"], None), &deps).unwrap();

    assert_eq!(swipe.scene.characters_present.len(), 1);
    assert_eq!(swipe.scene.characters_present[0].id, "carla");
    assert!(swipe.encounters.is_currently_meeting("carla"));
    assert_eq!(swipe.encounters.get_times_met("carla"), 1);
}

#[test]
fn test_the_step_walks_into_a_room_of_the_map() {
    let deps = rules_deps(
        TestMap::two_rooms("room1", "room2"),
        vec![TestCharacter::named("carla", "Carla")],
    );
    let mut swipe = swipe_in("room1");

    rule_step(&mut swipe, &quantifier(&["carla"], Some("room2")), &deps).unwrap();

    assert_eq!(swipe.movement.current_room_id, "room2");
    assert!(swipe.movement.dynamic_rooms.is_empty());
}

#[test]
fn test_the_step_adds_a_room_the_map_does_not_know() {
    let deps = deps_with_carla("room1");

    for destination in ["nonexistent_room", ""] {
        let mut swipe = swipe_in("room1");

        rule_step(&mut swipe, &quantifier(&[], Some(destination)), &deps).unwrap();

        assert!(
            swipe.movement.current_room_id.starts_with("dynamic_"),
            "an unknown destination becomes a room of its own"
        );
        assert!(
            swipe
                .movement
                .dynamic_rooms
                .contains_key(&swipe.movement.current_room_id),
            "the rules added the room they walked into"
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

    rule_step(&mut swipe, &quantifier(&[], Some(&added_id)), &deps).unwrap();

    assert_eq!(swipe.movement.current_room_id, added_id);
    assert_eq!(
        swipe.movement.dynamic_rooms.len(),
        1,
        "the room is used, not added again"
    );
}

#[test]
fn test_the_step_does_not_meet_a_character_again() {
    let deps = deps_with_carla("room1");
    let carla = TestCharacter::named("carla", "Carla");
    let mut swipe = swipe_with("room1", vec![carla], log_of("carla", 1, true));

    rule_step(&mut swipe, &quantifier(&["carla"], Some("room1")), &deps).unwrap();

    assert_eq!(
        swipe.encounters.get_times_met("carla"),
        1,
        "a Character already in the scene does not enter again"
    );
}

#[test]
fn test_the_step_removes_a_character_that_left() {
    let deps = deps_with_carla("room1");
    let carla = TestCharacter::named("carla", "Carla");
    let mut swipe = swipe_with("room1", vec![carla], log_of("carla", 1, true));

    rule_step(&mut swipe, &quantifier(&[], Some("room1")), &deps).unwrap();

    assert!(!swipe.encounters.is_currently_meeting("carla"));
    assert!(swipe.scene.characters_present.is_empty());
    assert_eq!(swipe.encounters.get_times_met("carla"), 1);
}

#[test]
fn test_the_step_matches_no_trigger_without_one() {
    let deps = rules_deps(
        TestMap::single_room("room_1"),
        vec![TestCharacter::named("gabriella", "Gabriella")],
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

    assert_eq!(matched.trigger.character_id, "gabriella");
    assert_eq!(matched.trigger.trigger_idx, 0);
    assert!(!matched.trigger.trigger_repeat);
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
        case.swipe.scene.characters_present.is_empty(),
        "the scene is empty"
    );

    let matched = matched(&mut case.swipe, &case.deps, &[])
        .expect("a global Trigger fires for a Character not in the scene");

    assert_eq!(matched.trigger.character_id, "gabriella");
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
    let character = make_character(
        "gabriella",
        vec![
            make_trigger(requirement(ComparisonOperator::Eq, 0), false),
            make_trigger(requirement(ComparisonOperator::Gte, 0), false),
        ],
    );
    let deps = rules_deps(TestMap::single_room("room_1"), vec![character]);
    let mut swipe = swipe_with("room_1", vec![], {
        let mut log = Encounters::default();
        log.mark_trigger_fired("gabriella", 0);
        log
    });

    let matched = rule_step(&mut swipe, &quantifier(&[], None), &deps)
        .expect("the rule step keeps the state consistent")
        .fired
        .expect("the second Trigger matches");
    assert_eq!(
        matched.trigger.trigger_idx, 1,
        "the index is the Trigger's own"
    );

    swipe.encounters.mark_trigger_fired("gabriella", 1);
    assert!(
        rule_step(&mut swipe, &quantifier(&[], None), &deps)
            .expect("the rule step keeps the state consistent")
            .fired
            .is_none(),
        "both Triggers are now skipped"
    );
}

#[test]
fn test_a_one_shot_trigger_does_not_fire_again_from_the_state_of_its_event() {
    let mut case = trigger_case(
        make_trigger(requirement(ComparisonOperator::Gte, 0), false),
        false,
        false,
        0,
    );

    let fired = matched(&mut case.swipe, &case.deps, &["gabriella"]).expect("the Trigger fires");

    let mut event_state = event_state_of(&case.swipe, &fired);
    assert!(
        matched(&mut event_state, &case.deps, &["gabriella"]).is_none(),
        "a one-shot Trigger fires once"
    );
}

#[test]
fn game_rules_times_met_before_encounters() {
    let character =
        TestCharacter::with_times_met_trigger("carla", "Carla", ComparisonOperator::Eq, 0);
    let deps = rules_deps(TestMap::single_room("test_room"), vec![character]);
    let mut swipe = swipe_in("test_room");

    let outcome = rule_step(&mut swipe, &quantifier(&["carla"], None), &deps).unwrap();

    assert!(
        outcome.fired.is_some(),
        "a first-encounter Trigger fires on the Narration that brings its Character"
    );
    assert_eq!(swipe.encounters.get_times_met("carla"), 1);
}

#[test]
fn game_rules_event_no_movement() {
    let deps = rules_deps(
        TestMap::two_rooms("room1", "room2"),
        vec![TestCharacter::named("carla", "Carla")],
    );
    let mut swipe = swipe_in("room1");

    rule_step_of(
        Kind::Event,
        &mut swipe,
        &quantifier(&["carla"], Some("room2")),
        &deps,
    )
    .unwrap();

    assert_eq!(swipe.movement.current_room_id, "room1");
    assert!(swipe.movement.dynamic_rooms.is_empty());
    assert!(
        swipe.encounters.is_currently_meeting("carla"),
        "an event still updates the Characters present"
    );
}

#[test]
fn game_rules_event_no_trigger() {
    let mut case = trigger_case(
        make_trigger(requirement(ComparisonOperator::Gte, 0), true),
        false,
        false,
        0,
    );
    case.swipe.event_record = Some(event_record_of("gabriella", "Earlier Event", false));

    let outcome = rule_step_of(
        Kind::Event,
        &mut case.swipe,
        &quantifier(&["gabriella"], None),
        &case.deps,
    )
    .unwrap();

    assert!(
        outcome.fired.is_none(),
        "a Trigger condition holds, and none fires"
    );
    assert!(outcome.state.event_record.is_none());

    let mut control = trigger_case(
        make_trigger(requirement(ComparisonOperator::Gte, 0), true),
        false,
        false,
        0,
    );
    let control_outcome = rule_step_of(
        Kind::Narration,
        &mut control.swipe,
        &quantifier(&["gabriella"], None),
        &control.deps,
    )
    .unwrap();
    assert!(
        control_outcome.fired.is_some(),
        "the same input fires the Trigger for a Narration"
    );
}

#[test]
fn game_rules_narration_replaces_the_event_record() {
    let deps = deps_with_carla("room1");
    let mut swipe = swipe_in("room1");
    swipe.event_record = Some(event_record_of("carla", "Earlier Event", false));

    let outcome = rule_step(&mut swipe, &quantifier(&["carla"], None), &deps).unwrap();

    assert!(outcome.fired.is_none());
    assert!(
        swipe.event_record.is_none(),
        "a Narration that fires no Trigger leaves no event record"
    );
}

#[test]
fn game_rules_fired_record_rendered() {
    let mut trigger = make_trigger(requirement(ComparisonOperator::Eq, 0), false);
    trigger.narration.narration_prompt = "{{user}} hears the gate open.".to_string();
    let mut case = trigger_case(trigger, false, false, 0);
    let persona_name = case.deps.persona.sheet.name.clone();

    let outcome = rule_step(
        &mut case.swipe,
        &quantifier(&["gabriella"], None),
        &case.deps,
    )
    .unwrap();

    let fired = outcome.fired.expect("the Trigger fires");
    assert_eq!(
        fired.direction,
        format!("{persona_name} hears the gate open.")
    );
    assert_eq!(fired.header_name, "Test Event");
    assert_eq!(
        outcome.state.event_record,
        Some(fired),
        "the event record is the fired record"
    );
}

#[test]
fn test_state_mutation_order() {
    let character_id = "shopkeeper";
    let deps = rules_deps(TestMap::single_room("room1"), vec![shopkeeper_character()]);
    let mut swipe = swipe_in("room1");

    assert_eq!(
        swipe.encounters.get_times_met(character_id),
        0,
        "pre-condition: times_met should be 0"
    );

    let outcome = rule_step(&mut swipe, &quantifier(&[character_id], None), &deps).unwrap();

    assert!(
        outcome.fired.is_some(),
        "trigger should have fired (evaluated before times_met increment)"
    );
    assert_eq!(
        swipe.encounters.get_times_met(character_id),
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
        let carla = TestCharacter::named("carla", "Carla");
        let deps = rules_deps(TestMap::two_rooms("room1", "room2"), vec![carla.clone()]);
        let present = if present_before { vec![carla] } else { vec![] };
        let mut swipe = swipe_with("room1", present, log_of("carla", times_met, present_before));
        let named: Vec<&str> = if named_now { vec!["carla"] } else { vec![] };

        rule_step(&mut swipe, &quantifier(&named, destination), &deps)
            .expect("the rule step leaves a consistent state");

        let scene: Vec<&str> = swipe.scene.characters_present.iter().map(|character| character.id.as_str()).collect();
        prop_assert_eq!(scene, named.clone(), "the scene is what the Quantifier named");
        let known = matches!(destination, None | Some("room1") | Some("room2"));
        prop_assert_eq!(
            swipe.movement.dynamic_rooms.is_empty(),
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
    fn prop_a_character_that_enters_or_leaves_keeps_the_state_consistent(
        character_id in "[a-z]{1,10}",
        present_before in any::<bool>(),
        named_now in any::<bool>(),
    ) {
        let character = TestCharacter::named(&character_id, &character_id);
        let deps = rules_deps(TestMap::single_room("room1"), vec![character.clone()]);
        let present = if present_before { vec![character] } else { vec![] };
        let mut swipe = swipe_with("room1", present, log_of(&character_id, 1, present_before));
        let named: Vec<&str> = if named_now { vec![character_id.as_str()] } else { vec![] };

        rule_step(&mut swipe, &quantifier(&named, None), &deps)
            .expect("the Character events leave a consistent state");
    }

    #[test]
    fn prop_the_step_preserves_the_mutation_order(
        times_met in 0u32..3,
    ) {
        let character = TestCharacter::with_times_met_trigger(
            "carla",
            "Carla",
            ComparisonOperator::Eq,
            times_met,
        );
        let deps = rules_deps(TestMap::single_room("room1"), vec![character]);
        let mut swipe = swipe_with("room1", vec![], log_of("carla", times_met, false));

        let outcome = rule_step(&mut swipe, &quantifier(&["carla"], None), &deps)
            .expect("the rule step keeps the state consistent");

        prop_assert!(
            outcome.fired.is_some(),
            "a Trigger that meets its requirement before the events fires"
        );
        prop_assert_eq!(
            swipe.encounters.get_times_met("carla"),
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
            prop_assert_eq!(matched.trigger.character_id, "gabriella");
            prop_assert_eq!(matched.trigger.trigger_idx, 0);
            prop_assert_eq!(matched.trigger.trigger_repeat, repeat);
        }
    }

    #[test]
    fn prop_a_one_shot_trigger_fires_at_most_once(
        repeat in any::<bool>(),
        fired_before in any::<bool>(),
    ) {
        let trigger = make_trigger(requirement(ComparisonOperator::Gte, 0), repeat);
        let case = trigger_case(trigger, fired_before, false, 0);
        let record = event_record_of("gabriella", "Test Event", repeat);

        let mut event_state = event_state_of(&case.swipe, &record);

        prop_assert_eq!(
            matched(&mut event_state, &case.deps, &["gabriella"]).is_some(),
            repeat,
            "only a repeating Trigger matches from the state of its event"
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
            outcome.fired.is_some(),
            room_id.is_none_or(|room_id| room_id == arrived),
            "a room-limited Trigger fires only in its room"
        );
    }

    #[test]
    fn prop_characters_that_leave_are_removed(
        present_before in any::<bool>(),
        named_now in any::<bool>(),
    ) {
        let carla = TestCharacter::named("carla", "Carla");
        let deps = deps_with_carla("room1");
        let present = if present_before { vec![carla] } else { vec![] };
        let mut swipe = swipe_with("room1", present, log_of("carla", 1, present_before));
        let named: Vec<&str> = if named_now { vec!["carla"] } else { vec![] };

        rule_step(&mut swipe, &quantifier(&named, None), &deps)
            .expect("the rule step keeps the state consistent");

        let in_scene = swipe.scene.characters_present.iter().any(|character| character.id == "carla");
        prop_assert_eq!(in_scene, named_now, "the scene is what the Quantifier named");
        prop_assert_eq!(
            swipe.encounters.is_currently_meeting("carla"),
            named_now,
            "a Character that left is not meeting"
        );
    }
}
