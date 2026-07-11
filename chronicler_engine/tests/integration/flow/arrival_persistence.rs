//! Integration flow tests for arrival narration persistence — confirms the arrival narration survives a state reload, exercising the `ArrivalTaskContext` end-to-end against SQLite storage.

use std::sync::Arc;

use chronicler_engine::application::arrival_service::ArrivalTaskContext;
use chronicler_engine::application::application_service::DefaultApplicationService;
use chronicler_engine::application::game_service::GameService;
use chronicler_engine::adapters::driven::llm::providers::MockBackend;
use chronicler_engine::adapters::driven::storage::TestOverride;
use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::domain::model::character::NpcCard;
use chronicler_engine::domain::model::settings::AppSettings;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::test_support::{
    make_test_recorder_with_storage, seed_test_world_into_storage, TestDataBuilder,
};

use crate::pipeline_helpers::{create_test_state_with_map, latest_state};
use crate::sqlite_test_app_builder::SqliteTestAppBuilder;

#[test]
fn test_arrival_narration_survives_reload() {
    let preset_storage = chronicler_engine::test_support::default_test_preset_storage();
    let arrival_preset = preset_storage
        .get_preset("system_default")
        .ok()
        .flatten()
        .expect("Should have system_default preset");

    let nearby_npcs: Vec<NpcCard> = vec![];
    let all_npcs: Vec<NpcCard> = vec![];

    let llm: Arc<dyn chronicler_engine::application::ports::llm_provider::LlmProvider> =
        Arc::new(MockBackend::default());
    let data = TestDataBuilder::default_test()
        .world(crate::fixtures::create_test_world())
        .map(crate::fixtures::create_test_map())
        .persona(crate::fixtures::create_test_player())
        .npcs(crate::fixtures::create_test_npcs())
        .build();
    let llm_for_closure = Arc::clone(&llm);
    let app = SqliteTestAppBuilder::with_data(data)
        .game_service_fn(move |storage| {
            let recorder = make_test_recorder_with_storage(
                Arc::clone(&llm_for_closure)
                    as Arc<dyn chronicler_engine::application::ports::llm_provider::LlmProvider>,
                Arc::clone(storage),
            );
            Arc::new(GameService::with_mock_quantifier(
                Arc::clone(&recorder),
                Arc::new(MockBackend::default()),
            ))
        })
        .build_service()
        .unwrap();

    let recorder = make_test_recorder_with_storage(Arc::clone(&llm), Arc::clone(app.storage()));
    let task_ctx = ArrivalTaskContext::new_for_test(
        Arc::clone(&app),
        "room1".to_string(),
        nearby_npcs,
        all_npcs,
        Some(arrival_preset),
        "short".to_string(),
        1024,
        None,
        recorder,
    );

    task_ctx.run_sync();

    let messages = app.storage().list_latest_llm_messages(50).unwrap();
    let narration_msgs: Vec<_> = messages
        .iter()
        .filter(|m| m.agent_name == "narrator")
        .collect();

    assert!(
        !narration_msgs.is_empty(),
        "Should have at least one narration message in llm_messages table after ArrivalTaskContext::run"
    );

    let first_narration = narration_msgs.first().unwrap();
    assert_ne!(
        first_narration.id, 0,
        "Narration message should have non-zero id (persisted to llm_messages table)"
    );

    let guard = latest_state(&app);
    let history_narrations: Vec<_> = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.message_type == MessageType::Narration)
        .collect();

    assert!(
        !history_narrations.is_empty(),
        "GameState should contain narration after ArrivalTaskContext::run"
    );

    let history_narration = history_narrations.first().unwrap();
    let history_narration_text = history_narration.text.clone();

    let reloaded_messages =
        chronicler_engine::application::application_service::load_messages_with_swipes(
            app.storage(),
        )
        .unwrap();

    assert!(
        !reloaded_messages.is_empty(),
        "load_messages_with_swipes should return the persisted narration message"
    );

    let reloaded_narration = reloaded_messages
        .iter()
        .find(|m| m.message_type == MessageType::Narration)
        .expect("Should find narration message after reload");

    assert_eq!(
        reloaded_narration.id, history_narration.id,
        "Reloaded narration message id should match the one in snapshot history (survives reload)"
    );

    assert_eq!(
        reloaded_narration.text(),
        history_narration_text,
        "Reloaded narration should match the one produced by ArrivalTaskContext::run"
    );
}

#[test]
fn arrival_service_tests_falls_back_to_fresh_state_on_load_error() {
    let state = create_test_state_with_map();

    let (failing_storage, handle) = Storage::new_in_memory().with_test_failures();
    let failing_storage = Arc::new(failing_storage);
    seed_test_world_into_storage(&failing_storage, &state);
    handle.set(
        "load_latest_snapshot",
        TestOverride::internal("simulated load_latest_snapshot failure"),
    );

    let preset_storage = Storage::new_in_memory();
    let _ = preset_storage.save_preset(
        &chronicler_engine::domain::model::prompt_preset::PromptPreset {
            id: "system_default".to_string(),
            name: "Default System".to_string(),
            role: Some("You are a narrator.".to_string()),
            instructions: None,
            writing_style: None,
            output_format: None,
            is_default: true,
            preset_type: chronicler_engine::domain::model::prompt_preset::PresetType::System,
        },
    );
    let arrival_preset = preset_storage
        .get_preset("system_default")
        .ok()
        .flatten()
        .expect("Should have system_default preset");

    let llm: Arc<dyn chronicler_engine::application::ports::llm_provider::LlmProvider> =
        Arc::new(MockBackend::default());
    let recorder = make_test_recorder_with_storage(Arc::clone(&llm), Arc::clone(&failing_storage));
    let game_service = Arc::new(GameService::with_mock_quantifier(
        Arc::clone(&recorder),
        Arc::new(MockBackend::default()),
    ));

    let app = Arc::new(DefaultApplicationService::new(
        Arc::clone(&failing_storage),
        Arc::new(preset_storage),
        Arc::new(std::sync::RwLock::new(AppSettings::default())),
        tokio_util::sync::CancellationToken::new(),
        Arc::new(std::sync::atomic::AtomicBool::new(false)),
        Arc::clone(&game_service),
    ));

    let task_ctx = ArrivalTaskContext::new_for_test(
        Arc::clone(&app),
        "room1".to_string(),
        Vec::<NpcCard>::new(),
        Vec::<NpcCard>::new(),
        Some(arrival_preset),
        "short".to_string(),
        1024,
        None,
        recorder,
    );

    task_ctx.run_sync();

    handle.clear("load_latest_snapshot");

    let guard = app.load_or_fresh();
    let narrations: Vec<_> = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.message_type == MessageType::Narration)
        .collect();
    assert!(
        !narrations.is_empty(),
        "arrival_service should have run (fresh-state fallback); expected at least one narration message after load failure, got {} total messages",
        guard.narrative.history().len(),
    );
}

// B2 fail-loud: `ArrivalTaskContext::run` must bail and emit no narration when `get_world` returns `Err`.

#[test]
fn arrival_service_returns_early_without_narration_on_world_fetch_failure() {
    let state = create_test_state_with_map();

    let (failing_storage, handle) = Storage::new_in_memory().with_test_failures();
    let failing_storage = Arc::new(failing_storage);
    seed_test_world_into_storage(&failing_storage, &state);
    let _ = failing_storage.save_snapshot(
        &chronicler_engine::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &state,
        ),
    );
    handle.set(
        "get_world",
        TestOverride::internal("simulated get_world failure"),
    );

    let preset_storage = Storage::new_in_memory();
    let _ = preset_storage.save_preset(
        &chronicler_engine::domain::model::prompt_preset::PromptPreset {
            id: "system_default".to_string(),
            name: "Default System".to_string(),
            role: Some("You are a narrator.".to_string()),
            instructions: None,
            writing_style: None,
            output_format: None,
            is_default: true,
            preset_type: chronicler_engine::domain::model::prompt_preset::PresetType::System,
        },
    );
    let arrival_preset = preset_storage
        .get_preset("system_default")
        .ok()
        .flatten()
        .expect("Should have system_default preset");

    let llm: Arc<dyn chronicler_engine::application::ports::llm_provider::LlmProvider> =
        Arc::new(MockBackend::default());
    let recorder = make_test_recorder_with_storage(Arc::clone(&llm), Arc::clone(&failing_storage));
    let game_service = Arc::new(GameService::with_mock_quantifier(
        Arc::clone(&recorder),
        Arc::new(MockBackend::default()),
    ));

    let app = Arc::new(DefaultApplicationService::new(
        Arc::clone(&failing_storage),
        Arc::new(preset_storage),
        Arc::new(std::sync::RwLock::new(AppSettings::default())),
        tokio_util::sync::CancellationToken::new(),
        Arc::new(std::sync::atomic::AtomicBool::new(false)),
        Arc::clone(&game_service),
    ));

    let task_ctx = ArrivalTaskContext::new_for_test(
        Arc::clone(&app),
        "room1".to_string(),
        Vec::<NpcCard>::new(),
        Vec::<NpcCard>::new(),
        Some(arrival_preset),
        "short".to_string(),
        1024,
        None,
        recorder,
    );

    task_ctx.run_sync();

    handle.clear("get_world");

    let guard = app.load_or_fresh();
    let narrations: Vec<_> = guard
        .narrative
        .history()
        .into_iter()
        .filter(|e| e.message_type == MessageType::Narration)
        .collect();
    assert!(
        narrations.is_empty(),
        "ArrivalTaskContext must not add narration when get_world fails, got {} narrations",
        narrations.len(),
    );
}
