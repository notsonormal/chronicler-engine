//! Test application builder for HTTP and integration tests.
#![allow(clippy::expect_used)]

use std::sync::{Arc, RwLock};

use axum::Router;

use crate::application::pipeline::pipeline::ActionPipeline;
use crate::bootstrap::wiring::build_app_graph_for_tests;
use crate::domain::model::settings::AppSettings;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::MessageType;
use crate::domain::model::state::trigger_context::StoredTriggerContext;
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::adapters::driving::http::builders::router::build_router;
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
    pipeline: Option<ActionPipeline>,
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
            pipeline: None,
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

    pub fn pipeline(mut self, pipeline: ActionPipeline) -> Self {
        self.pipeline = Some(pipeline);
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
        let (app_state, _storage) = self.build_service_with_storage();
        build_router(app_state)
    }

    /// Build returning `(Router, AppState)`.
    pub fn build_with_state(self) -> (Router, AppState) {
        let (app_state, _storage) = self.build_service_with_storage();
        (build_router(app_state.clone()), app_state)
    }

    /// Build returning the `AppState`.
    pub fn build_service(self) -> AppState {
        let (app_state, _storage) = self.build_service_with_storage();
        app_state
    }

    /// Build returning `(AppState, Arc<Storage>)`.
    pub fn build_service_with_storage(mut self) -> (AppState, Arc<Storage>) {
        let test_data = self.test_data.expect(
            "test setup: TestAppBuilder requires test_data (use default_test() or with_data())",
        );

        let storage = match self.storage.take() {
            Some(s) => s,
            None => Arc::new(Storage::new_in_memory()),
        };

        crate::test_support::seed_default_preset(&storage);

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

        let mut state = GameState::new(starting_room);

        for npc in test_data
            .npcs
            .iter()
            .filter(|n| test_data.room_npcs.contains(&n.id))
        {
            state.scene.npcs_in_area.push(npc.clone());
        }

        if let Some(trigger) = self.last_trigger {
            state.narrative.last_trigger = Some(trigger);
        }

        if let Some((status, phase)) = self.generation.clone() {
            state.narrative.input_buffer.status = status;
            state.narrative.input_buffer.phase = phase;
        }

        for (text, sender, log_type) in self.logs {
            state.add_message(text, sender, log_type);
        }

        if self.is_generating {
            state.narrative.input_buffer.status = GenerationStatus::Generating;
            state.narrative.input_buffer.phase = GenerationPhase::Narrating;
        }

        if !self.skip_seeding {
            let snapshot = GameStateSnapshot::from_game_state(&state);
            let _ = storage.save_snapshot(&snapshot);
            for msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
                let _ = storage.insert_message(&msg);
            }
        }

        let settings_arc = Arc::new(RwLock::new(self.settings.clone()));
        let pipeline_override = self.pipeline.take();

        let wired = build_app_graph_for_tests(
            Arc::clone(&settings_arc),
            Arc::clone(&storage),
            pipeline_override,
        )
        .expect("build_app_graph_for_tests should succeed");

        if self.is_generating {
            let mut state = wired.message_service.load_or_fresh();
            state.narrative.input_buffer.status = GenerationStatus::Generating;
            state.narrative.input_buffer.phase = GenerationPhase::Narrating;
            let _ = wired.message_service.save_state(&state);
            // Mirror the persisted Generating status into the in-memory gate so
            // handlers that consult `is_busy` see the same truth.
            let game_id = wired.storage.current_game_id();
            let _ = wired
                .generation_gate
                .try_claim(game_id, &mut state, &wired.message_service);
        }

        if let Some((status, phase)) = self.generation.clone() {
            let mut state = wired.message_service.load_or_fresh();
            state.narrative.input_buffer.status = status;
            state.narrative.input_buffer.phase = phase;
            let _ = wired.message_service.save_state(&state);
        }

        (AppState::from_wired(wired), storage)
    }
}
