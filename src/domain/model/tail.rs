//! [DOC: docs/diataxis/reference/game_flow.md]
//! The end of the story, and the rules derived from it

use crate::domain::model::agent::Kind;
use crate::domain::model::generation::{GenerationOrder, Place};
use crate::domain::model::message::{Message, StoredInputs, Swipe, SwipeAt};
use crate::domain::model::state::message_types::MessageKind;
use crate::domain::model::state::swipe_state::SwipeState;
use crate::domain::model::state::trigger_context::EventRecord;

#[derive(Debug, Clone, PartialEq)]
pub struct Tail {
    pub last: Message,
    /// The active Swipe of the Message before the last, with its id. The id comes
    /// with the state, because a `SwipeState` carries none and a generation that
    /// starts from it must name the Swipe it starts from.
    pub state_before_last: Option<(SwipeAt, SwipeState)>,
    pub count: usize,
}

impl Tail {
    pub fn current_state(&self) -> Option<&SwipeState> {
        self.last
            .swipes
            .get(self.last.active_swipe_index)
            .map(|swipe| &swipe.state)
    }

    pub fn current_at(&self) -> SwipeAt {
        SwipeAt {
            message: self.last.id,
            swipe: self.last.active_swipe_index,
        }
    }

    /// The state the last Message was written from: the active Swipe of the Message
    /// before it, or the opening's first Swipe for a story that holds only its
    /// opening.
    pub fn start_at(&self) -> SwipeAt {
        match &self.state_before_last {
            Some((at, _)) => *at,
            None => SwipeAt {
                message: self.last.id,
                swipe: 0,
            },
        }
    }

    /// The order of the last Message again, of the kind the last Message holds.
    pub fn redo(&self) -> GenerationOrder {
        let kind = self.redo_kind();
        let last_swipe = self.last.swipes.get(self.last.active_swipe_index);
        GenerationOrder {
            kind,
            place: Place::NewSwipeOnLast,
            start: self.start_at(),
            inputs: match kind {
                // The opening re-applies nothing the player asked for.
                Kind::Narration if self.state_before_last.is_none() => StoredInputs::default(),
                _ => last_swipe.map(Swipe::stored_inputs).unwrap_or_default(),
            },
            event: self.event_record(),
        }
    }

    /// The event Narration as a new Message at the end of the story, from the
    /// event record in the trigger context of the current state.
    pub fn retrigger(&self) -> Option<GenerationOrder> {
        if !self.retrigger_allowed() {
            return None;
        }
        Some(GenerationOrder {
            kind: Kind::Event,
            place: Place::NewMessage,
            start: self.current_at(),
            inputs: StoredInputs::default(),
            event: self.current_state()?.trigger_context.clone(),
        })
    }

    fn redo_kind(&self) -> Kind {
        match self.last.kind {
            MessageKind::Narration => Kind::Narration,
            MessageKind::EventNarration(_) => Kind::Event,
            MessageKind::ImpersonateInput => Kind::Impersonate,
            MessageKind::TypedInput => Kind::UserRegen,
        }
    }

    fn event_record(&self) -> Option<EventRecord> {
        match &self.last.kind {
            MessageKind::EventNarration(record) => Some(record.clone()),
            _ => None,
        }
    }

    pub fn retrigger_allowed(&self) -> bool {
        let kind_allows = matches!(
            self.last.kind,
            MessageKind::Narration | MessageKind::ImpersonateInput
        );
        kind_allows
            && self
                .current_state()
                .is_some_and(|state| state.trigger_context.is_some())
    }

    /// The story keeps its last Message: a delete needs more than one.
    pub fn can_delete(&self) -> bool {
        self.count > 1
    }
}
