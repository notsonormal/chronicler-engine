//! [DOC: docs/system/game_flow.md]
//! Action enum and semantic command types

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    FreeAction(String),
}
