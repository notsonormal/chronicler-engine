//! Tests for `SettingsService`.

use std::sync::Arc;

use crate::application::settings_service::SettingsService;
use crate::domain::model::settings::AppSettings;

type Storage = crate::adapters::driven::storage::Storage;

fn make_service() -> (SettingsService, Arc<Storage>) {
    let storage = Arc::new(Storage::new_in_memory());
    let service = SettingsService::new(Arc::clone(&storage));
    (service, storage)
}

#[test]
fn test_save_settings_roundtrip() {
    let (service, storage) = make_service();
    let settings = AppSettings {
        narration_connection_id: "narration-1".to_string(),
        quantifier_connection_id: "quantifier-1".to_string(),
        ..AppSettings::default()
    };
    service.save_settings(&settings).unwrap();

    let loaded = storage.get_settings().unwrap();
    assert_eq!(loaded.narration_connection_id, "narration-1");
    assert_eq!(loaded.quantifier_connection_id, "quantifier-1");
}
