//! [DOC: docs/reference/testing.md — section "SqliteTestAppBuilder"]
//! Integration-only SQLite-backed application builder for integration tests.
#![allow(clippy::expect_used)]
#![allow(dead_code)]

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};

use tokio_util::sync::CancellationToken;

use chronicler_engine::adapters::driven::llm::providers::MockBackend;
use chronicler_engine::adapters::driven::storage::db::DbPool;
use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::application::agents::registry::AgentRegistry;
use chronicler_engine::application::application_service::DefaultApplicationService;
use chronicler_engine::application::game_service::GameService;
use chronicler_engine::domain::model::message::Message;
use chronicler_engine::domain::model::settings::AppSettings;
use chronicler_engine::domain::model::state::game_state::GameState;
use chronicler_engine::domain::model::state::game_state_snapshot::GameStateSnapshot;
use chronicler_engine::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::domain::model::state::trigger_context::StoredTriggerContext;
use chronicler_engine::error::Result;
use chronicler_engine::test_support::{
    default_test_preset_storage, make_test_recorder, seed_default_game_row, TestData,
    TestDataBuilder,
};

/// Backend wiring for the resulting `GameService`. Mutually exclusive variants; `build_service` consumes the last set one.
type MockBackendFn = Box<dyn Fn() -> MockBackend>;
type GameServiceBuilder = Box<dyn FnOnce(&Arc<Storage>) -> Arc<GameService>>;

enum BackendSpec {
    /// `with_mock_quantifier(make_recorder(backend()), backend())`. Closure invoked twice.
    MockBackend(MockBackendFn),
    /// `with_backends(make_recorder(backend()), AgentRegistry::default())`.
    Backends(MockBackendFn),
    /// `with_mock_quantifier(make_recorder(narrator()), quantifier())` with two factories.
    SeparateBackends {
        narrator: MockBackendFn,
        quantifier: MockBackendFn,
    },
    /// Caller builds the full `GameService` from the seeded `Storage`.
    GameServiceFn(GameServiceBuilder),
}

/// SQLite-backed `DefaultApplicationService` builder for integration tests. Emits the service directly (no HTTP router), backed by in-memory SQLite with full snapshot + message persistence.
type StateMut = Box<dyn FnOnce(&mut GameState)>;

pub struct SqliteTestAppBuilder {
    test_data: TestData,
    logs: Vec<(String, Option<String>, MessageType)>,
    messages: Vec<Message>,
    last_trigger: Option<StoredTriggerContext>,
    generation: Option<(GenerationStatus, GenerationPhase)>,
    settings: AppSettings,
    backend: Option<BackendSpec>,
    is_generating: bool,
    state_mut: Option<StateMut>,
}

impl SqliteTestAppBuilder {
    pub fn with_data(data: TestData) -> Self {
        Self {
            test_data: data,
            logs: vec![],
            messages: vec![],
            last_trigger: None,
            generation: None,
            settings: AppSettings::default(),
            backend: None,
            is_generating: false,
            state_mut: None,
        }
    }

    /// Canonical setup: Test World + Test Map + Test Player + `npc_1`.
    pub fn default_test() -> Self {
        Self::with_data(TestDataBuilder::default_test().build())
    }

    pub fn data(mut self, data: TestData) -> Self {
        self.test_data = data;
        self
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

    /// Append a pre-built `Message` to the builder's pending history. Persists `snapshot_id` on `Input` messages.
    pub fn message(mut self, msg: Message) -> Self {
        self.messages.push(msg);
        self
    }

    /// Append a `Vec` of pre-built `Message`s. Same persistence semantics as `Self::message`.
    pub fn messages(mut self, msgs: Vec<Message>) -> Self {
        self.messages.extend(msgs);
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

    #[allow(clippy::wrong_self_convention)]
    pub fn is_generating(mut self, value: bool) -> Self {
        self.is_generating = value;
        self
    }

    pub fn mock_backend<F>(mut self, make_backend: F) -> Self
    where
        F: Fn() -> MockBackend + 'static,
    {
        self.backend = Some(BackendSpec::MockBackend(Box::new(make_backend)));
        self
    }

    pub fn backends<F>(mut self, make_narrator: F) -> Self
    where
        F: Fn() -> MockBackend + 'static,
    {
        self.backend = Some(BackendSpec::Backends(Box::new(make_narrator)));
        self
    }

    pub fn separate_backends<N, Q>(mut self, make_narrator: N, make_quantifier: Q) -> Self
    where
        N: Fn() -> MockBackend + 'static,
        Q: Fn() -> MockBackend + 'static,
    {
        self.backend = Some(BackendSpec::SeparateBackends {
            narrator: Box::new(make_narrator),
            quantifier: Box::new(make_quantifier),
        });
        self
    }

    pub fn game_service_fn<F>(mut self, build: F) -> Self
    where
        F: FnOnce(&Arc<Storage>) -> Arc<GameService> + 'static,
    {
        self.backend = Some(BackendSpec::GameServiceFn(Box::new(build)));
        self
    }

    /// Escape hatch for state mutations with no dedicated builder method. Applied after world-data seeding, before snapshot save + message persistence.
    pub fn state_mut<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut GameState) + 'static,
    {
        self.state_mut = Some(Box::new(f));
        self
    }

    /// Build the `DefaultApplicationService`: seed sqlite storage, persist snapshot + messages, dispatch to `BackendSpec`.
    pub fn build_service(mut self) -> Result<Arc<DefaultApplicationService>> {
        let starting_room = self
            .test_data
            .map
            .overworld
            .regions
            .first()
            .and_then(|r| r.rooms.first())
            .map(|r| r.id.clone())
            .unwrap_or_else(|| "room_1".to_string());

        let mut state = GameState::new(starting_room);

        for npc in self
            .test_data
            .npcs
            .iter()
            .filter(|n| self.test_data.room_npcs.contains(&n.id))
        {
            state.scene.npcs_in_area.push(npc.clone());
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

        for msg in self.messages {
            state.narrative.history.append(msg);
        }

        let db_pool = DbPool::new(":memory:")?;
        seed_default_game_row(&db_pool, 1)?;
        let storage = Arc::new(Storage::new_sqlite(db_pool, 1));

        let _world_id = self.test_data.seed_into(&storage);

        if let Some(state_mut) = self.state_mut.take() {
            state_mut(&mut state);
        }

        let snapshot = GameStateSnapshot::from_game_state(&state);
        let pre_main_id = storage
            .save_snapshot(&snapshot)
            .expect("snapshot save must succeed in test builder");

        let mut messages: Vec<_> = state.narrative.history.iter().cloned().collect();
        for msg in messages.iter_mut() {
            if msg.message_type == MessageType::Input {
                msg.set_snapshot_id(Some(pre_main_id));
                if let Some(swipe) = msg.swipes.first_mut() {
                    swipe.snapshot_id = Some(pre_main_id);
                }
            }
        }
        for msg in messages {
            if let Ok(id) = storage.insert_message(&msg) {
                for (idx, swipe) in msg.swipes.iter().enumerate() {
                    let _ = storage.insert_swipe(id, swipe, idx);
                }
            }
        }

        let _ = storage.save_snapshot(&snapshot);

        let game_service = match self.backend {
            None => panic!(
                "SqliteTestAppBuilder: no backend set; call .game_service_fn(...) or .mock_backend(...) before .build_service()"
            ),
            Some(BackendSpec::MockBackend(f)) => Arc::new(GameService::with_mock_quantifier(
                make_test_recorder(Arc::new(f())),
                Arc::new(f()),
            )),
            Some(BackendSpec::Backends(f)) => Arc::new(GameService::with_backends(
                make_test_recorder(Arc::new(f())),
                AgentRegistry::default(),
            )),
            Some(BackendSpec::SeparateBackends {
                narrator,
                quantifier,
            }) => Arc::new(GameService::with_mock_quantifier(
                make_test_recorder(Arc::new(narrator())),
                Arc::new(quantifier()),
            )),
            Some(BackendSpec::GameServiceFn(f)) => f(&storage),
        };

        Ok(finalize_app(
            storage,
            game_service,
            self.settings,
            self.is_generating,
        ))
    }
}

fn finalize_app(
    storage: Arc<Storage>,
    game_service: Arc<GameService>,
    settings: AppSettings,
    is_generating: bool,
) -> Arc<DefaultApplicationService> {
    let settings_arc = Arc::new(RwLock::new(settings));
    let preset_storage = default_test_preset_storage();
    Arc::new(DefaultApplicationService::new(
        storage,
        preset_storage,
        settings_arc,
        CancellationToken::new(),
        Arc::new(AtomicBool::new(is_generating)),
        game_service,
    ))
}
