//! [DOC: docs/diataxis/reference/startup.md]
//! Composition root for application orchestrators

use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use crate::adapters::driven::llm::providers::{
    DeepSeekBackend, MockBackend, OllamaBackend, OpenRouterBackend,
};
use crate::adapters::driven::storage::Storage;
use crate::adapters::driven::text_check::HarperTextChecker;
use crate::application::agents::registry::AgentRegistry;
use crate::application::games::catalogue::GameCatalogue;
use crate::application::games::view_query::GameViewQuery;
use crate::application::generation::gate::GenerationGate;
use crate::application::llm_message::SaveLlmMessageFn;
use crate::application::llm_recorder::{LlmCallRecorder, ProviderResolver};
use crate::application::message_service::MessageService;
use crate::application::persona_catalogue::PersonaCatalogue;
use crate::application::pipeline::ActionPipeline;
use crate::application::ports::llm_provider::LlmProvider;
use crate::application::prompt_preset_service::PromptPresetService;
use crate::application::settings_service::SettingsService;
use crate::application::text_check_service::TextCheckService;
use crate::application::world_catalogue::WorldCatalogue;
use crate::domain::model::llm_backend::LlmBackendType;
use crate::domain::model::llm_message::LlmMessage;
use crate::domain::model::settings::{AppSettings, LlmProviderConfig};
use crate::error::Result;

fn provider_from_config(config: &LlmProviderConfig) -> Arc<dyn LlmProvider> {
    match config.provider {
        LlmBackendType::Mock => Arc::new(MockBackend::new().with_model(config.model.as_str())),
        LlmBackendType::DeepSeek => Arc::new(DeepSeekBackend::from_config(config)),
        LlmBackendType::OpenRouter => Arc::new(OpenRouterBackend::from_config(config)),
        LlmBackendType::Ollama => Arc::new(OllamaBackend::from_config(config)),
    }
}

/// Resolves the provider per call, so a dashboard change takes effect without a
/// restart. A `pick` failure (a dangling connection id) or a missing API key
/// surfaces as a call error.
fn recorder_with_storage(
    storage: Arc<Storage>,
    role: &'static str,
    pick: fn(&AppSettings) -> Result<LlmProviderConfig>,
) -> Arc<LlmCallRecorder> {
    let resolver_storage = Arc::clone(&storage);
    let resolve: ProviderResolver = Arc::new(move || {
        let settings = resolver_storage.get_settings()?;
        let config = pick(&settings)?;
        config.check_api_key_available()?;
        tracing::info!(
            "Resolved LLM provider for {role}: provider={:?}, model={}, api_key={}",
            config.provider,
            config.model,
            if config.resolve_api_key().is_some() {
                "present"
            } else {
                "absent"
            }
        );
        Ok(provider_from_config(&config))
    });

    let save_fn: SaveLlmMessageFn =
        Arc::new(move |message: &LlmMessage| storage.save_llm_message(message));

    Arc::new(LlmCallRecorder::with_resolver(resolve, save_fn))
}

/// Named-backend seam: a connection whose id is "options" routes options
/// generation to it (Roadway's cheap-model pattern); absent, the narration
/// connection serves.
fn options_recorder(storage: Arc<Storage>) -> Arc<LlmCallRecorder> {
    recorder_with_storage(storage, "options", |s| match s.find_connection("options") {
        Some(c) => Ok(c.clone()),
        None => s.narration_connection(),
    })
}

pub struct WiredApp {
    pub settings_service: SettingsService,
    pub prompt_preset_service: PromptPresetService,
    pub storage: Arc<Storage>,
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
    recorder: Arc<LlmCallRecorder>,
    agent_registry: AgentRegistry,
    text_check_service: Arc<TextCheckService>,
) -> Result<WiredApp> {
    let shutdown_token = CancellationToken::new();

    let settings_service = SettingsService::new(Arc::clone(&storage));
    let prompt_preset_service = PromptPresetService::new(Arc::clone(&storage));
    let message_service = Arc::new(MessageService::new(Arc::clone(&storage)));
    let world_catalogue = WorldCatalogue::new(Arc::clone(&storage));
    let persona_catalogue = PersonaCatalogue::new(Arc::clone(&storage));
    let generation_gate = GenerationGate::new();
    let game_catalogue = GameCatalogue::new(Arc::clone(&storage), Arc::clone(&message_service));
    let game_view_query = GameViewQuery::new(Arc::clone(&storage), Arc::clone(&message_service));
    let pipeline = ActionPipeline::with_storage(
        shutdown_token.clone(),
        recorder,
        agent_registry,
        Arc::clone(&message_service),
        Arc::clone(&storage),
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

pub fn build_app_graph(storage: Arc<Storage>) -> Result<WiredApp> {
    let settings = storage.get_settings()?;
    // Fail fast on a dangling reference at boot: the resolvers would otherwise
    // defer the error to the first generation.
    settings.narration_connection()?;
    settings.quantifier_connection()?;

    let narration_recorder = recorder_with_storage(
        Arc::clone(&storage),
        "narrator",
        AppSettings::narration_connection,
    );
    let quantifier_recorder = recorder_with_storage(
        Arc::clone(&storage),
        "quantifier",
        AppSettings::quantifier_connection,
    );
    let options_recorder = options_recorder(Arc::clone(&storage));
    let registry = AgentRegistry::from_configs_with_storage(
        &settings.agents,
        Arc::clone(&quantifier_recorder),
        options_recorder,
        Some(Arc::clone(&storage)),
    )
    .unwrap_or_default();
    let checker = Arc::new(HarperTextChecker::new());
    let text_check_service = Arc::new(TextCheckService::new(checker));

    build_wired_app(storage, narration_recorder, registry, text_check_service)
}

#[cfg(feature = "testing")]
pub fn build_app_graph_for_tests(
    storage: Arc<Storage>,
    pipeline_override: Option<ActionPipeline>,
) -> Result<WiredApp> {
    let settings = storage.get_settings()?;
    let checker = Arc::new(HarperTextChecker::new());
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
        let options_recorder = options_recorder(Arc::clone(&storage));
        AgentRegistry::from_configs_with_storage(
            &settings.agents,
            Arc::clone(&recorder),
            options_recorder,
            Some(Arc::clone(&storage)),
        )
        .unwrap_or_default()
    };

    let mut wired = build_wired_app(storage, recorder, registry, text_check_service)?;

    if let Some(pipeline) = pipeline_override {
        // Override backends must use this graph's persistence.
        wired.pipeline = pipeline.rebind_for_test(
            Arc::clone(&wired.message_service),
            Arc::clone(&wired.storage),
            wired.shutdown_token.clone(),
        );
    }

    Ok(wired)
}
