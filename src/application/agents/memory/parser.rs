//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! The read of a writer answer: the one JSON object, the check of the scene starts, and the ranges the engine sets

use serde::Deserialize;

use crate::application::ports::llm_provider::LlmCallResult;
use crate::application::prompting::sanitize::json_object_in;
use crate::domain::model::agent::StoryMemoryHookInput;
use crate::domain::model::memory_entry::MemoryEntry;
use crate::error::{EngineError, LlmFailure};

const MEMORY_ENTRIES_FORMAT: &str = "memory entries";
pub(crate) const SCENE_STARTS_FORMAT: &str =
    "scene starts that are Messages of the call, in story order";

#[derive(Deserialize)]
struct Answer {
    entries: Vec<AnswerEntry>,
}

/// The prompt asks for `to` as well, but the engine sets each range from the scene starts.
#[derive(Deserialize)]
struct AnswerEntry {
    from: u64,
    time: String,
    place: String,
    took_part: Vec<String>,
    others: Vec<String>,
    what_happened: String,
}

pub struct MemoryEntriesAnswer<'a> {
    input: &'a StoryMemoryHookInput<'a>,
}

impl<'a> MemoryEntriesAnswer<'a> {
    pub fn new(input: &'a StoryMemoryHookInput<'a>) -> Self {
        Self { input }
    }

    /// A cut-off answer loses its newest scenes, so it is refused whole.
    pub fn read(&self, answer: &LlmCallResult) -> Result<Vec<MemoryEntry>, EngineError> {
        let text = answer.text.trim();
        if text.is_empty() {
            return Err(LlmFailure::EmptyResponse {
                raw_response: answer.raw_response_json.clone(),
                finish_reason: None,
            }
            .into());
        }
        if answer.stopped_at_limit {
            return Err(LlmFailure::AnswerTooLong {
                raw_response: answer.raw_response_json.clone(),
                stopped_at_limit: true,
            }
            .into());
        }
        let scenes = json_object_in(text)
            .and_then(|body| serde_json::from_str::<serde_json::Value>(body).ok())
            // A struct refuses a key written twice; a `Value` keeps the last value of it.
            .and_then(|value| serde_json::from_value::<Answer>(value).ok())
            .map(|parsed| parsed.entries)
            .filter(|entries| !entries.is_empty())
            .ok_or_else(|| Self::refused(answer, MEMORY_ENTRIES_FORMAT))?;
        if !self.starts_are_messages_in_order(&scenes) {
            return Err(Self::refused(answer, SCENE_STARTS_FORMAT));
        }
        Ok(self.with_ranges(scenes))
    }

    /// Message ids have gaps after a delete, so a later start must be an id of the
    /// call, not only a number inside its range.
    fn starts_are_messages_in_order(&self, scenes: &[AnswerEntry]) -> bool {
        let is_message = |id: u64| self.input.leaving().iter().any(|row| row.id == id);
        let later_starts: Vec<u64> = scenes.iter().skip(1).map(|scene| scene.from).collect();
        later_starts
            .iter()
            .all(|&id| is_message(id) && id > self.input.oldest())
            && later_starts.windows(2).all(|pair| pair[0] < pair[1])
    }

    fn with_ranges(&self, scenes: Vec<AnswerEntry>) -> Vec<MemoryEntry> {
        let leaving = self.input.leaving();
        let starts: Vec<u64> = scenes.iter().map(|scene| scene.from).collect();
        scenes
            .into_iter()
            .enumerate()
            .map(|(index, scene)| {
                let from = if index == 0 {
                    self.input.oldest()
                } else {
                    scene.from
                };
                let to = starts.get(index + 1).map_or(self.input.newest(), |next| {
                    leaving
                        .iter()
                        .take_while(|row| row.id < *next)
                        .last()
                        .map_or(from, |row| row.id)
                });
                MemoryEntry {
                    from,
                    to,
                    time: scene.time,
                    place: scene.place,
                    took_part: scene.took_part,
                    others: scene.others,
                    what_happened: scene.what_happened,
                }
            })
            .collect()
    }

    fn refused(answer: &LlmCallResult, expected_format: &'static str) -> EngineError {
        LlmFailure::ParseError {
            raw_response: answer.raw_response_json.clone(),
            expected_format,
        }
        .into()
    }
}
