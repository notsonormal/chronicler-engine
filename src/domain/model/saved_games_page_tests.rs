//! The page rules of the saved Games.

use super::saved_games_page::{SavedGamesPage, SAVED_GAMES_PAGE_SIZE};

#[test]
fn a_list_with_no_saved_game_is_one_page() {
    assert_eq!(SavedGamesPage::page_count_of(0), 1);
}

#[test]
fn a_full_page_is_one_page_and_one_more_game_adds_a_page() {
    assert_eq!(SavedGamesPage::page_count_of(SAVED_GAMES_PAGE_SIZE), 1);
    assert_eq!(SavedGamesPage::page_count_of(SAVED_GAMES_PAGE_SIZE + 1), 2);
}

#[test]
fn a_page_past_the_end_is_the_last_page() {
    let total = SAVED_GAMES_PAGE_SIZE * 2 + 1;

    assert_eq!(SavedGamesPage::clamp_page(99, total), 3);
}

#[test]
fn page_zero_is_the_first_page() {
    assert_eq!(SavedGamesPage::clamp_page(0, SAVED_GAMES_PAGE_SIZE * 3), 1);
}

#[test]
fn a_page_inside_the_range_stays() {
    assert_eq!(SavedGamesPage::clamp_page(2, SAVED_GAMES_PAGE_SIZE * 3), 2);
}
