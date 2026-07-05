//! Integration tests for storage repositories exercised against a real SQLite-backed `Storage`: messages, snapshots, worlds, LLM message log, prompt presets, and the prompt-presets HTTP fragment.

mod llm_message_storage;
mod message_storage;
mod preset_storage;
mod prompt_presets;
mod snapshot_storage;
mod world_storage;
