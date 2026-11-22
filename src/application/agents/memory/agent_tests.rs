//! Seam contract of the story memory writer, the before-narration hook's Agent.

use std::sync::Arc;

use serde_json::json;

use crate::adapters::driven::storage::Storage;
use crate::application::agents::memory::agent::StoryMemoryWriter;
use crate::application::agents::memory::parser::SCENE_STARTS_FORMAT;
use crate::application::agents::trait_def::BeforeNarrationAgent;
use crate::domain::model::agent::{Cast, Role, StoryMemoryHookInput};
use crate::domain::model::game::GameId;
use crate::domain::model::generation_parameters::CallSettings;
use crate::domain::model::llm_message::LlmMessage;
use crate::domain::model::memory_entry::MemoryEntry;
use crate::domain::model::state::message_types::{MessageType, StoryRow};
use crate::error::{EngineError, LlmFailure};
use crate::test_support::fixtures::{default_story_row, TestCharacter};
use crate::test_support::{
    make_test_recorder, make_test_recorder_with_storage, sent_user_text, ScriptedModel,
};

const PERSONA: &str = "Julian";
const NOT_JSON: &str = "Julian asked Mara when they would leave.";
const MEMORY_ENTRIES_FORMAT: &str = "memory entries";

fn cast() -> Cast {
    Cast::new(PERSONA, &[TestCharacter::named("mara", "Mara")])
}

fn writer(model: Arc<ScriptedModel>) -> StoryMemoryWriter {
    StoryMemoryWriter::new(make_test_recorder(model))
}

/// Message ids have a gap, as after a delete: 14 is no Message of the call.
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
        StoryRow {
            id: 15,
            text: "At dawn, the gate is empty.".to_string(),
            ..default_story_row(MessageType::Narration)
        },
    ]
}

fn previous_entry() -> MemoryEntry {
    MemoryEntry {
        from: 8,
        to: 11,
        time: "Day 1, evening".to_string(),
        place: "The inn".to_string(),
        took_part: vec![PERSONA.to_string()],
        others: vec![],
        what_happened: "Julian rented a room for three nights.".to_string(),
    }
}

fn settings() -> CallSettings {
    CallSettings {
        temperature: Some(0.3),
        max_tokens: 2_000,
        game: Some(GameId(7)),
        ..CallSettings::default()
    }
}

fn answer_entry(from: u64, to: u64) -> serde_json::Value {
    json!({
        "from": from,
        "to": to,
        "time": format!("Day 2, scene at #{from}"),
        "place": format!("Place of #{from}"),
        "took_part": [PERSONA, "Mara"],
        "others": ["the guard"],
        "what_happened": format!("What happened from #{from}."),
    })
}

/// One JSON answer, with the `from` and `to` the model gives for each scene.
fn answer(scenes: &[(u64, u64)]) -> String {
    let entries: Vec<_> = scenes
        .iter()
        .map(|&(from, to)| answer_entry(from, to))
        .collect();
    json!({ "entries": entries }).to_string()
}

/// The entry the engine keeps for the scene the model started at `answered_from`.
fn kept_entry(answered_from: u64, from: u64, to: u64) -> MemoryEntry {
    MemoryEntry {
        from,
        to,
        time: format!("Day 2, scene at #{answered_from}"),
        place: format!("Place of #{answered_from}"),
        took_part: vec![PERSONA.to_string(), "Mara".to_string()],
        others: vec!["the guard".to_string()],
        what_happened: format!("What happened from #{answered_from}."),
    }
}

fn run_with(
    model: &Arc<ScriptedModel>,
    previous: Option<&MemoryEntry>,
) -> Result<Vec<MemoryEntry>, EngineError> {
    let rows = leaving();
    let cast = cast();
    let input = StoryMemoryHookInput::new(previous, &cast, &rows).expect("Messages leave");
    writer(Arc::clone(model)).run(&input, &settings())
}

fn entries_of(model: ScriptedModel) -> Vec<MemoryEntry> {
    run_with(&Arc::new(model), Some(&previous_entry())).expect("a good answer gives entries")
}

/// A model that gives the same answer on both tries.
fn twice(answer: impl Into<String>) -> ScriptedModel {
    let answer = answer.into();
    ScriptedModel::new()
        .story_memory(answer.clone())
        .story_memory(answer)
}

fn failure_of(model: ScriptedModel) -> (EngineError, usize) {
    let model = Arc::new(model);
    let error = run_with(&model, Some(&previous_entry())).expect_err("the writer fails");
    (error, model.calls().len())
}

/// The records of the calls that one run of the writer makes, oldest first. The run may fail.
fn recorded_run(model: ScriptedModel) -> Vec<LlmMessage> {
    let storage = Arc::new(Storage::new_in_memory());
    let rows = leaving();
    let cast = cast();
    let input = StoryMemoryHookInput::new(None, &cast, &rows).expect("Messages leave");
    let recorder = make_test_recorder_with_storage(Arc::new(model), Arc::clone(&storage));
    let _ = StoryMemoryWriter::new(recorder).run(&input, &settings());
    let mut recorded = storage
        .list_latest_llm_messages(10)
        .expect("the forensics read back");
    recorded.sort_by_key(|call| call.id);
    recorded
}

fn first_user_prompt(previous: Option<&MemoryEntry>) -> String {
    let model = Arc::new(ScriptedModel::new().story_memory(answer(&[(12, 15)])));
    run_with(&model, previous).expect("a good answer gives entries");
    model
        .calls()
        .into_iter()
        .next()
        .expect("one call")
        .user_prompt()
}

fn is_parse_error(error: &EngineError, format: &str) -> bool {
    matches!(
        error,
        EngineError::Llm(LlmFailure::ParseError { expected_format, .. }) if *expected_format == format
    )
}

// CASE memory-result: a good answer gives its entries in order, and their ranges cover the first to the last Message of the call with no gap.
#[test]
fn test_a_good_answer_gives_its_entries_over_the_call() {
    let entries = entries_of(ScriptedModel::new().story_memory(answer(&[(12, 13), (15, 15)])));

    assert_eq!(
        entries,
        vec![kept_entry(12, 12, 13), kept_entry(15, 15, 15)]
    );
}

// CASE memory-ranges-from-starts: the model's `to` numbers are ignored; each `to` is the Message before the next scene's start, and the last is the last Message of the call.
#[test]
fn test_the_ranges_come_from_the_scene_starts() {
    let entries = entries_of(ScriptedModel::new().story_memory(answer(&[(12, 15), (15, 13)])));

    assert_eq!(
        entries,
        vec![kept_entry(12, 12, 13), kept_entry(15, 15, 15)]
    );
}

// CASE memory-first-from-set: the `from` of the first scene is not checked, even when it is no Message of the call; the first scene starts at the first Message.
#[test]
fn test_the_first_scene_starts_at_the_first_message() {
    let entries = entries_of(ScriptedModel::new().story_memory(answer(&[(8, 15)])));

    assert_eq!(entries, vec![kept_entry(8, 12, 15)]);
}

// CASE memory-fenced-answer: an answer inside a Markdown code fence is read.
#[test]
fn test_a_fenced_answer_is_read() {
    let fenced = format!("```json\n{}\n```", answer(&[(12, 15)]));

    let entries = entries_of(ScriptedModel::new().story_memory(fenced));

    assert_eq!(entries, vec![kept_entry(12, 12, 15)]);
}

// CASE memory-text-around-answer: text before and after the one JSON object is ignored.
#[test]
fn test_text_around_the_answer_is_ignored() {
    let wrapped = format!(
        "Here are the entries:\n{}\nI hope they help.",
        answer(&[(12, 15)])
    );

    let entries = entries_of(ScriptedModel::new().story_memory(wrapped));

    assert_eq!(entries, vec![kept_entry(12, 12, 15)]);
}

// CASE memory-repeated-key: a key written twice in one entry is read, and the last value counts.
#[test]
fn test_a_key_written_twice_keeps_the_last_value() {
    let repeated = answer(&[(12, 15)]).replacen(
        r#""others":["the guard"]"#,
        r#""others":["a stranger"],"others":["the guard"]"#,
        1,
    );
    assert!(repeated.contains("a stranger"), "the answer repeats a key");

    let entries = entries_of(ScriptedModel::new().story_memory(repeated));

    assert_eq!(entries, vec![kept_entry(12, 12, 15)]);
}

// CASE memory-prompt: the prompt holds the cast, the player character, the previous entry in the entry form, the range of the call, and each leaving Message with its id and its label.
#[test]
fn test_the_prompt_holds_the_cast_the_previous_entry_and_the_messages() {
    let model = Arc::new(ScriptedModel::new().story_memory(answer(&[(12, 15)])));

    run_with(&model, Some(&previous_entry())).expect("a good answer gives entries");

    let call = model.calls().into_iter().next().expect("one call");
    let user = call.user_prompt();
    assert!(
        user.starts_with("The cast: Julian, Mara. Julian is the player character."),
        "the cast and the player character: {user}"
    );
    assert!(
        user.contains(
            "<PreviousEntry>\n[#8-#11] Day 1, evening. The inn. Took part: Julian.\n\
             Julian rented a room for three nights.\n</PreviousEntry>"
        ),
        "the previous entry: {user}"
    );
    assert!(
        user.contains("These are the story Messages #12 to #15,"),
        "the range of the call: {user}"
    );
    assert!(
        user.contains(
            "<Messages>\n[#12 Julian]\nI ask Mara when we leave.\n\n\
             [#13 Narration]\nMara says she will wait at the gate at dawn.\n\n\
             [#15 Narration]\nAt dawn, the gate is empty.\n</Messages>"
        ),
        "each leaving Message with its id and label: {user}"
    );
    assert!(
        user.ends_with(
            "{\"entries\": [{\"from\": 0, \"to\": 0, \"time\": \"\", \"place\": \"\", \
             \"took_part\": [], \"others\": [], \"what_happened\": \"\"}]}"
        ),
        "the answer shape: {user}"
    );
    assert!(
        call.system_prompt()
            .starts_with("You write the memory entries of the memory of an interactive story."),
        "the writer's instructions: {}",
        call.system_prompt()
    );
}

// CASE memory-prompt-first-cut: with no previous entry, the prompt says these are the first Messages of the story.
#[test]
fn test_the_first_cut_has_no_previous_entry() {
    let user = first_user_prompt(None);

    assert!(
        user.contains(
            "<PreviousEntry>\n(none: these are the first Messages of the story)\n</PreviousEntry>"
        ),
        "no previous entry: {user}"
    );
}

// CASE memory-call-settings: the call keeps the call settings it gets.
#[test]
fn test_the_call_keeps_the_call_settings_it_gets() {
    let model = Arc::new(ScriptedModel::new().story_memory(answer(&[(12, 15)])));

    run_with(&model, None).expect("a good answer gives entries");

    let call = model.calls().into_iter().next().expect("one call");
    assert_eq!(call.role, Role::StoryMemory);
    assert_eq!(call.settings.max_tokens, 2_000);
    assert_eq!(call.settings.temperature, Some(0.3));
    assert_eq!(call.settings.game, Some(GameId(7)));
}

// CASE memory-second-try: a first try that fails and a second that succeeds give the entries of the second try, from the same prompt.
#[test]
fn test_a_second_try_after_a_failed_one_gives_its_entries() {
    let model = Arc::new(
        ScriptedModel::new()
            .story_memory(NOT_JSON)
            .story_memory(answer(&[(12, 15)])),
    );

    let entries = run_with(&model, Some(&previous_entry())).expect("the second try is good");

    assert_eq!(entries, vec![kept_entry(12, 12, 15)]);
    let calls = model.calls();
    assert_eq!(calls.len(), 2, "two tries");
    assert_eq!(
        calls[0].parts, calls[1].parts,
        "the same prompt on both tries"
    );
}

// CASE memory-stopped-at-limit [failure]: an answer stopped at the answer limit, on both tries, fails the writer.
#[test]
fn test_an_answer_stopped_at_the_limit_fails() {
    let cut = "{\"entries\": [{\"from\": 12, \"time\": \"Day 2";
    let (error, calls) = failure_of(
        ScriptedModel::new()
            .cut_off(Role::StoryMemory, cut)
            .cut_off(Role::StoryMemory, cut),
    );

    assert!(
        matches!(
            error,
            EngineError::Llm(LlmFailure::AnswerTooLong {
                stopped_at_limit: true,
                ..
            })
        ),
        "a cut-off answer gives no entries: {error:?}"
    );
    assert_eq!(calls, 2);
}

// CASE memory-empty-answer [failure]: an empty answer, on both tries, fails the writer.
#[test]
fn test_an_empty_answer_fails() {
    let (error, calls) = failure_of(ScriptedModel::new().story_memory("   \n").story_memory(""));

    assert!(
        matches!(error, EngineError::Llm(LlmFailure::EmptyResponse { .. })),
        "a blank answer gives no entries: {error:?}"
    );
    assert_eq!(calls, 2);
}

// CASE memory-unparseable [failure]: an answer with no JSON object, on both tries, fails the writer.
#[test]
fn test_an_answer_that_does_not_parse_fails() {
    let (error, calls) = failure_of(twice(NOT_JSON));

    assert!(is_parse_error(&error, MEMORY_ENTRIES_FORMAT), "{error:?}");
    assert_eq!(calls, 2);
}

// CASE memory-missing-field [failure]: an entry with a missing field, on both tries, fails the writer.
#[test]
fn test_an_entry_with_a_missing_field_fails() {
    let missing = json!({ "entries": [{ "from": 12, "to": 15, "time": "Day 2", "place": "The gate",
        "took_part": [], "others": [] }] })
    .to_string();
    let (error, _) = failure_of(twice(missing));

    assert!(is_parse_error(&error, MEMORY_ENTRIES_FORMAT), "{error:?}");
}

// CASE memory-empty-list [failure]: an answer with no entry, on both tries, fails the writer.
#[test]
fn test_an_answer_with_no_entry_fails() {
    let empty = json!({ "entries": [] }).to_string();
    let (error, _) = failure_of(twice(empty));

    assert!(is_parse_error(&error, MEMORY_ENTRIES_FORMAT), "{error:?}");
}

// CASE memory-from-outside-call [failure]: a scene start that is no Message of the call, on both tries, fails the writer.
#[test]
fn test_a_scene_start_outside_the_call_fails() {
    let outside = answer(&[(12, 13), (14, 15)]);
    let (error, _) = failure_of(twice(outside));

    assert!(is_parse_error(&error, SCENE_STARTS_FORMAT), "{error:?}");
}

// CASE memory-from-not-rising [failure]: later scene starts that do not go up, on both tries, fail the writer.
#[test]
fn test_scene_starts_that_do_not_go_up_fail() {
    let falling = answer(&[(12, 13), (15, 15), (13, 13)]);
    let (error, _) = failure_of(twice(falling));

    assert!(is_parse_error(&error, SCENE_STARTS_FORMAT), "{error:?}");
}

// CASE memory-later-start-at-first-message [failure]: a later scene that starts at the first Message of the call, on both tries, fails the writer.
#[test]
fn test_a_later_scene_start_at_the_first_message_fails() {
    let doubled = answer(&[(13, 15), (12, 12)]);
    let (error, _) = failure_of(twice(doubled));

    assert!(is_parse_error(&error, SCENE_STARTS_FORMAT), "{error:?}");
}

// CASE memory-transport-failure [failure]: two failed calls fail the writer.
#[test]
fn test_two_failed_calls_fail_the_writer() {
    let (_, calls) = failure_of(
        ScriptedModel::new()
            .failed(Role::StoryMemory)
            .failed(Role::StoryMemory),
    );

    assert_eq!(calls, 2);
}

// CASE memory-call-record: a good call is recorded with its agent name, its prompt and its answer.
#[test]
fn test_a_good_call_is_recorded_with_its_prompts() {
    let storage = Arc::new(Storage::new_in_memory());
    let good = answer(&[(12, 15)]);
    let model = Arc::new(ScriptedModel::new().story_memory(good.clone()));
    let rows = leaving();
    let cast = cast();
    let input = StoryMemoryHookInput::new(None, &cast, &rows).expect("Messages leave");
    StoryMemoryWriter::new(make_test_recorder_with_storage(model, Arc::clone(&storage)))
        .run(&input, &settings())
        .expect("a good answer gives entries");

    let recorded = storage
        .list_latest_llm_messages(10)
        .expect("the forensics read back");
    assert_eq!(recorded.len(), 1);
    let call = &recorded[0];
    assert_eq!(call.agent_name, "memory");
    assert!(call.error_message.is_none());
    assert_eq!(call.parsed_response, good);
    assert!(sent_user_text(call).contains("[#13 Narration]"));
}

// CASE memory-call-record-failure [failure]: a refused try is recorded with its agent name and its failure text.
#[test]
fn test_a_refused_try_is_recorded_with_its_failure_text() {
    let recorded = recorded_run(twice(answer(&[(12, 13), (14, 15)])));

    let failures: Vec<_> = recorded
        .iter()
        .map(|call| (call.agent_name.as_str(), call.error_message.as_deref()))
        .collect();
    let refused = Some(
        "LLM Error: unexpected response format (expected scene starts that are Messages of the call, in story order)",
    );
    assert_eq!(failures, vec![("memory", refused), ("memory", refused)]);
}

// CASE memory-call-record-second-try: a good second try after a refused one is recorded as a good call, after the record of the refused try.
#[test]
fn test_a_good_second_try_is_recorded_after_the_refused_one() {
    let good = answer(&[(12, 15)]);

    let recorded = recorded_run(
        ScriptedModel::new()
            .story_memory(NOT_JSON)
            .story_memory(good.clone()),
    );

    assert_eq!(recorded.len(), 2);
    assert!(
        recorded[0].error_message.is_some(),
        "the refused try comes first"
    );
    assert_eq!(recorded[1].agent_name, "memory");
    assert!(recorded[1].error_message.is_none());
    assert_eq!(recorded[1].parsed_response, good);
}
