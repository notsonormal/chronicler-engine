//! [DOC: docs/diataxis/reference/storage.md]
//! Message types and conversation history (Message, Swipe, stored generation inputs)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::domain::model::state::message_types::{MessageKind, MessageType};
use crate::domain::model::state::swipe_state::SwipeState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwipeAt {
    pub message: u64,
    pub swipe: usize,
}

/// What the player asked for when the Swipe's text was written.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StoredInputs {
    pub impersonated: bool,
    pub guide_text: Option<String>,
    pub impersonate_direction: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Maker {
    /// The World's opening text, written at create.
    WorldOpening,
    /// The player.
    Player,
    /// The narrator.
    Narrator,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Swipe {
    pub text: String,
    pub room_report: Option<String>,
    pub event_header: Option<String>,
    #[serde(default)]
    pub impersonated: bool,
    #[serde(default)]
    pub guide_text: Option<String>,
    #[serde(default)]
    pub impersonate_direction: Option<String>,
    pub state: SwipeState,
    pub maker: Maker,
}

impl Swipe {
    pub fn new(text: impl Into<String>, maker: Maker, state: SwipeState) -> Self {
        Self {
            text: text.into(),
            room_report: None,
            event_header: None,
            impersonated: false,
            guide_text: None,
            impersonate_direction: None,
            state,
            maker,
        }
    }

    pub fn stored_inputs(&self) -> StoredInputs {
        StoredInputs {
            impersonated: self.impersonated,
            guide_text: self.guide_text.clone(),
            impersonate_direction: self.impersonate_direction.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub id: u64,
    pub message_type: MessageType,
    pub kind: MessageKind,
    pub timestamp: DateTime<Utc>,
    pub active_swipe_index: usize,
    pub swipes: Vec<Swipe>,
}

impl Message {
    /// A Message of one Swipe. The id is the storage's to give.
    pub fn new(message_type: MessageType, kind: MessageKind, swipe: Swipe) -> Self {
        Self {
            id: 0,
            message_type,
            kind,
            timestamp: Utc::now(),
            active_swipe_index: 0,
            swipes: vec![swipe],
        }
    }

    pub fn swipe_count(&self) -> usize {
        self.swipes.len()
    }

    fn active_swipe(&self) -> Option<&Swipe> {
        self.swipes.get(self.active_swipe_index)
    }

    fn active_swipe_mut(&mut self) -> Option<&mut Swipe> {
        self.swipes.get_mut(self.active_swipe_index)
    }

    pub fn text(&self) -> &str {
        self.active_swipe().map(|s| s.text.as_str()).unwrap_or("")
    }

    pub fn room_report(&self) -> Option<&str> {
        self.active_swipe().and_then(|s| s.room_report.as_deref())
    }

    pub fn event_header(&self) -> Option<&str> {
        self.active_swipe().and_then(|s| s.event_header.as_deref())
    }

    pub fn impersonated(&self) -> bool {
        self.active_swipe().is_some_and(|s| s.impersonated)
    }

    pub fn guide_text(&self) -> Option<&str> {
        self.active_swipe().and_then(|s| s.guide_text.as_deref())
    }

    pub fn impersonate_direction(&self) -> Option<&str> {
        self.active_swipe()
            .and_then(|s| s.impersonate_direction.as_deref())
    }

    pub fn is_input_or_narration(&self) -> bool {
        self.message_type == MessageType::Narration || self.message_type == MessageType::Input
    }

    pub fn is_guided(&self) -> bool {
        self.guide_text().is_some()
    }

    pub fn set_active_swipe(&mut self, index: usize) {
        if index >= self.swipes.len() {
            return;
        }
        self.active_swipe_index = index;
    }

    pub fn update_active_swipe_text(&mut self, new_text: impl Into<String>) {
        let new_text = new_text.into();
        if let Some(swipe) = self.active_swipe_mut() {
            swipe.text = new_text;
        }
    }

    pub(crate) fn from_db(
        id: u64,
        message_type: MessageType,
        kind: MessageKind,
        timestamp: DateTime<Utc>,
        active_swipe_index: usize,
    ) -> Self {
        Self {
            id,
            message_type,
            kind,
            timestamp,
            active_swipe_index,
            swipes: Vec::new(),
        }
    }

    pub fn ensure_valid_swipe_index(&mut self) -> bool {
        if self.active_swipe_index >= self.swipes.len() {
            self.active_swipe_index = 0;
            true
        } else {
            false
        }
    }
}

impl Message {
    #[cfg(test)]
    pub(crate) fn set_event_header(&mut self, header: Option<String>) {
        if let Some(swipe) = self.active_swipe_mut() {
            swipe.event_header = header;
        }
    }

    pub fn set_stored_inputs(
        &mut self,
        impersonated: bool,
        guide_text: Option<String>,
        impersonate_direction: Option<String>,
    ) {
        if let Some(swipe) = self.active_swipe_mut() {
            swipe.impersonated = impersonated;
            swipe.guide_text = guide_text;
            swipe.impersonate_direction = impersonate_direction;
        }
    }
}
