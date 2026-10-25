//! [DOC: docs/diataxis/reference/game_flow.md]
//! The end of the story, and the rules derived from it

use crate::domain::model::message::{Message, SwipeAt};
use crate::domain::model::state::message_types::{MessageKind, MessageType};
use crate::domain::model::state::swipe_state::SwipeState;
use crate::domain::model::state::event_record::EventRecord;

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

    /// The event record a Retrigger repeats (R-65): a Narration that is not an event
    /// Narration, or an Impersonate Input, with an event record in its current
    /// state.
    pub fn retrigger_record(&self) -> Option<&EventRecord> {
        let kind_allows = matches!(
            self.last.kind,
            MessageKind::Narration | MessageKind::ImpersonateInput
        );
        if !kind_allows {
            return None;
        }
        self.current_state()?.event_record.as_ref()
    }

    pub fn retrigger_allowed(&self) -> bool {
        self.retrigger_record().is_some()
    }

    pub fn awaits_narration(&self) -> bool {
        self.last.message_type == MessageType::Input
    }

    /// The story keeps its last Message: a delete needs more than one.
    pub fn can_delete(&self) -> bool {
        self.count > 1
    }
}
