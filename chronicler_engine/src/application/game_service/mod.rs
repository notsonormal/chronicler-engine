//! [DOC: docs/architecture/system.md]

mod service;

pub use crate::application::context::GameServiceContext;
pub use crate::application::context::{
    delete_and_remove_message, map_llm_error, save_message_and_snapshot,
};
pub use service::{DefaultGameService, GameService};
