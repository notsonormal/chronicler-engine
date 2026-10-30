//! The storage seam of the saved-Games pages: the Games that a page lists, their order, and the count that the page rules read.

use chronicler_engine::adapters::driven::storage::{Storage, StorageCall, TestOverride};
use chronicler_engine::domain::model::game::{GameId, NewGame};
use chronicler_engine::test_support::{
    empty_sqlite_storage, seed_game_row, set_game_updated_at_row, test_new_game,
};

fn new_game(name: &str) -> NewGame {
    test_new_game(
        "Page World",
        "page_world",
        "page_persona",
        "Page Persona",
        name,
    )
}

fn seeded(storage: &Storage, names: &[&str]) -> Vec<GameId> {
    names
        .iter()
        .map(|name| seed_game_row(storage, &new_game(name)))
        .collect()
}

fn names_of(games: &[chronicler_engine::domain::model::game::Game]) -> Vec<&str> {
    games.iter().map(|game| game.name.as_str()).collect()
}

// CASE saved-games-page-excludes-active: the page and the count leave out the active Game
#[test]
fn the_page_and_the_count_leave_out_the_active_game() {
    let storage = empty_sqlite_storage();
    let ids = seeded(&storage, &["One", "Two", "Three"]);

    let page = storage
        .list_saved_games_page(Some(ids[1]), 50, 0)
        .expect("list the saved Games");
    let count = storage.count_saved_games(Some(ids[1])).expect("count");

    assert_eq!(names_of(&page), ["Three", "One"]);
    assert_eq!(count, 2);
    assert_eq!(storage.count_saved_games(None).expect("count"), 3);
}

// CASE saved-games-page-order: the newest updated_at lists first, and equal updated_at lists the greater id first
#[test]
fn the_page_lists_the_newest_first_and_breaks_a_tie_by_the_greater_id() {
    let storage = empty_sqlite_storage();
    let ids = seeded(&storage, &["Old", "TieLow", "TieHigh", "New"]);
    set_game_updated_at_row(&storage, ids[0], "2026-01-01T00:00:00+00:00");
    set_game_updated_at_row(&storage, ids[1], "2026-02-01T00:00:00+00:00");
    set_game_updated_at_row(&storage, ids[2], "2026-02-01T00:00:00+00:00");
    set_game_updated_at_row(&storage, ids[3], "2026-03-01T00:00:00+00:00");

    let page = storage
        .list_saved_games_page(None, 50, 0)
        .expect("list the saved Games");

    assert_eq!(names_of(&page), ["New", "TieHigh", "TieLow", "Old"]);
}

// CASE saved-games-page-window: the limit and the offset cut the ordered list
#[test]
fn the_limit_and_the_offset_cut_the_ordered_list() {
    let storage = empty_sqlite_storage();
    let ids = seeded(&storage, &["A", "B", "C", "D", "E"]);
    for (index, id) in ids.iter().enumerate() {
        set_game_updated_at_row(
            &storage,
            *id,
            &format!("2026-01-0{}T00:00:00+00:00", index + 1),
        );
    }

    let second_page = storage
        .list_saved_games_page(None, 2, 2)
        .expect("list the saved Games");
    let past_the_end = storage
        .list_saved_games_page(None, 2, 10)
        .expect("list the saved Games");

    assert_eq!(names_of(&second_page), ["C", "B"]);
    assert!(past_the_end.is_empty());
}

// CASE saved-games-page-failure [failure]: a failed page read and a failed count return the error
#[test]
fn a_failed_page_read_and_a_failed_count_return_the_error() {
    let storage = empty_sqlite_storage()
        .with_failure(
            StorageCall::ListSavedGames,
            TestOverride::internal("page read fails"),
        )
        .with_failure(
            StorageCall::CountSavedGames,
            TestOverride::internal("count fails"),
        );

    let page = storage.list_saved_games_page(None, 50, 0).unwrap_err();
    let count = storage.count_saved_games(None).unwrap_err();

    assert!(page.to_string().contains("page read fails"));
    assert!(count.to_string().contains("count fails"));
}
