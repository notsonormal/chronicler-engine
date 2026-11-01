//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! The generation parameters of a preset, and the settings one model call carries

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::domain::model::game::GameId;

/// The request keys the engine writes itself. A preset cannot set them in `custom`.
const ENGINE_KEYS: [&str; 4] = ["model", "messages", "stream", "max_tokens"];

/// The generation parameters of a preset. A value that is not set is not sent.
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
    /// The first key of `custom` that the engine owns, if any.
    pub fn refused_custom_key(&self) -> Option<&'static str> {
        ENGINE_KEYS
            .into_iter()
            .find(|key| self.custom.contains_key(*key))
    }
}

/// What one model call carries besides its messages: the parameters after the
/// engine resolved them, and the Game the call belongs to.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CallSettings {
    pub max_tokens: Option<u32>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub custom: Map<String, Value>,
    pub game: Option<GameId>,
}

impl CallSettings {
    /// The settings of a call: the samplers of a preset, `max_tokens` as the
    /// engine resolved it, and the Game of the Job.
    pub fn of(parameters: &GenerationParameters, max_tokens: u32, game: GameId) -> Self {
        Self {
            max_tokens: Some(max_tokens),
            temperature: parameters.temperature,
            top_p: parameters.top_p,
            custom: parameters.custom.clone(),
            game: Some(game),
        }
    }
}
