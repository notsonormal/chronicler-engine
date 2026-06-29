//! [DOC: docs/system/game_flow.md]
//! Parser for game data formats

use crate::domain::engine::action::Action;

pub fn parse_command(input: &str) -> Action {
    Action::FreeAction(input.to_string())
}
