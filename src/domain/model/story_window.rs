//! [DOC: docs/diataxis/reference/game_flow.md]
//! The story window: the newest entries of the story, with the room each one shows, and the end-of-story facts that go with them

use crate::domain::model::state::message_types::StoryRow;

/// The entries are oldest first, and the last one is the last Message of the story.
#[derive(Debug, Clone, PartialEq)]
pub struct StoryWindow {
    pub entries: Vec<StoryWindowEntry>,
    pub retrigger_allowed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StoryWindowEntry {
    pub row: StoryRow,
    /// The room name the entry shows above its text.
    pub room_header: Option<String>,
}

impl StoryWindow {
    /// The room name each entry shows, from the room name of each entry's active Swipe
    /// state, oldest first. An entry shows its name when it differs from the name of the
    /// entry before it, and the oldest entry always shows its name. An entry with no
    /// room name shows nothing, so the next entry with a name shows it.
    pub fn room_headers(room_names: &[Option<String>]) -> Vec<Option<String>> {
        let mut before: Option<&String> = None;
        room_names
            .iter()
            .map(|name| {
                let shown = match name {
                    Some(name) if before != Some(name) => Some(name.clone()),
                    _ => None,
                };
                before = name.as_ref();
                shown
            })
            .collect()
    }
}
