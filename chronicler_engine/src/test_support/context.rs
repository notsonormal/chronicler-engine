//! [DOC: docs/reference/test_support.md — section "Survivor Helpers"]
//! Builds `DefaultApplicationService` instances for integration tests.

use std::sync::Arc;
use std::sync::RwLock;

use crate::adapters::driven::storage::Storage;
use crate::application::application_service::DefaultApplicationService;
use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::error::Result;
use crate::test_support::TestData;

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
    // Delegate to TestData::seed_into — single source of truth for world
    // data seeding (fail-loud .expect() lives there, not here).
    let data = TestData {
        world: Arc::clone(&state.world),
        map: Arc::clone(&state.map),
        persona: Arc::clone(&state.persona),
        npcs: state.npcs.values().cloned().collect(),
        room_npcs: Vec::new(),
    };
    let _ = data.seed_into(storage);
}

fn build_test_app(storage: Arc<Storage>) -> Result<Arc<DefaultApplicationService>> {
    let settings = Arc::new(RwLock::new(
        crate::domain::model::settings::AppSettings::default(),
    ));
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
