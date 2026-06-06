//! [DOC: docs/system/game_flow.md]

use crate::engine::action::Action;

pub fn parse_command(input: &str) -> Action {
    Action::FreeAction(input.to_string())
}
