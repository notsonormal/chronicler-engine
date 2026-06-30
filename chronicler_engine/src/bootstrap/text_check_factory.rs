//! [DOC: docs/system/text_check.md]
//! Text check factory - wires TextChecker port to HarperTextChecker impl

use std::sync::Arc;

use crate::application::ports::text_checker::TextChecker;
use crate::application::text_check_service::TextCheckService;
use crate::adapters::driven::text_check::HarperTextChecker;
use crate::domain::model::settings::AppSettings;

/// Create a TextCheckService with HarperTextChecker implementation.
/// This is the composition root for text checking - wires the adapter to the orchestrator.
pub fn create_text_check_service(settings: &AppSettings) -> TextCheckService {
    tracing::info!(
        "Creating text check service: mode={:?}, ignored_words={}",
        settings.text_check.mode,
        settings.text_check.ignored_words.len()
    );

    let checker: Arc<dyn TextChecker> =
        Arc::new(HarperTextChecker::new(&settings.text_check.ignored_words));
    TextCheckService::new(checker)
}
