//! [DOC: docs/diataxis/reference/game_flow.md]
//! The opening Narration of a new Game and the parts only it has.

use crate::domain::model::agent::Kind;
use crate::domain::model::map::MapDef;
use crate::domain::model::message::{Maker, Message};
use crate::domain::model::state::message_types::MessageKind;
use crate::domain::model::template::TemplateVars;
use crate::domain::model::world::WorldCard;
use crate::domain::model::utils::generation_record;

/// The opening Narration: the World's default scenario rendered for the Persona,
/// reported from the starting room, and made by the World rather than a model.
///
/// The Swipe it writes carries no state, because the state a Game starts from is
/// the caller's: the create unit derives it from the World, and the test builder
/// seeds its own.
pub fn opening_message(world: &WorldCard, map: &MapDef, persona_name: &str) -> Message {
    let scenario = world.default_scenario();
    let text = scenario
        .map(|scenario| TemplateVars::new(persona_name).render(&scenario.text))
        .unwrap_or_default();
    let starting_room = world.starting_room_id();
    let room_report = map
        .get_room_by_id(&starting_room)
        .map(|room| room.name.clone())
        .unwrap_or(starting_room);

    let mut message = Message::new(
        text,
        generation_record::message_type(Kind::Narration),
        None,
        None,
    );
    message.kind = Some(MessageKind::Narration);
    message.swipes[0].room_report = Some(room_report);
    message.swipes[0].maker = Some(Maker::WorldOpening);
    message
}
