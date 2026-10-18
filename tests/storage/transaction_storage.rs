//! The storage transaction seam: one unit of record writes that lands whole, and the newest Game of a World, on both backends.

use chrono::{DateTime, Utc};
use chronicler_engine::adapters::driven::storage::{Storage, TestOverride, TxWrite};
use chronicler_engine::domain::model::game::{GameId, NewGame};
use chronicler_engine::domain::model::settings::{NarrativePerspective, NarrativeTense, NarratorMode};

const WORLD_KEY: &str = "seam_world";
/// A Message time the wall clock cannot reach while these cases run.
const FUTURE_MESSAGE_TIME: &str = "2099-01-01T00:00:00+00:00";
/// A Message time older than every `updated_at` these cases write.
const PAST_MESSAGE_TIME: &str = "2020-01-01T00:00:00+00:00";

fn backends() -> Vec<(&'static str, Storage)> {
    vec![
        (
            "sqlite",
            chronicler_engine::test_support::empty_sqlite_storage(),
        ),
        ("in-memory", Storage::new_in_memory()),
    ]
}

fn new_game(name: &str, world_key: &str) -> NewGame {
    NewGame {
        world_name: "Seam World".to_string(),
        world_key: world_key.to_string(),
        persona_key: "seam_persona".to_string(),
        persona_name: "Seam Persona".to_string(),
        name: name.to_string(),
        narrator_mode: NarratorMode::Novel,
        narrative_perspective: NarrativePerspective::Third,
        narrative_tense: NarrativeTense::Past,
        system_prompt_preset_id: "system_default".to_string(),
        quantifier_prompt_preset_id: "quantifier_default".to_string(),
        impersonate_prompt_preset_id: "impersonate_default".to_string(),
        options_prompt_preset_id: "options_default".to_string(),
        options_always_on: false,
    }
}

fn create_game(storage: &Storage, name: &str) -> GameId {
    let id = storage
        .create_game_from_request(&new_game(name, WORLD_KEY))
        .expect("create_game_from_request should succeed");
    GameId(id)
}

/// The newest Message of `game_id` carries the fixed `stamp`, so its key is
/// known without reading the wall clock.
fn insert_message_at(storage: &Storage, game_id: GameId, stamp: &str) {
    let mut message = chronicler_engine::test_support::dummy_message("hello");
    message.timestamp = DateTime::parse_from_rfc3339(stamp)
        .expect("a fixed RFC 3339 timestamp")
        .with_timezone(&Utc);
    storage.insert_message(game_id, &message).unwrap();
}

// CASE storage-tx-rollback [failure]: a unit whose second record write fails leaves no row
#[test]
fn tx_rollback_leaves_no_row_when_the_second_write_fails() {
    let (storage, handle) =
        chronicler_engine::test_support::empty_sqlite_storage().with_test_failures();
    handle.set_write(
        TxWrite::DeleteGame,
        TestOverride::internal("the second write fails"),
    );

    let result = storage.transaction(|tx| {
        let game_id = tx.insert_game(&new_game("Rolled back", WORLD_KEY))?;
        tx.delete_game(game_id)?;
        Ok(())
    });

    assert!(result.is_err(), "the injected failure fails the unit");
    assert!(
        storage.list_games().unwrap().is_empty(),
        "the first write must roll back with the failed unit"
    );
}

// CASE storage-tx-new-game: the create unit writes the Game row and hands its id to the unit
#[test]
fn transaction_new_game_writes_the_game_row() {
    let storage = chronicler_engine::test_support::empty_sqlite_storage();

    let game_id = storage
        .transaction_new_game(&new_game("First", WORLD_KEY), |game_id, _tx| Ok(game_id))
        .expect("the create unit should succeed");

    let stored = storage
        .get_game(game_id.0)
        .unwrap()
        .expect("the Game row landed");
    assert_eq!(stored.world_key, WORLD_KEY);
    assert_eq!(stored.name, "First");
}

// CASE storage-tx-new-game-rollback [failure]: a failed write after the insert rolls the create unit back
#[test]
fn transaction_new_game_rolls_back_when_a_later_write_fails() {
    let (storage, handle) =
        chronicler_engine::test_support::empty_sqlite_storage().with_test_failures();
    handle.set_write(
        TxWrite::DeleteGame,
        TestOverride::internal("the later write fails"),
    );

    let result =
        storage.transaction_new_game(&new_game("Rolled back", WORLD_KEY), |game_id, tx| {
            tx.delete_game(game_id)?;
            Ok(game_id)
        });

    assert!(result.is_err(), "the injected failure fails the unit");
    assert!(
        storage.list_games().unwrap().is_empty(),
        "the create unit must leave no Game row"
    );
}

// CASE storage-tx-no-in-memory-arm [failure]: the in-memory backend refuses a unit
#[test]
fn transaction_refuses_the_in_memory_backend() {
    let storage = Storage::new_in_memory();

    let result = storage.transaction(|_tx| Ok(()));

    assert!(
        result.is_err(),
        "the in-memory backend holds no unit; tier 1 runs on SQLite"
    );
}

// CASE storage-newest-game-message: a Game's newest Message answers before its `updated_at`
#[test]
fn newest_game_of_world_uses_the_newest_message() {
    for (label, storage) in backends() {
        let with_message = create_game(&storage, "With message");
        // A second Game with a newer `updated_at`: the Message of the first
        // still answers.
        create_game(&storage, "Without message");
        insert_message_at(&storage, with_message, FUTURE_MESSAGE_TIME);

        assert_eq!(
            storage.newest_game_of_world(WORLD_KEY).unwrap(),
            Some(with_message),
            "{label}: the newest Message of a Game decides"
        );
    }
}

// CASE storage-newest-game-updated-at: without a Message, `updated_at` answers
#[test]
fn newest_game_of_world_falls_back_to_updated_at() {
    for (label, storage) in backends() {
        let with_old_message = create_game(&storage, "With old message");
        insert_message_at(&storage, with_old_message, PAST_MESSAGE_TIME);
        // A Game with no Message is keyed by its `updated_at`, which is now.
        let without_message = create_game(&storage, "Without message");

        assert_eq!(
            storage.newest_game_of_world(WORLD_KEY).unwrap(),
            Some(without_message),
            "{label}: a Game with no Message is keyed by `updated_at`"
        );
    }
}

// CASE storage-newest-game-tie: two Games share the newest key, and the higher id answers
#[test]
fn newest_game_of_world_breaks_a_tie_on_the_higher_id() {
    for (label, storage) in backends() {
        let first = create_game(&storage, "First");
        let second = create_game(&storage, "Second");
        insert_message_at(&storage, first, FUTURE_MESSAGE_TIME);
        insert_message_at(&storage, second, FUTURE_MESSAGE_TIME);

        assert_eq!(
            storage.newest_game_of_world(WORLD_KEY).unwrap(),
            Some(second),
            "{label}: the higher Game id breaks the tie"
        );
    }
}

// CASE storage-newest-game-unknown-world: a World with no Game answers None, and another World's Game does not answer
#[test]
fn newest_game_of_world_ignores_other_worlds() {
    for (label, storage) in backends() {
        create_game(&storage, "Elsewhere");
        let other = storage
            .create_game_from_request(&new_game("Other", "other_world"))
            .unwrap();

        assert_eq!(
            storage.newest_game_of_world("no_such_world").unwrap(),
            None,
            "{label}: a World with no Game has no newest Game"
        );
        assert_eq!(
            storage.newest_game_of_world("other_world").unwrap(),
            Some(GameId(other)),
            "{label}: the lookup filters by World"
        );
    }
}
