//! [DOC: docs/reference/test_support.md — section "Integration Test Helpers"]
use std::sync::{Arc, RwLock};

use crate::application::OpContext;
use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::domain::model::settings::AppSettings;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::adapters::driven::storage::Storage;

pub fn make_test_context(state: GameState) -> OpContext {
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

    build_test_context(state, storage)
}

pub fn make_test_context_without_snapshot(state: GameState) -> OpContext {
    let storage = Arc::new(Storage::new_in_memory());
    seed_test_world_into_storage(&storage, &state);
    for msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        if let Ok(id) = storage.insert_message(&msg) {
            for (idx, swipe) in msg.swipes.iter().enumerate() {
                let _ = storage.insert_swipe(id, swipe, idx);
            }
        }
    }

    build_test_context(state, storage)
}

fn default_test_preset_storage() -> Arc<Storage> {
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

fn build_test_context(state: GameState, storage: Arc<Storage>) -> OpContext {
    OpContext {
        storage,
        world_snapshot: crate::application::context::WorldSnapshot {
            world: state.world.clone(),
            map: state.map.clone(),
            player: state.player.clone(),
            npcs: Arc::new(state.npcs.clone()),
        },
        cancel_token: tokio_util::sync::CancellationToken::new(),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        settings: Arc::new(RwLock::new(AppSettings::default())),
        preset_storage: default_test_preset_storage(),
    }
}

pub fn make_test_context_with_sqlite(state: GameState) -> crate::error::Result<OpContext> {
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

    Ok(OpContext {
        storage,
        world_snapshot: crate::application::context::WorldSnapshot {
            world: state.world.clone(),
            map: state.map.clone(),
            player: state.player.clone(),
            npcs: Arc::new(state.npcs.clone()),
        },
        cancel_token: tokio_util::sync::CancellationToken::new(),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        settings: Arc::new(RwLock::new(AppSettings::default())),
        preset_storage: default_test_preset_storage(),
    })
}
