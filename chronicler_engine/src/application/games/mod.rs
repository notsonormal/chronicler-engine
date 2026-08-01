//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! Game lifecycle orchestration and read-side queries.

pub mod catalogue;
pub mod view_query;
pub mod world_persona_catalogue;

pub use catalogue::GameCatalogue;
pub use view_query::GameViewQuery;
pub use world_persona_catalogue::WorldPersonaCatalogue;
