//! Unit tests for GameCatalogue.

use std::sync::{Arc, RwLock};

use crate::application::errors::ApplicationError;
use crate::application::games::catalogue::GameCatalogue;
use crate::application::message_service::MessageService;
use crate::adapters::driven::storage::{Storage, TestOverride};
use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::domain::model::settings::{AppSettings, NarratorMode, NarrativePerspective, NarrativeTense};
use crate::domain::model::state::message_types::MessageType;
use crate::domain::model::utils::settings_defaults;
use crate::test_support::TestDataBuilder;

fn seeded_catalogue() -> (GameCatalogue, Arc<Storage>, String, String) {
    let data = TestDataBuilder::default_test().build();
    let storage = Arc::new(Storage::new_in_memory());
    data.seed_into(&storage);
    let message_service = Arc::new(MessageService::new(Arc::clone(&storage)));
    let settings = Arc::new(RwLock::new(AppSettings::default()));
    let world_key = data.world_key();
    let persona_key = data.persona.key.clone();
    (
        GameCatalogue::new(Arc::clone(&storage), message_service, settings),
        storage,
        world_key,
        persona_key,
    )
}

#[test]
fn test_create_game_returns_positive_id() {
    let (catalogue, storage, world_key, persona_key) = seeded_catalogue();

    let id = catalogue
        .create_game(&world_key, &persona_key)
        .expect("create_game should succeed");

    assert!(id > 0, "Game ID should be positive");
    let game = storage
        .get_game(id)
        .unwrap()
        .expect("game should be persisted");
    assert_eq!(game.world_key, world_key);
    assert_eq!(game.persona_key, persona_key);
    assert_eq!(
        storage.current_game_id(),
        id,
        "new game should become current"
    );
}

#[test]
fn test_create_game_inherits_world_posture_and_mode_matched_bundle() {
    let if_world = {
        let mut world = TestDataBuilder::default_test()
            .build()
            .world
            .as_ref()
            .clone();
        world.key = "test_if".to_string();
        world.narrator_mode = NarratorMode::InteractiveFiction;
        world
    };
    let data = TestDataBuilder::default_test()
        .world(if_world.clone())
        .build();
    let storage = Arc::new(Storage::new_in_memory());
    data.seed_into(&storage);
    let message_service = Arc::new(MessageService::new(Arc::clone(&storage)));
    let settings = Arc::new(RwLock::new(AppSettings::default()));
    let world_key = data.world_key();
    let persona_key = data.persona.key.clone();
    let catalogue = GameCatalogue::new(Arc::clone(&storage), message_service, settings);

    let id = catalogue
        .create_game(&world_key, &persona_key)
        .expect("create_game should succeed");
    let game = storage
        .get_game(id)
        .unwrap()
        .expect("game should be persisted");

    assert_eq!(game.narrator_mode, NarratorMode::InteractiveFiction);
    assert_eq!(game.active_system_prompt_preset_id, "system_if_default");
    assert_eq!(
        game.active_quantifier_prompt_preset_id,
        "quantifier_default"
    );
    assert_eq!(
        game.active_impersonate_prompt_preset_id,
        "impersonate_default"
    );
}

#[test]
fn test_create_game_errors_when_world_missing() {
    let (catalogue, _storage, _world_key, persona_key) = seeded_catalogue();

    let result = catalogue.create_game("no_such_world", &persona_key);

    assert!(
        matches!(result, Err(ApplicationError::Validation(ref msg)) if msg.contains("World not found")),
        "Expected world-not-found validation error, got {result:?}"
    );
}

#[test]
fn test_create_game_errors_when_persona_missing() {
    let (catalogue, _storage, world_key, _persona_key) = seeded_catalogue();

    let result = catalogue.create_game(&world_key, "no_such_persona");

    assert!(
        matches!(result, Err(ApplicationError::Validation(ref msg)) if msg.contains("Persona not found")),
        "Expected persona-not-found validation error, got {result:?}"
    );
}

#[test]
fn test_create_game_generates_unique_names() {
    let (catalogue, _storage, world_key, persona_key) = seeded_catalogue();

    let id1 = catalogue.create_game(&world_key, &persona_key).unwrap();
    let id2 = catalogue.create_game(&world_key, &persona_key).unwrap();
    let id3 = catalogue.create_game(&world_key, &persona_key).unwrap();

    let g1 = catalogue
        .list_games()
        .unwrap()
        .into_iter()
        .find(|g| g.id == id1)
        .unwrap();
    let g2 = catalogue
        .list_games()
        .unwrap()
        .into_iter()
        .find(|g| g.id == id2)
        .unwrap();
    let g3 = catalogue
        .list_games()
        .unwrap()
        .into_iter()
        .find(|g| g.id == id3)
        .unwrap();

    assert_ne!(g1.name, g2.name, "game names should be unique");
    assert_ne!(g2.name, g3.name, "game names should be unique");
    assert_ne!(g1.name, g3.name, "game names should be unique");
}

#[test]
fn test_create_game_restores_current_game_on_persist_failure() {
    let data = TestDataBuilder::default_test().build();
    let raw_storage = Storage::new_in_memory();
    data.seed_into(&raw_storage);
    let (storage, handle) = raw_storage.with_test_failures();
    handle.set(
        "save_snapshot",
        TestOverride::internal("simulated initial snapshot failure"),
    );
    let storage = Arc::new(storage);
    let message_service = Arc::new(MessageService::new(Arc::clone(&storage)));
    let catalogue = GameCatalogue::new(
        Arc::clone(&storage),
        message_service,
        Arc::new(RwLock::new(AppSettings::default())),
    );
    let original_current = storage.current_game_id();

    let result = catalogue.create_game(&data.world_key(), &data.persona.key);

    assert!(
        result.is_err(),
        "create_game should fail when initial state cannot be persisted"
    );
    assert_eq!(
        storage.current_game_id(),
        original_current,
        "current game should be restored after rollback"
    );
}

#[test]
fn test_switch_game_changes_current_game() {
    let (catalogue, _storage, world_key, persona_key) = seeded_catalogue();
    let id1 = catalogue.create_game(&world_key, &persona_key).unwrap();
    let id2 = catalogue.create_game(&world_key, &persona_key).unwrap();

    catalogue
        .switch_game(id1)
        .expect("switch to first game should succeed");
    assert_eq!(catalogue.current_game_id(), id1);

    catalogue
        .switch_game(id2)
        .expect("switch to second game should succeed");
    assert_eq!(catalogue.current_game_id(), id2);
}

#[test]
fn test_switch_game_errors_when_game_missing() {
    let (catalogue, _storage, _world_key, _persona_key) = seeded_catalogue();

    let result = catalogue.switch_game(9999);

    assert!(
        matches!(result, Err(ApplicationError::Validation(ref msg)) if msg.contains("Game not found")),
        "Expected game-not-found validation error, got {result:?}"
    );
}

#[test]
fn test_delete_game_removes_non_active_game() {
    let (catalogue, storage, world_key, persona_key) = seeded_catalogue();
    let id1 = catalogue.create_game(&world_key, &persona_key).unwrap();
    let id2 = catalogue.create_game(&world_key, &persona_key).unwrap();
    catalogue.switch_game(id1).unwrap();

    catalogue
        .delete_game(id2)
        .expect("delete non-active game should succeed");

    assert!(
        storage.get_game(id2).unwrap().is_none(),
        "deleted game should not exist"
    );
}

#[test]
fn test_delete_game_errors_when_deleting_active_game() {
    let (catalogue, _storage, world_key, persona_key) = seeded_catalogue();
    let id = catalogue.create_game(&world_key, &persona_key).unwrap();

    let result = catalogue.delete_game(id);

    assert!(
        matches!(result, Err(ApplicationError::Validation(ref msg)) if msg.contains("Cannot delete the active game")),
        "Expected active-game deletion error, got {result:?}"
    );
}

#[test]
fn test_list_games_returns_all_games() {
    let (catalogue, _storage, world_key, persona_key) = seeded_catalogue();
    let id1 = catalogue.create_game(&world_key, &persona_key).unwrap();
    let id2 = catalogue.create_game(&world_key, &persona_key).unwrap();

    let games = catalogue.list_games().expect("list_games should succeed");
    let ids: Vec<_> = games.iter().map(|g| g.id).collect();

    assert!(ids.contains(&id1), "list should contain first game");
    assert!(ids.contains(&id2), "list should contain second game");
}

#[test]
fn test_reset_replaces_current_game() {
    let (catalogue, storage, world_key, persona_key) = seeded_catalogue();
    catalogue.create_game(&world_key, &persona_key).unwrap();
    let pre_reset_current = catalogue.current_game_id();

    catalogue.reset().expect("reset should succeed");

    let post_reset_current = catalogue.current_game_id();
    assert_ne!(
        post_reset_current, pre_reset_current,
        "reset should create a new current game"
    );
    assert!(
        storage.get_game(pre_reset_current).unwrap().is_none(),
        "pre-reset game should be deleted"
    );
}

#[test]
fn test_current_game_id_matches_storage() {
    let (catalogue, storage, _world_key, _persona_key) = seeded_catalogue();
    assert_eq!(catalogue.current_game_id(), storage.current_game_id());
}

#[test]
fn test_create_game_persists_scenario_message_and_swipe() {
    let (catalogue, storage, world_key, persona_key) = seeded_catalogue();

    let id = catalogue
        .create_game(&world_key, &persona_key)
        .expect("create_game should succeed");

    assert!(id > 0, "Game ID should be positive");
    let messages = storage.load_message_rows().unwrap();
    let narrations: Vec<_> = messages
        .into_iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .collect();
    assert_eq!(
        narrations.len(),
        1,
        "exactly one scenario Narration should be persisted"
    );
    let swipe_count = storage.count_swipes_for_message(narrations[0].id).unwrap();
    assert!(
        swipe_count > 0,
        "scenario message should have at least one swipe"
    );
}

#[test]
fn test_delete_game_succeeds_silently_for_nonexistent_game() {
    let (catalogue, _storage, _world_key, _persona_key) = seeded_catalogue();

    catalogue
        .delete_game(99999)
        .expect("delete_game should succeed silently for nonexistent game");
}

fn seed_library(storage: &Storage) {
    let presets = [
        (
            "system_default",
            "System Default",
            PresetType::System,
            vec![NarratorMode::Novel],
        ),
        (
            "system_if_default",
            "System IF",
            PresetType::System,
            vec![NarratorMode::InteractiveFiction],
        ),
        (
            "quantifier_default",
            "Quantifier Default",
            PresetType::Quantifier,
            settings_defaults::default_allowed_modes(),
        ),
        (
            "impersonate_default",
            "Impersonate Default",
            PresetType::Impersonate,
            settings_defaults::default_allowed_modes(),
        ),
        (
            "system_custom",
            "System Custom",
            PresetType::System,
            settings_defaults::default_allowed_modes(),
        ),
    ];
    for (id, name, preset_type, allowed_modes) in presets {
        storage
            .save_preset(&PromptPreset {
                id: id.to_string(),
                name: name.to_string(),
                role: None,
                instructions: None,
                writing_style: None,
                output_format: None,
                allowed_modes,
                is_default: true,
                preset_type,
            })
            .expect("test setup: save_preset must succeed");
    }
}

#[test]
fn test_current_game_returns_the_seeded_game() {
    let (catalogue, _storage, _world_key, _persona_key) = seeded_catalogue();

    let current = catalogue.current_game().expect("current_game should load");
    assert!(current.is_some(), "seeded catalogue has an active game");
}

#[test]
fn test_set_posture_updates_perspective_and_tense() {
    let (catalogue, storage, _world_key, _persona_key) = seeded_catalogue();
    let id = catalogue.current_game_id();

    let game = catalogue
        .set_posture(id, NarrativePerspective::Second, NarrativeTense::Present)
        .expect("set_posture should succeed");
    assert_eq!(game.narrative_perspective, NarrativePerspective::Second);
    assert_eq!(game.narrative_tense, NarrativeTense::Present);

    let stored = storage.get_game(id).unwrap().expect("game persisted");
    assert_eq!(stored.narrative_perspective, NarrativePerspective::Second);
    assert_eq!(stored.narrative_tense, NarrativeTense::Present);
    // Mode is untouched by the posture save.
    assert_eq!(stored.narrator_mode, NarratorMode::Novel);
}

#[test]
fn test_set_posture_errors_on_missing_game() {
    let (catalogue, _storage, _world_key, _persona_key) = seeded_catalogue();

    let result = catalogue.set_posture(9999, NarrativePerspective::Second, NarrativeTense::Past);
    assert!(
        matches!(result, Err(ApplicationError::Validation(ref msg)) if msg.contains("Game not found")),
        "Expected game-not-found validation error, got {result:?}"
    );
}

#[test]
fn test_switch_mode_retargets_presets_and_nudges_perspective() {
    let (catalogue, storage, _world_key, _persona_key) = seeded_catalogue();
    let id = catalogue.current_game_id();

    let game = catalogue
        .switch_mode(id, NarratorMode::InteractiveFiction)
        .expect("switch_mode should succeed");
    assert_eq!(game.narrator_mode, NarratorMode::InteractiveFiction);
    assert_eq!(game.narrative_perspective, NarrativePerspective::Second);
    assert_eq!(game.active_system_prompt_preset_id, "system_if_default");

    let stored = storage.get_game(id).unwrap().expect("game persisted");
    assert_eq!(stored.narrator_mode, NarratorMode::InteractiveFiction);
    assert_eq!(stored.active_system_prompt_preset_id, "system_if_default");
}

#[test]
fn test_switch_mode_keeps_a_deliberate_perspective() {
    let (catalogue, _storage, _world_key, _persona_key) = seeded_catalogue();
    let id = catalogue.current_game_id();

    // An IF game deliberately held in third person.
    catalogue
        .switch_mode(id, NarratorMode::InteractiveFiction)
        .unwrap();
    catalogue
        .set_posture(id, NarrativePerspective::Third, NarrativeTense::Past)
        .unwrap();

    let game = catalogue
        .switch_mode(id, NarratorMode::Novel)
        .expect("switch_mode should succeed");
    assert_eq!(game.narrator_mode, NarratorMode::Novel);
    assert_eq!(
        game.narrative_perspective,
        NarrativePerspective::Third,
        "a deliberately-set perspective is never clobbered"
    );
}

#[test]
fn test_switch_mode_same_mode_is_a_noop() {
    let (catalogue, storage, _world_key, _persona_key) = seeded_catalogue();
    let id = catalogue.current_game_id();
    seed_library(storage.as_ref());

    // Move to IF (retargets to the IF bundle), then customize the selection
    // away from the bundle default.
    catalogue
        .switch_mode(id, NarratorMode::InteractiveFiction)
        .unwrap();
    catalogue
        .set_preset_selection(
            id,
            "system_custom",
            "quantifier_default",
            "impersonate_default",
        )
        .unwrap();

    let game = catalogue
        .switch_mode(id, NarratorMode::InteractiveFiction)
        .expect("switch_mode should succeed");
    assert_eq!(
        game.active_system_prompt_preset_id, "system_custom",
        "same-mode switch must not retarget presets"
    );
}

#[test]
fn test_set_preset_selection_updates_ids() {
    let (catalogue, storage, _world_key, _persona_key) = seeded_catalogue();
    let id = catalogue.current_game_id();
    seed_library(storage.as_ref());

    // IF mode first: system_if_default is IF-only, and selecting it must be
    // a change from the Novel bundle default.
    catalogue
        .switch_mode(id, NarratorMode::InteractiveFiction)
        .unwrap();

    let game = catalogue
        .set_preset_selection(
            id,
            "system_if_default",
            "quantifier_default",
            "impersonate_default",
        )
        .expect("set_preset_selection should succeed");
    assert_eq!(game.active_system_prompt_preset_id, "system_if_default");

    let stored = storage.get_game(id).unwrap().expect("game persisted");
    assert_eq!(stored.active_system_prompt_preset_id, "system_if_default");
}

#[test]
fn test_set_preset_selection_rejects_unknown_preset() {
    let (catalogue, storage, _world_key, _persona_key) = seeded_catalogue();
    let id = catalogue.current_game_id();
    seed_library(storage.as_ref());

    let result =
        catalogue.set_preset_selection(id, "no_such", "quantifier_default", "impersonate_default");
    assert!(
        matches!(result, Err(ApplicationError::Validation(ref msg)) if msg.contains("Preset not found")),
        "Expected preset-not-found validation error, got {result:?}"
    );
}

#[test]
fn test_set_preset_selection_rejects_mode_disallowed_preset() {
    let (catalogue, storage, _world_key, _persona_key) = seeded_catalogue();
    let id = catalogue.current_game_id();
    seed_library(storage.as_ref());

    // system_if_default is IF-only; the game is in Novel mode.
    let result = catalogue.set_preset_selection(
        id,
        "system_if_default",
        "quantifier_default",
        "impersonate_default",
    );
    assert!(
        matches!(result, Err(ApplicationError::Validation(ref msg)) if msg.contains("not allowed")),
        "Expected mode-allow validation error, got {result:?}"
    );

    // And the stored selection must be untouched by the refusal.
    let stored = storage.get_game(id).unwrap().expect("game persisted");
    assert_eq!(stored.active_system_prompt_preset_id, "system_default");
}

#[test]
fn test_set_preset_selection_accepts_unchanged_stored_ids() {
    let (catalogue, _storage, _world_key, _persona_key) = seeded_catalogue();
    let id = catalogue.current_game_id();

    // No library seeds: the game's default bundle ids are not stored presets.
    // Saving the form unchanged must still succeed (no-op slots).
    let game = catalogue
        .set_preset_selection(
            id,
            "system_default",
            "quantifier_default",
            "impersonate_default",
        )
        .expect("unchanged stored ids must pass without a library");
    assert_eq!(game.active_system_prompt_preset_id, "system_default");
}
