//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! The pages of saved Games: the page size, and the rule that names the page a request gets.

use super::game::Game;

pub const SAVED_GAMES_PAGE_SIZE: u64 = 50;

/// One page of the saved Games, with the number of pages that all of them fill.
pub struct SavedGamesPage {
    pub games: Vec<Game>,
    pub page: u64,
    pub page_count: u64,
}

impl SavedGamesPage {
    /// A list that holds no saved Game is one empty page.
    pub fn page_count_of(total: u64) -> u64 {
        total.div_ceil(SAVED_GAMES_PAGE_SIZE).max(1)
    }

    /// A page past the end is the last page.
    pub fn clamp_page(requested: u64, total: u64) -> u64 {
        requested.clamp(1, Self::page_count_of(total))
    }
}
