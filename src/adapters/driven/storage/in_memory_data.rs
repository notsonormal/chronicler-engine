//! [DOC: docs/diataxis/reference/storage.md]
//! In-memory backend data structures and their inherent impls

use std::collections::HashMap;

use crate::domain::model::llm_message::LlmMessage;
use crate::domain::model::character::{NpcCard, PersonaCard};
use crate::domain::model::game::Game;
use crate::domain::model::map::MapDef;
use crate::domain::model::message::{Message, Swipe};
use crate::domain::model::prompt_preset::PromptPreset;
use crate::domain::model::settings::AppSettings;
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::domain::model::world::WorldCard;

pub struct InMemoryData {
    pub snapshots: HashMap<u64, Vec<GameStateSnapshot>>,
    pub next_snapshot_id: u64,
    pub games: Vec<Game>,
    pub next_game_id: u64,
    pub messages: HashMap<u64, Vec<Message>>,
    pub next_message_id: u64,
    pub swipes: HashMap<u64, Vec<Swipe>>,
    pub presets: Vec<PromptPreset>,
    pub llm_messages: Vec<LlmMessage>,
    pub worlds: Vec<InMemoryWorld>,
    pub personas: Vec<PersonaCardWithKey>,
    pub characters: Vec<CharacterSeed>,
    pub settings: AppSettings,
}

pub struct InMemoryWorld {
    pub world_id: i64,
    pub world_card: WorldCard,
    pub map: MapDef,
}

pub struct PersonaCardWithKey {
    pub key: String,
    pub card: PersonaCard,
}

pub struct CharacterSeed {
    pub world_id: i64,
    pub card: NpcCard,
}

impl InMemoryData {
    // Throwaway backend for `mem::replace` borrow-checker workaround.
    pub(crate) fn empty() -> Self {
        Self {
            snapshots: HashMap::new(),
            next_snapshot_id: 1,
            games: Vec::new(),
            next_game_id: 1,
            messages: HashMap::new(),
            next_message_id: 0,
            swipes: HashMap::new(),
            presets: Vec::new(),
            llm_messages: Vec::new(),
            worlds: Vec::new(),
            personas: Vec::new(),
            characters: Vec::new(),
            settings: AppSettings::default(),
        }
    }

    pub(crate) fn update_active_swipe(&mut self, game_id: u64, message_id: u64, index: usize) {
        if let Some(msg) = self
            .messages
            .get_mut(&game_id)
            .and_then(|vec| vec.iter_mut().find(|m| m.id == message_id))
        {
            msg.active_swipe_index = index;
        }
    }

    pub(crate) fn soft_delete_message(&mut self, game_id: u64, id: u64) {
        if let Some(msg) = self
            .messages
            .get_mut(&game_id)
            .and_then(|vec| vec.iter_mut().find(|m| m.id == id))
        {
            msg.is_deleted = true;
        }
    }

    pub(crate) fn restore_soft_deleted(&mut self, game_id: u64, ids: &[u64]) {
        if let Some(vec) = self.messages.get_mut(&game_id) {
            for m in vec.iter_mut().filter(|m| ids.contains(&m.id)) {
                m.is_deleted = false;
            }
        }
    }

    pub(crate) fn update_swipe_text(&mut self, message_id: u64, swipe_index: usize, text: &str) {
        if let Some(swipe) = self
            .swipes
            .get_mut(&message_id)
            .and_then(|vec| vec.get_mut(swipe_index))
        {
            swipe.text = text.to_string();
        }
    }

    pub(crate) fn load_swipes_for_messages(&self, message_ids: &[u64]) -> HashMap<u64, Vec<Swipe>> {
        message_ids
            .iter()
            .filter_map(|&msg_id| self.swipes.get(&msg_id).map(|v| (msg_id, v.clone())))
            .collect()
    }
}
