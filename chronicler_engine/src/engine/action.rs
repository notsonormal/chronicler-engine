#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Look,
    Inventory,
    Talk(String, Option<String>),
    FreeAction(String),
    Quit,
}
