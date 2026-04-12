#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Look,
    WalkTo(String),
    Inventory,
    Talk(String, Option<String>),
    FreeAction(String),
    Quit,
}
