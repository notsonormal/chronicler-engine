//! Shared fixtures for integration tests: builds storage instances with deterministic defaults so tests can focus on the behaviour under test.

use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::adapters::driven::storage::db::DbPool;

// Pre-seeds the games row so `game_state_snapshots.game_id` / `messages.game_id` FKs hold.
pub fn create_test_storage(game_id: u64) -> Storage {
    let pool = DbPool::new(":memory:").expect("in-memory db should open");
    chronicler_engine::test_support::seed_default_game_row(&pool, game_id).unwrap();
    Storage::new_sqlite(pool, game_id)
}
