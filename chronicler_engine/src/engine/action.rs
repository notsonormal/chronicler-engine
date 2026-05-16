#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Talk(String, Option<String>),
    FreeAction(String),
}
