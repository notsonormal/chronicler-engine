//! [DOC: docs/diataxis/reference/game_flow.md]
//! The order of one generation: ids and texts only

use crate::domain::model::agent::Kind;
use crate::domain::model::message::{StoredInputs, SwipeAt};
use crate::domain::model::state::trigger_context::EventRecord;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Place {
    /// A new Message at the end of the story.
    NewMessage,
    /// A new Swipe on the last Message.
    NewSwipeOnLast,
}

/// No Swipe state travels with the order: the generation's first step loads the
/// state of `start` itself.
#[derive(Debug, Clone, PartialEq)]
pub struct GenerationOrder {
    pub kind: Kind,
    pub place: Place,
    pub start: SwipeAt,
    pub inputs: StoredInputs,
    pub event: Option<EventRecord>,
}
