//! The plan rules of one request and the record one generation writes, one branch per test.

use super::agent::Kind;
use super::generation::{
    Generation, GenerationOrder, JobPlan, JobRequest, OptionsRun, PlanError, Place, StoryEnd,
};
use super::message::{Maker, Message, Swipe, SwipeAt};
use super::prompt_preset::PresetType;
use super::state::generation_status::{Record, Step};
use super::state::message_types::{MessageKind, MessageType};
use super::state::movement::MovementState;
use super::state::scene_state::SceneState;
use super::state::swipe_state::SwipeState;
use super::state::event_record::{EventRecord, TriggerRef};
use super::tail::Tail;
use super::trigger::Encounters;

fn state_in(room: &str, event_record: Option<EventRecord>) -> SwipeState {
    SwipeState {
        movement: MovementState {
            current_room_id: room.to_string(),
            dynamic_rooms: Default::default(),
        },
        scene: SceneState::default(),
        encounters: Encounters::default(),
        event_record,
    }
}

fn event_record_repeating(repeat: bool) -> EventRecord {
    EventRecord {
        trigger: TriggerRef {
            character_id: "character_1".to_string(),
            trigger_idx: 0,
            trigger_name: "The gate opens".to_string(),
            trigger_repeat: repeat,
        },
        header_name: "The gate opens".to_string(),
        direction: "the gate swings open".to_string(),
    }
}

fn event_record() -> EventRecord {
    event_record_repeating(false)
}

fn swipe(text: &str, state: SwipeState) -> Swipe {
    Swipe::new(text, Maker::Narrator, state)
}

/// A story of two Messages. The last one is id 7 with two Swipes, the second one
/// active, and the Message before it is id 6.
fn tail_after_one_message(kind: MessageKind) -> Tail {
    let mut last = Message::new(
        MessageType::Narration,
        kind,
        swipe("first", state_in("room2", None)),
    );
    last.id = 7;
    last.swipes = vec![
        swipe("first", state_in("room2", None)),
        swipe("second", state_in("room3", None)),
    ];
    last.active_swipe_index = 1;
    Tail {
        last,
        state_before_last: Some((
            SwipeAt {
                message: 6,
                swipe: 0,
            },
            state_in("room1", None),
        )),
        count: 2,
    }
}

fn tail_of_the_opening() -> Tail {
    let mut opening = Message::new(
        MessageType::Narration,
        MessageKind::Narration,
        swipe("opening", state_in("room1", None)),
    );
    opening.id = 3;
    opening.swipes = vec![
        swipe("opening", state_in("room1", None)),
        swipe("retold", state_in("room2", None)),
    ];
    opening.active_swipe_index = 1;
    Tail {
        last: opening,
        state_before_last: None,
        count: 1,
    }
}

fn story_of(tail: Tail) -> StoryEnd {
    StoryEnd {
        tail: Some(tail),
        has_text: true,
    }
}

fn plan(request: JobRequest, tail: Tail) -> Result<JobPlan, PlanError> {
    JobPlan::for_request(&request, story_of(tail))
}

fn first_order(plan: Result<JobPlan, PlanError>) -> GenerationOrder {
    plan.expect("the request has a plan")
        .first
        .expect("the plan has a generation")
}

#[test]
fn plan_retry_by_last_kind() {
    let rows = [
        (
            "the opening",
            tail_of_the_opening(),
            Generation::Narration { guide: None },
            SwipeAt {
                message: 3,
                swipe: 0,
            },
        ),
        (
            "a Narration",
            tail_after_one_message(MessageKind::Narration),
            Generation::Narration { guide: None },
            SwipeAt {
                message: 6,
                swipe: 0,
            },
        ),
        (
            "an event Narration",
            tail_after_one_message(MessageKind::EventNarration(event_record())),
            Generation::Event(event_record()),
            SwipeAt {
                message: 6,
                swipe: 0,
            },
        ),
        (
            "an Impersonate Input",
            tail_after_one_message(MessageKind::ImpersonateInput),
            Generation::Impersonate { direction: None },
            SwipeAt {
                message: 6,
                swipe: 0,
            },
        ),
        (
            "a typed Input",
            tail_after_one_message(MessageKind::TypedInput),
            Generation::InputRetry,
            SwipeAt {
                message: 6,
                swipe: 0,
            },
        ),
    ];

    for (last, tail, generation, start) in rows {
        let order = first_order(plan(JobRequest::Retry(tail.last.id), tail));

        assert_eq!(order.generation, generation, "a Retry of {last}");
        assert_eq!(order.place, Place::NewSwipeOnLast, "a Retry of {last}");
        assert_eq!(
            order.start, start,
            "a Retry of {last} starts from its own start"
        );
    }
}

#[test]
fn plan_retry_keeps_the_guide_of_a_narration() {
    let mut tail = tail_after_one_message(MessageKind::Narration);
    tail.last.swipes[1].guide_text = Some("go north".to_string());

    let order = first_order(plan(JobRequest::Retry(tail.last.id), tail));

    assert_eq!(
        order.generation,
        Generation::Narration {
            guide: Some("go north".to_string())
        }
    );
}

#[test]
fn plan_retry_of_the_opening_applies_no_guide() {
    let mut tail = tail_of_the_opening();
    tail.last.swipes[1].guide_text = Some("go north".to_string());

    let order = first_order(plan(JobRequest::Retry(tail.last.id), tail));

    assert_eq!(order.generation, Generation::Narration { guide: None });
}

#[test]
fn plan_retry_bare_impersonate() {
    let mut tail = tail_after_one_message(MessageKind::ImpersonateInput);
    tail.last.swipes[1].impersonated = true;

    let order = first_order(plan(JobRequest::Retry(tail.last.id), tail));

    assert_eq!(
        order.generation,
        Generation::Impersonate { direction: None }
    );
}

#[test]
fn plan_retry_keeps_the_direction_of_an_impersonate_input() {
    let mut tail = tail_after_one_message(MessageKind::ImpersonateInput);
    tail.last.swipes[1].impersonate_direction = Some("be bold".to_string());

    let order = first_order(plan(JobRequest::Retry(tail.last.id), tail));

    assert_eq!(
        order.generation,
        Generation::Impersonate {
            direction: Some("be bold".to_string())
        }
    );
}

#[test]
fn plan_retry_no_story() {
    let story = StoryEnd {
        tail: None,
        has_text: false,
    };

    assert_eq!(
        JobPlan::for_request(&JobRequest::Retry(0), story),
        Err(PlanError::BadRequest("No input to retry".to_string()))
    );
}

#[test]
fn plan_retrigger_without_a_record_is_a_bad_request() {
    let with_context = |kind: MessageKind| {
        let mut tail = tail_after_one_message(kind);
        tail.last.swipes[1].state = state_in("room3", Some(event_record()));
        tail
    };
    let rows = [
        (
            "no event record",
            tail_after_one_message(MessageKind::Narration),
        ),
        (
            "an event Narration last",
            with_context(MessageKind::EventNarration(event_record())),
        ),
        ("a typed Input last", with_context(MessageKind::TypedInput)),
    ];

    for (case, tail) in rows {
        assert_eq!(
            plan(JobRequest::Retrigger(tail.last.id), tail),
            Err(PlanError::BadRequest(
                "No event record available".to_string()
            )),
            "a Retrigger with {case}"
        );
    }
}

#[test]
fn plan_names_a_message_that_is_not_the_last_one() {
    for request in [
        JobRequest::Retry as fn(u64) -> JobRequest,
        JobRequest::Retrigger as fn(u64) -> JobRequest,
    ] {
        let tail = tail_after_one_message(MessageKind::Narration);
        let stale = tail.last.id + 1;

        assert_eq!(
            plan(request(stale), tail),
            Err(PlanError::LastMessageChanged),
            "a request that names a Message that is not the last one changes nothing"
        );
    }
}

#[test]
fn plan_retrigger_no_story() {
    let story = StoryEnd {
        tail: None,
        has_text: false,
    };

    assert!(matches!(
        JobPlan::for_request(&JobRequest::Retrigger(0), story),
        Err(PlanError::BadRequest(_))
    ));
}

#[test]
fn plan_retrigger_copies_record() {
    for kind in [MessageKind::Narration, MessageKind::ImpersonateInput] {
        let mut tail = tail_after_one_message(kind.clone());
        tail.last.swipes[1].state = state_in("room3", Some(event_record()));
        let current = tail.current_at();

        let order = first_order(plan(JobRequest::Retrigger(tail.last.id), tail));

        assert_eq!(
            order,
            GenerationOrder {
                generation: Generation::Event(event_record()),
                place: Place::NewMessage,
                start: current,
            },
            "a Retrigger after {kind:?} copies the record of the current state"
        );
    }
}

#[test]
fn plan_action_no_story() {
    let story = StoryEnd {
        tail: None,
        has_text: false,
    };

    assert_eq!(
        JobPlan::for_request(&JobRequest::Action("look".to_string()), story),
        Err(PlanError::Missing(Record::Story))
    );
}

#[test]
fn plan_action_saves_the_input_and_narrates_from_the_current_state() {
    let tail = tail_after_one_message(MessageKind::Narration);
    let current = tail.current_at();

    let job = plan(JobRequest::Action("look".to_string()), tail).expect("an Action plans");

    assert_eq!(job.input.as_deref(), Some("look"));
    assert_eq!(job.options, OptionsRun::AfterNarration);
    assert_eq!(
        job.first,
        Some(GenerationOrder {
            generation: Generation::Narration { guide: None },
            place: Place::NewMessage,
            start: current,
        })
    );
}

#[test]
fn plan_continue_saves_no_input() {
    let tail = tail_after_one_message(MessageKind::Narration);

    let job = plan(JobRequest::Action(String::new()), tail).expect("a Continue plans");

    assert_eq!(job.input, None);
}

#[test]
fn plan_guide_carries_the_direction() {
    let tail = tail_after_one_message(MessageKind::Narration);

    let order = first_order(plan(JobRequest::Guide("go north".to_string()), tail));

    assert_eq!(
        order.generation,
        Generation::Narration {
            guide: Some("go north".to_string())
        }
    );
}

#[test]
fn plan_bare_impersonate() {
    let tail = tail_after_one_message(MessageKind::Narration);

    let order = first_order(plan(JobRequest::Impersonate(None), tail));

    assert_eq!(
        order.generation,
        Generation::Impersonate { direction: None }
    );
    assert_eq!(order.place, Place::NewMessage);
}

#[test]
fn plan_impersonate_carries_the_direction() {
    let tail = tail_after_one_message(MessageKind::Narration);

    let order = first_order(plan(
        JobRequest::Impersonate(Some("be bold".to_string())),
        tail,
    ));

    assert_eq!(
        order.generation,
        Generation::Impersonate {
            direction: Some("be bold".to_string())
        }
    );
}

#[test]
fn plan_options_no_text() {
    let story = StoryEnd {
        tail: Some(tail_of_the_opening()),
        has_text: false,
    };

    assert_eq!(
        JobPlan::for_request(&JobRequest::Options, story),
        Err(PlanError::BadRequest(
            "No scene to generate options for yet".to_string()
        ))
    );
}

#[test]
fn plan_options_runs_alone() {
    let job = plan(JobRequest::Options, tail_of_the_opening()).expect("/options plans");

    assert_eq!(job.first, None);
    assert_eq!(job.options, OptionsRun::OnDemand);
    assert_eq!(job.first_step(), Step::Options);
}

#[test]
fn plan_first_step_of_an_event_retry_is_the_event_step() {
    let tail = tail_after_one_message(MessageKind::EventNarration(event_record()));

    let job = plan(JobRequest::Retry(tail.last.id), tail).expect("a Retry plans");

    assert_eq!(job.first_step(), Step::GeneratingEvent);
}

#[test]
fn plan_input_retry_not_analysed() {
    let tail = tail_after_one_message(MessageKind::TypedInput);

    let order = first_order(plan(JobRequest::Retry(tail.last.id), tail));

    assert!(
        !order.is_analysed(),
        "the Retry of a typed Input ends after its text save"
    );
}

#[test]
fn plan_other_kinds_are_analysed() {
    let rows = [
        MessageKind::Narration,
        MessageKind::ImpersonateInput,
        MessageKind::EventNarration(event_record()),
    ];

    for kind in rows {
        let tail = tail_after_one_message(kind.clone());
        let order = first_order(plan(JobRequest::Retry(tail.last.id), tail));

        assert!(order.is_analysed(), "a Retry of {kind:?} is analysed");
    }
}

fn order_of(generation: Generation) -> GenerationOrder {
    GenerationOrder {
        generation,
        place: Place::NewMessage,
        start: SwipeAt {
            message: 1,
            swipe: 0,
        },
    }
}

#[test]
fn write_the_record_of_each_kind() {
    let start = state_in("room1", None);
    let rows = [
        (
            Generation::Narration { guide: None },
            Kind::Narration,
            MessageType::Narration,
            MessageKind::Narration,
            Maker::Narrator,
        ),
        (
            Generation::Impersonate { direction: None },
            Kind::Impersonate,
            MessageType::Input,
            MessageKind::ImpersonateInput,
            Maker::Player,
        ),
        (
            Generation::InputRetry,
            Kind::InputRetry,
            MessageType::Input,
            MessageKind::TypedInput,
            Maker::Player,
        ),
        (
            Generation::Event(event_record()),
            Kind::Event,
            MessageType::Narration,
            MessageKind::EventNarration(event_record()),
            Maker::Narrator,
        ),
    ];

    for (generation, kind, message_type, message_kind, maker) in rows {
        let order = order_of(generation);
        let written = order.write("text".to_string(), &start);

        assert_eq!(order.kind(), kind);
        assert_eq!(written.message_type, message_type, "{kind:?}");
        assert_eq!(written.kind, message_kind, "{kind:?}");
        assert_eq!(written.swipe.maker, maker, "{kind:?}");
        assert_eq!(written.swipe.text, "text");
    }
}

#[test]
fn preset_type_by_kind() {
    let rows = [
        (Generation::Narration { guide: None }, PresetType::System),
        (
            Generation::Impersonate { direction: None },
            PresetType::Impersonate,
        ),
        (Generation::InputRetry, PresetType::System),
        (Generation::Event(event_record()), PresetType::System),
    ];

    for (generation, preset_type) in rows {
        assert_eq!(
            generation.preset_type(),
            preset_type,
            "{:?}",
            generation.kind()
        );
    }
}

#[test]
fn write_narration_stores_its_guide() {
    let order = order_of(Generation::Narration {
        guide: Some("go north".to_string()),
    });

    let swipe = order
        .write("text".to_string(), &state_in("room1", None))
        .swipe;

    assert_eq!(swipe.guide_text.as_deref(), Some("go north"));
    assert!(!swipe.impersonated);
}

#[test]
fn write_impersonate_stores_its_direction_and_flag() {
    let order = order_of(Generation::Impersonate {
        direction: Some("be bold".to_string()),
    });

    let swipe = order
        .write("text".to_string(), &state_in("room1", None))
        .swipe;

    assert!(swipe.impersonated);
    assert_eq!(swipe.impersonate_direction.as_deref(), Some("be bold"));
}

#[test]
fn write_bare_impersonate_stores_no_direction() {
    let order = order_of(Generation::Impersonate { direction: None });

    let swipe = order
        .write("text".to_string(), &state_in("room1", None))
        .swipe;

    assert!(swipe.impersonated);
    assert_eq!(swipe.impersonate_direction, None);
}

#[test]
fn write_event_stores_its_header() {
    let order = order_of(Generation::Event(event_record()));

    let swipe = order
        .write("text".to_string(), &state_in("room1", None))
        .swipe;

    assert_eq!(swipe.event_header.as_deref(), Some("The gate opens"));
}

#[test]
fn write_event_marks_one_shot() {
    let started_from = state_in("room1", Some(event_record()));

    let one_shot = order_of(Generation::Event(event_record_repeating(false)))
        .write("text".to_string(), &started_from)
        .swipe
        .state;
    let repeating = order_of(Generation::Event(event_record_repeating(true)))
        .write("text".to_string(), &started_from)
        .swipe
        .state;

    assert!(one_shot.event_record.is_none());
    assert!(one_shot.encounters.is_trigger_fired("character_1", 0));
    assert!(repeating.event_record.is_none());
    assert!(
        !repeating.encounters.is_trigger_fired("character_1", 0),
        "a repeating Trigger is not added to the fired set"
    );
}

#[test]
fn write_narration_clears_the_event_record() {
    let started_from = state_in("room1", Some(event_record()));

    let state = order_of(Generation::Narration { guide: None })
        .write("text".to_string(), &started_from)
        .swipe
        .state;

    assert!(state.event_record.is_none());
    assert_eq!(state.movement.current_room_id, "room1");
}

#[test]
fn write_input_retry_not_analysed() {
    let started_from = state_in("room1", Some(event_record()));

    let written = order_of(Generation::InputRetry).write("text".to_string(), &started_from);

    assert_eq!(written.swipe.state, started_from);
    assert_eq!(written.swipe.maker, Maker::Player);
}

#[test]
fn options_follow() {
    // (run, a Narration was saved, the failure is EntryMoved, the run follows)
    let rows = [
        (OptionsRun::OnDemand, false, false, true),
        (OptionsRun::AfterNarration, true, false, true),
        (OptionsRun::AfterNarration, true, true, false),
        (OptionsRun::AfterNarration, false, false, false),
        (OptionsRun::AfterNarration, false, true, false),
    ];
    for (run, narration_saved, entry_moved, expected) in rows {
        assert_eq!(
            run.follows(narration_saved, entry_moved),
            expected,
            "{run:?}, narration saved {narration_saved}, entry moved {entry_moved}"
        );
    }
}

fn tail_ending_in(message_type: MessageType, kind: MessageKind) -> Tail {
    let mut last = Message::new(message_type, kind, swipe("last", state_in("room1", None)));
    last.id = 9;
    Tail {
        last,
        state_before_last: None,
        count: 2,
    }
}

#[test]
fn next_in_autoplay_continues_an_unanswered_input() {
    let rows = [
        ("a typed Input", MessageKind::TypedInput),
        ("an Impersonate Input", MessageKind::ImpersonateInput),
    ];
    for (label, kind) in rows {
        let tail = tail_ending_in(MessageType::Input, kind);
        let next = JobRequest::next_in_autoplay(Some(&tail));
        assert_eq!(next, JobRequest::Action(String::new()), "{label}");
    }
}

#[test]
fn next_in_autoplay_impersonates_after_any_other_end_of_the_story() {
    let rows = [
        (
            "a Narration",
            Some(tail_ending_in(
                MessageType::Narration,
                MessageKind::Narration,
            )),
        ),
        (
            "an event Narration",
            Some(tail_ending_in(
                MessageType::Narration,
                MessageKind::EventNarration(event_record()),
            )),
        ),
        ("an empty story", None),
    ];
    for (label, tail) in rows {
        let next = JobRequest::next_in_autoplay(tail.as_ref());
        assert_eq!(next, JobRequest::Impersonate(None), "{label}");
    }
}

#[test]
fn a_cycle_ends_where_no_input_waits_for_its_narration() {
    let rows = [
        (
            "a typed Input",
            Some(tail_ending_in(MessageType::Input, MessageKind::TypedInput)),
            false,
        ),
        (
            "an Impersonate Input",
            Some(tail_ending_in(
                MessageType::Input,
                MessageKind::ImpersonateInput,
            )),
            false,
        ),
        (
            "a Narration",
            Some(tail_ending_in(
                MessageType::Narration,
                MessageKind::Narration,
            )),
            true,
        ),
        (
            "an event Narration",
            Some(tail_ending_in(
                MessageType::Narration,
                MessageKind::EventNarration(event_record()),
            )),
            true,
        ),
        ("an empty story", None, false),
    ];
    for (label, tail, ends) in rows {
        let story = StoryEnd {
            tail,
            has_text: false,
        };
        assert_eq!(story.ends_a_cycle(), ends, "{label}");
    }
}
