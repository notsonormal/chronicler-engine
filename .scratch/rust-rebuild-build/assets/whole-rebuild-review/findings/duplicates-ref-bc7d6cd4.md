# Duplicate Summary

- Scope: changed files only (340 in scope)
- Total clone pairs: 438 (filtered: min-lines=5, cross-file-only=True)
- Unique file pairs: 174
- Showing top 25 pairs by dup lines

## tests/http/retrigger.rs  <->  tests/http/swipe_new.rs

Clones: 14 | Dup lines: 285

### Clone 1 (12L)
- tests/http/retrigger.rs:378-389
- tests/http/swipe_new.rs:1260-1271

```rust
let body = axum::body::to_bytes(response.into_body(), 16384)
        .await
        .expect("read body");
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Sti  ... [truncated]
```

### Clone 2 (11L)
- tests/http/retrigger.rs:100-110
- tests/http/swipe_new.rs:572-582

```rust
let data = TestDataBuilder::default_test()
        .map(TestMap::two_rooms("room_1", "room2"))
        .npcs(vec![TestNpc::with_times_met_trigger(
            "npc_1",
            "Test NPC",
          ... [truncated]
```

### Clone 3 (10L)
- tests/http/retrigger.rs:113-122
- tests/http/swipe_new.rs:526-535

```rust
.event_narration("The event fires again.")
        .analysis(analysis_report(&[], Some("room_1")));
    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .model(Arc::new(mo  ... [truncated]
```


## tests/http/actions.rs  <->  tests/http/swipe_new.rs

Clones: 13 | Dup lines: 278

### Clone 1 (10L)
- tests/http/actions.rs:189-198
- tests/http/swipe_new.rs:462-471

```rust
assert!(wait_settled(&state, 1000).await, "action should complete");

    let messages = state
        .game_view_query
        .story_messages(active_game_of(&state))
        .unwrap();
    let narra  ... [truncated]
```

### Clone 2 (10L)
- tests/http/actions.rs:917-926
- tests/http/swipe_new.rs:182-191

```rust
);

    let messages = state
        .game_view_query
        .story_messages(active_game_of(&state))
        .unwrap();
    let narration = messages
        .iter()
        .rfind(|m| m.message_type   ... [truncated]
```

### Clone 3 (9L)
- tests/http/actions.rs:47-55
- tests/http/swipe_new.rs:127-135

```rust
let narrations: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .collect();
    assert_eq!(
        narrations.len(),
        2,
        "the op  ... [truncated]
```


## tests/http/actions.rs  <->  tests/http/options.rs

Clones: 9 | Dup lines: 123

### Clone 1 (11L)
- tests/http/actions.rs:35-45
- tests/http/options.rs:740-751

```rust
assert!(wait_settled(&state, 1000).await, "action should complete");

    let messages = state
        .game_view_query
        .story_messages(active_game_of(&state))
        .unwrap();
    let input  ... [truncated]
```

### Clone 2 (10L)
- tests/http/actions.rs:113-122
- tests/http/options.rs:519-528

```rust
sheet: chronicler_engine::test_support::TestPersona::standard().sheet,
        inventory: vec![],
        triggers: vec![Trigger {
            requirement: TriggerRequirement {
                operato  ... [truncated]
```

### Clone 3 (6L)
- tests/http/actions.rs:129-134
- tests/http/options.rs:111-115

```rust
let data = TestDataBuilder::default_test().npcs(vec![npc]).build();

    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .model(Arc::new(
            ScriptedModel::new()
```


## tests/http/prompt_presets.rs  <->  tests/http/settings.rs

Clones: 8 | Dup lines: 111

### Clone 1 (12L)
- tests/http/prompt_presets.rs:1083-1094
- tests/http/settings.rs:522-533

```rust
"name=++alpha++&instructions=Second.&preset_type=system",
        ))
        .await
        .unwrap();
    assert_eq!(
        second.status(),
        StatusCode::BAD_REQUEST,
        "a case-and-spa  ... [truncated]
```

### Clone 2 (11L)
- tests/http/prompt_presets.rs:31-41
- tests/http/settings.rs:28-38

```rust
fn post_form_request(uri: &str, body: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
     ... [truncated]
```

### Clone 3 (8L)
- tests/http/prompt_presets.rs:1041-1048
- tests/http/settings.rs:469-476

```rust
"name=Alpha&instructions=First.&preset_type=system",
        ))
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);

    let second = app
        .oneshot(post_form_reque  ... [truncated]
```


## tests/http/actions.rs  <->  tests/http/story_log.rs

Clones: 7 | Dup lines: 82

### Clone 1 (8L)
- tests/http/actions.rs:75-82
- tests/http/story_log.rs:107-114

```rust
assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await);

    let messages = state
        .game_view_query
        .story_messages(active_game_of(&state))
        .unwrap()  ... [truncated]
```

### Clone 2 (7L)
- tests/http/actions.rs:129-135
- tests/http/story_log.rs:223-229

```rust
let data = TestDataBuilder::default_test().npcs(vec![npc]).build();

    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .model(Arc::new(
            ScriptedModel::new()  ... [truncated]
```

### Clone 3 (7L)
- tests/http/actions.rs:391-397
- tests/http/story_log.rs:36-42

```rust
let narrations = state
        .game_view_query
        .story_messages(active_game_of(&state))
        .unwrap()
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
          ... [truncated]
```


## tests/http/actions.rs  <->  tests/http/narrator_mode.rs

Clones: 4 | Dup lines: 73

### Clone 1 (11L)
- tests/http/actions.rs:35-45
- tests/http/narrator_mode.rs:69-79

```rust
assert!(wait_settled(&state, 1000).await, "action should complete");

    let messages = state
        .game_view_query
        .story_messages(active_game_of(&state))
        .unwrap();
    let input  ... [truncated]
```

### Clone 2 (6L)
- tests/http/actions.rs:733-738
- tests/http/narrator_mode.rs:226-231

```rust
let (app, state) = app_with_model(Arc::new(ScriptedModel::new().turn("You say hello.")));

    let resp = post_action(&app, "/impersonate hello").await;
    assert!(resp.status().is_success(), "/imper  ... [truncated]
```

### Clone 3 (5L)
- tests/http/actions.rs:31-35
- tests/http/narrator_mode.rs:97-101

```rust
));

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await, "action should complete");
```


## tests/http/games_delete.rs  <->  tests/http/requires_migration/fragment.rs

Clones: 2 | Dup lines: 73

### Clone 1 (7L)
- tests/http/games_delete.rs:62-68
- tests/http/requires_migration/fragment.rs:302-309

```rust
.method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUE  ... [truncated]
```

### Clone 2 (5L)
- tests/http/games_delete.rs:36-40
- tests/http/requires_migration/fragment.rs:196-200

```rust
.uri(format!("/games/{other_id}/delete"))
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
```


## src/application/agents/options/agent_tests.rs  <->  src/application/agents/quantifier/agent_tests.rs

Clones: 7 | Dup lines: 71

### Clone 1 (28L)
- src/application/agents/options/agent_tests.rs:62-89
- src/application/agents/quantifier/agent_tests.rs:64-91

```rust
}],
        }
    }

    fn world(&self) -> WorldView<'_> {
        WorldView {
            room: self.map.get_room_by_id("start"),
            map: &self.map,
            persona: &self.persona,
      ... [truncated]
```

### Clone 2 (10L)
- src/application/agents/options/agent_tests.rs:49-58
- src/application/agents/quantifier/agent_tests.rs:50-59

```rust
let npcs = vec![TestNpc::named("npc_1", "Test NPC")];
        let characters = npcs
            .iter()
            .map(|npc| (npc.id.clone(), npc.clone()))
            .collect();
        Self {
     ... [truncated]
```

### Clone 3 (10L)
- src/application/agents/options/agent_tests.rs:247-256
- src/application/agents/quantifier/agent_tests.rs:300-309

```rust
let _ = agent.run(&scene.input(&world), &AgentSettings::default());

    let recorded = storage
        .list_latest_llm_messages(10)
        .expect("the forensics read back");
    let failure = reco  ... [truncated]
```


## tests/http/games_delete.rs  <->  tests/http/games_switch.rs

Clones: 4 | Dup lines: 62

### Clone 1 (22L)
- tests/http/games_delete.rs:15-36
- tests/http/games_switch.rs:14-35

```rust
async fn test_delete_game_handler_success() {
    let (storage, _world_key, persona_key, _initial_game_id) = seeded_storage_with_initial_game();

    let (app, state) = TestAppBuilder::default_test()
  ... [truncated]
```

### Clone 2 (9L)
- tests/http/games_delete.rs:3-11
- tests/http/games_switch.rs:3-11

```rust
use std::sync::Arc;

use chronicler_engine::test_support::active_game_of;
use axum::{body::Body, http::Request, http::StatusCode};
use tower::util::ServiceExt;
use chronicler_engine::TestAppBuilder;
u  ... [truncated]
```

### Clone 3 (7L)
- tests/http/games_delete.rs:62-68
- tests/http/games_switch.rs:56-60

```rust
.method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUE  ... [truncated]
```


## tests/http/games_create.rs  <->  tests/http/requires_migration/worlds_fragment_handlers.rs

Clones: 1 | Dup lines: 56

### Clone 1 (7L)
- tests/http/games_create.rs:111-117
- tests/http/requires_migration/worlds_fragment_handlers.rs:125-131

```rust
.uri("/games")
        .method(http::Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();

    let response = app.  ... [truncated]
```


## src/application/agents/quantifier/prompt_tests.rs  <->  src/application/agents/quantifier/utils/orchestration_tests.rs

Clones: 1 | Dup lines: 52

### Clone 1 (18L)
- src/application/agents/quantifier/prompt_tests.rs:91-108
- src/application/agents/quantifier/utils/orchestration_tests.rs:421-438

```rust
fn test_quantifier_prompt_builder_empty_history() {
    let room = make_room();
    let all_npcs: Vec<NpcCard> = vec![];
    let previous_npcs: Vec<NpcCard> = vec![];
    let history: Vec<StoryEntry>   ... [truncated]
```


## tests/http/requires_migration/fragment.rs  <->  tests/http/requires_migration/text_check.rs

Clones: 5 | Dup lines: 46

### Clone 1 (15L)
- tests/http/requires_migration/fragment.rs:126-140
- tests/http/requires_migration/text_check.rs:53-68

```rust
.method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=look"))
        .unwra  ... [truncated]
```

### Clone 2 (10L)
- tests/http/requires_migration/fragment.rs:153-162
- tests/http/requires_migration/text_check.rs:79-88

```rust
.method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command="))
        .unwrap();  ... [truncated]
```

### Clone 3 (9L)
- tests/http/requires_migration/fragment.rs:3-11
- tests/http/requires_migration/text_check.rs:3-11

```rust
use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::util::ServiceExt;

use chronicler_engine::test_support::{
```


## tests/browser/stub/dashboard.rs  <->  tests/browser/stub/failure_display.rs

Clones: 3 | Dup lines: 44

### Clone 1 (10L)
- tests/browser/stub/dashboard.rs:149-158
- tests/browser/stub/failure_display.rs:639-648

```rust
confirm_text_check_preview(&page).await;

    status.set(StubStatus::Error(
        GenerationFailureKind::Other,
        "narration failed".to_string(),
    ));
    poll_now(&page, "#status-display")  ... [truncated]
```

### Clone 2 (8L)
- tests/browser/stub/dashboard.rs:271-278
- tests/browser/stub/failure_display.rs:546-553

```rust
.click(None)
        .await
        .unwrap();
    wait_until_hidden(&page, ".text-check-preview", Duration::from_secs(5)).await;

    assert!(
        wait_for_condition_async(
            Duration::  ... [truncated]
```

### Clone 3 (6L)
- tests/browser/stub/dashboard.rs:126-131
- tests/browser/stub/failure_display.rs:730-735

```rust
);

    status.set(StubStatus::Idle);
    poll_now(&page, "#status-display").await;
    wait_for_status_ready(&page).await;
    let (disabled, label) = read_submit_button(&page).await;
```


## tests/http/games_create.rs  <->  tests/http/requires_migration/fragment.rs

Clones: 3 | Dup lines: 44

### Clone 1 (10L)
- tests/http/games_create.rs:64-73
- tests/http/requires_migration/fragment.rs:581-590

```rust
world.key, player.key
        )))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().g  ... [truncated]
```

### Clone 2 (7L)
- tests/http/games_create.rs:56-62
- tests/http/requires_migration/fragment.rs:376-382

```rust
let old_id = active_game_of(&state);

    let req = Request::builder()
        .uri("/games")
        .method(http::Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
   ... [truncated]
```

### Clone 3 (7L)
- tests/http/games_create.rs:115-121
- tests/http/requires_migration/fragment.rs:256-261

```rust
.unwrap();

    let response = app.oneshot(req).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
```


## tests/http/llm_messages.rs  <->  tests/http/options.rs

Clones: 1 | Dup lines: 44

### Clone 1 (8L)
- tests/http/llm_messages.rs:72-79
- tests/http/options.rs:373-380

```rust
.narration("You listen.")
            .analysis(analysis_report(&[], None)),
    );

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(
        wait_  ... [truncated]
```


## tests/http/prompt_presets.rs  <->  tests/http/requires_migration/fragment.rs

Clones: 2 | Dup lines: 40

### Clone 1 (7L)
- tests/http/prompt_presets.rs:33-39
- tests/http/requires_migration/fragment.rs:125-131

```rust
.uri(uri)
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from(body.to_strin  ... [truncated]
```

### Clone 2 (5L)
- tests/http/prompt_presets.rs:813-817
- tests/http/requires_migration/fragment.rs:539-543

```rust
storage.save_preset(&preset).unwrap();

    let app = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build();
```


## tests/http/requires_migration/connections.rs  <->  tests/http/requires_migration/fragment.rs

Clones: 3 | Dup lines: 39

### Clone 1 (7L)
- tests/http/requires_migration/connections.rs:245-251
- tests/http/requires_migration/fragment.rs:162-168

```rust
);
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Connecti  ... [truncated]
```

### Clone 2 (6L)
- tests/http/requires_migration/connections.rs:24-29
- tests/http/requires_migration/fragment.rs:131-136

```rust
))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 16384)
```

### Clone 3 (6L)
- tests/http/requires_migration/connections.rs:157-162
- tests/http/requires_migration/fragment.rs:198-203

```rust
.body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 1638  ... [truncated]
```


## tests/http/dashboard.rs  <->  tests/http/swipe_new.rs

Clones: 2 | Dup lines: 38

### Clone 1 (5L)
- tests/http/dashboard.rs:67-71
- tests/http/swipe_new.rs:1008-1013

```rust
.model(Arc::new(ScriptedModel::new().failed(ModelRole::Narration)))
        .build_with_state();
    assert!(post_action(&app, "look").await.status().is_success());
    assert!(
        wait_settled(&  ... [truncated]
```

### Clone 2 (5L)
- tests/http/dashboard.rs:88-92
- tests/http/swipe_new.rs:731-736

```rust
.model(Arc::new(ScriptedModel::new().turn("You look around.")))
        .build_with_state();
    assert!(post_action(&app, "look around").await.status().is_success());
    assert!(
        wait_settle  ... [truncated]
```


## tests/http/games_switch.rs  <->  tests/http/requires_migration/worlds_fragment_handlers.rs

Clones: 1 | Dup lines: 36

### Clone 1 (9L)
- tests/http/games_switch.rs:59-67
- tests/http/requires_migration/worlds_fragment_handlers.rs:164-172

```rust
let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unw  ... [truncated]
```


## tests/http/prompt_presets.rs  <->  tests/http/requires_migration/connections.rs

Clones: 1 | Dup lines: 35

### Clone 1 (7L)
- tests/http/prompt_presets.rs:33-39
- tests/http/requires_migration/connections.rs:16-22

```rust
.uri(uri)
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from(body.to_strin  ... [truncated]
```


## src/adapters/driven/storage/characters_tests.rs  <->  src/adapters/driven/storage/worlds_tests.rs

Clones: 2 | Dup lines: 32

### Clone 1 (27L)
- src/adapters/driven/storage/characters_tests.rs:133-159
- src/adapters/driven/storage/worlds_tests.rs:219-245

```rust
name: "Test World".to_string(),
        description: "Test description".to_string(),
        ..Default::default()
    };

    let map = MapDef {
        overworld: Overworld {
            id: "overwor  ... [truncated]
```

### Clone 2 (5L)
- src/adapters/driven/storage/characters_tests.rs:119-123
- src/adapters/driven/storage/worlds_tests.rs:157-161

```rust
},
        inventory: Vec::new(),
        triggers: Vec::new(),
        relationships: Vec::new(),
    }
```


## tests/http/actions.rs  <->  tests/http/llm_messages.rs

Clones: 3 | Dup lines: 30

### Clone 1 (9L)
- tests/http/actions.rs:31-39
- tests/http/llm_messages.rs:11-19

```rust
));

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await, "action should complete");

    let messages = state
         ... [truncated]
```

### Clone 2 (8L)
- tests/http/actions.rs:243-250
- tests/http/llm_messages.rs:76-83

```rust
let resp = post_action(&app, "talk to shopkeeper").await;
    assert!(resp.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
        "first action should complete"
    );
  ... [truncated]
```

### Clone 3 (8L)
- tests/http/actions.rs:250-257
- tests/http/llm_messages.rs:83-90

```rust
let resp = post_action(&app, "talk to shopkeeper").await;
    assert!(resp.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
        "second action should complete"
    );  ... [truncated]
```


## src/application/agents/options/agent.rs  <->  src/application/agents/quantifier/agent.rs

Clones: 3 | Dup lines: 29

### Clone 1 (16L)
- src/application/agents/options/agent.rs:34-49
- src/application/agents/quantifier/agent.rs:37-52

```rust
name: "options".to_string(),
            recorder,
        })
    }

    #[cfg(feature = "testing")]
    pub fn with_provider(name: String, provider: Arc<dyn LlmProvider>) -> Self {
        use crate:  ... [truncated]
```

### Clone 2 (7L)
- src/application/agents/options/agent.rs:28-34
- src/application/agents/quantifier/agent.rs:31-37

```rust
impl OptionsGeneratorAgent {
    pub fn from_config(
        _config: &AgentConfig,
        recorder: Arc<LlmCallRecorder>,
    ) -> Result<Self, EngineError> {
        Ok(Self {
            name: "op  ... [truncated]
```

### Clone 3 (6L)
- src/application/agents/options/agent.rs:49-54
- src/application/agents/quantifier/agent.rs:52-60

```rust
impl OptionsAgent for OptionsGeneratorAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn run(&self, input: &HookInput, settings: &AgentSettings) -> Result<OfferedSet, EngineError> {
```


## tests/http/dashboard.rs  <->  tests/http/story_log.rs

Clones: 2 | Dup lines: 29

### Clone 1 (15L)
- tests/http/dashboard.rs:43-57
- tests/http/story_log.rs:267-283

```rust
.narration("You look around.")
            .analysis(analysis_report(&[], None)),
    );
    let hold = model.hold(ModelRole::Analysis);
    let (app, state) = TestAppBuilder::default_test()
        .  ... [truncated]
```

### Clone 2 (5L)
- tests/http/dashboard.rs:88-92
- tests/http/story_log.rs:232-236

```rust
.model(Arc::new(ScriptedModel::new().turn("You look around.")))
        .build_with_state();
    assert!(post_action(&app, "look around").await.status().is_success());
    assert!(
        wait_settle  ... [truncated]
```


## tests/http/reset.rs  <->  tests/http/story_log.rs

Clones: 3 | Dup lines: 29

### Clone 1 (8L)
- tests/http/reset.rs:63-70
- tests/http/story_log.rs:86-92

```rust
.turn("You look around."),
        ))
        .build_with_state();

    let _ = post_action(&app, "examine room").await;
    assert!(wait_settled(&state, 1000).await);

    let resp = post_empty(&app,  ... [truncated]
```

### Clone 2 (6L)
- tests/http/reset.rs:15-20
- tests/http/story_log.rs:162-167

```rust
let (app, state) = TestAppBuilder::default_test()
        .model(Arc::new(ScriptedModel::new().turn("You examine the room.")))
        .build_with_state();

    let _ = post_action(&app, "examine room  ... [truncated]
```

### Clone 3 (5L)
- tests/http/reset.rs:59-63
- tests/http/story_log.rs:24-28

```rust
let (app, state) = TestAppBuilder::default_test()
        .model(Arc::new(
            ScriptedModel::new()
                .turn("You examine the room.")
                .turn("You look around."),
```


