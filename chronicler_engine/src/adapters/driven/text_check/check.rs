//! [DOC: docs/system/text_check.md]
//! Text check execution

use crate::error::EngineError;
use crate::domain::model::settings::TextCheckMode;

use super::harper_backend::HarperBackend;
use super::types::CheckResult;

/// Check player input text for spelling and grammar issues.
/// Returns `None` when the mode is `Disabled` or no issues are found.
pub fn check_player_input(
    text: &str,
    mode: TextCheckMode,
    ignored_words: &[String],
) -> Result<Option<CheckResult>, EngineError> {
    if mode == TextCheckMode::Disabled {
        return Ok(None);
    }

    let backend = HarperBackend::new(ignored_words);
    backend.check(text, mode)
}
