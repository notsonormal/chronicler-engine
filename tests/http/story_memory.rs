//! HTTP tests of the story memory: the summary a history cut writes, and the block the narrator sees.

use std::sync::Arc;

use axum::http::StatusCode;

use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::adapters::driving::http::AppState;
use chronicler_engine::domain::model::agent::Role;
use chronicler_engine::domain::model::game::GameId;
use chronicler_engine::domain::model::settings::LlmConnection;
use chronicler_engine::domain::model::trigger::ComparisonOperator;
use chronicler_engine::application::prompting::token_budget::STORY_MEMORY_TOKENS;
use chronicler_engine::test_support::{
    analysis_report, ModelCall, ScriptedModel, TestCharacter, TestDataBuilder,
};

use crate::support::app_wiring::{pad_story, padded_story_app, padded_story_app_over, story_connection};
use crate::support::http_fixtures::story_of;
use crate::support::http_requests::{
    fetch_part, post_action, post_empty, post_form, post_on_last_message, wait_settled,
};
use crate::SettingsTestGuard;

const CUTTING_WINDOW: u32 = 8192;
const WIDE_WINDOW: u32 = 131_072;
const STORY_LENGTH: usize = 300;

const FIRST_MEMORY: &str = "Mara promised to wait at the gate at dawn.";
const SECOND_MEMORY: &str = "Mara waited at the gate; the caravan left at noon.";
const THIRD_MEMORY: &str = "The caravan reached the river by nightfall.";

/// The reply of a writer call that the next Action may make: that Action shows the
/// story memory of the current state either way.
const NEXT_MEMORY: &str = "The memory the next Action writes.";
const NEXT_TURN: &str = "You listen.";

struct StoryApp {
    app: axum::Router,
    state: AppState,
    storage: Arc<Storage>,
    game: GameId,
    model: Arc<ScriptedModel>,
}

impl StoryApp {
    fn new(model: ScriptedModel, window: u32) -> Self {
        let model = Arc::new(model);
        let (app, state, storage, game) =
            padded_story_app(Arc::clone(&model), Some(window), STORY_LENGTH);
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

    /// The script must hold that Action's replies: see `then_next_action`.
    async fn next_action_shows(&self) -> Option<String> {
        shown_story_memory(&self.calls_of_action("listen").await)
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

/// Whether the Action that `next_action_shows` makes cuts depends on how the
/// history sits in its space, so its writer reply may stay unused.
fn then_next_action(model: ScriptedModel) -> ScriptedModel {
    model.story_memory(NEXT_MEMORY).optional().turn(NEXT_TURN)
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

fn asked_words(writer: &ModelCall) -> u32 {
    between(&writer.user_prompt(), "in at most ", " words")
        .and_then(|words| words.parse().ok())
        .unwrap_or_else(|| panic!("a word count in {}", writer.user_prompt()))
}

fn memory_block(text: &str) -> String {
    format!("<StoryMemory>\n{text}\n</StoryMemory>")
}

fn between<'a>(prompt: &'a str, open: &str, close: &str) -> Option<&'a str> {
    let start = prompt.find(open)? + open.len();
    let end = start + prompt[start..].find(close)?;
    Some(&prompt[start..end])
}

fn narrator_block(narrator: &ModelCall) -> Option<String> {
    between(
        &narrator.user_prompt(),
        "<StoryMemory>\n",
        "\n</StoryMemory>",
    )
    .map(str::to_string)
}

fn previous_memory(writer: &ModelCall) -> Option<String> {
    between(
        &writer.user_prompt(),
        "<PreviousMemory>\n",
        "\n</PreviousMemory>",
    )
    .filter(|text| *text != "(none)")
    .map(str::to_string)
}

/// The story memory that the start state of an Action keeps: the writer's previous
/// story memory when the Action cuts, else the narrator's block.
fn shown_story_memory(calls: &[ModelCall]) -> Option<String> {
    match calls.iter().find(|call| call.role == Role::StoryMemory) {
        Some(writer) => previous_memory(writer),
        None => narrator_block(&of_role(calls, Role::Narrator)),
    }
}

/// `later` scripts the replies after that Action's.
async fn story_after_one_cut(
    later: impl FnOnce(ScriptedModel) -> ScriptedModel,
) -> (StoryApp, u64) {
    let story = StoryApp::new(
        later(
            ScriptedModel::new()
                .story_memory(FIRST_MEMORY)
                .turn("You look around."),
        ),
        CUTTING_WINDOW,
    );
    let calls = story.calls_of_action("look").await;
    assert_eq!(
        narrator_block(&of_role(&calls, Role::Narrator)).as_deref(),
        Some(FIRST_MEMORY),
        "the first cut writes the story memory"
    );
    let covered = *story
        .transcribed_ids(&of_role(&calls, Role::StoryMemory))
        .last()
        .expect("Messages leave the history");
    (story, covered)
}

async fn story_after_two_cuts(later: impl FnOnce(ScriptedModel) -> ScriptedModel) -> StoryApp {
    let (story, _) =
        story_after_one_cut(|model| later(model.story_memory(SECOND_MEMORY).turn("You wait.")))
            .await;
    let calls = story.calls_of_action("wait").await;
    assert_eq!(
        roles(&calls),
        [Role::StoryMemory, Role::Narrator, Role::Quantifier],
        "the second Action cuts the history again"
    );
    assert_eq!(
        narrator_block(&of_role(&calls, Role::Narrator)).as_deref(),
        Some(SECOND_MEMORY)
    );
    story
}

// [docs/specs/story_memory.md] SCENARIO: 46.1
#[tokio::test]
async fn test_a_cut_writes_the_story_memory_before_the_narration() {
    let story = StoryApp::new(
        ScriptedModel::new()
            .story_memory(FIRST_MEMORY)
            .turn("You look around.")
            .story_memory(SECOND_MEMORY)
            .turn("You wait."),
        CUTTING_WINDOW,
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
    let first_uncovered = newest_leaving + 1;

    assert!(
        writer.user_prompt().contains(left.text()),
        "the writer's prompt holds the Messages that leave: {}",
        writer.user_prompt()
    );
    let block_at = narration
        .find(&memory_block(FIRST_MEMORY))
        .expect("the narrator's prompt holds the story memory");
    let history_at = narration
        .find(first_kept.text())
        .expect("the narrator's prompt holds the kept history");
    assert!(
        block_at < history_at,
        "the story memory stands before the history"
    );
    assert!(
        !narration.contains(left.text()),
        "the narrator's prompt holds none of the Messages that left"
    );
    assert_eq!(
        messages.len(),
        1 + STORY_LENGTH + 2,
        "the story log still holds every Message"
    );

    let next = of_role(&story.calls_of_action("wait").await, Role::StoryMemory);
    assert_eq!(
        previous_memory(&next).as_deref(),
        Some(FIRST_MEMORY),
        "the new Narration keeps the summary"
    );
    assert_eq!(
        story.transcribed_ids(&next).first(),
        Some(&first_uncovered),
        "the summary covers up to the newest Message the first writer call held"
    );
}

// [docs/specs/story_memory.md] SCENARIO: 46.2
#[tokio::test]
async fn test_an_action_that_drops_no_new_message_shows_the_story_memory_again() {
    let _guard = SettingsTestGuard::new();
    let (story, _) = story_after_one_cut(|model| model.turn("You wait.").turn("You listen.")).await;
    let widened = post_form(
        &story.app,
        "/connections/narration-conn/edit",
        &format!(
            "conn_name=Wide&conn_provider=mock&conn_model=mock-model&conn_api_key=\
             &conn_base_url=&conn_max_context_tokens={WIDE_WINDOW}"
        ),
    )
    .await;
    assert_eq!(widened.status(), StatusCode::OK, "the window edit saves");

    let calls = story.calls_of_action("wait").await;

    assert_eq!(roles(&calls), [Role::Narrator, Role::Quantifier]);
    assert_eq!(
        narrator_block(&of_role(&calls, Role::Narrator)).as_deref(),
        Some(FIRST_MEMORY),
        "the narrator's prompt holds the same story memory"
    );
    let next = story.calls_of_action("listen").await;
    assert_eq!(roles(&next), [Role::Narrator, Role::Quantifier]);
    assert_eq!(
        narrator_block(&of_role(&next, Role::Narrator)).as_deref(),
        Some(FIRST_MEMORY),
        "the new Narration keeps the same story memory"
    );
}

// [docs/specs/story_memory.md] SCENARIO: 46.3
#[tokio::test]
async fn test_a_later_cut_gives_the_writer_only_the_messages_it_does_not_cover() {
    let (story, covered) = story_after_one_cut(|model| {
        then_next_action(model.story_memory(SECOND_MEMORY).turn("You look again."))
    })
    .await;
    story.pad(STORY_LENGTH);

    let writer = of_role(&story.calls_of_action("look").await, Role::StoryMemory);

    assert_eq!(
        previous_memory(&writer).as_deref(),
        Some(FIRST_MEMORY),
        "the writer's prompt holds the previous story memory"
    );
    let leaving = story.transcribed_ids(&writer);
    assert!(
        leaving.first().is_some_and(|id| *id == covered + 1),
        "the writer starts after the covered Messages: {leaving:?}, covered up to {covered}"
    );
    assert_eq!(
        story.next_action_shows().await.as_deref(),
        Some(SECOND_MEMORY),
        "the new Narration keeps the new summary"
    );
}

// [docs/specs/story_memory.md] SCENARIO: 46.11
#[tokio::test]
async fn test_the_writer_call_asks_for_no_more_than_the_connection_ceiling() {
    const CEILING: u32 = 700;
    const { assert!(CEILING < STORY_MEMORY_TOKENS) };
    let model = Arc::new(
        ScriptedModel::new()
            .story_memory(FIRST_MEMORY)
            .turn("You look around."),
    );
    let connection = LlmConnection {
        max_tokens_ceiling: Some(CEILING),
        ..story_connection(Some(CUTTING_WINDOW))
    };
    let (app, state, storage, game) = padded_story_app_over(
        Arc::clone(&model),
        connection,
        TestDataBuilder::default_test().build(),
        STORY_LENGTH,
    );
    let story = StoryApp::of(model, app, state, storage, game);

    let capped = of_role(&story.calls_of_action("look").await, Role::StoryMemory);
    let uncapped_story = StoryApp::new(
        ScriptedModel::new()
            .story_memory(FIRST_MEMORY)
            .turn("You look around."),
        CUTTING_WINDOW,
    );
    let uncapped = of_role(
        &uncapped_story.calls_of_action("look").await,
        Role::StoryMemory,
    );

    assert_eq!(
        capped.settings.max_tokens, CEILING,
        "the writer call asks for the ceiling, not the size of the story memory"
    );
    assert!(
        asked_words(&capped) < asked_words(&uncapped),
        "the writer prompt asks for fewer words under the ceiling: {} against {}",
        asked_words(&capped),
        asked_words(&uncapped)
    );
}

// [docs/specs/story_memory.md] SCENARIO: 46.13
#[tokio::test]
async fn test_one_writer_call_holds_at_most_one_history_space() {
    const LONG_STORY: usize = 2_000;
    let model = Arc::new(
        ScriptedModel::new()
            .story_memory(FIRST_MEMORY)
            .turn("You look around.")
            .story_memory(SECOND_MEMORY)
            .turn("You wait."),
    );
    let (app, state, storage, game) =
        padded_story_app(Arc::clone(&model), Some(CUTTING_WINDOW), LONG_STORY);
    let story = StoryApp::of(model, app, state, storage, game);
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
        writer_tokens < CUTTING_WINDOW as usize,
        "the writer's prompt fits the context window: {writer_tokens} tokens"
    );

    let next = of_role(&story.calls_of_action("wait").await, Role::StoryMemory);
    assert_eq!(
        story.transcribed_ids(&next).first(),
        Some(&(covered + 1)),
        "the next writer call starts after the covered Messages"
    );
}

// [docs/specs/story_memory.md] SCENARIO: 46.4
#[tokio::test]
async fn test_a_history_inside_its_space_shows_no_story_memory() {
    let story = StoryApp::new(
        ScriptedModel::new()
            .turn("You look around.")
            .turn(NEXT_TURN),
        WIDE_WINDOW,
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
        "the new Narration keeps no story memory"
    );
}

async fn assert_banner_names_the_story_memory(story: &StoryApp) {
    let banner = fetch_part(&story.app, "failure-banner").await;
    assert!(
        banner.contains("Story memory failed")
            && banner.contains("keeping the previous story memory"),
        "the banner names the story memory: {banner}"
    );
}

/// Gives the ids the writer call missed.
fn assert_the_job_goes_on(story: &StoryApp, calls: &[ModelCall], narration: &str) -> Vec<u64> {
    assert_eq!(
        roles(calls),
        [Role::StoryMemory, Role::Narrator, Role::Quantifier],
        "the Job continues after the failed writer"
    );
    assert_eq!(
        narrator_block(&of_role(calls, Role::Narrator)).as_deref(),
        Some(FIRST_MEMORY),
        "the narrator's prompt holds the previous story memory"
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
async fn test_a_failed_writer_keeps_the_previous_story_memory() {
    let (story, _) = story_after_one_cut(|model| {
        model
            .failed(Role::StoryMemory)
            .turn("You look again.")
            .story_memory(THIRD_MEMORY)
            .turn("You wait.")
    })
    .await;
    story.pad(STORY_LENGTH);

    let calls = story.calls_of_action("look").await;

    let missed = assert_the_job_goes_on(&story, &calls, "You look again.");
    assert_banner_names_the_story_memory(&story).await;

    let retry = of_role(&story.calls_of_action("wait").await, Role::StoryMemory);
    assert_eq!(
        previous_memory(&retry).as_deref(),
        Some(FIRST_MEMORY),
        "the new Narration kept the previous story memory"
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
async fn test_a_story_memory_cut_off_at_the_limit_keeps_the_previous_one() {
    let (story, _) = story_after_one_cut(|model| {
        then_next_action(
            model
                .cut_off(Role::StoryMemory, "Mara waited at the gate; the caravan le")
                .turn("You look again."),
        )
    })
    .await;
    story.pad(STORY_LENGTH);

    let calls = story.calls_of_action("look").await;

    assert_the_job_goes_on(&story, &calls, "You look again.");
    assert_banner_names_the_story_memory(&story).await;
    assert_eq!(
        story.next_action_shows().await.as_deref(),
        Some(FIRST_MEMORY),
        "the new Narration keeps the previous story memory"
    );
}

// [docs/specs/story_memory.md] SCENARIO: 46.7
#[tokio::test]
async fn test_the_impersonate_call_shows_the_story_memory() {
    let (story, _) = story_after_one_cut(|model| {
        model
            .story_memory(SECOND_MEMORY)
            .optional()
            .narration("I wave to Mara.")
            .after_narration(analysis_report(&[], None))
    })
    .await;

    let calls = story.calls_of_action("/impersonate I wave").await;

    let expected = if roles(&calls).contains(&Role::StoryMemory) {
        SECOND_MEMORY
    } else {
        FIRST_MEMORY
    };
    let impersonate = of_role(&calls, Role::Narrator);
    assert!(
        impersonate.user_prompt().contains(&memory_block(expected)),
        "the Impersonate call's prompt holds the story memory: {}",
        impersonate.user_prompt()
    );
}

// [docs/specs/story_memory.md] SCENARIO: 46.8
#[tokio::test]
async fn test_a_retry_starts_from_the_story_memory_before_the_last_message() {
    let story =
        story_after_two_cuts(|model| model.story_memory(THIRD_MEMORY).turn("You wait, and wait."))
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
    assert_eq!(
        previous_memory(&of_role(&calls, Role::StoryMemory)).as_deref(),
        Some(FIRST_MEMORY),
        "the Retry starts from the story memory before the last Message"
    );
    assert_eq!(
        narrator_block(&of_role(&calls, Role::Narrator)).as_deref(),
        Some(THIRD_MEMORY)
    );
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
async fn test_a_swipe_switch_gives_the_story_memory_of_that_swipe_back() {
    let story = story_after_two_cuts(|model| {
        then_next_action(model.story_memory(THIRD_MEMORY).turn("You wait, and wait."))
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

    assert_eq!(
        story.next_action_shows().await.as_deref(),
        Some(SECOND_MEMORY),
        "the first Swipe's story memory is back"
    );
}

// [docs/specs/story_memory.md] SCENARIO: 46.10
#[tokio::test]
async fn test_a_delete_of_the_last_message_gives_the_older_story_memory_back() {
    let story = story_after_two_cuts(then_next_action).await;

    assert!(
        post_on_last_message(&story.app, "/history/delete")
            .await
            .status()
            .is_success()
    );

    assert_eq!(
        story.next_action_shows().await.as_deref(),
        Some(FIRST_MEMORY),
        "the story memory of the Message before the deleted one is back"
    );
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
            .story_memory(FIRST_MEMORY)
            .story_memory(SECOND_MEMORY)
            .optional()
            .narration("You look around.")
            .after_narration(analysis_report(&["character_1"], None))
            .event_narration("The Character greets you warmly.")
            .after_narration(analysis_report(&["character_1"], None)),
    );
    let (app, state, storage, game) = padded_story_app_over(
        Arc::clone(&model),
        story_connection(Some(CUTTING_WINDOW)),
        TestDataBuilder::default_test()
            .characters(vec![character])
            .build(),
        STORY_LENGTH,
    );
    let story = StoryApp::of(model, app, state, storage, game);

    let calls = story.calls_of_action("look").await;

    let event_at = calls
        .iter()
        .position(|call| call.role == Role::Trigger)
        .expect("an event Narration call");
    let writer_calls = roles(&calls[..event_at])
        .into_iter()
        .filter(|role| *role == Role::StoryMemory)
        .count();
    let newest = [FIRST_MEMORY, SECOND_MEMORY][writer_calls - 1];
    let event = &calls[event_at];
    assert!(
        event.user_prompt().contains(&memory_block(newest)),
        "the event Narration call's prompt holds the story memory: {}",
        event.user_prompt()
    );
}
