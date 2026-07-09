//! [DOC: docs/reference/test_support.md — section "Integration Test Helpers"]
//! Builds `DefaultApplicationService` instances for integration tests.
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};

use tokio_util::sync::CancellationToken;

use crate::adapters::driven::llm::providers::MockBackend;
use crate::adapters::driven::storage::Storage;
use crate::application::application_service::DefaultApplicationService;
use crate::application::game_service::GameService;
use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::domain::model::settings::AppSettings;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::error::Result;
use crate::test_support::noop_forensics::make_test_recorder;

pub fn default_test_preset_storage() -> Arc<Storage> {
    let storage = Storage::new_in_memory();
    let _ = storage.save_preset(&PromptPreset {
        id: "system_default".to_string(),
        name: "Default Test System".to_string(),
        role: Some("You are a test narrator.".to_string()),
        instructions: None,
        writing_style: None,
        output_format: None,
        is_default: true,
        preset_type: PresetType::System,
    });
    Arc::new(storage)
}

pub fn seed_test_world_into_storage(storage: &Storage, state: &GameState) {
    let world_id = storage.seed_world(&state.world, &state.map).unwrap_or(1);
    let _ = storage.seed_persona(&state.player.key, &state.player);
    for (_, npc) in state.npcs.iter() {
        let _ = storage.seed_character(world_id, npc);
    }
    let game_id = storage
        .create_game(
            &state.world.name,
            &state.world.key,
            &state.player.key,
            &state.player.sheet.name,
            "Test Game",
        )
        .unwrap_or(1);
    storage.set_game_id(game_id);
}

fn build_test_app(storage: Arc<Storage>) -> Result<Arc<DefaultApplicationService>> {
    let settings = Arc::new(RwLock::new(AppSettings::default()));
    let preset_storage = default_test_preset_storage();

    let game_service = crate::bootstrap::wiring::build_game_service_for_tests(
        Arc::clone(&settings),
        Arc::clone(&storage),
        Arc::clone(&preset_storage),
    )?;

    Ok(Arc::new(DefaultApplicationService::new(
        storage,
        preset_storage,
        settings,
        tokio_util::sync::CancellationToken::new(),
        Arc::new(std::sync::atomic::AtomicBool::new(false)),
        Arc::new(game_service),
    )))
}

pub fn make_test_app(state: GameState) -> Result<Arc<DefaultApplicationService>> {
    let snapshot = GameStateSnapshot::from_game_state(&state);
    let storage = Arc::new(Storage::new_in_memory());
    seed_test_world_into_storage(&storage, &state);
    let _ = storage.save_snapshot(&snapshot);
    for msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        if let Ok(id) = storage.insert_message(&msg) {
            for (idx, swipe) in msg.swipes.iter().enumerate() {
                let _ = storage.insert_swipe(id, swipe, idx);
            }
        }
    }
    build_test_app(storage)
}

pub fn make_test_app_without_snapshot(state: GameState) -> Result<Arc<DefaultApplicationService>> {
    let storage = Arc::new(Storage::new_in_memory());
    seed_test_world_into_storage(&storage, &state);
    for msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        if let Ok(id) = storage.insert_message(&msg) {
            for (idx, swipe) in msg.swipes.iter().enumerate() {
                let _ = storage.insert_swipe(id, swipe, idx);
            }
        }
    }
    build_test_app(storage)
}

pub fn make_test_app_with_sqlite(state: GameState) -> Result<Arc<DefaultApplicationService>> {
    let snapshot = GameStateSnapshot::from_game_state(&state);
    let db_pool = crate::adapters::driven::storage::db::DbPool::new(":memory:")?;
    crate::test_support::seed_default_game_row(&db_pool, 1)?;
    let storage = Arc::new(Storage::new_sqlite(db_pool, 1));
    seed_test_world_into_storage(&storage, &state);
    let _ = storage.save_snapshot(&snapshot);
    for msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        if let Ok(id) = storage.insert_message(&msg) {
            for (idx, swipe) in msg.swipes.iter().enumerate() {
                let _ = storage.insert_swipe(id, swipe, idx);
            }
        }
    }
    build_test_app(storage)
}

pub fn make_test_app_with_mock_backend<F>(
    state: GameState,
    make_mock_backend: F,
) -> Result<Arc<DefaultApplicationService>>
where
    F: Fn() -> MockBackend,
{
    let storage = build_seeded_sqlite_storage(&state)?;
    let game_service = Arc::new(GameService::with_mock_quantifier(
        make_test_recorder(Arc::new(make_mock_backend())),
        Arc::new(make_mock_backend()),
    ));
    Ok(finalize_app(storage, game_service))
}

/// Build an app backed by `GameService::with_backends` (no quantifier agent).
/// Mirrors `tests/integration/mod.rs::working_service()`.
pub fn make_test_app_with_backends<F>(
    state: GameState,
    make_narrator: F,
) -> Result<Arc<DefaultApplicationService>>
where
    F: Fn() -> MockBackend,
{
    let storage = build_seeded_sqlite_storage(&state)?;
    let game_service = Arc::new(GameService::with_backends(
        make_test_recorder(Arc::new(make_narrator())),
        crate::application::agents::registry::AgentRegistry::default(),
    ));
    Ok(finalize_app(storage, game_service))
}

/// Build an app backed by `GameService::with_mock_quantifier` with separate
/// narrator and quantifier factories. Mirrors `tests/integration/mod.rs::failing_service()`.
pub fn make_test_app_with_separate_backends<N, Q>(
    state: GameState,
    make_narrator: N,
    make_quantifier: Q,
) -> Result<Arc<DefaultApplicationService>>
where
    N: Fn() -> MockBackend,
    Q: Fn() -> MockBackend,
{
    let storage = build_seeded_sqlite_storage(&state)?;
    let game_service = Arc::new(GameService::with_mock_quantifier(
        make_test_recorder(Arc::new(make_narrator())),
        Arc::new(make_quantifier()),
    ));
    Ok(finalize_app(storage, game_service))
}

/// Most flexible factory: caller supplies a closure that builds the entire
/// `GameService` given the seeded `Storage`.
pub fn make_test_app_with_game_service<F>(
    state: GameState,
    build: F,
) -> Result<Arc<DefaultApplicationService>>
where
    F: FnOnce(&Arc<Storage>) -> Arc<GameService>,
{
    let storage = build_seeded_sqlite_storage(&state)?;
    let game_service = build(&storage);
    Ok(finalize_app(storage, game_service))
}

/// Rebuild an app over an EXISTING storage with a new GameService. No state
/// seeding — storage contents are preserved.
pub fn make_test_app_with_storage_and_service(
    storage: Arc<Storage>,
    game_service: Arc<GameService>,
) -> Arc<DefaultApplicationService> {
    finalize_app(storage, game_service)
}

fn build_seeded_sqlite_storage(state: &GameState) -> Result<Arc<Storage>> {
    let snapshot = GameStateSnapshot::from_game_state(state);
    let db_pool = crate::adapters::driven::storage::db::DbPool::new(":memory:")?;
    crate::test_support::seed_default_game_row(&db_pool, 1)?;
    let storage = Arc::new(Storage::new_sqlite(db_pool, 1));
    seed_test_world_into_storage(&storage, state);
    let pre_main_id = storage.save_snapshot(&snapshot).unwrap_or(0);

    let mut messages: Vec<_> = state.narrative.history.iter().cloned().collect();
    for msg in messages.iter_mut() {
        if msg.message_type == crate::domain::model::state::message_types::MessageType::Input {
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
    Ok(storage)
}

fn finalize_app(
    storage: Arc<Storage>,
    game_service: Arc<GameService>,
) -> Arc<DefaultApplicationService> {
    let settings = Arc::new(RwLock::new(AppSettings::default()));
    let preset_storage = default_test_preset_storage();
    Arc::new(DefaultApplicationService::new(
        storage,
        preset_storage,
        settings,
        CancellationToken::new(),
        Arc::new(AtomicBool::new(false)),
        game_service,
    ))
}
