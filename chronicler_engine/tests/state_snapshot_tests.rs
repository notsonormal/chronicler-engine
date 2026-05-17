mod test_data;

use std::sync::Arc;

use chronicler_engine::model::state::GameState;
use chronicler_engine::model::state_snapshot::GameStateSnapshot;

use test_data::create_test_state;

#[test]
fn test_apply_to_restores_state() {
    let original = create_test_state();
    let snapshot = GameStateSnapshot::from_game_state(&original);

    // Create a fresh empty-ish state with different starting room
    let mut target = GameState::new(
        Arc::clone(&original.world),
        Arc::clone(&original.map),
        Arc::clone(&original.player),
        original.npcs.values().cloned().collect(),
        "room2".to_string(), // different room
    );

    // Verify target starts differently
    assert_ne!(
        target.movement.current_room_id,
        original.movement.current_room_id
    );

    snapshot.apply_to(&mut target);

    // After apply_to, target should match original's snapshot data
    assert_eq!(
        target.movement.current_room_id,
        original.movement.current_room_id
    );
    assert_eq!(
        target.narrative.history().len(),
        original.narrative.history().len()
    );
    assert_eq!(target.scene.npcs_in_area, original.scene.npcs_in_area);
    assert_eq!(
        target.character_state.npcs.len(),
        original.character_state.npcs.len()
    );
}

#[test]
fn test_from_game_state_sets_defaults() {
    let state = create_test_state();
    let snapshot = GameStateSnapshot::from_game_state(&state);

    assert!(snapshot.db_id.is_none());
    assert!(!snapshot.committed, "New snapshot should not be committed");
    assert!(
        snapshot.created_at <= chrono::Utc::now(),
        "created_at should be in the past"
    );
}
