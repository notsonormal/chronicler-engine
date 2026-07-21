//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! TextCheckService orchestrator for text checking

use std::sync::Arc;

use crate::domain::model::settings::TextCheckMode;
use crate::error::EngineError;
use crate::application::ports::text_checker::{CheckResult, TextChecker};

pub struct TextCheckService {
    checker: Arc<dyn TextChecker>,
}

impl TextCheckService {
    pub fn new(checker: Arc<dyn TextChecker>) -> Self {
        Self { checker }
    }

    pub fn check_player_input(
        &self,
        text: &str,
        mode: TextCheckMode,
        ignored_words: &[String],
    ) -> Result<Option<CheckResult>, EngineError> {
        if mode == TextCheckMode::Disabled {
            return Ok(None);
        }
        self.checker.check(text, mode, ignored_words)
    }
}
