//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Story memory writer prompt construction

use crate::application::prompting::builders::sections::render_memory_entry;
use crate::domain::model::agent::StoryMemoryHookInput;
use crate::domain::model::state::message_types::{MessageType, StoryRow};

const WRITER_SYSTEM: &str = "You write the memory entries of the memory of an interactive story. The oldest Messages of the story leave the narrator's context now. You record them as memory entries, so that the narrator can continue the story without contradicting them. An entry is written once and never changes. Answer with one JSON object only.";

const NO_PREVIOUS_ENTRY: &str = "(none: these are the first Messages of the story)";

const NARRATION_LABEL: &str = "Narration";

/// No preset type holds the writer's instructions, so they are code.
pub struct StoryMemoryPrompt<'a> {
    input: &'a StoryMemoryHookInput<'a>,
}

impl<'a> StoryMemoryPrompt<'a> {
    pub fn new(input: &'a StoryMemoryHookInput<'a>) -> Self {
        Self { input }
    }

    pub fn build(&self) -> (String, String) {
        (WRITER_SYSTEM.to_string(), self.build_user_prompt())
    }

    fn build_user_prompt(&self) -> String {
        let cast = self.input.cast.names().join(", ");
        let persona = self.input.cast.persona_name();
        let previous_entry = self
            .input
            .previous_entry
            .map_or_else(|| NO_PREVIOUS_ENTRY.to_string(), render_memory_entry);
        let first = self.input.oldest();
        let last = self.input.newest();
        let entries = self
            .input
            .leaving()
            .iter()
            .map(|row| self.transcript_entry(row))
            .collect::<Vec<_>>()
            .join("\n\n");
        format!(
            "The cast: {cast}. {persona} is the player character.\n\nThe last memory entry written before these Messages. Use it only for the story time and the place. Do not repeat it.\n\n<PreviousEntry>\n{previous_entry}\n</PreviousEntry>\n\nThese are the story Messages #{first} to #{last}, which leave the narrator's context now. Messages marked {persona} are written by the player character. Messages marked {NARRATION_LABEL} are written by the narrator.\n\n<Messages>\n{entries}\n</Messages>\n\nSplit the Messages into scenes. A new scene starts when the place changes, when the story time jumps, or when the people who take part change. The first Messages can continue the scene of the previous entry, and the last scene can go on after #{last}. Write each part as its own entry.\n\nFor each scene, write one entry with these fields:\n- \"from\", \"to\": the numbers of its first and its last Message. The entries cover #{first} to #{last} in order, with no gap and no overlap.\n- \"time\": the story time, as the day of the story and the time of day, for example \"Day 1, evening\". Count the days from the start of the story. If the Messages do not say, carry on from the previous entry.\n- \"place\": where the scene happens.\n- \"took_part\": the names of the people who were in the scene, or spoke in it by phone. Use only names from the cast. A character merely mentioned, remembered, discussed, or addressed while absent is NOT a participant.\n- \"others\": the people outside the cast who took part, by the name the story uses. An empty list if none.\n- \"what_happened\": 60 to 150 words, in the past tense. Record what happened and its outcomes without treating past states as current. Keep the exact names, numbers, sums, dates and times, the objects and where they were left, the promises, plans and appointments with their times, the reasons that people gave, and who learned which fact (who was told, who overheard, who was not there).\n\nAnswer with this JSON object and nothing else:\n{{\"entries\": [{{\"from\": 0, \"to\": 0, \"time\": \"\", \"place\": \"\", \"took_part\": [], \"others\": [], \"what_happened\": \"\"}}]}}"
        )
    }

    fn transcript_entry(&self, row: &StoryRow) -> String {
        let label = match row.message_type {
            MessageType::Input => self.input.cast.persona_name(),
            MessageType::Narration => NARRATION_LABEL,
            MessageType::System => "System",
        };
        format!("[#{} {label}]\n{}", row.id, row.text.trim())
    }
}
