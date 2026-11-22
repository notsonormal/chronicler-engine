//! HTTP tests of the story memory: the memory entries a history cut writes, and the block the narrator sees.

use std::sync::Arc;

use axum::http::StatusCode;

use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::adapters::driving::http::AppState;
use chronicler_engine::application::prompting::token_budget::{
    estimate_tokens, MESSAGE_OVERHEAD_TOKENS, MEMORY_WRITER_ANSWER_TOKENS,
    MEMORY_WRITER_CALL_TOKENS,
};
use chronicler_engine::domain::model::agent::Role;
use chronicler_engine::domain::model::game::GameId;
use chronicler_engine::domain::model::settings::LlmConnection;
use chronicler_engine::domain::model::trigger::ComparisonOperator;
use chronicler_engine::test_support::{
    analysis_report, ModelCall, ScriptedModel, TestCharacter, TestDataBuilder,
};

use crate::support::app_wiring::{pad_story, padded_story_app_over, story_connection};
use crate::support::http_fixtures::story_of;
use crate::support::http_requests::{
    fetch_part, post_action, post_empty, post_form, post_on_last_message, wait_settled,
};
use crate::SettingsTestGuard;

const WIDE_WINDOW: u32 = 131_072;
const STORY_LENGTH: usize = 300;

#[derive(Clone, Copy)]
struct Shape {
    window: u32,
    history: Option<u32>,
    padding: usize,
}

const CUTTING: Shape = Shape {
    window: 8192,
    history: None,
    padding: STORY_LENGTH,
};
const LONG: Shape = Shape {
    window: 8192,
    history: None,
    padding: 2_000,
};
/// A history space of 6,000 tokens in a wide window: one cut holds about 6,000 tokens of
/// Messages, so it makes two writer calls of at most 4,000 tokens.
const SPLITTING: Shape = Shape {
    window: WIDE_WINDOW,
    history: Some(6_000),
    padding: 400,
};
const FITTING: Shape = Shape {
    window: WIDE_WINDOW,
    history: None,
    padding: STORY_LENGTH,
};

const FIRST_ENTRY: &str = "Mara promised to wait at the gate at dawn.";
const SECOND_ENTRY: &str = "Mara waited at the gate; the caravan left at noon.";
const THIRD_ENTRY: &str = "The caravan reached the river by nightfall.";

const NEXT_TURN: &str = "You listen.";

/// `from` and `to` are placeholders: the engine sets the first scene's start and every
/// range itself.
fn scene(what: &str) -> String {
    format!(
        r#"{{"entries": [{{"from": 1, "to": 1, "time": "Day 1, evening", "place": "The inn", "took_part": ["Mara"], "others": [], "what_happened": "{what}"}}]}}"#
    )
}

struct StoryApp {
    app: axum::Router,
    state: AppState,
    storage: Arc<Storage>,
    game: GameId,
    model: Arc<ScriptedModel>,
}

impl StoryApp {
    fn new(model: ScriptedModel, shape: Shape) -> Self {
        let model = Arc::new(model);
        let connection = LlmConnection {
            max_history_tokens: shape.history,
            ..story_connection(Some(shape.window))
        };
        let (app, state, storage, game) = padded_story_app_over(
            Arc::clone(&model),
            connection,
            TestDataBuilder::default_test().build(),
            shape.padding,
        );
        Self::of(model, app, state, storage, game)
    }

    fn of(
        model: Arc<ScriptedModel>,
        app: axum::Router,
        state: AppState,
        storage: Arc<Storage>,
        game: GameId,
    ) -> Self {
        Self {
            app,
            state,
            storage,
            game,
            model,
        }
    }

    fn pad(&self, from: usize) {
        pad_story(&self.storage, self.game, from, STORY_LENGTH);
    }

    async fn act(&self, command: &str) {
        assert!(post_action(&self.app, command).await.status().is_success());
        self.settle().await;
    }

    async fn settle(&self) {
        assert!(
            wait_settled(&self.state, 5000).await,
            "the Job should settle"
        );
    }

    async fn calls_of_action(&self, command: &str) -> Vec<ModelCall> {
        let seen = self.model.calls().len();
        self.act(command).await;
        self.calls_since(seen)
    }

    fn calls_since(&self, seen: usize) -> Vec<ModelCall> {
        self.model.calls().into_iter().skip(seen).collect()
    }

    /// After the edit the history holds every Message, so the next Action cuts nothing.
    async fn widen_window(&self) {
        let widened = post_form(
            &self.app,
            "/connections/narration-conn/edit",
            &format!(
                "conn_name=Wide&conn_provider=mock&conn_model=mock-model&conn_api_key=\
                 &conn_base_url=&conn_max_context_tokens={WIDE_WINDOW}"
            ),
        )
        .await;
        assert_eq!(widened.status(), StatusCode::OK, "the window edit saves");
    }

    /// The script must hold that Action's Narration: see `NEXT_TURN`.
    async fn next_action_shows(&self) -> Option<String> {
        self.widen_window().await;
        let calls = self.calls_of_action("listen").await;
        assert_eq!(
            roles(&calls),
            [Role::Narrator, Role::Quantifier],
            "the widened history cuts nothing"
        );
        narrator_block(&of_role(&calls, Role::Narrator))
    }

    fn transcribed_ids(&self, writer: &ModelCall) -> Vec<u64> {
        let prompt = writer.user_prompt();
        story_of(&self.state)
            .iter()
            .map(|message| message.id)
            .filter(|id| prompt.contains(&format!("[#{id} ")))
            .collect()
    }
}

fn roles(calls: &[ModelCall]) -> Vec<Role> {
    calls.iter().map(|call| call.role).collect()
}

fn of_role(calls: &[ModelCall], role: Role) -> ModelCall {
    calls
        .iter()
        .find(|call| call.role == role)
        .cloned()
        .unwrap_or_else(|| panic!("a {role:?} call in {:?}", roles(calls)))
}

fn writer_calls(calls: &[ModelCall]) -> Vec<ModelCall> {
    calls
        .iter()
        .filter(|call| call.role == Role::StoryMemory)
        .cloned()
        .collect()
}

fn between<'a>(prompt: &'a str, open: &str, close: &str) -> Option<&'a str> {
    let start = prompt.find(open)? + open.len();
    let end = start + prompt[start..].find(close)?;
    Some(&prompt[start..end])
}

fn narrator_block(narrator: &ModelCall) -> Option<String> {
    between(&narrator.user_prompt(), "<StoryMemory>\n", "</StoryMemory>").map(str::to_string)
}

fn previous_entry(writer: &ModelCall) -> Option<String> {
    between(
        &writer.user_prompt(),
        "<PreviousEntry>\n",
        "\n</PreviousEntry>",
    )
    .filter(|text| !text.starts_with("(none"))
    .map(str::to_string)
}

fn assert_in_order(text: &str, parts: &[&str]) {
    let mut from = 0;
    for part in parts {
        let at = text[from..]
            .find(part)
            .unwrap_or_else(|| panic!("`{part}` after byte {from} in: {text}"));
        from += at + part.len();
    }
}

fn assert_block_lacks(block: &str, entry: &str) {
    assert!(
        !block.contains(entry),
        "the block does not hold `{entry}`: {block}"
    );
}

/// `later` scripts the replies after that Action's.
async fn story_after_one_cut(
    later: impl FnOnce(ScriptedModel) -> ScriptedModel,
) -> (StoryApp, u64) {
    let story = StoryApp::new(
        later(
            ScriptedModel::new()
                .story_memory(scene(FIRST_ENTRY))
                .turn("You look around."),
        ),
        CUTTING,
    );
    let calls = story.calls_of_action("look").await;
    let block = narrator_block(&of_role(&calls, Role::Narrator)).expect("the cut writes an entry");
    assert!(
        block.contains(FIRST_ENTRY),
        "the first cut writes a memory entry: {block}"
    );
    let covered = *story
        .transcribed_ids(&of_role(&calls, Role::StoryMemory))
        .last()
        .expect("Messages leave the history");
    (story, covered)
}

async fn story_after_two_cuts(later: impl FnOnce(ScriptedModel) -> ScriptedModel) -> StoryApp {
    let (story, _) = story_after_one_cut(|model| {
        later(model.story_memory(scene(SECOND_ENTRY)).turn("You wait."))
    })
    .await;
    let calls = story.calls_of_action("wait").await;
    assert_eq!(
        roles(&calls),
        [Role::StoryMemory, Role::Narrator, Role::Quantifier],
        "the second Action cuts the history again"
    );
    let block = narrator_block(&of_role(&calls, Role::Narrator)).expect("a story memory block");
    assert_in_order(&block, &[FIRST_ENTRY, SECOND_ENTRY]);
    story
}

// [docs/specs/story_memory.md] SCENARIO: 46.1
#[tokio::test]
async fn test_a_cut_writes_a_memory_entry_before_the_narration() {
    let story = StoryApp::new(
        ScriptedModel::new()
            .story_memory(scene(FIRST_ENTRY))
            .turn("You look around.")
            .story_memory(scene(SECOND_ENTRY))
            .turn("You wait."),
        CUTTING,
    );

    let calls = story.calls_of_action("look").await;

    assert_eq!(
        roles(&calls),
        [Role::StoryMemory, Role::Narrator, Role::Quantifier],
        "the writer runs before the narrator"
    );
    let writer = of_role(&calls, Role::StoryMemory);
    let narration = of_role(&calls, Role::Narrator).user_prompt();
    let leaving = story.transcribed_ids(&writer);
    let newest_leaving = *leaving.last().expect("Messages leave the history");
    let messages = story_of(&story.state);
    let left = messages
        .iter()
        .find(|message| message.id == newest_leaving)
        .expect("the newest leaving Message is in the story");
    let first_kept = messages
        .iter()
        .find(|message| message.id > newest_leaving && narration.contains(message.text()))
        .expect("a Message stays in the history");

    assert!(
        writer.user_prompt().contains(left.text()),
        "the writer's prompt holds the Messages that leave: {}",
        writer.user_prompt()
    );
    let block_at = narration
        .find("<StoryMemory>")
        .expect("the narrator's prompt holds the story memory");
    let history_at = narration
        .find(first_kept.text())
        .expect("the narrator's prompt holds the kept history");
    assert!(
        block_at < history_at,
        "the story memory stands before the history"
    );
    let block = between(&narration, "<StoryMemory>\n", "</StoryMemory>").expect("the block");
    assert!(
        block.starts_with(&format!("<Opening>\n{}\n</Opening>", messages[0].text())),
        "the block opens with the first Message of the story, word for word: {block}"
    );
    assert!(
        between(block, "<MemoryEntries>\n", "\n</MemoryEntries>")
            .is_some_and(|entries| entries.contains(FIRST_ENTRY)),
        "the block holds the memory entry in <MemoryEntries>: {block}"
    );
    assert!(
        !narration.contains(left.text()),
        "the narrator's prompt holds none of the other Messages that left"
    );
    assert_eq!(
        messages.len(),
        1 + STORY_LENGTH + 2,
        "the story log still holds every Message"
    );

    let second = of_role(&story.calls_of_action("wait").await, Role::StoryMemory);
    let previous = previous_entry(&second).expect("the writer gets the memory entry");
    assert!(
        previous.starts_with(&format!("[#{}-#{newest_leaving}]", leaving[0]))
            && previous.contains(FIRST_ENTRY),
        "the entry covers the Messages of the first call, though the model named its last Message wrongly: {previous}"
    );
    assert_eq!(
        story.transcribed_ids(&second).first(),
        Some(&(newest_leaving + 1)),
        "the second call starts after the Messages that the first call held"
    );
}

// [docs/specs/story_memory.md] SCENARIO: 46.2
#[tokio::test]
async fn test_an_action_that_drops_no_new_message_shows_the_memory_entries_again() {
    let _guard = SettingsTestGuard::new();
    let (story, _) = story_after_one_cut(|model| model.turn("You wait.").turn("You listen.")).await;
    story.widen_window().await;

    let calls = story.calls_of_action("wait").await;

    assert_eq!(roles(&calls), [Role::Narrator, Role::Quantifier]);
    let block = narrator_block(&of_role(&calls, Role::Narrator)).expect("a story memory block");
    assert!(
        block.contains(FIRST_ENTRY),
        "the narrator's prompt holds the same memory entry: {block}"
    );
    let next = story.calls_of_action("listen").await;
    assert_eq!(roles(&next), [Role::Narrator, Role::Quantifier]);
    assert_eq!(
        narrator_block(&of_role(&next, Role::Narrator)),
        Some(block),
        "the new Narration shows the same block"
    );
}

// [docs/specs/story_memory.md] SCENARIO: 46.3
#[tokio::test]
async fn test_a_later_cut_gives_the_writer_the_last_entry_and_only_the_uncovered_messages() {
    let (story, covered) = story_after_one_cut(|model| {
        model
            .story_memory(scene(SECOND_ENTRY))
            .turn("You look again.")
    })
    .await;
    story.pad(STORY_LENGTH);

    let calls = story.calls_of_action("look").await;

    let writer = of_role(&calls, Role::StoryMemory);
    assert!(
        previous_entry(&writer).is_some_and(|entry| entry.contains(FIRST_ENTRY)),
        "the writer's prompt holds the last memory entry as the previous entry"
    );
    let leaving = story.transcribed_ids(&writer);
    assert!(
        leaving.first().is_some_and(|id| *id == covered + 1),
        "the writer starts after the covered Messages: {leaving:?}, covered up to {covered}"
    );
    let block = narrator_block(&of_role(&calls, Role::Narrator)).expect("a story memory block");
    assert_in_order(&block, &[FIRST_ENTRY, SECOND_ENTRY]);
}

// [docs/specs/story_memory.md] SCENARIO: 46.11
#[tokio::test]
async fn test_the_writer_call_asks_for_no_more_than_the_connection_ceiling() {
    const CEILING: u32 = 700;
    const { assert!(CEILING < MEMORY_WRITER_ANSWER_TOKENS) };
    let model = Arc::new(
        ScriptedModel::new()
            .story_memory(scene(FIRST_ENTRY))
            .turn("You look around."),
    );
    let connection = LlmConnection {
        max_tokens_ceiling: Some(CEILING),
        ..story_connection(Some(CUTTING.window))
    };
    let (app, state, storage, game) = padded_story_app_over(
        Arc::clone(&model),
        connection,
        TestDataBuilder::default_test().build(),
        CUTTING.padding,
    );
    let story = StoryApp::of(model, app, state, storage, game);

    let capped = of_role(&story.calls_of_action("look").await, Role::StoryMemory);

    assert_eq!(
        capped.settings.max_tokens, CEILING,
        "the writer call asks for the ceiling, not the answer size of a writer call"
    );
}

// [docs/specs/story_memory.md] SCENARIO: 46.13
#[tokio::test]
async fn test_one_cut_writes_at_most_one_history_space() {
    let story = StoryApp::new(
        ScriptedModel::new()
            .story_memory(scene(FIRST_ENTRY))
            .turn("You look around.")
            .story_memory(scene(SECOND_ENTRY))
            .turn("You wait."),
        LONG,
    );
    let story_ids: Vec<u64> = story_of(&story.state).iter().map(|m| m.id).collect();

    let calls = story.calls_of_action("look").await;

    let writer = of_role(&calls, Role::StoryMemory);
    let given = story.transcribed_ids(&writer);
    assert!(
        !given.is_empty() && story_ids.starts_with(&given),
        "the writer gets the oldest Messages of the story, in order: {given:?}"
    );
    let covered = *given.last().expect("Messages leave the history");
    let narration = of_role(&calls, Role::Narrator).user_prompt();
    let first_left_out = story_of(&story.state)
        .into_iter()
        .find(|message| message.id == covered + 1)
        .expect("a Message after the covered ones");
    assert!(
        !narration.contains(first_left_out.text()),
        "the next Message left the history too, and the writer did not get it"
    );
    let writer_tokens = (writer.system_prompt().len() + writer.user_prompt().len()) / 4;
    assert!(
        writer_tokens < LONG.window as usize,
        "the writer's prompt fits the context window: {writer_tokens} tokens"
    );

    let second = of_role(&story.calls_of_action("wait").await, Role::StoryMemory);
    assert_eq!(
        story.transcribed_ids(&second).first(),
        Some(&(covered + 1)),
        "the next writer call starts after the covered Messages"
    );
}

/// The token count of one Message in a history cut: the engine's own count, as the
/// `HistoryCut` takes it from the text of the Message.
fn cut_tokens_of(story: &StoryApp, id: u64) -> usize {
    let messages = story_of(&story.state);
    let message = messages
        .iter()
        .find(|message| message.id == id)
        .expect("the Message is in the story");
    estimate_tokens(message.text()) + MESSAGE_OVERHEAD_TOKENS
}

// [docs/specs/story_memory.md] SCENARIO: 46.14
#[tokio::test]
async fn test_a_cut_over_one_call_makes_several_writer_calls_before_the_narration() {
    let story = StoryApp::new(
        ScriptedModel::new()
            .story_memory(scene(FIRST_ENTRY))
            .story_memory(scene(SECOND_ENTRY))
            .turn("You look around."),
        SPLITTING,
    );

    let calls = story.calls_of_action("look").await;

    assert_eq!(
        roles(&calls),
        [
            Role::StoryMemory,
            Role::StoryMemory,
            Role::Narrator,
            Role::Quantifier
        ],
        "the cut of this story has two writer calls, one after the other, before the narrator"
    );
    let writers = writer_calls(&calls);
    let first = story.transcribed_ids(&writers[0]);
    let second = story.transcribed_ids(&writers[1]);
    let first_tokens: usize = first.iter().map(|id| cut_tokens_of(&story, *id)).sum();
    let second_tokens: usize = second.iter().map(|id| cut_tokens_of(&story, *id)).sum();
    assert!(
        first_tokens <= MEMORY_WRITER_CALL_TOKENS && second_tokens <= MEMORY_WRITER_CALL_TOKENS,
        "a writer call holds at most 4,000 tokens of Messages: {first_tokens} and {second_tokens}"
    );
    assert!(
        first_tokens + cut_tokens_of(&story, second[0]) > MEMORY_WRITER_CALL_TOKENS,
        "the first call is full: the next Message would pass the limit"
    );
    assert_eq!(
        first.last().map(|id| id + 1),
        second.first().copied(),
        "the Messages of the calls follow each other with no gap"
    );
    assert_eq!(
        previous_entry(&writers[0]),
        None,
        "the first call has no previous entry"
    );
    let previous = previous_entry(&writers[1]).expect("the second call has a previous entry");
    assert!(
        previous.contains(FIRST_ENTRY)
            && previous.starts_with(&format!(
                "[#{}-#{}]",
                first[0],
                first.last().expect("a Message")
            )),
        "the second call holds the last entry of the first call: {previous}"
    );
    let block = narrator_block(&of_role(&calls, Role::Narrator)).expect("a story memory block");
    assert_in_order(&block, &[FIRST_ENTRY, SECOND_ENTRY]);
}

async fn assert_banner_names_the_story_memory(story: &StoryApp) {
    let banner = fetch_part(&story.app, "failure-banner").await;
    assert!(
        banner.contains("Story memory failed")
            && banner.contains("keeping the earlier memory entries"),
        "the banner names the story memory: {banner}"
    );
}

// [docs/specs/story_memory.md] SCENARIO: 46.15
#[tokio::test]
async fn test_a_failed_call_in_a_split_keeps_the_entries_of_the_calls_before_it() {
    let story = StoryApp::new(
        ScriptedModel::new()
            .story_memory(scene(FIRST_ENTRY))
            .failed(Role::StoryMemory)
            .failed(Role::StoryMemory)
            .turn("You look around.")
            .story_memory(scene(SECOND_ENTRY))
            .story_memory(scene(SECOND_ENTRY))
            .optional()
            .turn("You wait."),
        SPLITTING,
    );

    let calls = story.calls_of_action("look").await;

    assert_eq!(
        roles(&calls),
        [
            Role::StoryMemory,
            Role::StoryMemory,
            Role::StoryMemory,
            Role::Narrator,
            Role::Quantifier
        ],
        "the second call fails on both tries, and the Job goes on"
    );
    let failed_call = story.transcribed_ids(&writer_calls(&calls)[1]);
    let block = narrator_block(&of_role(&calls, Role::Narrator)).expect("a story memory block");
    assert!(
        block.contains(FIRST_ENTRY),
        "the narrator's prompt holds the entry of the first call: {block}"
    );
    assert!(
        story_of(&story.state)
            .last()
            .is_some_and(|message| message.text() == "You look around."),
        "the Narration is saved"
    );
    assert_banner_names_the_story_memory(&story).await;

    let next = writer_calls(&story.calls_of_action("wait").await);
    assert!(
        previous_entry(&next[0]).is_some_and(|entry| entry.contains(FIRST_ENTRY)),
        "the writer's prompt holds the entry of the first call as the previous entry"
    );
    let retried = story.transcribed_ids(&next[0]);
    assert_eq!(
        retried.first(),
        failed_call.first(),
        "the next cut starts at the first Message of the failed call"
    );
    assert!(
        failed_call.iter().all(|id| retried.contains(id)),
        "the next writer call holds the Messages the failed call missed"
    );
}

// [docs/specs/story_memory.md] SCENARIO: 46.4
#[tokio::test]
async fn test_a_history_inside_its_space_shows_no_story_memory() {
    let _guard = SettingsTestGuard::new();
    let story = StoryApp::new(
        ScriptedModel::new()
            .turn("You look around.")
            .turn(NEXT_TURN),
        FITTING,
    );

    let calls = story.calls_of_action("look").await;

    assert_eq!(roles(&calls), [Role::Narrator, Role::Quantifier]);
    assert!(
        !of_role(&calls, Role::Narrator)
            .user_prompt()
            .contains("<StoryMemory>"),
        "no story memory block"
    );
    assert_eq!(
        story.next_action_shows().await,
        None,
        "the next Action shows no memory entry"
    );
}

// [docs/specs/story_memory.md] SCENARIO: 46.16
#[tokio::test]
async fn test_a_writer_call_that_fails_once_is_tried_again() {
    let story = StoryApp::new(
        ScriptedModel::new()
            .failed(Role::StoryMemory)
            .story_memory(scene(FIRST_ENTRY))
            .turn("You look around."),
        CUTTING,
    );

    let calls = story.calls_of_action("look").await;

    assert_eq!(
        roles(&calls),
        [
            Role::StoryMemory,
            Role::StoryMemory,
            Role::Narrator,
            Role::Quantifier
        ],
        "the writer is tried twice"
    );
    let tries = writer_calls(&calls);
    assert_eq!(tries[0].system_prompt(), tries[1].system_prompt());
    assert_eq!(tries[0].user_prompt(), tries[1].user_prompt());
    let block = narrator_block(&of_role(&calls, Role::Narrator)).expect("a story memory block");
    assert!(
        block.contains(FIRST_ENTRY),
        "the narrator's prompt holds the entry of the second try: {block}"
    );
    let banner = fetch_part(&story.app, "failure-banner").await;
    assert!(
        !banner.contains("Story memory"),
        "the failure banner does not name the story memory: {banner}"
    );
}

/// Gives the ids the writer calls missed.
fn assert_the_job_goes_on(story: &StoryApp, calls: &[ModelCall], narration: &str) -> Vec<u64> {
    assert_eq!(
        roles(calls),
        [
            Role::StoryMemory,
            Role::StoryMemory,
            Role::Narrator,
            Role::Quantifier
        ],
        "the Job continues after the writer fails on both tries"
    );
    let block = narrator_block(&of_role(calls, Role::Narrator)).expect("a story memory block");
    assert!(
        block.contains(FIRST_ENTRY),
        "the narrator's prompt holds the earlier memory entry: {block}"
    );
    assert!(
        story_of(&story.state)
            .last()
            .is_some_and(|message| message.text() == narration),
        "the Narration is saved"
    );
    story.transcribed_ids(&of_role(calls, Role::StoryMemory))
}

// [docs/specs/story_memory.md] SCENARIO: 46.5
#[tokio::test]
async fn test_a_failed_writer_keeps_the_earlier_memory_entries() {
    let (story, _) = story_after_one_cut(|model| {
        model
            .failed(Role::StoryMemory)
            .failed(Role::StoryMemory)
            .turn("You look again.")
            .story_memory(scene(THIRD_ENTRY))
            .turn("You wait.")
    })
    .await;
    story.pad(STORY_LENGTH);

    let calls = story.calls_of_action("look").await;

    let missed = assert_the_job_goes_on(&story, &calls, "You look again.");
    assert_banner_names_the_story_memory(&story).await;

    let retry = of_role(&story.calls_of_action("wait").await, Role::StoryMemory);
    assert!(
        previous_entry(&retry).is_some_and(|entry| entry.contains(FIRST_ENTRY)),
        "the writer's prompt holds the earlier memory entry as the previous entry"
    );
    let retried = story.transcribed_ids(&retry);
    assert!(
        missed.iter().all(|id| retried.contains(id)),
        "the next writer call holds the Messages the failed call missed: \
         missed {missed:?}, given {retried:?}"
    );
}

// [docs/specs/story_memory.md] SCENARIO: 46.6
#[tokio::test]
async fn test_a_memory_entry_cut_off_at_the_limit_on_both_tries_keeps_the_earlier_one() {
    const CUT_OFF: &str = r#"{"entries": [{"from": 1, "to": 1, "time": "Day 2, mor"#;
    let _guard = SettingsTestGuard::new();
    let (story, _) = story_after_one_cut(|model| {
        model
            .cut_off(Role::StoryMemory, CUT_OFF)
            .cut_off(Role::StoryMemory, CUT_OFF)
            .turn("You look again.")
            .turn(NEXT_TURN)
    })
    .await;
    story.pad(STORY_LENGTH);

    let calls = story.calls_of_action("look").await;

    assert_the_job_goes_on(&story, &calls, "You look again.");
    assert_banner_names_the_story_memory(&story).await;
    let shown = story
        .next_action_shows()
        .await
        .expect("the earlier memory entry is still shown");
    assert!(
        shown.contains(FIRST_ENTRY),
        "the next Action shows the earlier memory entry: {shown}"
    );
}

// [docs/specs/story_memory.md] SCENARIO: 46.7
#[tokio::test]
async fn test_the_impersonate_call_shows_the_story_memory() {
    let _guard = SettingsTestGuard::new();
    let (story, _) = story_after_one_cut(|model| {
        model
            .narration("I wave to Mara.")
            .after_narration(analysis_report(&[], None))
    })
    .await;
    story.widen_window().await;

    let calls = story.calls_of_action("/impersonate I wave").await;

    assert_eq!(roles(&calls), [Role::Narrator, Role::Quantifier]);
    let impersonate = of_role(&calls, Role::Narrator);
    assert!(
        narrator_block(&impersonate).is_some_and(|block| block.contains(FIRST_ENTRY)),
        "the Impersonate call's prompt holds the memory entry: {}",
        impersonate.user_prompt()
    );
}

// [docs/specs/story_memory.md] SCENARIO: 46.8
#[tokio::test]
async fn test_a_retry_shows_the_entries_of_the_messages_before_the_last_message() {
    let story = story_after_two_cuts(|model| {
        model
            .story_memory(scene(THIRD_ENTRY))
            .turn("You wait, and wait.")
    })
    .await;
    let seen = story.model.calls().len();

    assert!(
        post_on_last_message(&story.app, "/swipe/new")
            .await
            .status()
            .is_success()
    );
    story.settle().await;

    let calls = story.calls_since(seen);
    assert_eq!(
        roles(&calls),
        [Role::StoryMemory, Role::Narrator, Role::Quantifier],
        "the Retry cuts the history again"
    );
    assert!(
        previous_entry(&of_role(&calls, Role::StoryMemory))
            .is_some_and(|entry| entry.contains(FIRST_ENTRY) && !entry.contains(SECOND_ENTRY)),
        "the Retry starts from the memory entry before the last Message"
    );
    let block = narrator_block(&of_role(&calls, Role::Narrator)).expect("a story memory block");
    assert_in_order(&block, &[FIRST_ENTRY, THIRD_ENTRY]);
    assert_block_lacks(&block, SECOND_ENTRY);
}

/// The id of the last Narration and the index of the Swipe a switch control of
/// the story log offers.
fn switch_target(story_log: &str) -> (u64, usize) {
    let route = between(story_log, r#"hx-post="/message/"#, "\"")
        .expect("the log renders an enabled swipe switch");
    let (id, index) = route
        .split_once("/swipe/")
        .expect("the switch route names a Swipe");
    (
        id.parse().expect("the Message id is a number"),
        index.parse().expect("the Swipe index is a number"),
    )
}

// [docs/specs/story_memory.md] SCENARIO: 46.9
#[tokio::test]
async fn test_a_swipe_switch_gives_the_entries_of_that_swipe_back() {
    let _guard = SettingsTestGuard::new();
    let story = story_after_two_cuts(|model| {
        model
            .story_memory(scene(THIRD_ENTRY))
            .turn("You wait, and wait.")
            .turn(NEXT_TURN)
    })
    .await;
    assert!(
        post_on_last_message(&story.app, "/swipe/new")
            .await
            .status()
            .is_success()
    );
    story.settle().await;
    let (message, index) = switch_target(&fetch_part(&story.app, "story-log").await);
    assert_eq!(index, 0, "the switch goes back to the first Swipe");

    let switched = post_empty(&story.app, &format!("/message/{message}/swipe/{index}")).await;
    assert!(switched.status().is_success(), "the switch answers");

    let shown = story
        .next_action_shows()
        .await
        .expect("a story memory block");
    assert_in_order(&shown, &[FIRST_ENTRY, SECOND_ENTRY]);
    assert_block_lacks(&shown, THIRD_ENTRY);
}

// [docs/specs/story_memory.md] SCENARIO: 46.10
#[tokio::test]
async fn test_a_delete_of_the_last_message_gives_the_older_entries_back() {
    let _guard = SettingsTestGuard::new();
    let story = story_after_two_cuts(|model| model.turn(NEXT_TURN)).await;

    assert!(
        post_on_last_message(&story.app, "/history/delete")
            .await
            .status()
            .is_success()
    );

    let shown = story
        .next_action_shows()
        .await
        .expect("a story memory block");
    assert!(
        shown.contains(FIRST_ENTRY),
        "the memory entry of the Message before the deleted one is back: {shown}"
    );
    assert_block_lacks(&shown, SECOND_ENTRY);
}

// [docs/specs/story_memory.md] SCENARIO: 46.12
#[tokio::test]
async fn test_the_event_narration_call_shows_the_story_memory() {
    let character = TestCharacter::with_times_met_trigger(
        "character_1",
        "Test Character",
        ComparisonOperator::Eq,
        0,
    );
    let model = Arc::new(
        ScriptedModel::new()
            .story_memory(scene(FIRST_ENTRY))
            // The cut of the event Narration may call the writer again.
            .story_memory(scene(SECOND_ENTRY))
            .optional()
            .narration("You look around.")
            .after_narration(analysis_report(&["character_1"], None))
            .event_narration("The Character greets you warmly.")
            .after_narration(analysis_report(&["character_1"], None)),
    );
    let (app, state, storage, game) = padded_story_app_over(
        Arc::clone(&model),
        story_connection(Some(CUTTING.window)),
        TestDataBuilder::default_test()
            .characters(vec![character])
            .build(),
        CUTTING.padding,
    );
    let story = StoryApp::of(model, app, state, storage, game);

    let calls = story.calls_of_action("look").await;

    let event = of_role(&calls, Role::Trigger);
    assert!(
        narrator_block(&event).is_some_and(|block| block.contains(FIRST_ENTRY)),
        "the event Narration call's prompt holds the memory entry: {}",
        event.user_prompt()
    );
}
