//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! The memory entries: the written record of the Messages that left the history, and the newest Message they cover

use crate::domain::model::state::message_types::StoryRow;

/// One scene of the story. The engine sets `from` and `to`, so the entries of one writer
/// call cover its Messages with no gap and no overlap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryEntry {
    /// The id of the first Message of the scene.
    pub from: u64,
    /// The id of the last Message of the scene.
    pub to: u64,
    pub time: String,
    pub place: String,
    pub took_part: Vec<String>,
    pub others: Vec<String>,
    pub what_happened: String,
}

/// The memory entries of the active Swipes before a place, oldest first.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MemoryEntries(Vec<MemoryEntry>);

impl MemoryEntries {
    pub fn new(entries: Vec<MemoryEntry>) -> Self {
        Self(entries)
    }

    pub fn none() -> &'static Self {
        static NONE: MemoryEntries = MemoryEntries(Vec::new());
        &NONE
    }

    pub fn entries(&self) -> &[MemoryEntry] {
        &self.0
    }

    /// The last memory entry written before: the writer reads it as the scene that came
    /// just before its own Messages.
    pub fn last(&self) -> Option<&MemoryEntry> {
        self.0.last()
    }

    /// The id of the newest Message that the entries cover. Ids only grow, so a delete
    /// of a newer Message leaves the mark true.
    pub fn covers_up_to(&self) -> Option<u64> {
        self.0.last().map(|entry| entry.to)
    }

    /// `dropped` must be in story order: the cover mark is found by binary search.
    pub fn uncovered<'a>(&self, dropped: &'a [StoryRow]) -> &'a [StoryRow] {
        let Some(covers_up_to) = self.covers_up_to() else {
            return dropped;
        };
        let first_uncovered = dropped.partition_point(|row| row.id <= covers_up_to);
        &dropped[first_uncovered..]
    }

    pub fn with_written(&self, written: &[MemoryEntry]) -> Self {
        let mut entries = self.0.clone();
        entries.extend_from_slice(written);
        Self(entries)
    }
}
