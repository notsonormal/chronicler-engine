use crate::domain::model::state::game_state::GameStateBuilder;
use crate::domain::model::state::generation_status::GenerationStatus;
use crate::domain::model::state::game_state_snapshot::{GameStateSnapshot, NarrativeSnapshot};
use crate::domain::model::state::narrative_state::NarrativeState;
use crate::test_support::create_test_state;

#[test]
fn test_narrative_snapshot_default() {
    let snap = NarrativeSnapshot::default();
    assert_eq!(snap.input_buffer.status, GenerationStatus::Idle);
    assert!(snap.last_trigger.is_none());
    assert!(snap.pending_location.is_none());
    assert!(snap.pending_event.is_none());
}

#[test]
fn test_from_game_state_sets_defaults() {
    let state = GameStateBuilder::new("start").build();

    let snapshot = GameStateSnapshot::from_game_state(&state);
    assert!(snapshot.db_id.is_none());
    assert!(
        snapshot.created_at <= chrono::Utc::now(),
        "created_at should be in the past"
    );
}

#[test]
fn test_snapshot_captures_state_fields() {
    let mut state = create_test_state();
    state.movement.current_room_id = "room2".to_string();
    state.scene.npcs_in_area.clear();
    state.narrative.last_trigger = None;

    let snapshot = GameStateSnapshot::from_game_state(&state);
    assert_eq!(
        snapshot.movement.current_room_id, "room2",
        "snapshot should capture modified current_room_id"
    );
    assert_eq!(
        snapshot.scene.npcs_in_area.len(),
        0,
        "snapshot should capture empty npcs_in_area"
    );
    assert_eq!(
        snapshot.narrative.last_trigger, None,
        "snapshot should capture None last_trigger"
    );
}

#[test]
fn test_narrative_snapshot_roundtrips_current_options() {
    let mut state = create_test_state();
    state.narrative.current_options = vec![
        "Search the desk".to_string(),
        "Question the guard".to_string(),
    ];

    let snapshot = GameStateSnapshot::from_game_state(&state);
    assert_eq!(snapshot.narrative.current_options.len(), 2);

    let restored = NarrativeState::from_snapshot(&snapshot.narrative);
    assert_eq!(restored.current_options, state.narrative.current_options);
}

#[test]
fn test_narrative_snapshot_without_current_options_deserializes_empty() {
    // A pre-options snapshot JSON must still load: the field is serde-defaulted.
    let legacy = r#"{
        "generation": {"input": "", "status": "Idle", "phase": "Narrating"},
        "last_trigger": null,
        "pending_location": null,
        "pending_event": null,
        "last_backend_name": null,
        "last_model_name": null
    }"#;
    let snapshot: NarrativeSnapshot =
        serde_json::from_str(legacy).expect("legacy snapshot JSON should deserialize");
    assert!(snapshot.current_options.is_empty());
}
