//! The storage transaction seam: one unit of record writes that lands whole, and the newest Game of a World.

use chrono::{DateTime, Utc};
use chronicler_engine::adapters::driven::storage::{Storage, TestOverride, TxWrite};
use chronicler_engine::domain::model::game::{ConfigChange, GameId, NewGame, PresetSelection};
use chronicler_engine::domain::model::message::SwipeAt;
use chronicler_engine::domain::model::settings::{NarrativePerspective, NarrativeTense, NarratorMode};
use chronicler_engine::test_support::{test_new_game, tx_token};

const WORLD_KEY: &str = "seam_world";
const FUTURE_MESSAGE_TIME: &str = "2099-01-01T00:00:00+00:00";
const PAST_MESSAGE_TIME: &str = "2020-01-01T00:00:00+00:00";

fn sqlite() -> Storage {
    chronicler_engine::test_support::empty_sqlite_storage()
}

fn new_game(name: &str, world_key: &str) -> NewGame {
    test_new_game(
        "Seam World",
        world_key,
        "seam_persona",
        "Seam Persona",
        name,
    )
}

fn create_game(storage: &Storage, name: &str) -> GameId {
    chronicler_engine::test_support::seed_game_row(storage, &new_game(name, WORLD_KEY))
}

fn insert_message_at(storage: &Storage, game_id: GameId, stamp: &str) {
    let mut message = chronicler_engine::test_support::dummy_message("hello");
    message.timestamp = DateTime::parse_from_rfc3339(stamp)
        .expect("a fixed RFC 3339 timestamp")
        .with_timezone(&Utc);
    storage
        .transaction(game_id, tx_token(), |tx| {
            let message_id = tx.insert_message(&message)?;
            tx.insert_swipe(
                SwipeAt {
                    message: message_id,
                    swipe: 0,
                },
                &message.swipes[0],
            )
        })
        .expect("the unit should succeed");
}

// CASE storage-tx-rollback [failure]: a unit whose second record write fails leaves no row
#[test]
fn tx_rollback_leaves_no_row_when_the_second_write_fails() {
    let (storage, handle) =
        chronicler_engine::test_support::empty_sqlite_storage().with_test_failures();
    let game_id = create_game(&storage, "Rolled back");
    handle.set_write(
        TxWrite::InsertSwipe,
        TestOverride::internal("the second write fails"),
    );

    let result = storage.transaction(game_id, tx_token(), |tx| {
        let message = chronicler_engine::test_support::dummy_message("hello");
        let message_id = tx.insert_message(&message)?;
        tx.insert_swipe(
            SwipeAt {
                message: message_id,
                swipe: 0,
            },
            &chronicler_engine::test_support::dummy_swipe("hello"),
        )
    });

    assert!(result.is_err(), "the injected failure fails the unit");
    assert!(
        storage.stories().tail(game_id).unwrap().is_none(),
        "the first write must roll back with the failed unit"
    );
}

// CASE storage-tx-new-game: the create unit writes the Game row and hands its id to the unit
#[test]
fn transaction_new_game_writes_the_game_row() {
    let storage = chronicler_engine::test_support::empty_sqlite_storage();

    let game_id = storage
        .transaction_new_game(&new_game("First", WORLD_KEY), tx_token(), |game_id, _tx| {
            Ok(game_id)
        })
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

    let result = storage.transaction_new_game(
        &new_game("Rolled back", WORLD_KEY),
        tx_token(),
        |game_id, tx| {
            tx.delete_game()?;
            Ok(game_id)
        },
    );

    assert!(result.is_err(), "the injected failure fails the unit");
    assert!(
        storage.list_games().unwrap().is_empty(),
        "the create unit must leave no Game row"
    );
}

// CASE storage-tx-new-game-failure [failure]: a refused Game-row write leaves no row
#[test]
fn transaction_new_game_refuses_when_the_game_row_write_fails() {
    let (storage, handle) =
        chronicler_engine::test_support::empty_sqlite_storage().with_test_failures();
    handle.set_write(
        TxWrite::InsertGame,
        TestOverride::internal("the Game-row write fails"),
    );

    let result = storage.transaction_new_game(
        &new_game("Refused", WORLD_KEY),
        tx_token(),
        |game_id, _tx| Ok(game_id),
    );

    assert!(result.is_err(), "the injected failure fails the unit");
    assert!(
        storage.list_games().unwrap().is_empty(),
        "a refused create leaves no Game row"
    );
}

// CASE storage-tx-delete-game: the unit deletes the Game row and the records the schema cascades
#[test]
fn tx_delete_game_removes_the_game_and_its_records() {
    let storage = sqlite();
    let game_id = create_game(&storage, "Removed");
    insert_message_at(&storage, game_id, PAST_MESSAGE_TIME);

    storage
        .transaction(game_id, tx_token(), |tx| tx.delete_game())
        .expect("the unit should succeed");

    assert!(
        storage.get_game(game_id.0).unwrap().is_none(),
        "the Game row is gone"
    );
    assert!(
        storage.list_games().unwrap().is_empty(),
        "the Game's records cascade with it"
    );
}

// CASE storage-tx-config-posture: a Posture change writes the perspective and the tense, and leaves the mode and the presets as they were
#[test]
fn tx_config_posture_leaves_the_mode_and_the_presets() {
    let storage = sqlite();
    let game_id = create_game(&storage, "Posture");
    let before = storage.require_game(game_id.0).unwrap();

    storage
        .transaction(game_id, tx_token(), |tx| {
            tx.set_game_config(&ConfigChange::Posture {
                perspective: NarrativePerspective::Second,
                tense: NarrativeTense::Present,
            })
        })
        .expect("the unit should succeed");

    let after = storage.require_game(game_id.0).unwrap();
    assert_eq!(after.narrative_perspective, NarrativePerspective::Second);
    assert_eq!(after.narrative_tense, NarrativeTense::Present);
    assert_eq!(after.narrator_mode, before.narrator_mode);
    assert_eq!(
        after.active_system_prompt_preset_id,
        before.active_system_prompt_preset_id
    );
    assert_eq!(
        after.active_quantifier_prompt_preset_id,
        before.active_quantifier_prompt_preset_id
    );
    assert_eq!(
        after.active_impersonate_prompt_preset_id,
        before.active_impersonate_prompt_preset_id
    );
    assert_eq!(
        after.active_options_prompt_preset_id,
        before.active_options_prompt_preset_id
    );
}

// CASE storage-tx-config-presets: a Presets change writes the four preset slots, and leaves the mode and the posture as they were
#[test]
fn tx_config_presets_leaves_the_mode_and_the_posture() {
    let storage = sqlite();
    let game_id = create_game(&storage, "Presets");
    let before = storage.require_game(game_id.0).unwrap();

    storage
        .transaction(game_id, tx_token(), |tx| {
            tx.set_game_config(&ConfigChange::Presets(PresetSelection::new(
                "system_next",
                "quantifier_next",
                "impersonate_next",
                "options_next",
            )))
        })
        .expect("the unit should succeed");

    let after = storage.require_game(game_id.0).unwrap();
    assert_eq!(after.active_system_prompt_preset_id, "system_next");
    assert_eq!(after.active_quantifier_prompt_preset_id, "quantifier_next");
    assert_eq!(
        after.active_impersonate_prompt_preset_id,
        "impersonate_next"
    );
    assert_eq!(after.active_options_prompt_preset_id, "options_next");
    assert_eq!(after.narrator_mode, before.narrator_mode);
    assert_eq!(after.narrative_perspective, before.narrative_perspective);
    assert_eq!(after.narrative_tense, before.narrative_tense);
}

// CASE storage-tx-config-mode: a Mode change writes the mode, three presets and the perspective, and leaves the tense and the options preset as they were
#[test]
fn tx_config_mode_leaves_the_tense_and_the_options_preset() {
    let storage = sqlite();
    let game_id = create_game(&storage, "Mode");
    let before = storage.require_game(game_id.0).unwrap();

    storage
        .transaction(game_id, tx_token(), |tx| {
            tx.set_game_config(&ConfigChange::Mode {
                mode: NarratorMode::InteractiveFiction,
                system_id: "system_if".to_string(),
                quantifier_id: "quantifier_if".to_string(),
                impersonate_id: "impersonate_if".to_string(),
                perspective: NarrativePerspective::Second,
            })
        })
        .expect("the unit should succeed");

    let after = storage.require_game(game_id.0).unwrap();
    assert_eq!(after.narrator_mode, NarratorMode::InteractiveFiction);
    assert_eq!(after.narrative_perspective, NarrativePerspective::Second);
    assert_eq!(after.active_system_prompt_preset_id, "system_if");
    assert_eq!(after.active_quantifier_prompt_preset_id, "quantifier_if");
    assert_eq!(after.active_impersonate_prompt_preset_id, "impersonate_if");
    assert_eq!(after.narrative_tense, before.narrative_tense);
    assert_eq!(
        after.active_options_prompt_preset_id,
        before.active_options_prompt_preset_id
    );
}

// CASE storage-newest-game-message: a Game's newest Message answers before its `updated_at`
#[test]
fn newest_game_of_world_uses_the_newest_message() {
    let storage = sqlite();
    let with_message = create_game(&storage, "With message");
    create_game(&storage, "Without message");
    insert_message_at(&storage, with_message, FUTURE_MESSAGE_TIME);

    assert_eq!(
        storage.newest_game_of_world(WORLD_KEY).unwrap(),
        Some(with_message),
        "the newest Message of a Game decides"
    );
}

// CASE storage-newest-game-updated-at: `updated_at` answers when a Game has no Message, and a newer Game with no Message beats one with an old Message; the higher id alone does not win
#[test]
fn newest_game_of_world_falls_back_to_updated_at() {
    let storage = sqlite();
    let touched_later = create_game(&storage, "Touched later");
    let higher_id = create_game(&storage, "Higher id");
    storage
        .touch_game_updated_at(touched_later)
        .expect("touching the Game should succeed");

    assert!(
        higher_id.0 > touched_later.0,
        "the test data needs the newer id"
    );
    assert_eq!(
        storage.newest_game_of_world(WORLD_KEY).unwrap(),
        Some(touched_later),
        "a Game with no Message is keyed by `updated_at`, and the higher id does not win"
    );

    let crossover = sqlite();
    let with_old_message = create_game(&crossover, "With old message");
    insert_message_at(&crossover, with_old_message, PAST_MESSAGE_TIME);
    let without_message = create_game(&crossover, "Without message");

    assert_eq!(
        crossover.newest_game_of_world(WORLD_KEY).unwrap(),
        Some(without_message),
        "a Game with an old Message loses to a newer Game with no Message"
    );
}

// CASE storage-newest-game-tie: two Games share the newest key, and the higher id answers
#[test]
fn newest_game_of_world_breaks_a_tie_on_the_higher_id() {
    let storage = sqlite();
    let first = create_game(&storage, "First");
    let second = create_game(&storage, "Second");
    insert_message_at(&storage, first, FUTURE_MESSAGE_TIME);
    insert_message_at(&storage, second, FUTURE_MESSAGE_TIME);

    assert_eq!(
        storage.newest_game_of_world(WORLD_KEY).unwrap(),
        Some(second),
        "the higher Game id breaks the tie"
    );
}

// CASE storage-newest-game-unknown-world: a World with no Game answers None, and another World's Game does not answer
#[test]
fn newest_game_of_world_ignores_other_worlds() {
    let storage = sqlite();
    create_game(&storage, "Elsewhere");
    let other =
        chronicler_engine::test_support::seed_game_row(&storage, &new_game("Other", "other_world"));

    assert_eq!(
        storage.newest_game_of_world("no_such_world").unwrap(),
        None,
        "a World with no Game has no newest Game"
    );
    assert_eq!(
        storage.newest_game_of_world("other_world").unwrap(),
        Some(other),
        "the lookup filters by World"
    );
}
