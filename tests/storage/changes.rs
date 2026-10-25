//! The storage seam of the change counters: what a committed transaction, a rolled-back transaction, a storage call, a failed call and a read do to the numbers.

use chronicler_engine::adapters::driven::storage::{Storage, StorageCall, TestOverride, TxWrite};
use chronicler_engine::domain::model::change::ChangeVersions;
use chronicler_engine::domain::model::game::{GameId, NewGame};
use chronicler_engine::domain::model::dock::DockRecord;
use chronicler_engine::domain::model::message::SwipeAt;
use chronicler_engine::domain::model::settings::AppSettings;
use chronicler_engine::domain::model::state::generation_status::GenerationFailure;
use chronicler_engine::test_support::{
    test_new_game, dummy_message, dummy_swipe, empty_sqlite_storage, seed_game_row, tx_token,
};

fn new_game(name: &str) -> NewGame {
    test_new_game(
        "Seam World",
        "seam_world",
        "seam_persona",
        "Seam Persona",
        name,
    )
}

fn versions_of(storage: &Storage, game: GameId) -> ChangeVersions {
    storage.changes().versions(Some(game))
}

fn write_message_with_two_swipe_writes(
    storage: &Storage,
    game: GameId,
) -> Result<(), chronicler_engine::error::EngineError> {
    storage.transaction(game, tx_token(), |tx| {
        let message_id = tx.insert_message(&dummy_message("hello"))?;
        let at = SwipeAt {
            message: message_id,
            swipe: 0,
        };
        tx.insert_swipe(at, &dummy_swipe("hello"))?;
        tx.set_swipe_text(at, "hello again")
    })
}

// CASE storage-changes-tx-commit: a committed unit raises each kind of its writes once, for its Game
#[test]
fn a_committed_unit_raises_each_kind_of_its_writes_once() {
    let storage = empty_sqlite_storage();
    let game = seed_game_row(&storage, &new_game("Committed"));
    let before = versions_of(&storage, game);

    write_message_with_two_swipe_writes(&storage, game).expect("the unit commits");

    let after = versions_of(&storage, game);
    let (before_game, after_game) = (before.of_game.unwrap(), after.of_game.unwrap());
    assert_eq!(
        after_game.story,
        before_game.story + 1,
        "three writes of the story raise story once"
    );
    assert_eq!(
        after.games,
        before.games + 1,
        "a new Message raises games once"
    );
    assert_eq!(after_game.options, before_game.options);
    assert_eq!(after_game.busy_state, before_game.busy_state);
    assert_eq!(
        (
            after.worlds,
            after.settings,
            after.prompt_presets,
            after.llm_messages
        ),
        (
            before.worlds,
            before.settings,
            before.prompt_presets,
            before.llm_messages
        ),
        "no other engine-wide number moves"
    );
}

// CASE storage-changes-tx-after-commit: a unit raises its kinds when it has committed, not while it runs
#[test]
fn a_unit_raises_nothing_before_its_commit() {
    let storage = empty_sqlite_storage();
    let game = seed_game_row(&storage, &new_game("Running"));
    let before = versions_of(&storage, game);

    let inside = storage
        .transaction(game, tx_token(), |tx| {
            tx.insert_message(&dummy_message("hello"))?;
            Ok(versions_of(&storage, game))
        })
        .expect("the unit commits");

    assert_eq!(inside, before, "the numbers stand while the unit runs");
    assert_ne!(
        versions_of(&storage, game),
        before,
        "the commit raised them"
    );
}

// CASE storage-changes-tx-rollback [failure]: a unit whose later write fails raises nothing, also for its earlier writes
#[test]
fn a_unit_whose_later_write_fails_raises_nothing() {
    let (storage, failures) = empty_sqlite_storage().with_test_failures();
    let game = seed_game_row(&storage, &new_game("Rolled back"));
    failures.set_write(
        TxWrite::InsertSwipe,
        TestOverride::internal("the second write fails"),
    );
    let before = versions_of(&storage, game);

    let result = write_message_with_two_swipe_writes(&storage, game);

    assert!(result.is_err(), "the injected failure fails the unit");
    assert_eq!(
        versions_of(&storage, game),
        before,
        "the insert of the Message that rolled back raised nothing"
    );
}

// CASE storage-changes-tx-game-scope: a unit raises the per-Game numbers of its own Game only
#[test]
fn a_unit_raises_the_numbers_of_its_own_game_only() {
    let storage = empty_sqlite_storage();
    let written = seed_game_row(&storage, &new_game("Written"));
    let other = seed_game_row(&storage, &new_game("Other"));
    let other_before = versions_of(&storage, other).of_game;

    write_message_with_two_swipe_writes(&storage, written).expect("the unit commits");

    assert_eq!(
        versions_of(&storage, other).of_game,
        other_before,
        "the other Game keeps its numbers"
    );
}

// CASE storage-changes-tx-dock: a dock write raises options and no other per-Game number
#[test]
fn a_dock_write_raises_options_only() {
    let storage = empty_sqlite_storage();
    let game = seed_game_row(&storage, &new_game("Dock"));
    let before = versions_of(&storage, game).of_game.unwrap();

    storage
        .transaction(game, tx_token(), |tx| tx.put_dock(&DockRecord::default()))
        .expect("the unit commits");

    let after = versions_of(&storage, game).of_game.unwrap();
    assert_eq!(after.options, before.options + 1);
    assert_eq!(
        (after.story, after.busy_state),
        (before.story, before.busy_state)
    );
}

// CASE storage-changes-tx-failure-line: a write of the failure line raises the busy state and no other per-Game number
#[test]
fn a_write_of_the_failure_line_raises_the_busy_state_only() {
    let storage = empty_sqlite_storage();
    let game = seed_game_row(&storage, &new_game("Failure"));
    let before = versions_of(&storage, game).of_game.unwrap();

    storage
        .transaction(game, tx_token(), |tx| {
            tx.set_game_failure(Some(&GenerationFailure::save_failed(
                &chronicler_engine::error::EngineError::Config("probe".to_string()),
            )))
        })
        .expect("the unit commits");

    let after = versions_of(&storage, game).of_game.unwrap();
    assert_eq!(after.busy_state, before.busy_state + 1);
    assert_eq!((after.story, after.options), (before.story, before.options));
}

// CASE storage-changes-call: a successful write call raises the kind of that call
#[test]
fn a_successful_write_call_raises_its_kind() {
    let storage = empty_sqlite_storage();
    let before = storage.changes().versions(None);

    storage
        .save_settings(&AppSettings::default())
        .expect("the save succeeds");

    let after = storage.changes().versions(None);
    assert_eq!(after.settings, before.settings + 1);
    assert_eq!(
        (
            after.games,
            after.worlds,
            after.prompt_presets,
            after.llm_messages
        ),
        (
            before.games,
            before.worlds,
            before.prompt_presets,
            before.llm_messages
        ),
        "no other engine-wide number moves"
    );
}

// CASE storage-changes-call-failure [failure]: a failed storage call raises nothing
#[test]
fn a_failed_write_call_raises_nothing() {
    let storage = empty_sqlite_storage().with_failure(
        StorageCall::UpdateGameDisplayName,
        TestOverride::internal("the call fails"),
    );
    let game = seed_game_row(&storage, &new_game("Failing"));
    let before = versions_of(&storage, game);

    let result = storage.update_game_display_name(game.0, "Renamed");

    assert!(result.is_err(), "the injected failure fails the call");
    assert_eq!(versions_of(&storage, game), before);
}

// CASE storage-changes-read: a read raises nothing
#[test]
fn a_read_raises_nothing() {
    let storage = empty_sqlite_storage();
    let game = seed_game_row(&storage, &new_game("Read"));
    let before = versions_of(&storage, game);

    storage.list_games().expect("list_games");
    storage.get_settings().expect("get_settings");
    storage.list_worlds().expect("list_worlds");
    storage.stories().tail(game).expect("tail");

    assert_eq!(versions_of(&storage, game), before);
}

// CASE storage-changes-no-game: a snapshot with no Game holds no per-Game numbers, and a Game with no raise reads as zeros
#[test]
fn a_snapshot_names_its_game_and_a_new_game_reads_as_zeros() {
    let storage = empty_sqlite_storage();

    let none = storage.changes().versions(None);
    assert_eq!((none.game, none.of_game), (None, None));

    let unknown = storage.changes().versions(Some(GameId(4242)));
    assert_eq!(unknown.game, Some(4242));
    let numbers = unknown.of_game.expect("a named Game has numbers");
    assert_eq!(
        (numbers.story, numbers.options, numbers.busy_state),
        (0, 0, 0)
    );
}

// CASE storage-changes-server-id: one storage keeps its server id through its raises, and two storages hold two ids
#[test]
fn the_server_id_is_stable_for_one_storage_and_differs_between_two() {
    let storage = empty_sqlite_storage();
    let first = storage.changes().versions(None).server;
    storage
        .save_settings(&AppSettings::default())
        .expect("the save succeeds");

    assert_eq!(storage.changes().versions(None).server, first);
    assert_ne!(
        empty_sqlite_storage().changes().versions(None).server,
        first
    );
}
