use std::sync::Arc;

use chronicler_engine::application::game_service::{DefaultGameService, GameService};
use chronicler_engine::model::character::{CharacterSheet, PlayerCard};
use chronicler_engine::model::map::{MapDef, Overworld};
use chronicler_engine::model::settings::AppSettings;
use chronicler_engine::model::world::WorldCard;
use chronicler_engine::server::AppState;
use chronicler_engine::storage::llm_message_storage::InMemoryLlmMessageStorage;
use chronicler_engine::storage::prompt_preset_storage::InMemoryPromptPresetStorage;
use chronicler_engine::test_support::{InMemoryMessageRepository, InMemorySnapshotRepository};
use tokio_util::sync::CancellationToken;

#[test]
fn test_settings_recover_from_poisoned_rwlock() {
    let settings = Arc::new(std::sync::RwLock::new(AppSettings {
        narration_connection_id: "poison-test".to_string(),
        ..AppSettings::default()
    }));

    let settings_clone = Arc::clone(&settings);
    let _ = std::thread::spawn(move || {
        let _guard = settings_clone.write().unwrap();
        panic!("intentional panic to poison lock");
    })
    .join();

    let game_service: Arc<dyn GameService> = Arc::new(DefaultGameService::with_storage(
        None,
        Arc::new(std::sync::RwLock::new(AppSettings::default())),
    ));
    let app_state = AppState {
        snapshot_storage: Arc::new(InMemorySnapshotRepository::new()),
        message_storage: Arc::new(InMemoryMessageRepository::new()),
        llm_message_storage: Arc::new(InMemoryLlmMessageStorage::new()),
        prompt_preset_storage: Arc::new(InMemoryPromptPresetStorage::new()),
        world: Arc::new(WorldCard::default()),
        map: Arc::new(MapDef {
            overworld: Overworld {
                id: "overworld".to_string(),
                name: "Overworld".to_string(),
                regions: vec![],
            },
        }),
        player: Arc::new(PlayerCard {
            sheet: CharacterSheet {
                name: "Hero".to_string(),
                description: "A hero".to_string(),
                personality: "Brave".to_string(),
                scenario: "Default".to_string(),
                example_dialogue: "".to_string(),
                summary: None,
                profile_image: None,
                headshot_image: None,
            },
            inventory: vec![],
        }),
        npcs: Arc::new(std::collections::HashMap::new()),
        game_service: Arc::clone(&game_service),
        application_service: Arc::new(
            chronicler_engine::application::application_service::DefaultApplicationService::new(
                game_service,
            ),
        ),
        settings,
        cancel_token: Arc::new(std::sync::RwLock::new(CancellationToken::new())),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    };

    let recovered = app_state.settings();
    assert_eq!(
        recovered.narration_connection_id, "poison-test",
        "settings() should recover actual settings from poisoned RwLock"
    );
}

#[test]
fn test_cancel_token_recover_from_poisoned_rwlock() {
    let token = CancellationToken::new();
    let cancel_token = Arc::new(std::sync::RwLock::new(token.clone()));

    let cancel_clone = Arc::clone(&cancel_token);
    let _ = std::thread::spawn(move || {
        let _guard = cancel_clone.write().unwrap();
        panic!("intentional panic to poison lock");
    })
    .join();

    let game_service: Arc<dyn GameService> = Arc::new(DefaultGameService::with_storage(
        None,
        Arc::new(std::sync::RwLock::new(AppSettings::default())),
    ));
    let app_state = AppState {
        snapshot_storage: Arc::new(InMemorySnapshotRepository::new()),
        message_storage: Arc::new(InMemoryMessageRepository::new()),
        llm_message_storage: Arc::new(InMemoryLlmMessageStorage::new()),
        prompt_preset_storage: Arc::new(InMemoryPromptPresetStorage::new()),
        world: Arc::new(WorldCard::default()),
        map: Arc::new(MapDef {
            overworld: Overworld {
                id: "overworld".to_string(),
                name: "Overworld".to_string(),
                regions: vec![],
            },
        }),
        player: Arc::new(PlayerCard {
            sheet: CharacterSheet {
                name: "Hero".to_string(),
                description: "A hero".to_string(),
                personality: "Brave".to_string(),
                scenario: "Default".to_string(),
                example_dialogue: "".to_string(),
                summary: None,
                profile_image: None,
                headshot_image: None,
            },
            inventory: vec![],
        }),
        npcs: Arc::new(std::collections::HashMap::new()),
        game_service: Arc::clone(&game_service),
        application_service: Arc::new(
            chronicler_engine::application::application_service::DefaultApplicationService::new(
                game_service,
            ),
        ),
        settings: Arc::new(std::sync::RwLock::new(AppSettings::default())),
        cancel_token,
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    };

    let recovered = app_state.current_cancel_token();
    assert!(
        !recovered.is_cancelled(),
        "current_cancel_token() should recover the actual token from poisoned RwLock"
    );
}
