use crate::engine::action::Action;

pub fn parse_command(input: &str) -> Action {
    // [DOC: docs/architecture/system.md]
    Action::FreeAction(input.to_string())
}
