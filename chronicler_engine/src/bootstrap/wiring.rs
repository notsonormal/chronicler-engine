//! [DOC: chronicler_engine/docs/diataxis/reference/startup.md]
//! Composition root for application orchestrators

use std::sync::{Arc, RwLock};

use tokio_util::sync::CancellationToken;

use crate::adapters::driven::llm::providers::{
    DeepSeekBackend, MockBackend, OllamaBackend, OpenRouterBackend,
};
use crate::adapters::driven::storage::{PresetStore, Storage};
use crate::adapters::driven::text_check::HarperTextChecker;
use crate::application::pipeline::pipeline::ActionPipeline;
use crate::application::agents::registry::AgentRegistry;
use crate::application::application_service::DefaultApplicationService;
use crate::application::games::catalogue::GameCatalogue;
use crate::application::games::view_query::GameViewQuery;
use crate::application::generation::gate::GenerationGate;
use crate::application::llm_message::{LlmMessage, SaveLlmMessageFn};
use crate::application::llm_recorder::LlmCallRecorder;
use crate::application::persistence_gate::PersistenceGate;
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
    pub persistence_gate: Arc<PersistenceGate>,
    pub generation_gate: GenerationGate,
    pub game_catalogue: GameCatalogue,
    pub game_view_query: GameViewQuery,
    pub pipeline: ActionPipeline,
    pub application_service: Arc<DefaultApplicationService>,
    pub text_check_service: Arc<TextCheckService>,
}

fn build_collaborators(
    storage: Arc<Storage>,
    preset_storage: Arc<Storage>,
    settings: Arc<RwLock<AppSettings>>,
    recorder: Arc<LlmCallRecorder>,
    agent_registry: AgentRegistry,
) -> (
    Arc<PersistenceGate>,
    GenerationGate,
    GameCatalogue,
    GameViewQuery,
    ActionPipeline,
) {
    let preset_store = Arc::new(PresetStore::new(preset_storage));
    let persistence_gate = Arc::new(PersistenceGate::new(
        Arc::clone(&storage),
        Arc::clone(&preset_store),
    ));
    let generation_gate = GenerationGate::new();
    let game_catalogue = GameCatalogue::new(Arc::clone(&persistence_gate));
    let game_view_query = GameViewQuery::new(Arc::clone(&persistence_gate), Arc::clone(&settings));
    let pipeline = ActionPipeline::with_storage(
        recorder,
        agent_registry,
        Arc::clone(&persistence_gate),
        Arc::clone(&settings),
    );
    (
        persistence_gate,
        generation_gate,
        game_catalogue,
        game_view_query,
        pipeline,
    )
}

fn wire_application_service(
    persistence_gate: Arc<PersistenceGate>,
    generation_gate: GenerationGate,
    game_catalogue: GameCatalogue,
    game_view_query: GameViewQuery,
    settings: Arc<RwLock<AppSettings>>,
    pipeline: ActionPipeline,
) -> Arc<DefaultApplicationService> {
    Arc::new(DefaultApplicationService::new(
        persistence_gate,
        generation_gate,
        game_catalogue,
        game_view_query,
        Arc::clone(&settings),
        pipeline,
        CancellationToken::new(),
    ))
}

fn build_wired_app(
    storage: Arc<Storage>,
    preset_storage: Arc<Storage>,
    settings: Arc<RwLock<AppSettings>>,
    recorder: Arc<LlmCallRecorder>,
    agent_registry: AgentRegistry,
    text_check_service: Arc<TextCheckService>,
) -> Result<WiredApp> {
    let (persistence_gate, generation_gate, game_catalogue, game_view_query, pipeline) =
        build_collaborators(
            Arc::clone(&storage),
            Arc::clone(&preset_storage),
            Arc::clone(&settings),
            recorder,
            agent_registry,
        );

    // Boot heal: a crash/restart may have left the current game persisted as Generating.
    let current_game_id = storage.current_game_id();
    let mut boot_state = persistence_gate.load_or_fresh();
    let pre_heal = boot_state.narrative.input_buffer.status.clone();
    generation_gate.heal_stale(current_game_id, &mut boot_state);
    if boot_state.narrative.input_buffer.status != pre_heal {
        let _ = persistence_gate.save_state(&boot_state);
    }

    let application_service = wire_application_service(
        persistence_gate,
        generation_gate.clone(),
        game_catalogue.clone(),
        game_view_query.clone(),
        Arc::clone(&settings),
        pipeline.clone(),
    );

    Ok(WiredApp {
        storage,
        preset_storage,
        settings,
        persistence_gate: Arc::clone(&application_service.persistence_gate),
        generation_gate: application_service.generation_gate.clone(),
        game_catalogue: application_service.game_catalogue.clone(),
        game_view_query: application_service.game_view_query.clone(),
        pipeline: application_service.pipeline.clone(),
        application_service,
        text_check_service,
    })
}

pub fn build_app_graph(
    settings: Arc<RwLock<AppSettings>>,
    storage: Arc<Storage>,
    preset_storage: Arc<Storage>,
) -> Result<WiredApp> {
    let (recorder, agent_registry, text_check_service) = {
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
        let checker = Arc::new(HarperTextChecker::new(&guard.text_check.ignored_words));
        let text_check_service = Arc::new(TextCheckService::new(checker));
        (narration_recorder, registry, text_check_service)
    };

    build_wired_app(
        storage,
        preset_storage,
        settings,
        recorder,
        agent_registry,
        text_check_service,
    )
}

#[cfg(feature = "testing")]
pub fn build_app_graph_for_tests(
    settings: Arc<RwLock<AppSettings>>,
    storage: Arc<Storage>,
    preset_storage: Arc<Storage>,
    pipeline_override: Option<ActionPipeline>,
) -> Result<WiredApp> {
    let (recorder, agent_registry, text_check_service) = {
        let guard = settings.read().unwrap_or_else(|e| e.into_inner());
        let checker = Arc::new(HarperTextChecker::new(&guard.text_check.ignored_words));
        let text_check_service = Arc::new(TextCheckService::new(checker));

        let mock_provider: Arc<dyn LlmProvider> = Arc::new(MockBackend::new());
        let recorder = crate::test_support::make_test_recorder_with_storage(
            Arc::clone(&mock_provider),
            Arc::clone(&storage),
        );
        // Build placeholder collaborators; a pipeline override replaces them below.
        let registry = if pipeline_override.is_some() {
            AgentRegistry::default()
        } else {
            AgentRegistry::from_configs_with_storage(
                &guard.agents,
                Arc::clone(&recorder),
                Some(Arc::clone(&preset_storage)),
                Arc::clone(&settings),
            )
            .unwrap_or_default()
        };
        drop(mock_provider);
        (recorder, registry, text_check_service)
    };

    let mut wired = build_wired_app(
        storage,
        preset_storage,
        settings,
        recorder,
        agent_registry,
        text_check_service,
    )?;

    if let Some(pipeline) = pipeline_override {
        // Override backends must use this graph's persistence and live settings.
        let pipeline = pipeline.rebind_for_test(
            Arc::clone(&wired.persistence_gate),
            Arc::clone(&wired.settings),
        );
        wired.pipeline = pipeline.clone();
        wired.application_service = Arc::new(DefaultApplicationService::new(
            Arc::clone(&wired.persistence_gate),
            wired.generation_gate.clone(),
            wired.game_catalogue.clone(),
            wired.game_view_query.clone(),
            Arc::clone(&wired.settings),
            pipeline,
            CancellationToken::new(),
        ));
    }

    Ok(wired)
}
