//! [DOC: docs/diataxis/reference/game_flow.md]
//! The end of the story, and the rules derived from it

use crate::domain::model::message::{Message, SwipeAt};
use crate::domain::model::state::message_types::MessageKind;
use crate::domain::model::state::swipe_state::SwipeState;

#[derive(Debug, Clone, PartialEq)]
pub struct Tail {
    /// The last Message of the story, with all its Swipes.
    pub last: Message,
    /// The active Swipe of the Message before the last, with its id. The id comes
    /// with the state, because a `SwipeState` carries none and a generation that
    /// starts from it must name the Swipe it starts from.
    pub state_before_last: Option<(SwipeAt, SwipeState)>,
    pub count: usize,
}

impl Tail {
    /// The state of the active Swipe of the last Message, when it holds one.
    pub fn current_state(&self) -> Option<&SwipeState> {
        self.last
            .swipes
            .get(self.last.active_swipe_index)
            .and_then(|swipe| swipe.state.as_ref())
    }

    /// The Swipe a generation of the last Message starts from: the state the last
    /// Message was written from, which is the active Swipe of the Message before
    /// it. A story that holds only its opening starts from the opening's first
    /// Swipe, which is the Game's starting state.
    pub fn start_at(&self) -> SwipeAt {
        match &self.state_before_last {
            Some((at, _)) => *at,
            None => SwipeAt {
                message: self.last.id,
                swipe: 0,
            },
        }
    }

    pub fn retrigger_allowed(&self) -> bool {
        let kind_allows = matches!(
            self.last.kind,
            Some(MessageKind::Narration) | Some(MessageKind::ImpersonateInput)
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
