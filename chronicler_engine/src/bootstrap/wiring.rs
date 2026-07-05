//! [DOC: docs/system/startup.md]
//! Composition root for application orchestrators — wires port impls to
//! `LlmCallRecorder`, `AgentRegistry`, and `GameService`. This is the only
//! module that imports both port traits and adapter impls (see ADR-027).

use std::sync::{Arc, RwLock};

use crate::adapters::driven::storage::Storage;
use crate::application::agents::registry::AgentRegistry;
use crate::application::game_service::GameService;
use crate::application::llm_recorder::LlmCallRecorder;
use crate::application::text_check_service::TextCheckService;
use crate::domain::model::settings::AppSettings;
use crate::error::Result;

/// Compose a `GameService` for production.
///
/// Reads settings, builds narration + quantifier LLM recorders via the LLM
/// factory, builds the agent registry, and returns a fully-wired
/// `GameService`. Application code receives the result — never reaches into
/// `crate::bootstrap::` itself.
pub fn build_game_service(
    settings: Arc<RwLock<AppSettings>>,
    storage: Arc<Storage>,
    preset_storage: Arc<Storage>,
) -> Result<GameService> {
    let (narration_recorder, quantifier_recorder, registry) = {
        let guard = settings.read().unwrap_or_else(|e| e.into_inner());
        let narration_recorder = super::llm_factory::get_llm_recorder_for(
            &guard.narration_connection(),
            Arc::clone(&storage),
        )?;
        let quantifier_recorder = super::llm_factory::get_llm_recorder_for(
            &guard.quantifier_connection(),
            Arc::clone(&storage),
        )?;
        let registry = AgentRegistry::from_configs_with_storage(
            &guard.agents,
            Arc::clone(&quantifier_recorder),
            Some(Arc::clone(&preset_storage)),
            Arc::clone(&settings),
        )
        .unwrap_or_default();
        (narration_recorder, quantifier_recorder, registry)
    };
    // quantifier_recorder is consumed by the registry; narration_recorder by GameService.
    drop(quantifier_recorder);
    Ok(GameService::with_storage(
        narration_recorder,
        registry,
        Arc::clone(&settings),
    ))
}

/// Build a single `LlmCallRecorder` for the narration connection.
/// Used by arrival-task wiring which only needs the recorder, not a full
/// `GameService`.
pub fn build_narration_recorder(
    settings: Arc<RwLock<AppSettings>>,
    storage: Arc<Storage>,
) -> Result<Arc<LlmCallRecorder>> {
    let guard = settings.read().unwrap_or_else(|e| e.into_inner());
    super::llm_factory::get_llm_recorder_for(&guard.narration_connection(), storage)
}

/// Build a `TextCheckService` from settings. Wraps `text_check_factory` so
/// driving adapters don't import bootstrap directly.
pub fn build_text_check_service(settings: Arc<RwLock<AppSettings>>) -> Arc<TextCheckService> {
    let guard = settings.read().unwrap_or_else(|e| e.into_inner());
    Arc::new(super::text_check_factory::create_text_check_service(&guard))
}

/// Convenience: build a default `GameService` backed by `MockBackend` LLM
/// providers. Used by tests that don't need a real LLM.
#[cfg(feature = "testing")]
pub fn build_game_service_for_tests(
    settings: Arc<RwLock<AppSettings>>,
    storage: Arc<Storage>,
    preset_storage: Arc<Storage>,
) -> Result<GameService> {
    // Test path: build mock recorders without touching real provider impls.
    let mock_provider: Arc<dyn crate::application::ports::llm_provider::LlmProvider> =
        Arc::new(crate::adapters::driven::llm::providers::MockBackend::new());
    let recorder = crate::test_support::make_test_recorder_with_storage(
        Arc::clone(&mock_provider),
        Arc::clone(&storage),
    );
    let registry = AgentRegistry::from_configs_with_storage(
        &settings.read().unwrap_or_else(|e| e.into_inner()).agents,
        Arc::clone(&recorder),
        Some(Arc::clone(&preset_storage)),
        Arc::clone(&settings),
    )
    .unwrap_or_default();
    drop(mock_provider); // consumed by recorder (Arc); drop the local handle
    Ok(GameService::with_storage(recorder, registry, settings))
}
