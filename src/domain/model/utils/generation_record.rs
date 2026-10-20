//! [DOC: docs/diataxis/reference/game_flow.md]
//! What a generation writes as a record: its Message kind, its message type and its maker

use crate::domain::model::agent::Kind;
use crate::domain::model::message::Maker;
use crate::domain::model::state::message_types::{MessageKind, MessageType};
use crate::domain::model::state::trigger_context::EventRecord;

/// The kind of Message a generation of `kind` writes, from the event record it was
/// ordered with. An event generation without a record writes none.
pub fn message_kind(kind: Kind, event: Option<&EventRecord>) -> Option<MessageKind> {
    match kind {
        Kind::Narration => Some(MessageKind::Narration),
        Kind::Event => event.cloned().map(MessageKind::EventNarration),
        Kind::Impersonate => Some(MessageKind::ImpersonateInput),
        Kind::UserRegen => Some(MessageKind::TypedInput),
    }
}

/// The `message_type` column a generation writes, beside the kind itself.
pub fn message_type(kind: Kind) -> MessageType {
    match kind {
        Kind::Narration | Kind::Event => MessageType::Narration,
        Kind::Impersonate | Kind::UserRegen => MessageType::Input,
    }
}

/// Who writes the text of a generation: the player for an Input, the narrator
/// otherwise.
pub fn maker(kind: Kind) -> Maker {
    match kind {
        Kind::Impersonate | Kind::UserRegen => Maker::Player,
        Kind::Narration | Kind::Event => Maker::Narrator,
    }
}
