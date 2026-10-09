//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Prompt presets askama templates.

pub mod prompt_presets;

pub use self::prompt_presets::PromptPresetsTemplate;

#[cfg(test)]
mod prompt_presets_tests;
