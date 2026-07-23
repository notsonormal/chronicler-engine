//! [DOC: chronicler_engine/docs/diataxis/reference/startup.md]
//! Composition root for application orchestrators — wires port impls to
//! `LlmCallRecorder`, `AgentRegistry`, and `GameService`. This is the only
//! module that imports both port traits and adapter impls (see ADR-027).

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};

use tokio_util::sync::CancellationToken;

use crate::adapters::driven::llm::providers::{
    DeepSeekBackend, MockBackend, OllamaBackend, OpenRouterBackend,
};
use crate::adapters::driven::storage::Storage;
use crate::adapters::driven::text_check::HarperTextChecker;
use crate::application::agents::registry::AgentRegistry;
use crate::application::application_service::DefaultApplicationService;
use crate::application::game_service::GameService;
use crate::application::llm_message::{LlmMessage, SaveLlmMessageFn};
use crate::application::llm_recorder::LlmCallRecorder;
use crate::application::ports::llm_provider::LlmProvider;
use crate::application::text_check_service::TextCheckService;
use crate::domain::model::llm_backend::LlmBackendType;
use crate::domain::model::settings::{AppSettings, LlmProviderConfig};
use crate::error::Result;

fn recorder_for(config: &LlmProviderConfig, storage: Arc<Storage>) -> Result<Arc<LlmCallRecorder>> {
    tracing::info!(
        "Creating LLM recorder: provider={:?}, model={}",
        config.provider,
        config.model
    );

    let provider: Arc<dyn LlmProvider> = match config.provider {
        LlmBackendType::Mock => Arc::new(MockBackend::new()),
        LlmBackendType::DeepSeek => Arc::new(DeepSeekBackend::from_config(config)),
        LlmBackendType::OpenRouter => Arc::new(OpenRouterBackend::from_config(config)),
        LlmBackendType::Ollama => Arc::new(OllamaBackend::from_config(config)),
    };

    let save_fn: SaveLlmMessageFn =
        Arc::new(move |message: &LlmMessage| storage.save_llm_message(message));

    Ok(Arc::new(LlmCallRecorder::new(provider, save_fn)))
}

pub struct WiredApp {
    pub storage: Arc<Storage>,
    pub preset_storage: Arc<Storage>,
    pub settings: Arc<RwLock<AppSettings>>,
    pub game_service: Arc<GameService>,
    pub application_service: Arc<DefaultApplicationService>,
    pub text_check_service: Arc<TextCheckService>,
}

pub fn build_app_graph(
    settings: Arc<RwLock<AppSettings>>,
    storage: Arc<Storage>,
    preset_storage: Arc<Storage>,
) -> Result<WiredApp> {
    let (game_service, text_check_service) = {
        let guard = settings.read().unwrap_or_else(|e| e.into_inner());
        let narration_recorder = recorder_for(&guard.narration_connection(), Arc::clone(&storage))?;
        let quantifier_recorder =
            recorder_for(&guard.quantifier_connection(), Arc::clone(&storage))?;
        let registry = AgentRegistry::from_configs_with_storage(
            &guard.agents,
            Arc::clone(&quantifier_recorder),
            Some(Arc::clone(&preset_storage)),
            Arc::clone(&settings),
        )
        .unwrap_or_default();
        drop(quantifier_recorder);
        let game_service = Arc::new(GameService::with_storage(
            narration_recorder,
            registry,
            Arc::clone(&settings),
        ));
        let checker = Arc::new(HarperTextChecker::new(&guard.text_check.ignored_words));
        let text_check_service = Arc::new(TextCheckService::new(checker));
        (game_service, text_check_service)
    };

    let application_service = Arc::new(DefaultApplicationService::new(
        Arc::clone(&storage),
        Arc::clone(&preset_storage),
        Arc::clone(&settings),
        CancellationToken::new(),
        Arc::new(AtomicBool::new(false)),
        Arc::clone(&game_service),
    ));

    Ok(WiredApp {
        storage,
        preset_storage,
        settings,
        game_service,
        application_service,
        text_check_service,
    })
}

#[cfg(feature = "testing")]
pub fn build_app_graph_for_tests(
    settings: Arc<RwLock<AppSettings>>,
    storage: Arc<Storage>,
    preset_storage: Arc<Storage>,
    game_service_override: Option<Arc<GameService>>,
) -> Result<WiredApp> {
    let (game_service, text_check_service) = {
        let guard = settings.read().unwrap_or_else(|e| e.into_inner());
        let checker = Arc::new(HarperTextChecker::new(&guard.text_check.ignored_words));
        let text_check_service = Arc::new(TextCheckService::new(checker));

        let game_service = if let Some(override_) = game_service_override {
            override_
        } else {
            let mock_provider: Arc<dyn LlmProvider> = Arc::new(MockBackend::new());
            let recorder = crate::test_support::make_test_recorder_with_storage(
                Arc::clone(&mock_provider),
                Arc::clone(&storage),
            );
            let registry = AgentRegistry::from_configs_with_storage(
                &guard.agents,
                Arc::clone(&recorder),
                Some(Arc::clone(&preset_storage)),
                Arc::clone(&settings),
            )
            .unwrap_or_default();
            drop(mock_provider);
            Arc::new(GameService::with_storage(
                recorder,
                registry,
                Arc::clone(&settings),
            ))
        };
        (game_service, text_check_service)
    };

    let application_service = Arc::new(DefaultApplicationService::new(
        Arc::clone(&storage),
        Arc::clone(&preset_storage),
        Arc::clone(&settings),
        CancellationToken::new(),
        Arc::new(AtomicBool::new(false)),
        Arc::clone(&game_service),
    ));

    Ok(WiredApp {
        storage,
        preset_storage,
        settings,
        game_service,
        application_service,
        text_check_service,
    })
}
