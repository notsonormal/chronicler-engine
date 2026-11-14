//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Story memory writer prompt construction

use crate::domain::model::agent::StoryMemoryHookInput;
use crate::domain::model::state::message_types::{MessageType, StoryRow};

const WRITER_SYSTEM: &str = "You keep the memory of an interactive story. The oldest entries of the story are about to leave the narrator's context. Write the notes that the narrator needs to continue the story without contradicting them.";

const NO_PREVIOUS_MEMORY: &str = "(none)";

const NARRATION_LABEL: &str = "Narration";

/// The writer's instructions. No preset type holds them, so they are code.
pub struct StoryMemoryPrompt<'a> {
    input: &'a StoryMemoryHookInput<'a>,
    words: u32,
}

impl<'a> StoryMemoryPrompt<'a> {
    /// `response_tokens` is the response length of the writer call.
    pub fn new(input: &'a StoryMemoryHookInput<'a>, response_tokens: u32) -> Self {
        Self {
            input,
            words: Self::words_for(response_tokens),
        }
    }

    /// The model writes about twice the words the prompt asks for (measured on five
    /// cuts of a stored story), so the prompt asks for well under the response length.
    fn words_for(response_tokens: u32) -> u32 {
        response_tokens * 3 / 10
    }

    pub fn build(&self) -> (String, String) {
        (WRITER_SYSTEM.to_string(), self.build_user_prompt())
    }

    fn build_user_prompt(&self) -> String {
        let first = self.input.oldest();
        let last = self.input.newest();
        let previous = self
            .input
            .previous
            .map_or(NO_PREVIOUS_MEMORY, |memory| memory.text.as_str());
        let persona = self.input.persona_name;
        let words = self.words;
        let entries = self
            .input
            .leaving()
            .iter()
            .map(|row| self.transcript_entry(row))
            .collect::<Vec<_>>()
            .join("\n\n");
        format!(
            "This is the memory of the story so far.\n\n\
             <PreviousMemory>\n{previous}\n</PreviousMemory>\n\n\
             These are the story entries #{first} to #{last}, which leave the narrator's context now. \
             Entries marked {persona} are written by the player character; entries marked \
             {NARRATION_LABEL} are written by the narrator.\n\n\
             <Entries>\n{entries}\n</Entries>\n\n\
             Rewrite the memory so that it holds the previous memory and these entries, in at most \
             {words} words, as short plain lines. Keep:\n\
             - names, numbers, objects and where they are, places, promises, plans and appointments \
             with their times;\n\
             - the story time: which day of the story it is, the time of day, and how much time passed;\n\
             - who was present at each conversation, and who learned which fact.\n\
             If the memory would pass {words} words, shorten the oldest parts first: keep \
             the open promises, plans and appointments, the names, and who knows what, and drop the \
             small steps that are finished.\n\
             Write only the memory."
        )
    }

    fn transcript_entry(&self, row: &StoryRow) -> String {
        let label = match row.message_type {
            MessageType::Input => self.input.persona_name,
            MessageType::Narration => NARRATION_LABEL,
            MessageType::System => "System",
        };
        format!("[#{} {label}]\n{}", row.id, row.text.trim())
    }
}
