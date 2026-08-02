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
use crate::application::games::catalogue::GameCatalogue;
use crate::application::games::view_query::GameViewQuery;
use crate::application::generation::gate::GenerationGate;
use crate::application::llm_message::{LlmMessage, SaveLlmMessageFn};
use crate::application::llm_recorder::LlmCallRecorder;
use crate::application::persona_catalogue::PersonaCatalogue;
use crate::application::prompt_preset_service::PromptPresetService;
use crate::application::settings_service::SettingsService;
use crate::application::message_service::MessageService;
use crate::application::world_catalogue::WorldCatalogue;
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
    pub settings_service: SettingsService,
    pub prompt_preset_service: PromptPresetService,
    pub storage: Arc<Storage>,
    pub preset_storage: Arc<Storage>,
    pub settings: Arc<RwLock<AppSettings>>,
    pub message_service: Arc<MessageService>,
    pub generation_gate: GenerationGate,
    pub game_catalogue: GameCatalogue,
    pub game_view_query: GameViewQuery,
    pub world_catalogue: WorldCatalogue,
    pub persona_catalogue: PersonaCatalogue,
    pub pipeline: ActionPipeline,
    pub text_check_service: Arc<TextCheckService>,
    pub shutdown_token: CancellationToken,
}

fn build_wired_app(
    storage: Arc<Storage>,
    preset_storage: Arc<Storage>,
    settings: Arc<RwLock<AppSettings>>,
    recorder: Arc<LlmCallRecorder>,
    agent_registry: AgentRegistry,
    text_check_service: Arc<TextCheckService>,
) -> Result<WiredApp> {
    let shutdown_token = CancellationToken::new();

    let settings_service = SettingsService::new(Arc::clone(&storage));
    let prompt_preset_service = PromptPresetService::new(Arc::clone(&preset_storage));
    let preset_store = Arc::new(PresetStore::new(Arc::clone(&preset_storage)));
    let message_service = Arc::new(MessageService::new(Arc::clone(&storage)));
    let world_catalogue = WorldCatalogue::new(Arc::clone(&storage));
    let persona_catalogue = PersonaCatalogue::new(Arc::clone(&storage));
    let generation_gate = GenerationGate::new();
    let game_catalogue = GameCatalogue::new(Arc::clone(&storage), Arc::clone(&message_service));
    let game_view_query = GameViewQuery::new(
        Arc::clone(&storage),
        Arc::clone(&message_service),
        Arc::clone(&preset_store),
        Arc::clone(&settings),
    );
    let pipeline = ActionPipeline::with_storage(
        shutdown_token.clone(),
        recorder,
        agent_registry,
        Arc::clone(&message_service),
        Arc::clone(&storage),
        Arc::clone(&preset_store),
        Arc::clone(&settings),
    );

    // Boot heal: a crash/restart may have left the current game persisted as Generating.
    let current_game_id = storage.current_game_id();
    let mut boot_state = message_service.load_or_fresh();
    let pre_heal = boot_state.narrative.input_buffer.status.clone();
    generation_gate.heal_stale(current_game_id, &mut boot_state);
    if boot_state.narrative.input_buffer.status != pre_heal {
        let _ = message_service.save_state(&boot_state);
    }

    Ok(WiredApp {
        settings_service,
        prompt_preset_service,
        storage,
        preset_storage,
        settings,
        message_service,
        generation_gate,
        game_catalogue,
        game_view_query,
        world_catalogue,
        persona_catalogue,
        pipeline,
        text_check_service,
        shutdown_token,
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
        let preset_store = Arc::new(PresetStore::new(Arc::clone(&wired.preset_storage)));
        wired.pipeline = pipeline.rebind_for_test(
            Arc::clone(&wired.message_service),
            Arc::clone(&wired.storage),
            Arc::clone(&preset_store),
            Arc::clone(&wired.settings),
            wired.shutdown_token.clone(),
        );
    }

    Ok(wired)
}
