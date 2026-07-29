//! [DOC: chronicler_engine/docs/diataxis/reference/startup.md]
//! Composition root for application orchestrators

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};

use tokio_util::sync::CancellationToken;

use crate::adapters::driven::llm::providers::{
    DeepSeekBackend, MockBackend, OllamaBackend, OpenRouterBackend,
};
use crate::adapters::driven::storage::{PresetStore, Storage};
use crate::adapters::driven::text_check::HarperTextChecker;
use crate::application::action_pipeline::pipeline::ActionPipeline;
use crate::application::agents::registry::AgentRegistry;
use crate::application::application_service::DefaultApplicationService;
use crate::application::game_catalogue::GameCatalogue;
use crate::application::game_service::GameService;
use crate::application::game_view_query::GameViewQuery;
use crate::application::generation_gate::GenerationGate;
use crate::application::llm_message::{LlmMessage, SaveLlmMessageFn};
use crate::application::llm_recorder::LlmCallRecorder;
use crate::application::persistence_gate::PersistenceGate;
use crate::application::ports::llm_provider::LlmProvider;
use crate::application::text_check_service::TextCheckService;
use crate::application::world_catalogue::WorldCatalogue;
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
    pub persistence_gate: Arc<PersistenceGate>,
    pub generation_gate: GenerationGate,
    pub game_catalogue: GameCatalogue,
    pub game_view_query: GameViewQuery,
    pub world_catalogue: WorldCatalogue,
    pub pipeline: ActionPipeline,
    pub application_service: Arc<DefaultApplicationService>,
    pub text_check_service: Arc<TextCheckService>,
}

struct AppCollaborators {
    pub(crate) persistence_gate: Arc<PersistenceGate>,
    pub(crate) generation_gate: GenerationGate,
    pub(crate) game_catalogue: GameCatalogue,
    pub(crate) game_view_query: GameViewQuery,
    pub(crate) world_catalogue: WorldCatalogue,
    pub(crate) pipeline: ActionPipeline,
}

fn build_app_collaborators(
    storage: Arc<Storage>,
    preset_storage: Arc<Storage>,
    settings: Arc<RwLock<AppSettings>>,
    is_generating: Arc<AtomicBool>,
    game_service: &Arc<GameService>,
) -> AppCollaborators {
    let preset_store = Arc::new(PresetStore::new(preset_storage));
    let persistence_gate = Arc::new(PersistenceGate::new(
        Arc::clone(&storage),
        Arc::clone(&preset_store),
    ));
    let generation_gate = GenerationGate::new(Arc::clone(&is_generating));
    // Direct atomic access per ADR-030 hot-path.
    let game_catalogue = GameCatalogue::new(Arc::clone(&persistence_gate));
    let game_view_query = GameViewQuery::new(Arc::clone(&persistence_gate), Arc::clone(&settings));
    // WorldCatalogue only performs worlds/personas CRUD, so Arc<Storage> is the narrowest collaborator.
    let world_catalogue = WorldCatalogue::new(storage);
    let pipeline = ActionPipeline::new(
        Arc::clone(&game_service.prompt_assembler),
        Arc::clone(&game_service.llm_recorder),
        Arc::clone(&game_service.agent_registry),
        Arc::clone(&persistence_gate),
        Arc::clone(&settings),
    );
    AppCollaborators {
        persistence_gate,
        generation_gate,
        game_catalogue,
        game_view_query,
        world_catalogue,
        pipeline,
    }
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

    let collaborators = build_app_collaborators(
        Arc::clone(&storage),
        Arc::clone(&preset_storage),
        Arc::clone(&settings),
        Arc::new(AtomicBool::new(false)),
        &game_service,
    );
    let application_service = Arc::new(DefaultApplicationService::new(
        Arc::clone(&collaborators.persistence_gate),
        collaborators.generation_gate.clone(),
        collaborators.game_catalogue.clone(),
        collaborators.game_view_query.clone(),
        collaborators.world_catalogue.clone(),
        Arc::clone(&settings),
        Arc::clone(&game_service),
        collaborators.pipeline.clone(),
        CancellationToken::new(),
    ));

    Ok(WiredApp {
        storage,
        preset_storage,
        settings,
        game_service: Arc::clone(&game_service),
        persistence_gate: collaborators.persistence_gate,
        generation_gate: collaborators.generation_gate,
        game_catalogue: collaborators.game_catalogue,
        game_view_query: collaborators.game_view_query,
        world_catalogue: collaborators.world_catalogue,
        pipeline: collaborators.pipeline,
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

    let collaborators = build_app_collaborators(
        Arc::clone(&storage),
        Arc::clone(&preset_storage),
        Arc::clone(&settings),
        Arc::new(AtomicBool::new(false)),
        &game_service,
    );
    let application_service = Arc::new(DefaultApplicationService::new(
        Arc::clone(&collaborators.persistence_gate),
        collaborators.generation_gate.clone(),
        collaborators.game_catalogue.clone(),
        collaborators.game_view_query.clone(),
        collaborators.world_catalogue.clone(),
        Arc::clone(&settings),
        Arc::clone(&game_service),
        collaborators.pipeline.clone(),
        CancellationToken::new(),
    ));

    Ok(WiredApp {
        storage,
        preset_storage,
        settings,
        game_service: Arc::clone(&game_service),
        persistence_gate: collaborators.persistence_gate,
        generation_gate: collaborators.generation_gate,
        game_catalogue: collaborators.game_catalogue,
        game_view_query: collaborators.game_view_query,
        world_catalogue: collaborators.world_catalogue,
        pipeline: collaborators.pipeline,
        application_service,
        text_check_service,
    })
}
