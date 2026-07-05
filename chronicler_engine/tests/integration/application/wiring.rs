//! Integration wiring test — exercises the prod composition path
//! `bootstrap::wiring::build_game_service` → `llm_factory::get_llm_recorder_for`
//! end-to-end.
//!
//! Catch silent-fallback regression (Fix 2 in phase2-thermonuclear-review-fixes.md):
//! if someone reintroduces `unwrap_or_else(Mock+Noop)`, the forensics assertion
//! fails because the recorder's forensics repo must be the real `Storage`.

use std::sync::{Arc, RwLock};

use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::domain::model::llm_backend::LlmBackendType;
use chronicler_engine::domain::model::settings::{AppSettings, LlmProviderConfig};

#[test]
fn with_storage_wires_recorder_to_provider_and_storage() {
    // AppSettings with Mock as narration connection — avoids real API calls.
    let mock_connection = LlmProviderConfig {
        id: "test-mock".into(),
        name: "test-mock".into(),
        provider: LlmBackendType::Mock,
        model: "mock-model".into(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    };
    let settings = AppSettings {
        connections: vec![mock_connection],
        narration_connection_id: "test-mock".into(),
        quantifier_connection_id: "test-mock".into(),
        ..Default::default()
    };

    let storage = Arc::new(Storage::new_in_memory());
    let preset_storage = Arc::new(Storage::new_in_memory());
    let settings_arc = Arc::new(RwLock::new(settings));

    let game_service = chronicler_engine::bootstrap::wiring::build_game_service(
        Arc::clone(&settings_arc),
        Arc::clone(&storage),
        Arc::clone(&preset_storage),
    )
    .expect("build_game_service should succeed with Mock connection");

    // Recorder wired to Mock provider (not OpenRouter default).
    assert_eq!(game_service.llm_recorder.provider().name(), "Mock");

    // Drive the recorder directly — this proves the factory wired Storage as
    // the forensics repository. If the silent fallback to NoopForensics is
    // reintroduced, the saved message never reaches Storage and the
    // list_latest_llm_messages assertion below fails.
    game_service
        .llm_recorder
        .complete("wiring-test-agent", "sys", "usr", None)
        .expect("recorder.complete should succeed against MockBackend");

    // The forensics row must have landed in the real Storage, proving
    // factory + recorder + storage are all in sync.
    let messages = storage
        .list_latest_llm_messages(10)
        .expect("Storage::list_latest_llm_messages should succeed");
    assert_eq!(
        messages.len(),
        1,
        "expected exactly one LlmMessage persisted to Storage"
    );
    assert_eq!(messages[0].agent_name, "wiring-test-agent");
    assert_eq!(messages[0].backend_name, "Mock");
}
