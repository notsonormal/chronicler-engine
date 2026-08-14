//! [DOC: docs/diataxis/reference/game_flow.md]
//! Game lifecycle orchestration and read-side queries.

pub mod catalogue;
pub mod view_query;

#[cfg(test)]
mod catalogue_tests;

#[cfg(test)]
mod view_query_tests;

pub use catalogue::GameCatalogue;
pub use view_query::GameViewQuery;
