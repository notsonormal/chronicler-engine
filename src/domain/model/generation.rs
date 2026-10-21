//! [DOC: docs/diataxis/reference/game_flow.md]
//! The order of one generation, and the plan rules that turn one request into a plan

use crate::domain::model::agent::Kind;
use crate::domain::model::message::{Maker, Swipe, SwipeAt};
use crate::domain::model::prompt_preset::PresetType;
use crate::domain::model::state::generation_status::{Record, Step};
use crate::domain::model::state::message_types::{MessageKind, MessageType};
use crate::domain::model::state::swipe_state::SwipeState;
use crate::domain::model::state::trigger_context::EventRecord;
use crate::domain::model::tail::Tail;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Place {
    /// A new Message at the end of the story.
    NewMessage,
    /// A new Swipe on the last Message.
    NewSwipeOnLast,
}

/// What one generation is, with the data that only its kind has.
///
/// [TRIVIAL_ENUM]
#[derive(Debug, Clone, PartialEq)]
pub enum Generation {
    Narration { guide: Option<String> },
    Impersonate { direction: Option<String> },
    UserRegen,
    Event(EventRecord),
}

impl Generation {
    pub fn kind(&self) -> Kind {
        match self {
            Self::Narration { .. } => Kind::Narration,
            Self::Impersonate { .. } => Kind::Impersonate,
            Self::UserRegen => Kind::UserRegen,
            Self::Event(_) => Kind::Event,
        }
    }

    /// The kind of preset the narration call renders: an Impersonate has its own.
    pub fn preset_type(&self) -> PresetType {
        match self {
            Self::Impersonate { .. } => PresetType::Impersonate,
            _ => PresetType::System,
        }
    }
}

/// The order holds ids and texts only: the first step of the generation loads the
/// Swipe state of `start` itself.
#[derive(Debug, Clone, PartialEq)]
pub struct GenerationOrder {
    pub generation: Generation,
    pub place: Place,
    pub start: SwipeAt,
}

/// What a generation writes for its text. The Message parts apply when the order
/// places a new Message, and `swipe` is the Swipe in both places.
#[derive(Debug, Clone, PartialEq)]
pub struct NewSwipe {
    pub message_type: MessageType,
    pub kind: MessageKind,
    pub swipe: Swipe,
}

impl GenerationOrder {
    pub fn kind(&self) -> Kind {
        self.generation.kind()
    }

    /// The event Narration of a fired Trigger, a new Message after the Message the
    /// firing generation saved.
    pub fn event_after(saved: SwipeAt, record: EventRecord) -> Self {
        Self {
            generation: Generation::Event(record),
            place: Place::NewMessage,
            start: saved,
        }
    }

    /// A user regen ends after its text save: the Quantifier and the Game rules
    /// read a Narration or an Impersonate Input, not a typed Input again.
    pub fn is_analysed(&self) -> bool {
        !matches!(self.generation, Generation::UserRegen)
    }

    /// The one home of the Message kind, type, maker, stored inputs and event
    /// header. The new Swipe holds the state the Game rules read: the start state
    /// with no trigger context, and for an event its one-shot Trigger marked fired.
    /// A user regen keeps the start state as it is.
    pub fn write(&self, text: String, start: &SwipeState) -> NewSwipe {
        match &self.generation {
            Generation::Narration { guide } => NewSwipe {
                message_type: MessageType::Narration,
                kind: MessageKind::Narration,
                swipe: Swipe {
                    guide_text: guide.clone(),
                    ..Swipe::new(text, Maker::Narrator, Self::without_context(start))
                },
            },
            Generation::Impersonate { direction } => NewSwipe {
                message_type: MessageType::Input,
                kind: MessageKind::ImpersonateInput,
                swipe: Swipe {
                    impersonated: true,
                    impersonate_direction: direction.clone(),
                    ..Swipe::new(text, Maker::Player, Self::without_context(start))
                },
            },
            Generation::UserRegen => NewSwipe {
                message_type: MessageType::Input,
                kind: MessageKind::TypedInput,
                swipe: Swipe::new(text, Maker::Player, start.clone()),
            },
            Generation::Event(record) => {
                let mut state = Self::without_context(start);
                if !record.trigger.trigger_repeat {
                    state
                        .npc_encounter_log
                        .mark_trigger_fired(&record.trigger.npc_id, record.trigger.trigger_idx);
                }
                NewSwipe {
                    message_type: MessageType::Narration,
                    kind: MessageKind::EventNarration(record.clone()),
                    swipe: Swipe {
                        event_header: Some(record.header_name.clone()),
                        ..Swipe::new(text, Maker::Narrator, state)
                    },
                }
            }
        }
    }

    fn without_context(start: &SwipeState) -> SwipeState {
        SwipeState {
            trigger_context: None,
            ..start.clone()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum JobRequest {
    /// Free text the player typed. Empty text is a Continue.
    Action(String),
    /// `/guide`: a steering direction that shapes the next Narration.
    Guide(String),
    /// `/impersonate`: the player dictates their own Persona's Message.
    Impersonate(Option<String>),
    /// `/options`: the Options run alone.
    Options,
    /// A Retry of the last Message, of the kind the last Message holds.
    Retry,
    /// A Retrigger of the Trigger the last Message holds.
    Retrigger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionsRun {
    /// The run follows the Job's last Narration, when the toggle is on.
    AfterNarration,
    /// The run is the whole Job: `/options`.
    OnDemand,
}

impl OptionsRun {
    /// Whether the run happens at the end of the Job. `OnDemand` always runs. After
    /// a Narration, a saved Narration earns the run, also when a later step failed;
    /// a Job whose entry moved on runs nothing more.
    pub fn follows(self, narration_saved: bool, entry_moved: bool) -> bool {
        match self {
            Self::OnDemand => true,
            Self::AfterNarration => narration_saved && !entry_moved,
        }
    }
}

/// The order of one request: ids and texts only.
#[derive(Debug, Clone, PartialEq)]
pub struct JobPlan {
    /// The free text to save before the first generation, when the request carries
    /// any.
    pub input: Option<String>,
    /// The first generation. `None` for `/options`.
    pub first: Option<GenerationOrder>,
    pub options: OptionsRun,
}

/// What the plan rules read of the story: its last Message with the state before
/// it, and whether any Message holds text.
#[derive(Debug, Clone, PartialEq)]
pub struct StoryEnd {
    pub tail: Option<Tail>,
    pub has_text: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanRefusal {
    /// The request is wrong; the line says why.
    BadRequest(String),
    /// A record the request needs is missing; the failure line names it.
    Missing(Record),
}

impl JobPlan {
    pub fn for_request(request: &JobRequest, story: StoryEnd) -> Result<Self, PlanRefusal> {
        let StoryEnd { tail, has_text } = story;
        match request {
            JobRequest::Options if !has_text => Err(PlanRefusal::BadRequest(
                "No scene to generate options for yet".to_string(),
            )),
            JobRequest::Options => Ok(Self {
                input: None,
                first: None,
                options: OptionsRun::OnDemand,
            }),
            JobRequest::Retry => {
                let tail =
                    tail.ok_or_else(|| PlanRefusal::BadRequest("No input to retry".to_string()))?;
                Ok(Self::of_one(Self::redo(&tail), None))
            }
            JobRequest::Retrigger => {
                let refused =
                    || PlanRefusal::BadRequest("No trigger context available".to_string());
                let tail = tail.ok_or_else(refused)?;
                let record = tail.retrigger_record().ok_or_else(refused)?.clone();
                Ok(Self::of_one(
                    GenerationOrder::event_after(tail.current_at(), record),
                    None,
                ))
            }
            JobRequest::Action(text) => {
                let input = (!text.is_empty()).then(|| text.clone());
                Self::narration_like(tail.as_ref(), Generation::Narration { guide: None }, input)
            }
            JobRequest::Guide(guide) => Self::narration_like(
                tail.as_ref(),
                Generation::Narration {
                    guide: Some(guide.clone()),
                },
                None,
            ),
            JobRequest::Impersonate(direction) => Self::narration_like(
                tail.as_ref(),
                Generation::Impersonate {
                    direction: direction.clone(),
                },
                None,
            ),
        }
    }

    /// The step the status shows from the accept, until the Job names its own: a
    /// Retry of an event Narration starts in the event step.
    pub fn first_step(&self) -> Step {
        match &self.first {
            Some(order) => order.kind().step(),
            None => Step::Options,
        }
    }

    fn of_one(order: GenerationOrder, input: Option<String>) -> Self {
        Self {
            input,
            first: Some(order),
            options: OptionsRun::AfterNarration,
        }
    }

    fn narration_like(
        tail: Option<&Tail>,
        generation: Generation,
        input: Option<String>,
    ) -> Result<Self, PlanRefusal> {
        let start = tail
            .map(Tail::current_at)
            .ok_or(PlanRefusal::Missing(Record::Story))?;
        Ok(Self::of_one(
            GenerationOrder {
                generation,
                place: Place::NewMessage,
                start,
            },
            input,
        ))
    }

    /// The order of the last Message again, of the kind the last Message holds.
    fn redo(tail: &Tail) -> GenerationOrder {
        let last_swipe = tail.last.swipes.get(tail.last.active_swipe_index);
        let generation = match &tail.last.kind {
            // The opening re-applies nothing the player asked for.
            MessageKind::Narration if tail.state_before_last.is_none() => {
                Generation::Narration { guide: None }
            }
            MessageKind::Narration => Generation::Narration {
                guide: last_swipe.and_then(|swipe| swipe.guide_text.clone()),
            },
            MessageKind::EventNarration(record) => Generation::Event(record.clone()),
            MessageKind::ImpersonateInput => Generation::Impersonate {
                direction: last_swipe.and_then(|swipe| swipe.impersonate_direction.clone()),
            },
            MessageKind::TypedInput => Generation::UserRegen,
        };
        GenerationOrder {
            generation,
            place: Place::NewSwipeOnLast,
            start: tail.start_at(),
        }
    }
}
