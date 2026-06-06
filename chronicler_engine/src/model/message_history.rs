//! [DOC: docs/system/game_flow.md]
//! Message history tracking

use serde::{Deserialize, Serialize};

use crate::model::message::Message;
use crate::model::state::{MessageEntry, MessageType};

const MAX_MESSAGES: usize = 1000;

/// Owns `Vec<Message>` and all operations on it. Callers cannot bypass
/// rules with direct `.push()`.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MessageHistory {
    messages: Vec<Message>,
}

impl MessageHistory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_messages(messages: Vec<Message>) -> Self {
        Self { messages }
    }

    pub fn append(&mut self, message: Message) {
        if self.messages.len() >= MAX_MESSAGES {
            self.messages.remove(0);
        }
        self.messages.push(message);
    }

    pub fn edit(&mut self, id: u64, new_text: String) -> crate::error::Result<()> {
        if let Some(msg) = self.messages.iter_mut().find(|m| m.id == id) {
            msg.update_active_swipe_text(new_text);
            Ok(())
        } else {
            Err(crate::error::EngineError::Internal(
                crate::error::internal_error(format!("Message entry not found: {id}")),
            ))
        }
    }

    pub fn delete_last(&mut self) -> crate::error::Result<()> {
        if self.messages.is_empty() {
            return Err(crate::error::EngineError::Internal(
                crate::error::internal_error("History is empty".to_string()),
            ));
        }

        self.messages.pop();
        Ok(())
    }

    pub fn get(&self, id: u64) -> Option<&Message> {
        self.messages.iter().find(|m| m.id == id)
    }

    pub fn last(&self) -> Option<&Message> {
        self.messages.last()
    }

    pub fn last_mut(&mut self) -> Option<&mut Message> {
        self.messages.last_mut()
    }

    pub fn is_last(&self, id: u64) -> bool {
        self.messages.last().map(|m| m.id == id).unwrap_or(false)
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Message> {
        self.messages.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Message> {
        self.messages.iter_mut()
    }

    pub fn retain(&mut self, f: impl FnMut(&Message) -> bool) {
        self.messages.retain(f);
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

    pub fn as_slice(&self) -> &[Message] {
        &self.messages
    }

    pub fn replace(&mut self, messages: Vec<Message>) {
        self.messages = messages;
    }

    pub fn last_ai_response_index(&self) -> Option<usize> {
        self.messages.iter().rposition(|m| {
            m.message_type == MessageType::Narration || m.message_type == MessageType::Dialogue
        })
    }

    pub fn last_input_index(&self) -> Option<usize> {
        self.messages
            .iter()
            .rposition(|m| m.message_type == MessageType::Input)
    }

    pub fn last_input_text(&self) -> Option<(String, String)> {
        let input = self
            .messages
            .iter()
            .rev()
            .find(|m| m.message_type == MessageType::Input)?;
        let sender = input.sender.clone().unwrap_or_default();
        Some((sender, input.text().to_string()))
    }

    pub fn is_last_ai_response_event_continuation(&self) -> bool {
        self.messages
            .iter()
            .rev()
            .find(|m| {
                m.message_type == MessageType::Narration || m.message_type == MessageType::Dialogue
            })
            .is_some_and(|m| m.event_header().is_some())
    }

    pub fn to_message_entries(&self) -> Vec<MessageEntry> {
        self.messages
            .iter()
            .map(|msg| MessageEntry {
                id: msg.id,
                sender: msg.sender.clone(),
                text: msg.text().to_string(),
                message_type: msg.message_type.clone(),
                timestamp: msg.timestamp,
                location_header: msg.location_header().map(|s| s.to_string()),
                event_header: msg.event_header().map(|s| s.to_string()),
                swipe_count: msg.swipe_count(),
                active_swipe_index: msg.active_swipe_index,
            })
            .collect()
    }
}
