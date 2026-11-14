//! Seam contract of the story memory writer, the before-narration hook's Agent.

use std::sync::Arc;

use crate::adapters::driven::storage::Storage;
use crate::application::agents::memory::agent::StoryMemoryWriter;
use crate::application::agents::trait_def::BeforeNarrationAgent;
use crate::application::prompting::token_budget::{STORY_MEMORY_CHARS, STORY_MEMORY_TOKENS};
use crate::domain::model::agent::{StoryMemoryHookInput, Role};
use crate::domain::model::game::GameId;
use crate::domain::model::generation_parameters::CallSettings;
use crate::domain::model::state::message_types::{MessageType, StoryRow};
use crate::domain::model::story_memory::StoryMemory;
use crate::error::{EngineError, LlmFailure};
use crate::test_support::fixtures::default_story_row;
use crate::test_support::{
    make_test_recorder, make_test_recorder_with_storage, sent_user_text, ScriptedModel,
};

const PERSONA: &str = "Julian";
const PREVIOUS: &str = "Julian rented a room at the inn for three nights.";
const ANSWER: &str = "Julian rented a room. Mara promised to meet him at dawn.";

fn writer(model: Arc<ScriptedModel>) -> StoryMemoryWriter {
    StoryMemoryWriter::new(make_test_recorder(model))
}

fn leaving() -> Vec<StoryRow> {
    vec![
        StoryRow {
            id: 12,
            text: "I ask Mara when we leave.".to_string(),
            ..default_story_row(MessageType::Input)
        },
        StoryRow {
            id: 13,
            text: "Mara says she will wait at the gate at dawn.".to_string(),
            ..default_story_row(MessageType::Narration)
        },
    ]
}

fn previous() -> StoryMemory {
    StoryMemory {
        text: PREVIOUS.to_string(),
        covers_up_to: 11,
    }
}

fn input<'a>(
    previous: Option<&'a StoryMemory>,
    leaving: &'a [StoryRow],
) -> StoryMemoryHookInput<'a> {
    StoryMemoryHookInput::new(previous, leaving, PERSONA).expect("Messages leave")
}

fn settings() -> CallSettings {
    CallSettings {
        temperature: Some(0.3),
        max_tokens: 700,
        game: Some(GameId(7)),
        ..CallSettings::default()
    }
}

fn run_once(model: &Arc<ScriptedModel>, previous: Option<&StoryMemory>) -> StoryMemory {
    let rows = leaving();
    writer(Arc::clone(model))
        .run(&input(previous, &rows), &settings())
        .expect("a good answer is a story memory")
}

fn prompt_for_response_length(max_tokens: u32) -> String {
    let model = Arc::new(ScriptedModel::new().story_memory(ANSWER));
    let rows = leaving();
    writer(Arc::clone(&model))
        .run(
            &input(Some(&previous()), &rows),
            &CallSettings {
                max_tokens,
                ..settings()
            },
        )
        .expect("a good answer is a story memory");
    model
        .calls()
        .into_iter()
        .next()
        .expect("one call")
        .user_prompt()
}

// CASE memory-result: the answer, trimmed, is the new story memory, and it covers the Messages up to the newest one that left.
#[test]
fn test_the_answer_is_the_story_memory_up_to_the_newest_leaving_message() {
    let model = Arc::new(ScriptedModel::new().story_memory(format!("\n  {ANSWER}  \n")));

    let memory = run_once(&model, Some(&previous()));

    assert_eq!(
        memory,
        StoryMemory {
            text: ANSWER.to_string(),
            covers_up_to: 13,
        }
    );
}

// CASE memory-prompt-rewrite: the prompt holds the previous story memory and each leaving Message with its id and its label, the Persona's name for an Input and Narration for a Narration.
#[test]
fn test_the_prompt_holds_the_previous_memory_and_the_leaving_messages() {
    let model = Arc::new(ScriptedModel::new().story_memory(ANSWER));

    run_once(&model, Some(&previous()));

    let call = model.calls().into_iter().next().expect("one call");
    let user = call.user_prompt();
    assert!(
        user.contains(&format!("<PreviousMemory>\n{PREVIOUS}\n</PreviousMemory>")),
        "the previous story memory: {user}"
    );
    assert!(
        user.contains("[#12 Julian]\nI ask Mara when we leave.")
            && user.contains("[#13 Narration]\nMara says she will wait at the gate at dawn."),
        "each leaving Message with its id and label: {user}"
    );
    assert!(
        user.contains("#12 to #13"),
        "the range of the entries: {user}"
    );
    assert!(
        call.system_prompt()
            .contains("You keep the memory of an interactive story."),
        "the writer's instructions: {}",
        call.system_prompt()
    );
}

// CASE memory-prompt-size: the prompt asks for at most three tenths of the call's response length in words, and says what to shorten when the memory would pass it.
#[test]
fn test_the_prompt_states_the_size_and_how_to_shorten() {
    let user = prompt_for_response_length(STORY_MEMORY_TOKENS);

    assert!(
        user.contains("in at most 600 words"),
        "the word limit comes from the response length: {user}"
    );
    assert!(
        user.contains("If the memory would pass 600 words, shorten the oldest parts first"),
        "the rule to shorten: {user}"
    );
}

// CASE memory-prompt-size-follows-call: a smaller response length gives a smaller word count, in the limit and in the rule to shorten.
#[test]
fn test_a_smaller_response_length_asks_for_fewer_words() {
    let user = prompt_for_response_length(700);

    assert!(
        user.contains("in at most 210 words")
            && user.contains("If the memory would pass 210 words"),
        "the word count follows the response length: {user}"
    );
}

// CASE memory-prompt-first-cut: with no previous story memory, the prompt says there is none.
#[test]
fn test_the_first_cut_has_no_previous_memory() {
    let model = Arc::new(ScriptedModel::new().story_memory(ANSWER));

    run_once(&model, None);

    let user = model
        .calls()
        .into_iter()
        .next()
        .expect("one call")
        .user_prompt();
    assert!(
        user.contains("<PreviousMemory>\n(none)\n</PreviousMemory>"),
        "no previous story memory: {user}"
    );
}

// CASE memory-call-settings: the call keeps the call settings it gets.
#[test]
fn test_the_call_keeps_the_call_settings_it_gets() {
    let model = Arc::new(ScriptedModel::new().story_memory(ANSWER));

    run_once(&model, None);

    let call = model.calls().into_iter().next().expect("one call");
    assert_eq!(call.role, Role::StoryMemory);
    assert_eq!(call.settings.max_tokens, 700);
    assert_eq!(call.settings.temperature, Some(0.3));
    assert_eq!(call.settings.game, Some(GameId(7)));
}

// CASE memory-at-size: an answer of exactly the story memory's size is the story memory, whole.
#[test]
fn test_an_answer_of_the_size_is_kept_whole() {
    let answer = "w".repeat(STORY_MEMORY_CHARS);
    let model = Arc::new(ScriptedModel::new().story_memory(answer.clone()));

    let memory = run_once(&model, None);

    assert_eq!(memory.text, answer);
}

fn run_recorded(model: ScriptedModel) -> (Result<StoryMemory, EngineError>, Option<String>) {
    let storage = Arc::new(Storage::new_in_memory());
    let rows = leaving();
    let result = StoryMemoryWriter::new(make_test_recorder_with_storage(
        Arc::new(model),
        Arc::clone(&storage),
    ))
    .run(&input(Some(&previous()), &rows), &settings());
    let recorded = storage
        .list_latest_llm_messages(10)
        .expect("the forensics read back");
    let call = recorded.last().expect("the call is recorded");
    assert_eq!(call.agent_name, "memory");
    (result, call.error_message.clone())
}

// CASE memory-stopped-at-limit [failure]: an answer that the provider stopped at the answer limit fails, and its record names the failure, so the role health shows it.
#[test]
fn test_an_answer_stopped_at_the_limit_fails() {
    let (result, failure) = run_recorded(
        ScriptedModel::new().cut_off(Role::StoryMemory, "Julian rented a room. Mara prom"),
    );

    assert!(
        matches!(
            result,
            Err(EngineError::Llm(LlmFailure::AnswerTooLong {
                stopped_at_limit: true,
                ..
            }))
        ),
        "a cut-off text is not a story memory: {result:?}"
    );
    assert_eq!(
        failure.as_deref(),
        Some("LLM Error: the answer is longer than its size limit (finish_reason: length)")
    );
}

// CASE memory-past-size [failure]: an answer longer than the story memory's size fails, and the writer never cuts it.
#[test]
fn test_an_answer_past_the_size_fails() {
    let (result, failure) =
        run_recorded(ScriptedModel::new().story_memory("w".repeat(STORY_MEMORY_CHARS + 1)));

    assert!(
        matches!(
            result,
            Err(EngineError::Llm(LlmFailure::AnswerTooLong {
                stopped_at_limit: false,
                ..
            }))
        ),
        "a text past the size is not a story memory: {result:?}"
    );
    assert_eq!(
        failure.as_deref(),
        Some("LLM Error: the answer is longer than its size limit")
    );
}

// CASE memory-empty-answer [failure]: an empty answer fails, so a blank text never replaces a story memory.
#[test]
fn test_an_empty_answer_fails() {
    let model = Arc::new(ScriptedModel::new().story_memory("   \n"));
    let rows = leaving();

    let result = writer(model).run(&input(Some(&previous()), &rows), &settings());

    assert!(result.is_err(), "a blank answer is not a story memory");
}

// CASE memory-transport-failure [failure]: a failed call fails the writer.
#[test]
fn test_a_failed_call_fails_the_writer() {
    let model = Arc::new(ScriptedModel::new().failed(Role::StoryMemory));
    let rows = leaving();

    let result = writer(model).run(&input(None, &rows), &settings());

    assert!(result.is_err());
}

// CASE memory-call-record: a good call is recorded with its agent name and prompts.
#[test]
fn test_a_good_call_is_recorded_with_its_prompts() {
    let storage = Arc::new(Storage::new_in_memory());
    let model = Arc::new(ScriptedModel::new().story_memory(ANSWER));
    let rows = leaving();
    StoryMemoryWriter::new(make_test_recorder_with_storage(model, Arc::clone(&storage)))
        .run(&input(None, &rows), &settings())
        .expect("a good answer is a story memory");

    let recorded = storage
        .list_latest_llm_messages(10)
        .expect("the forensics read back");
    let call = recorded.last().expect("the call is recorded");
    assert_eq!(call.agent_name, "memory");
    assert!(call.error_message.is_none());
    assert_eq!(call.parsed_response, ANSWER);
    assert!(sent_user_text(call).contains("[#13 Narration]"));
}

// CASE memory-call-record-failure: a failed call is recorded with its failure text.
#[test]
fn test_a_failed_call_is_recorded_with_its_failure_text() {
    let storage = Arc::new(Storage::new_in_memory());
    let model = Arc::new(ScriptedModel::new().failed(Role::StoryMemory));
    let rows = leaving();
    let _ = StoryMemoryWriter::new(make_test_recorder_with_storage(model, Arc::clone(&storage)))
        .run(&input(None, &rows), &settings());

    let recorded = storage
        .list_latest_llm_messages(10)
        .expect("the forensics read back");
    let failure = recorded.last().expect("the failed attempt is recorded");
    assert_eq!(failure.agent_name, "memory");
    assert_eq!(
        failure.error_message.as_deref(),
        Some("LLM Error: Narration generation failed at stage 'scripted': configured_failure")
    );
}
