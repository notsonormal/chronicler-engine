//! [DOC: docs/diataxis/reference/storage.md]
//! In-memory backend data structures and their inherent impls

use crate::domain::model::llm_message::LlmMessage;
use crate::domain::model::character::{NpcCard, PersonaCard};
use crate::domain::model::game::Game;
use crate::domain::model::map::MapDef;
use crate::domain::model::prompt_preset::PromptPreset;
use crate::domain::model::settings::AppSettings;
use crate::domain::model::world::WorldCard;

pub struct InMemoryData {
    pub games: Vec<Game>,
    pub next_game_id: u64,
    pub presets: Vec<PromptPreset>,
    pub llm_messages: Vec<LlmMessage>,
    pub worlds: Vec<InMemoryWorld>,
    pub next_world_id: i64,
    pub personas: Vec<PersonaCardWithKey>,
    pub characters: Vec<CharacterSeed>,
    pub settings: AppSettings,
    /// `false` means `settings` is the constructed default standing in for an
    /// absent row, so `seed_settings` may still write.
    pub settings_seeded: bool,
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
    pub(crate) fn empty() -> Self {
        Self {
            games: Vec::new(),
            next_game_id: 1,
            presets: Vec::new(),
            llm_messages: Vec::new(),
            worlds: Vec::new(),
            next_world_id: 1,
            personas: Vec::new(),
            characters: Vec::new(),
            settings: AppSettings::default(),
            settings_seeded: false,
        }
    }
}
