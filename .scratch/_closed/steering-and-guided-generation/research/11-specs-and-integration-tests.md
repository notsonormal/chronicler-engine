# Spec & Integration-Test Plan: AI Steering

Asset for wayfinder map `.scratch/steering-and-guided-generation/map.md`, ticket 11 (`issues/11-research-specs-and-integration-tests.md`). Audit of the existing spec/test landscape plus the proposed feature specs and HTTP integration tests that will gate implementation.

## Audit findings

### Existing specs

`docs/specs/` currently contains endpoint-named specs:

- `actions.md` — `POST /action` (scenarios 1.x–2.x, 3.3, 6.x).
- `swipe_new.md` — `POST /swipe/new` (scenarios 9.x–12.x).
- `retrigger.md` — `POST /retrigger` (scenarios 13.x–15.x).
- `browser.md`, `games.md`, `prompt_presets.md`, `reset.md`, `settings.md`, `story_log.md`.

None of the existing specs cover narrator messages, guide/steering injection, or impersonate. `actions.md` is the natural home for slash-command behavior because `/guide`, `/narrator`, and `/impersonate` all dispatch through `POST /action`. A separate `steering.md` is proposed below to keep the surface focused.

### `tests/http/requires_migration/`

The migrated-pending test files cover unrelated surfaces:

- `connections.rs` — settings/connections UI.
- `core.rs` — `/reset` error handling.
- `debug.rs` — debug endpoints.
- `fragment.rs` — fragment rendering, action handler basics, history edit/delete, text check, swipe switch.
- `games_fragment_handlers.rs` — games list fragment.
- `index_handler.rs` — dashboard index.
- `server_impl_wiring.rs` — server wiring.
- `text_check.rs` — text-check endpoints.
- `worlds_fragment_handlers.rs` — worlds list fragment.

None of these files cover steering behavior, and none need to be migrated or updated as part of this effort. Steering is a new surface with new scenarios.

### `validate_feature_spec.py`

The validator discovers specs in `docs/specs/*.md` and looks for `// [path/to/spec.md] SCENARIO: X.Y` comments immediately before `#[test]` attributes in `tests/http/` and `tests/browser/`. Every declared scenario must have at least one covering test; every annotated test must reference a declared scenario. Scenario IDs must be unique across specs.

## Proposed spec

The proposed spec lives in a new file `docs/specs/steering.md`.

```markdown
# Feature Spec: Steering (POST /action slash commands)

Endpoint: `POST /action`

Covers the three steering surfaces dispatched from the single command input:
`/guide`, `/narrator`, and `/impersonate`.

## Scenarios

### Narrator

#### Scenario 22.1: `/narrator` persists a Narrator message and triggers a generation

```gherkin
Given a fresh game state with narrative.history empty
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action with command="/narrator The ceiling collapses."
And the pipeline returns to idle
Then message_service.load_messages() contains a new Narrator entry
And the Narrator entry's text is "The ceiling collapses."
And the Narrator entry's sender is None
And message_service.load_messages() contains a new Narration entry (the continue response)
And message_service.load_or_fresh().narrative.input_buffer.status is Idle
```

#### Scenario 22.2: Narrator messages render without a speaker prefix in the prompt

```gherkin
Given a fresh game state with a Narrator message whose text is "The ceiling collapses."
When the client POST /action with command="look"
And the pipeline returns to idle
Then the LLM user prompt's <ConversationHistory> block contains the narrator text
And the narrator text is not prefixed with a sender name such as "Narrator:"
```

### Guide

#### Scenario 23.1: `/guide` triggers a generation without persisting an Input message

```gherkin
Given a fresh game state with narrative.history empty
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action with command="/guide mention the lantern"
And the pipeline returns to idle
Then message_service.load_messages() contains no new Input entry
And message_service.load_messages() contains a new Narration entry
And message_service.load_or_fresh().narrative.input_buffer.status is Idle
```

#### Scenario 23.2: Guide text appears in the prompt as the final layer

```gherkin
Given a fresh game state
And a narrator backend that records the user prompt
When the client POST /action with command="/guide mention the lantern"
And the pipeline returns to idle
Then the recorded user prompt contains "Take the following into special consideration for your next message: mention the lantern"
And the guide block appears after the <PlayerInput> block
```

#### Scenario 23.3: Guide is not persisted in narrative history

```gherkin
Given a fresh game state
When the client POST /action with command="/guide mention the lantern"
And the pipeline returns to idle
Then message_service.load_messages() contains no entry whose text equals "mention the lantern"
```

#### Scenario 23.4: Retry reapplies the guide from the replay blob

```gherkin
Given a fresh game state
And a narrator backend that records the user prompt
When the client POST /action with command="/guide mention the lantern"
And the pipeline returns to idle
And the active swipe of the new Narration message has a replay blob whose guide is "mention the lantern"
And the client POST /swipe/new
And the pipeline returns to idle
Then the retry's recorded user prompt contains "Take the following into special consideration for your next message: mention the lantern"
And the retried swipe's replay blob guide is "mention the lantern"
```

### Impersonate

#### Scenario 24.1: `/impersonate` selects the impersonate preset

```gherkin
Given a fresh game state seeded with an active impersonate preset
And a narrator backend that records the system prompt
When the client POST /action with command="/impersonate"
And the pipeline returns to idle
Then the recorded system prompt was assembled from the impersonate preset
And the recorded system prompt does not contain the default system preset's narrator voice apparatus
```

#### Scenario 24.2: `/impersonate` injects persona data into the instruction

```gherkin
Given a fresh game state
And a narrator backend that records the system prompt
When the client POST /action with command="/impersonate speak boldly"
And the pipeline returns to idle
Then the recorded system prompt contains the active persona's name and description
And the instruction tells the model to write as that persona
```

#### Scenario 24.3: `/impersonate` saves output as a player-voiced message

```gherkin
Given a fresh game state
And a narrator backend that returns "I am the Hero." for the impersonate generation
When the client POST /action with command="/impersonate speak boldly"
And the pipeline returns to idle
Then message_service.load_messages() contains a new Input entry
And the Input entry's sender is the active persona name
And the Input entry's text is "I am the Hero."
```

#### Scenario 24.4: Retry reapplies impersonate from the replay blob

```gherkin
Given a fresh game state
When the client POST /action with command="/impersonate speak boldly"
And the pipeline returns to idle
And the active swipe of the new Input message has a replay blob whose impersonate flag is true
And the client POST /swipe/new
And the pipeline returns to idle
Then the retried swipe's replay blob impersonate flag is true
And the retry uses the impersonate preset
```

### Slash parser

#### Scenario 25.1: Unknown slash command falls back to plain player action

```gherkin
Given a fresh game state
When the client POST /action with command="/unknown hello"
And the pipeline returns to idle
Then message_service.load_messages() contains an Input entry whose text is "/unknown hello"
And message_service.load_messages() contains a Narration entry (normal action flow)
```

#### Scenario 25.2: `/narrator`, `/guide`, and `/impersonate` are dispatched correctly

```gherkin
Given a fresh game state
When the client POST /action with command="/narrator The sky darkens."
And the pipeline returns to idle
Then message_service.load_messages() contains a Narrator entry
When the client POST /action with command="/guide mention the lantern"
And the pipeline returns to idle
Then message_service.load_messages() contains no entry whose text equals "mention the lantern"
When the client POST /action with command="/impersonate"
And the pipeline returns to idle
Then message_service.load_messages() contains an Input entry whose sender is the active persona name
```

#### Scenario 25.3: Guide and impersonate are mutually exclusive per turn

```gherkin
Given a fresh game state seeded with an active impersonate preset
When the client POST /action with command="/impersonate speak boldly"
And the pipeline returns to idle
Then the new Input message's replay blob has impersonate=true and guide is None
When the client POST /action with command="/guide mention the lantern"
And the pipeline returns to idle
Then the new Narration message's replay blob has guide="mention the lantern" and impersonate is false
And no single swipe's replay blob has both impersonate=true and a non-None guide
```

## Invariants

These properties hold across every steering command and are observable
through HTTP. Drift indicates a regression even if all scenarios pass.

- **I.1** After the pipeline returns to idle, `input_buffer.status` is
  `Idle` or `Error(msg)` — never stuck `Generating`.
- **I.2** A `/guide` turn never adds an `Input` message to history.
- **I.3** A `/narrator` turn always adds a `Narrator` message to history
  before the generated `Narration`.
- **I.4** An `/impersonate` turn always adds an `Input` message to history
  (the AI speaks as the player).
- **I.5** `MessageType::Narrator` is rendered bare in the prompt — no
  sender prefix.
- **I.6** No swipe's replay blob combines `impersonate=true` with a
  non-None `guide`.
```

### Scenario ID allocation

Proposed scenario IDs use the unused `22.x`–`25.x` ranges to avoid collisions with:

- `actions.md`: 1.x–2.x, 3.3, 6.x
- `story_log.md`: 8.x
- `swipe_new.md`: 9.x–12.x
- `retrigger.md`: 13.x–15.x
- `browser.md`: 16.x
- `games.md`: 17.x–19.x
- `settings.md`: 20.x
- `prompt_presets.md`: 21.x

## Proposed integration tests

The proposed HTTP E2E tests live in a new file `tests/http/steering.rs`. They are written against the design decisions in ticket 04 and reference these implementation additions:

- New `MessageType::Narrator` variant.
- New `PresetType::Impersonate` variant.
- New `Swipe::replay: Option<GenerationReplay>` field.
- New `GenerationReplay` blob with `guide`, `impersonate`, `impersonate_direction`, and `impersonate_preset_id` fields.
- New `AppSettings::active_impersonate_prompt_preset_id` field (or equivalent active-impersonate-preset mechanism).
- Slash-command dispatch in the `POST /action` handler.

Because those types do not exist yet, the file is intentionally not committed now; it will be created by the implementation tickets or after the grilling ticket resolves any open questions.

### Test outline

The test file defines a local helper `app_with_recorded_narrator` that builds an app whose narrator LLM calls are persisted via `make_test_recorder_with_storage`, and a helper `latest_narrator_llm_message` that reads the latest `AGENT_NARRATOR` call back through `game_view_query.list_latest_llm_messages`. Each proposed test maps to one scenario in the spec above.

| Scenario | Proposed test | Key assertion |
|----------|---------------|---------------|
| 22.1 | `test_narrator_persists_message_and_triggers_generation_http` | History contains one `Narrator` entry and at least one `Narration`; status is `Idle`. |
| 22.2 | `test_narrator_renders_without_sender_prefix_http` | Seed a `Narrator` message, send `look`, assert the recorded user prompt contains the text but not `"Narrator: "`. |
| 23.1 | `test_guide_triggers_generation_without_input_http` | Send `/guide mention the lantern`; assert no `Input` entry and a `Narration` entry exists. |
| 23.2 | `test_guide_appears_as_final_layer_http` | Record the user prompt; assert the guide wrapper text is present and appears after `<PlayerInput>`. |
| 23.3 | `test_guide_is_not_persisted_in_history_http` | Assert no history entry text equals `"mention the lantern"`. |
| 23.4 | `test_retry_reapplies_guide_from_replay_blob_http` | Assert replay blob has `guide = Some("mention the lantern")`; retry and assert the guide is re-applied. |
| 24.1 | `test_impersonate_selects_impersonate_preset_http` | Seed an impersonate preset, set it active, send `/impersonate`; assert the recorded system prompt uses the impersonate preset and suppresses default narrator voice. |
| 24.2 | `test_impersonate_injects_persona_data_http` | Assert the recorded system prompt contains the active persona name/description. |
| 24.3 | `test_impersonate_saves_output_as_player_voiced_http` | Assert the generated text is saved as an `Input` entry with sender = active persona name. |
| 24.4 | `test_retry_reapplies_impersonate_from_replay_blob_http` | Assert replay blob `impersonate=true`; retry and assert the flag persists and the impersonate preset is used. |
| 25.1 | `test_unknown_slash_command_falls_back_to_player_action_http` | Send `/unknown hello`; assert it becomes an `Input` entry. |
| 25.2 | `test_slash_commands_are_dispatched_http` | Send `/narrator`, `/guide`, and `/impersonate` in sequence and assert the three distinct outcomes. |
| 25.3 | `test_guide_and_impersonate_mutually_exclusive_http` | Assert impersonate turn has `impersonate=true, guide=None` and guide turn has `guide=Some(...), impersonate=false`. |

### Proposed test file

```rust
//! HTTP E2E tests for steering slash commands (`/guide`, `/narrator`, `/impersonate`).

use std::sync::Arc;

use axum::http::StatusCode;

use chronicler_engine::adapters::driven::llm::providers::MockBackend;
use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::application::ports::llm_provider::{AGENT_NARRATOR, LlmProvider};
use chronicler_engine::domain::model::prompt_preset::{PresetType, PromptPreset};
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::domain::model::state::generation_status::GenerationStatus;
use chronicler_engine::test_support::{
    make_test_pipeline_with_backends, make_test_recorder_with_storage, TestAppBuilder,
};

use crate::test_helpers::{post_action, post_empty, wait_idle};

fn app_with_recorded_narrator(
    narrator: Arc<MockBackend>,
    storage: Arc<Storage>,
) -> (axum::Router, chronicler_engine::adapters::driving::http::AppState) {
    let recorder = make_test_recorder_with_storage(
        narrator as Arc<dyn LlmProvider>,
        Arc::clone(&storage),
    );
    let pipeline = make_test_pipeline_with_backends(
        Arc::clone(&storage),
        recorder,
        chronicler_engine::application::agents::registry::AgentRegistry::default(),
    );
    TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .pipeline(pipeline)
        .build_with_state()
}

fn latest_narrator_llm_message(
    state: &chronicler_engine::adapters::driving::http::AppState,
) -> Option<chronicler_engine::domain::model::llm_message::LlmMessage> {
    state
        .game_view_query
        .list_latest_llm_messages(50)
        .ok()?
        .into_iter()
        .rev()
        .find(|m| m.agent_name == AGENT_NARRATOR)
}

// [docs/specs/steering.md] SCENARIO: 22.1
#[tokio::test]
async fn test_narrator_persists_message_and_triggers_generation_http() {
    let narrator = Arc::new(MockBackend::default().with_narrations(vec![
        "The dust settles.".to_string(),
    ]));
    let (app, state) = app_with_recorded_narrator(narrator, Arc::new(Storage::new_in_memory()));

    let resp = post_action(&app, "/narrator The ceiling collapses.").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    let messages = state.message_service.load_messages().unwrap();
    let narrator_entries: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narrator)
        .collect();
    assert_eq!(narrator_entries.len(), 1);
    assert_eq!(narrator_entries[0].text(), "The ceiling collapses.");
    assert!(narrator_entries[0].sender.is_none());

    let narrations: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .collect();
    assert!(!narrations.is_empty());
    assert!(matches!(
        state.message_service.load_or_fresh().narrative.input_buffer.status,
        GenerationStatus::Idle
    ));
}

// [docs/specs/steering.md] SCENARIO: 22.2
#[tokio::test]
async fn test_narrator_renders_without_sender_prefix_http() {
    let storage = Arc::new(Storage::new_in_memory());
    let app = TestAppBuilder::default_test()
        .log("The ceiling collapses.", None, MessageType::Narrator)
        .storage(Arc::clone(&storage))
        .build();

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    let msg = latest_narrator_llm_message(&state).expect("recorded narrator call");
    assert!(msg.user_prompt.contains("The ceiling collapses."));
    assert!(!msg.user_prompt.contains("Narrator: The ceiling collapses."));
}

// [docs/specs/steering.md] SCENARIO: 23.1
#[tokio::test]
async fn test_guide_triggers_generation_without_input_http() {
    let narrator = Arc::new(MockBackend::default().with_narrations(vec![
        "You notice the lantern.".to_string(),
    ]));
    let (app, state) = app_with_recorded_narrator(narrator, Arc::new(Storage::new_in_memory()));

    let resp = post_action(&app, "/guide mention the lantern").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    let messages = state.message_service.load_messages().unwrap();
    assert!(messages.iter().filter(|m| m.message_type == MessageType::Input).next().is_none());
    assert!(messages.iter().filter(|m| m.message_type == MessageType::Narration).next().is_some());
    assert!(matches!(
        state.message_service.load_or_fresh().narrative.input_buffer.status,
        GenerationStatus::Idle
    ));
}

// [docs/specs/steering.md] SCENARIO: 23.2
#[tokio::test]
async fn test_guide_appears_as_final_layer_http() {
    let narrator = Arc::new(MockBackend::default().with_narrations(vec![
        "You notice the lantern.".to_string(),
    ]));
    let (app, state) = app_with_recorded_narrator(narrator, Arc::new(Storage::new_in_memory()));

    let resp = post_action(&app, "/guide mention the lantern").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    let msg = latest_narrator_llm_message(&state).expect("recorded narrator call");
    let guide_text = "Take the following into special consideration for your next message: mention the lantern";
    assert!(msg.user_prompt.contains(guide_text));
    let input_pos = msg.user_prompt.find("<PlayerInput>").expect("<PlayerInput>");
    let guide_pos = msg.user_prompt.find("Take the following into special consideration").expect("guide");
    assert!(input_pos < guide_pos);
}

// [docs/specs/steering.md] SCENARIO: 23.3
#[tokio::test]
async fn test_guide_is_not_persisted_in_history_http() {
    let narrator = Arc::new(MockBackend::default().with_narrations(vec![
        "You notice the lantern.".to_string(),
    ]));
    let (app, state) = app_with_recorded_narrator(narrator, Arc::new(Storage::new_in_memory()));

    let resp = post_action(&app, "/guide mention the lantern").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    let messages = state.message_service.load_messages().unwrap();
    assert!(!messages.iter().any(|m| m.text() == "mention the lantern"));
}

// [docs/specs/steering.md] SCENARIO: 23.4
#[tokio::test]
async fn test_retry_reapplies_guide_from_replay_blob_http() {
    let narrator = Arc::new(MockBackend::default().with_narrations(vec![
        "First.".to_string(),
        "Second.".to_string(),
    ]));
    let (app, state) = app_with_recorded_narrator(narrator, Arc::new(Storage::new_in_memory()));

    let resp = post_action(&app, "/guide mention the lantern").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    let messages = state.message_service.load_messages().unwrap();
    let narration = messages.iter().find(|m| m.message_type == MessageType::Narration).unwrap();
    let replay = narration.swipes[narration.active_swipe_index].replay.as_ref().expect("replay blob");
    assert_eq!(replay.guide.as_deref(), Some("mention the lantern"));
    assert!(!replay.impersonate);

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_idle(&state, 1000).await);

    let retry_msg = latest_narrator_llm_message(&state).expect("retry call");
    assert!(retry_msg.user_prompt.contains("Take the following into special consideration for your next message: mention the lantern"));

    let messages = state.message_service.load_messages().unwrap();
    let narration = messages.iter().find(|m| m.message_type == MessageType::Narration).unwrap();
    let replay = narration.swipes[narration.active_swipe_index].replay.as_ref().expect("replay blob");
    assert_eq!(replay.guide.as_deref(), Some("mention the lantern"));
}

// [docs/specs/steering.md] SCENARIO: 24.1
#[tokio::test]
async fn test_impersonate_selects_impersonate_preset_http() {
    let storage = Arc::new(Storage::new_in_memory());
    let impersonate_preset = PromptPreset {
        id: "impersonate_default".to_string(),
        name: "Default Impersonate".to_string(),
        role: Some("You are writing as the player character.".to_string()),
        instructions: None,
        writing_style: None,
        output_format: None,
        is_default: true,
        preset_type: PresetType::Impersonate,
    };
    storage.save_preset(&impersonate_preset).unwrap();

    let narrator = Arc::new(MockBackend::default().with_narrations(vec![
        "I am the Hero.".to_string(),
    ]));
    let (app, state) = app_with_recorded_narrator(narrator, Arc::clone(&storage));

    {
        let mut settings = state.settings.write().unwrap();
        settings.active_impersonate_prompt_preset_id = "impersonate_default".to_string();
    }

    let resp = post_action(&app, "/impersonate").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    let msg = latest_narrator_llm_message(&state).expect("recorded call");
    assert!(msg.system_prompt.contains("You are writing as the player character."));
    assert!(!msg.system_prompt.contains("You are a test narrator."));
}

// [docs/specs/steering.md] SCENARIO: 24.2
#[tokio::test]
async fn test_impersonate_injects_persona_data_http() {
    let storage = Arc::new(Storage::new_in_memory());
    let impersonate_preset = PromptPreset {
        id: "impersonate_default".to_string(),
        name: "Default Impersonate".to_string(),
        role: Some("Write as {{user}}. {{persona_description}}".to_string()),
        instructions: None,
        writing_style: None,
        output_format: None,
        is_default: true,
        preset_type: PresetType::Impersonate,
    };
    storage.save_preset(&impersonate_preset).unwrap();

    let narrator = Arc::new(MockBackend::default().with_narrations(vec![
        "I am the Hero.".to_string(),
    ]));
    let (app, state) = app_with_recorded_narrator(narrator, Arc::clone(&storage));

    {
        let mut settings = state.settings.write().unwrap();
        settings.active_impersonate_prompt_preset_id = "impersonate_default".to_string();
    }

    let resp = post_action(&app, "/impersonate speak boldly").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    let msg = latest_narrator_llm_message(&state).expect("recorded call");
    assert!(msg.system_prompt.contains("Hero"));
    assert!(msg.system_prompt.contains("The protagonist named Hero."));
}

// [docs/specs/steering.md] SCENARIO: 24.3
#[tokio::test]
async fn test_impersonate_saves_output_as_player_voiced_http() {
    let storage = Arc::new(Storage::new_in_memory());
    let impersonate_preset = PromptPreset {
        id: "impersonate_default".to_string(),
        name: "Default Impersonate".to_string(),
        role: Some("Write as the player.".to_string()),
        instructions: None,
        writing_style: None,
        output_format: None,
        is_default: true,
        preset_type: PresetType::Impersonate,
    };
    storage.save_preset(&impersonate_preset).unwrap();

    let narrator = Arc::new(MockBackend::default().with_narrations(vec![
        "I am the Hero.".to_string(),
    ]));
    let (app, state) = app_with_recorded_narrator(narrator, Arc::clone(&storage));

    {
        let mut settings = state.settings.write().unwrap();
        settings.active_impersonate_prompt_preset_id = "impersonate_default".to_string();
    }

    let resp = post_action(&app, "/impersonate speak boldly").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    let messages = state.message_service.load_messages().unwrap();
    let impersonation = messages.iter().find(|m| m.message_type == MessageType::Input).unwrap();
    assert_eq!(impersonation.text(), "I am the Hero.");
    assert_eq!(impersonation.sender.as_deref(), Some("Hero"));
}

// [docs/specs/steering.md] SCENARIO: 24.4
#[tokio::test]
async fn test_retry_reapplies_impersonate_from_replay_blob_http() {
    let storage = Arc::new(Storage::new_in_memory());
    let impersonate_preset = PromptPreset {
        id: "impersonate_default".to_string(),
        name: "Default Impersonate".to_string(),
        role: Some("Write as the player.".to_string()),
        instructions: None,
        writing_style: None,
        output_format: None,
        is_default: true,
        preset_type: PresetType::Impersonate,
    };
    storage.save_preset(&impersonate_preset).unwrap();

    let narrator = Arc::new(MockBackend::default().with_narrations(vec![
        "First.".to_string(),
        "Second.".to_string(),
    ]));
    let (app, state) = app_with_recorded_narrator(narrator, Arc::clone(&storage));

    {
        let mut settings = state.settings.write().unwrap();
        settings.active_impersonate_prompt_preset_id = "impersonate_default".to_string();
    }

    let resp = post_action(&app, "/impersonate speak boldly").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    let messages = state.message_service.load_messages().unwrap();
    let impersonation = messages.iter().find(|m| m.message_type == MessageType::Input).unwrap();
    let replay = impersonation.swipes[impersonation.active_swipe_index].replay.as_ref().expect("replay blob");
    assert!(replay.impersonate);
    assert_eq!(replay.impersonate_direction.as_deref(), Some("speak boldly"));

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_idle(&state, 1000).await);

    let messages = state.message_service.load_messages().unwrap();
    let impersonation = messages.iter().find(|m| m.message_type == MessageType::Input).unwrap();
    let replay = impersonation.swipes[impersonation.active_swipe_index].replay.as_ref().expect("replay blob");
    assert!(replay.impersonate);
    assert_eq!(replay.impersonate_direction.as_deref(), Some("speak boldly"));
}

// [docs/specs/steering.md] SCENARIO: 25.1
#[tokio::test]
async fn test_unknown_slash_command_falls_back_to_player_action_http() {
    let narrator = Arc::new(MockBackend::default());
    let (app, state) = app_with_recorded_narrator(narrator, Arc::new(Storage::new_in_memory()));

    let resp = post_action(&app, "/unknown hello").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    let messages = state.message_service.load_messages().unwrap();
    let inputs: Vec<_> = messages.iter().filter(|m| m.message_type == MessageType::Input).collect();
    assert_eq!(inputs.len(), 1);
    assert_eq!(inputs[0].text(), "/unknown hello");
}

// [docs/specs/steering.md] SCENARIO: 25.2
#[tokio::test]
async fn test_slash_commands_are_dispatched_http() {
    let storage = Arc::new(Storage::new_in_memory());
    let impersonate_preset = PromptPreset {
        id: "impersonate_default".to_string(),
        name: "Default Impersonate".to_string(),
        role: Some("Write as the player.".to_string()),
        instructions: None,
        writing_style: None,
        output_format: None,
        is_default: true,
        preset_type: PresetType::Impersonate,
    };
    storage.save_preset(&impersonate_preset).unwrap();

    let narrator = Arc::new(MockBackend::default().with_narrations(vec![
        "You notice the lantern.".to_string(),
        "The dust settles.".to_string(),
        "I am the Hero.".to_string(),
    ]));
    let (app, state) = app_with_recorded_narrator(narrator, Arc::clone(&storage));

    {
        let mut settings = state.settings.write().unwrap();
        settings.active_impersonate_prompt_preset_id = "impersonate_default".to_string();
    }

    let resp = post_action(&app, "/narrator The sky darkens.").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);
    assert!(state.message_service.load_messages().unwrap().iter().any(|m| m.message_type == MessageType::Narrator));

    let resp = post_action(&app, "/guide mention the lantern").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);
    assert!(!state.message_service.load_messages().unwrap().iter().any(|m| m.text() == "mention the lantern"));

    let resp = post_action(&app, "/impersonate").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);
    assert!(state.message_service.load_messages().unwrap().iter().any(|m| m.message_type == MessageType::Input && m.sender.as_deref() == Some("Hero")));
}

// [docs/specs/steering.md] SCENARIO: 25.3
#[tokio::test]
async fn test_guide_and_impersonate_mutually_exclusive_http() {
    let storage = Arc::new(Storage::new_in_memory());
    let impersonate_preset = PromptPreset {
        id: "impersonate_default".to_string(),
        name: "Default Impersonate".to_string(),
        role: Some("Write as the player.".to_string()),
        instructions: None,
        writing_style: None,
        output_format: None,
        is_default: true,
        preset_type: PresetType::Impersonate,
    };
    storage.save_preset(&impersonate_preset).unwrap();

    let narrator = Arc::new(MockBackend::default().with_narrations(vec![
        "First.".to_string(),
        "Second.".to_string(),
    ]));
    let (app, state) = app_with_recorded_narrator(narrator, Arc::clone(&storage));

    {
        let mut settings = state.settings.write().unwrap();
        settings.active_impersonate_prompt_preset_id = "impersonate_default".to_string();
    }

    let resp = post_action(&app, "/impersonate speak boldly").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    let messages = state.message_service.load_messages().unwrap();
    let impersonation = messages.iter().find(|m| m.message_type == MessageType::Input).unwrap();
    let replay = impersonation.swipes[impersonation.active_swipe_index].replay.as_ref().expect("replay blob");
    assert!(replay.impersonate);
    assert!(replay.guide.is_none());

    let resp = post_action(&app, "/guide mention the lantern").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    let messages = state.message_service.load_messages().unwrap();
    let narration = messages.iter().find(|m| m.message_type == MessageType::Narration).unwrap();
    let replay = narration.swipes[narration.active_swipe_index].replay.as_ref().expect("replay blob");
    assert!(!replay.impersonate);
    assert_eq!(replay.guide.as_deref(), Some("mention the lantern"));
}
```

## Open questions for grilling

The following questions are better resolved with a human before the spec/tests are committed to the repo:

1. **Scenario ID range.** The proposed `22.x`–`25.x` range is currently free. Confirm this allocation or pick a different range if the project prefers steering scenarios clustered elsewhere.
2. **Impersonate preset activation.** The tests assume a new `AppSettings::active_impersonate_prompt_preset_id` field. Is this the intended mechanism, or should the active impersonate preset be selected differently (e.g., always use a single hard-coded preset, a setting toggle, or a per-world default)?
3. **Unknown slash commands.** The spec treats `/unknown` as a plain player action. Should the parser instead reject unknown commands with a 400 or an inline error?
4. **Mutual-exclusivity enforcement.** The spec checks the replay blob. Should the parser also reject a command that tries to combine both (e.g., `/guide x /impersonate y`), or is mutual exclusivity purely a per-turn generation property?
5. **Narrator message placement.** Should `/narrator` insert the message at the end of history immediately before generation, or should it behave like a permanent system note that can be edited/deleted later?
6. **Impersonate output type.** The tests save impersonate output as `MessageType::Input`. Should it be a new `MessageType::Dialogue` or remain `Input`?

## Files to create after grilling

- `docs/specs/steering.md` — feature spec.
- `tests/http/steering.rs` — HTTP E2E tests.
- Update `tests/http/mod.rs` to add `mod steering;`.
- Regenerate `tests/AGENTS.md` after `steering.rs` is added.
