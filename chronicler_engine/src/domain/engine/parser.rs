//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! Parser for game data formats

use crate::domain::engine::action::Action;

pub fn parse_action(input: &str) -> Action {
    Action::FreeAction(input.to_string())
}
