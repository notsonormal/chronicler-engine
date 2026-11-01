//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! The generation parameters of a preset, and the settings one model call carries

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::domain::model::game::GameId;
use crate::domain::model::settings::DEFAULT_MAX_TOKENS;

/// The keys the engine writes into every request body.
pub const ENGINE_KEYS: [&str; 4] = ["model", "messages", "stream", "max_tokens"];

/// The samplers a preset sets in its own fields, so a second copy in `custom` would be
/// a second source for one value.
const TYPED_KEYS: [&str; 2] = ["temperature", "top_p"];

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GenerationParameters {
    pub max_tokens: Option<u32>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    /// Other samplers, written into the request body as they are.
    pub custom: Map<String, Value>,
}

impl GenerationParameters {
    pub fn refused_custom_key(&self) -> Option<&'static str> {
        ENGINE_KEYS
            .into_iter()
            .chain(TYPED_KEYS)
            .find(|key| self.custom.contains_key(*key))
    }
}

/// The settings of one model call. Unlike a preset's `max_tokens`, this `max_tokens` is the
/// value the engine sends.
#[derive(Debug, Clone, PartialEq)]
pub struct CallSettings {
    pub max_tokens: u32,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub custom: Map<String, Value>,
    pub game: Option<GameId>,
}

impl Default for CallSettings {
    fn default() -> Self {
        Self {
            max_tokens: DEFAULT_MAX_TOKENS,
            temperature: None,
            top_p: None,
            custom: Map::new(),
            game: None,
        }
    }
}

impl CallSettings {
    pub fn of(parameters: &GenerationParameters, max_tokens: u32, game: GameId) -> Self {
        Self {
            max_tokens,
            temperature: parameters.temperature,
            top_p: parameters.top_p,
            custom: parameters.custom.clone(),
            game: Some(game),
        }
    }
}
