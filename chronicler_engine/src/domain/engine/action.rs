//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! Action enum and semantic command types

/// [TRIVIAL_ENUM]
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    FreeAction(String),
}
