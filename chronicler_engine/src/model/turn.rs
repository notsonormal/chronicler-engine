use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::model::state::LogEntry;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Turn {
    pub id: String,
    pub input: LogEntry,
    pub swipes: Vec<Swipe>,
    pub active_swipe_index: u32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Swipe {
    pub index: u32,
    pub entries: Vec<LogEntry>,
}

impl Turn {
    pub fn new(input: LogEntry) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            input,
            swipes: vec![Swipe {
                index: 0,
                entries: Vec::new(),
            }],
            active_swipe_index: 0,
            created_at: Utc::now(),
        }
    }

    pub fn active_swipe(&self) -> Option<&Swipe> {
        self.swipes.get(self.active_swipe_index as usize)
    }

    pub fn active_swipe_mut(&mut self) -> Option<&mut Swipe> {
        self.swipes.get_mut(self.active_swipe_index as usize)
    }

    pub fn flattened_entries(&self) -> Vec<LogEntry> {
        let input = if self.input.text.is_empty() {
            Vec::new()
        } else {
            vec![self.input.clone()]
        };
        let swipe_entries = self
            .active_swipe()
            .map(|s| s.entries.clone())
            .unwrap_or_default();
        input.into_iter().chain(swipe_entries).collect()
    }

    /// Create a new empty swipe with the given index and activate it.
    pub fn create_swipe(&mut self, index: u32) {
        self.swipes.push(Swipe {
            index,
            entries: Vec::new(),
        });
        self.active_swipe_index = index;
    }

    /// Create a new swipe copying entries from the currently active swipe,
    /// then activate the new swipe.
    pub fn create_swipe_copying_active(&mut self, index: u32) {
        let entries = self
            .active_swipe()
            .map(|s| s.entries.clone())
            .unwrap_or_default();
        self.swipes.push(Swipe { index, entries });
        self.active_swipe_index = index;
    }
}
