//! Test application builder for HTTP and integration tests.
#![allow(clippy::expect_used)]

use std::collections::HashMap;
use std::sync::Arc;

use axum::Router;

use crate::application::pipeline::job::JobDeps;
use crate::test_support::context::{configured_runner, make_test_job_deps};
use crate::test_support::fixtures::make_test_recorder_with_storage;
use crate::test_support::scripted_model::ScriptedModel;
use crate::bootstrap::wiring::build_app_graph_for_tests;
use crate::domain::model::game::GameId;
use crate::domain::model::settings::AppSettings;
use crate::domain::model::message::SwipeAt;
use crate::domain::model::state::generation_status::GenerationFailure;
use crate::domain::model::state::swipe_state::SwipeState;
use crate::adapters::driving::http::builders::router::build_router;
use crate::adapters::driving::http::AppState;
use crate::adapters::driven::storage::Storage;
use crate::test_support::test_data_builder::TestData;

pub struct TestAppBuilder {
    test_data: Option<TestData>,
    failure: Option<GenerationFailure>,
    settings: AppSettings,
    storage: Option<Arc<Storage>>,
    job_deps: Option<JobDeps>,
    model: Option<Arc<ScriptedModel>>,
    skip_seeding: bool,
    active_game: Option<GameId>,
}

impl TestAppBuilder {
    pub fn default_test() -> Self {
        Self {
            test_data: Some(
                crate::test_support::test_data_builder::TestDataBuilder::default_test().build(),
            ),
            failure: None,
            settings: AppSettings::default(),
            storage: None,
            job_deps: None,
            model: None,
            skip_seeding: false,
            active_game: None,
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

    /// The record the status reads when no Job runs.
    pub fn failure(mut self, failure: GenerationFailure) -> Self {
        self.failure = Some(failure);
        self
    }

    pub fn settings(mut self, settings: AppSettings) -> Self {
        self.settings = settings;
        self
    }

    /// An injected `JobDeps` is rebound to this storage instance.
    pub fn storage(mut self, storage: Arc<Storage>) -> Self {
        self.storage = Some(storage);
        self
    }

    /// The Job's collaborators are rebound to this builder's storage: pass the
    /// same `Arc` via `.storage(...)` when the test mutates storage the Job
    /// should read.
    pub fn job_deps(mut self, job_deps: JobDeps) -> Self {
        self.job_deps = Some(job_deps);
        self
    }

    pub fn model(mut self, model: Arc<ScriptedModel>) -> Self {
        self.model = Some(model);
        self
    }

    pub fn skip_seeding(mut self, value: bool) -> Self {
        self.skip_seeding = value;
        self
    }

    /// Only needed with `skip_seeding(true)`: the builder cannot see the Game a
    /// test seeds itself.
    pub fn active_game(mut self, game_id: GameId) -> Self {
        self.active_game = Some(game_id);
        self
    }

    pub fn build(self) -> Router {
        let (app_state, _storage) = self.build_service_with_storage();
        build_router(app_state)
    }

    pub fn build_with_state(self) -> (Router, AppState) {
        let (app_state, _storage) = self.build_service_with_storage();
        (build_router(app_state.clone()), app_state)
    }

    /// The design's Game always holds an opening Narration, and the app's reads
    /// are the story reads, so a seeded Game gets one: the create unit's own
    /// build, with the state this fixture seeds.
    fn seed_opening(
        storage: &Arc<Storage>,
        game_id: GameId,
        data: &TestData,
        state: &SwipeState,
    ) -> crate::error::Result<()> {
        storage.transaction(game_id, crate::test_support::tx_token(), |tx| {
            let opening = crate::domain::model::utils::opening::opening_message(
                &data.world,
                &data.persona.sheet.name,
                state.clone(),
            );
            let message_id = tx.insert_message(&opening)?;
            tx.insert_swipe(
                SwipeAt {
                    message: message_id,
                    swipe: 0,
                },
                &opening.swipes[0],
            )?;
            Ok(())
        })
    }

    pub fn build_service(self) -> AppState {
        let (app_state, _storage) = self.build_service_with_storage();
        app_state
    }

    /// The returned storage is the instance the wired app holds.
    pub fn build_service_with_storage(mut self) -> (AppState, Arc<Storage>) {
        let test_data = self.test_data.expect(
            "test setup: TestAppBuilder requires test_data (use default_test() or with_data())",
        );

        let storage = match self.storage.take() {
            Some(s) => s,
            None => Arc::new(crate::test_support::empty_sqlite_storage()),
        };

        crate::test_support::seed_default_preset(&storage);

        let seeded_game = if self.skip_seeding {
            None
        } else {
            Some(test_data.seed_into(&storage))
        };

        let starting_room = test_data
            .map
            .overworld
            .regions
            .first()
            .and_then(|r| r.rooms.first())
            .map(|r| r.id.clone())
            .unwrap_or_else(|| "room_1".to_string());

        let mut state = SwipeState::starting(None, &HashMap::new(), &starting_room);

        for character in test_data
            .characters
            .iter()
            .filter(|n| test_data.room_characters.contains(&n.id))
        {
            state.scene.characters_present.push(character.clone());
        }

        if let Some(game_id) = seeded_game {
            let _ = Self::seed_opening(&storage, game_id, &test_data, &state);
        }

        let _ = storage.save_settings(&self.settings);
        let mut job_deps_override = self.job_deps.take();
        if job_deps_override.is_none() {
            if let Some(model) = self.model.take() {
                let recorder = make_test_recorder_with_storage(model, Arc::clone(&storage));
                let runner = configured_runner(Arc::clone(&recorder));
                job_deps_override =
                    Some(make_test_job_deps(Arc::clone(&storage), recorder, runner));
            }
        }

        let game_id = self.active_game.or(seeded_game);
        // Point the app before wiring, so the wired graph names the Game the test
        // seeded.
        let wired = build_app_graph_for_tests(Arc::clone(&storage), job_deps_override, game_id)
            .expect("build_app_graph_for_tests should succeed");

        if let (Some(game_id), Some(failure)) = (game_id, self.failure.take()) {
            let _ = storage.transaction(game_id, crate::test_support::tx_token(), |tx| {
                tx.set_game_failure(Some(&failure))
            });
        }

        (AppState::from_wired(wired), storage)
    }
}
