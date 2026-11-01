//! [DOC: docs/diataxis/reference/startup.md]
//! Composition root for application orchestrators

use std::sync::Arc;

use crate::adapters::driven::llm::providers::{ChatCompletionsBackend, MockBackend};
use crate::adapters::driven::storage::Storage;
use crate::adapters::driven::text_check::HarperTextChecker;
use crate::application::agents::AgentRunner;
use crate::application::connection_test_service::ConnectionTestService;
use crate::application::games::active::ActiveGame;
use crate::application::games::catalogue::GameCatalogue;
use crate::application::games::view_query::GameViewQuery;
use crate::application::generation::gate::GameWriter;
use crate::application::autoplay::pause::AutoplayPause;
use crate::application::generation::job_tasks::JobTasks;
use crate::application::llm_recorder::SaveLlmMessageFn;
use crate::application::llm_recorder::{LlmCallRecorder, ProviderResolver};
use crate::application::narration::Narrator;
use crate::application::persona_catalogue::PersonaCatalogue;
use crate::application::pipeline::job::JobDeps;
use crate::application::prompting::PromptAssembler;
use crate::application::ports::llm_provider::LlmProvider;
use crate::application::prompt_preset_service::PromptPresetService;
use crate::application::settings_service::SettingsService;
use crate::application::text_check_service::TextCheckService;
use crate::application::world_catalogue::WorldCatalogue;
use crate::domain::model::game::GameId;
use crate::domain::model::llm_backend::LlmBackendType;
use crate::domain::model::llm_message::LlmMessage;
use crate::domain::model::prompt_preset::PresetType;
use crate::domain::model::settings::{AppSettings, LlmConnection};
use crate::error::Result;

fn provider_from_config(config: &LlmConnection) -> Arc<dyn LlmProvider> {
    match config.provider {
        LlmBackendType::Mock => Arc::new(MockBackend::new().with_model(config.model.as_str())),
        LlmBackendType::OpenRouter | LlmBackendType::Ollama | LlmBackendType::OpenCodeGo => {
            Arc::new(ChatCompletionsBackend::from_config(config))
        }
    }
}

/// Resolves the provider per call, so a dashboard change takes effect without a
/// restart.
fn recorder_with_storage(storage: Arc<Storage>, preset_type: PresetType) -> Arc<LlmCallRecorder> {
    let resolver_storage = Arc::clone(&storage);
    let resolve: ProviderResolver = Arc::new(move || {
        let settings = resolver_storage.get_settings()?;
        let config = settings.connection_for(preset_type)?;
        config.check_api_key_available()?;
        tracing::info!(
            "Resolved LLM provider for {}: provider={}, model={}, api_key={}",
            AppSettings::slot_field(preset_type),
            config.provider.display_label(),
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

pub struct WiredApp {
    pub settings_service: SettingsService,
    pub prompt_preset_service: PromptPresetService,
    pub storage: Arc<Storage>,
    pub game_writer: GameWriter,
    pub game_catalogue: GameCatalogue,
    pub game_view_query: GameViewQuery,
    pub world_catalogue: WorldCatalogue,
    pub persona_catalogue: PersonaCatalogue,
    pub job_deps: JobDeps,
    pub text_check_service: Arc<TextCheckService>,
    pub connection_test_service: ConnectionTestService,
}

fn build_wired_app(
    storage: Arc<Storage>,
    recorder: Arc<LlmCallRecorder>,
    impersonate_recorder: Arc<LlmCallRecorder>,
    agent_runner: AgentRunner,
    text_check_service: Arc<TextCheckService>,
    initial_game: Option<GameId>,
) -> Result<WiredApp> {
    let active_game = Arc::new(ActiveGame::new(Arc::clone(&storage)));

    let settings_service = SettingsService::new(Arc::clone(&storage));
    let prompt_preset_service = PromptPresetService::new(Arc::clone(&storage));
    let connection_test_service =
        ConnectionTestService::new(settings_service.clone(), Arc::new(provider_from_config));
    let world_catalogue = WorldCatalogue::new(Arc::clone(&storage));
    let persona_catalogue = PersonaCatalogue::new(Arc::clone(&storage));
    let game_writer = GameWriter::new(Arc::clone(&storage));
    let game_catalogue = GameCatalogue::new(
        Arc::clone(&storage),
        game_writer.clone(),
        Arc::clone(&active_game),
        initial_game,
    )?;
    let game_view_query = GameViewQuery::new(Arc::clone(&storage), game_writer.clone());
    tracing::info!("Narrator: provider={}", recorder.provider_label());
    let job_deps = JobDeps {
        narrator: Arc::new(Narrator::new(
            PromptAssembler,
            recorder,
            impersonate_recorder,
        )),
        agent_runner: Arc::new(agent_runner),
        storage: Arc::clone(&storage),
        tasks: Arc::new(JobTasks::default()),
        pause: Arc::new(AutoplayPause::default()),
    };
    Ok(WiredApp {
        settings_service,
        prompt_preset_service,
        storage,
        game_writer,
        game_catalogue,
        game_view_query,
        world_catalogue,
        persona_catalogue,
        job_deps,
        text_check_service,
        connection_test_service,
    })
}

pub fn build_app_graph(storage: Arc<Storage>, initial_game: Option<GameId>) -> Result<WiredApp> {
    let settings = storage.get_settings()?;
    // Fail fast on a dangling reference at boot: the resolvers would otherwise
    // defer the error to the first generation.
    settings.check_connection_slots()?;

    let narration_recorder = recorder_with_storage(Arc::clone(&storage), PresetType::System);
    let quantifier_recorder = recorder_with_storage(Arc::clone(&storage), PresetType::Quantifier);
    let impersonate_recorder = recorder_with_storage(Arc::clone(&storage), PresetType::Impersonate);
    let options_recorder = recorder_with_storage(Arc::clone(&storage), PresetType::Options);
    let agent_runner = AgentRunner::from_configs(
        &settings.agents,
        Arc::clone(&quantifier_recorder),
        options_recorder,
    )
    .unwrap_or_default();
    let checker = Arc::new(HarperTextChecker::new());
    let text_check_service = Arc::new(TextCheckService::new(checker));

    build_wired_app(
        storage,
        narration_recorder,
        impersonate_recorder,
        agent_runner,
        text_check_service,
        initial_game,
    )
}

#[cfg(feature = "testing")]
pub fn build_app_graph_for_tests(
    storage: Arc<Storage>,
    job_deps_override: Option<JobDeps>,
    initial_game: Option<GameId>,
) -> Result<WiredApp> {
    let settings = storage.get_settings()?;
    let checker = Arc::new(HarperTextChecker::new());
    let text_check_service = Arc::new(TextCheckService::new(checker));

    let mock_provider: Arc<dyn LlmProvider> = Arc::new(MockBackend::new());
    let recorder = crate::test_support::make_test_recorder_with_storage(
        Arc::clone(&mock_provider),
        Arc::clone(&storage),
    );
    // Build placeholder collaborators; a Job-deps override replaces them below.
    let agent_runner = if job_deps_override.is_some() {
        AgentRunner::default()
    } else {
        let options_recorder = recorder_with_storage(Arc::clone(&storage), PresetType::Options);
        AgentRunner::from_configs(&settings.agents, Arc::clone(&recorder), options_recorder)
            .unwrap_or_default()
    };

    let mut wired = build_wired_app(
        storage,
        Arc::clone(&recorder),
        recorder,
        agent_runner,
        text_check_service,
        initial_game,
    )?;

    // The override's Job writes through this graph's storage.
    if let Some(job_deps) = job_deps_override {
        wired.job_deps = JobDeps {
            storage: Arc::clone(&wired.storage),
            ..job_deps
        };
    }

    Ok(wired)
}
