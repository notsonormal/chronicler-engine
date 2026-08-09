//! Driven-adapter storage seam tests: repositories exercised against a real SQLite-backed `Storage`.

#[path = "../test_utils/mod.rs"]
mod test_utils;

#[path = "../helpers/fixtures.rs"]
mod fixtures;

#[path = "../helpers/storage_ext.rs"]
mod storage_ext;

mod llm_message_storage;
mod message_storage;
mod preset_storage;
mod snapshot_storage;
mod world_storage;
