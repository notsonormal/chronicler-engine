//! [DOC: docs/reference/test_support.md — section "TestAppBuilder"]
//! Test application builder for HTTP and integration tests.
#![allow(clippy::expect_used)]

use std::sync::{Arc, RwLock};

use axum::Router;
use tokio_util::sync::CancellationToken;

use crate::application::application_service::DefaultApplicationService;
use crate::application::game_service::GameService;
use crate::bootstrap::text_check_factory::create_text_check_service;
use crate::domain::model::settings::AppSettings;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::MessageType;
use crate::domain::model::state::trigger_context::StoredTriggerContext;
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::adapters::driving::http::router::build_router;
use crate::adapters::driving::http::AppState;
use crate::adapters::driven::storage::Storage;
use crate::test_support::test_data_builder::TestData;

pub struct TestAppBuilder {
    test_data: Option<TestData>,
    logs: Vec<(String, Option<String>, MessageType)>,
    last_trigger: Option<StoredTriggerContext>,
    generation: Option<(GenerationStatus, GenerationPhase)>,
    settings: AppSettings,
    storage: Option<Arc<Storage>>,
    game_service: Option<Arc<GameService>>,
    skip_seeding: bool,
    is_generating: bool,
}

impl TestAppBuilder {
    pub fn default_test() -> Self {
        Self {
            test_data: Some(
                crate::test_support::test_data_builder::TestDataBuilder::default_test().build(),
            ),
            logs: vec![],
            last_trigger: None,
            generation: None,
            settings: AppSettings::default(),
            storage: None,
            game_service: None,
            skip_seeding: false,
            is_generating: false,
        }
    }

    pub fn with_data(data: TestData) -> Self {
        Self {
            test_data: Some(data),
            ..Self::default_test()
        }
    }

    pub fn data(mut self, data: TestData) -> Self {
        self.test_data = Some(data);
        self
    }

    pub fn default_app() -> Router {
        Self::default_test().build()
    }

    pub fn last_trigger(mut self, trigger: StoredTriggerContext) -> Self {
        self.last_trigger = Some(trigger);
        self
    }

    pub fn log(mut self, text: &str, speaker: Option<&str>, log_type: MessageType) -> Self {
        self.logs
            .push((text.to_string(), speaker.map(|s| s.to_string()), log_type));
        self
    }

    pub fn generation_status(mut self, status: GenerationStatus, phase: GenerationPhase) -> Self {
        self.generation = Some((status, phase));
        self
    }

    pub fn settings(mut self, settings: AppSettings) -> Self {
        self.settings = settings;
        self
    }

    pub fn storage(mut self, storage: Arc<Storage>) -> Self {
        self.storage = Some(storage);
        self
    }

    pub fn game_service(mut self, service: Arc<GameService>) -> Self {
        self.game_service = Some(service);
        self
    }

    pub fn is_generating(mut self, value: bool) -> Self {
        self.is_generating = value;
        self
    }

    pub fn skip_seeding(mut self, value: bool) -> Self {
        self.skip_seeding = value;
        self
    }

    pub fn build(self) -> Router {
        let app_state = self.build_app_state();
        build_router(app_state)
    }

    /// Build router and return the underlying service for tests that need to
    /// observe post-handler state (e.g., wait for the spawned pipeline to
    /// finalize via `is_generating`).
    pub fn build_with_service(self) -> (Router, Arc<DefaultApplicationService>) {
        let app_state = self.build_app_state();
        let service = Arc::clone(&app_state.application_service);
        (build_router(app_state), service)
    }

    /// Build without HTTP router. Returns the application service directly
    /// for non-HTTP integration tests.
    pub fn build_service(self) -> Arc<DefaultApplicationService> {
        let app_state = self.build_app_state();
        Arc::clone(&app_state.application_service)
    }

    /// Build a sibling application service that shares storage, settings,
    /// shutdown token, and `is_generating` flag with `base`, but installs a
    /// different `game_service`.
    pub fn from_base(
        base: &DefaultApplicationService,
        game_service: Arc<GameService>,
    ) -> Arc<DefaultApplicationService> {
        Arc::new(DefaultApplicationService::new(
            Arc::clone(base.storage()),
            Arc::clone(base.preset_storage().inner()),
            Arc::clone(base.settings()),
            base.cancel_token().clone(),
            Arc::clone(base.is_generating()),
            game_service,
        ))
    }

    pub fn build_app_state(mut self) -> AppState {
        let test_data = self.test_data.expect(
            "test setup: TestAppBuilder requires test_data (use default_test() or with_data())",
        );

        let storage = match self.storage.take() {
            Some(s) => s,
            None => Arc::new(Storage::new_in_memory()),
        };

        if !self.skip_seeding {
            let _ = test_data.seed_into(&storage);
        }

        let starting_room = test_data
            .map
            .overworld
            .regions
            .first()
            .and_then(|r| r.rooms.first())
            .map(|r| r.id.clone())
            .unwrap_or_else(|| "room_1".to_string());

        let mut state = GameState::new(
            Arc::clone(&test_data.world),
            Arc::clone(&test_data.map),
            Arc::clone(&test_data.persona),
            test_data.npcs.clone(),
            starting_room,
        );

        for npc_id in &test_data.room_npcs {
            if let Some(npc) = state.npcs.get(npc_id).cloned() {
                state.scene.npcs_in_area.push(npc);
            }
        }

        if let Some(trigger) = self.last_trigger {
            state.narrative.last_trigger = Some(trigger);
        }

        if let Some((status, phase)) = self.generation {
            state.narrative.input_buffer.status = status;
            state.narrative.input_buffer.phase = phase;
        }

        for (text, sender, log_type) in self.logs {
            state.add_message(text, sender, log_type);
        }

        if !self.skip_seeding {
            let snapshot = GameStateSnapshot::from_game_state(&state);
            let _ = storage.save_snapshot(&snapshot);
            for msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
                let _ = storage.insert_message(&msg);
            }
        }

        let settings_arc = Arc::new(RwLock::new(self.settings.clone()));
        let preset_storage = crate::test_support::default_test_preset_storage();
        let game_service: Arc<GameService> = if let Some(service) = self.game_service.take() {
            service
        } else {
            Arc::new(
                crate::bootstrap::wiring::build_game_service_for_tests(
                    Arc::clone(&settings_arc),
                    Arc::clone(&storage),
                    Arc::clone(&preset_storage),
                )
                .expect("build_game_service_for_tests should succeed"),
            )
        };
        let text_check_service = Arc::new(create_text_check_service(&self.settings));
        let is_generating = Arc::new(std::sync::atomic::AtomicBool::new(self.is_generating));
        let shutdown_token = CancellationToken::new();
        AppState {
            storage: Arc::clone(&storage),
            preset_storage: Arc::clone(&preset_storage),
            game_service: Arc::clone(&game_service),
            application_service: Arc::new(DefaultApplicationService::new(
                Arc::clone(&storage),
                Arc::clone(&preset_storage),
                Arc::clone(&settings_arc),
                shutdown_token.clone(),
                Arc::clone(&is_generating),
                game_service,
            )),
            text_check_service,
            settings: Arc::clone(&settings_arc),
            shutdown_token: Arc::new(RwLock::new(shutdown_token)),
        }
    }
}
