//! [DOC: docs/architecture/system.md]

mod service;

pub use crate::application::context::GameServiceContext;
pub use crate::application::context::{
    delete_and_remove_message, map_llm_error, persist_new_messages,
};
pub use service::{DefaultGameService, GameService};
