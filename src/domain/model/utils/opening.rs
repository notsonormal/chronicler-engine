//! [DOC: docs/diataxis/reference/game_flow.md]
//! The opening Narration of a new Game and the parts only it has.

use crate::domain::model::message::{Maker, Message, Swipe};
use crate::domain::model::state::message_types::{MessageKind, MessageType};
use crate::domain::model::state::swipe_state::SwipeState;
use crate::domain::model::template::TemplateVars;
use crate::domain::model::world::WorldCard;

/// The opening Narration: the World's default scenario rendered for the Persona,
/// and made by the World rather than a model.
///
/// The state the Swipe holds is the caller's: the create unit derives it from the
/// World, and the test builder seeds its own.
pub fn opening_message(world: &WorldCard, persona_name: &str, state: SwipeState) -> Message {
    let scenario = world.default_scenario();
    let text = scenario
        .map(|scenario| TemplateVars::new(persona_name).render(&scenario.text))
        .unwrap_or_default();
    let swipe = Swipe::new(text, Maker::WorldOpening, state);
    Message::new(MessageType::Narration, MessageKind::Narration, swipe)
}
