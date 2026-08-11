//! [DOC: docs/diataxis/reference/storage.md]
//! Row-to-domain object mapping

pub mod llm_message;
pub mod message;
pub mod state_snapshot;

#[cfg(test)]
mod llm_message_tests;
#[cfg(test)]
mod message_tests;
#[cfg(test)]
mod state_snapshot_tests;
