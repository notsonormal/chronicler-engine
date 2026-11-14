//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! The story memory: the summary of the Messages that left the history, and the newest Message it covers

use serde::{Deserialize, Serialize};

use crate::domain::model::state::message_types::StoryRow;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoryMemory {
    pub text: String,
    /// The id of the newest Message that the text covers. Ids only grow, so a
    /// delete of a newer Message leaves the mark true.
    pub covers_up_to: u64,
}

impl StoryMemory {
    /// `dropped` must be in story order: the cover mark is found by binary search.
    pub fn uncovered<'a>(memory: Option<&Self>, dropped: &'a [StoryRow]) -> &'a [StoryRow] {
        let Some(memory) = memory else {
            return dropped;
        };
        let first_uncovered = dropped.partition_point(|row| row.id <= memory.covers_up_to);
        &dropped[first_uncovered..]
    }
}
