//! Stub-browser server: the real dashboard shell plus canned fragments, with no engine behind it.

// Fragments the tests assert on render through the engine's own templates, so a
// hook change in a shipped template reaches the served fragment with no fixture
// to keep in step.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Form, State};
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use chrono::{TimeZone, Utc};

use chronicler_engine::adapters::driving::http::utils::response::BUSY_STATUS;
use chronicler_engine::adapters::driving::http::builders::headers::{
    failure_banner_html, header_fragment_html,
};
use chronicler_engine::adapters::driving::http::builders::presets::{
    preset_edit_form_html, PresetCardTemplate,
};
use chronicler_engine::adapters::driving::http::prompt_presets::templates::PromptPresetsTemplate;
use chronicler_engine::adapters::driving::http::worlds::templates::worlds::{
    WorldFormTemplate, WorldsPanelTemplate,
};
use chronicler_engine::adapters::driving::http::games::templates::games::{
    GamePostureTemplate, GameRowView, GamesPanelTemplate, PersonaRowView,
};
use chronicler_engine::adapters::driving::http::settings::templates::{
    ConnectionFormTemplate, SettingsTemplate,
};
use chronicler_engine::adapters::driving::http::layout::handlers::{
    busy_answer, failure_answer, idle_answer,
};
use chronicler_engine::adapters::driving::http::utils::error::render_error;
use chronicler_engine::application::generation::gate::{AutoplayProgress, Busy};
use chronicler_engine::application::games::view_query::RoleHealth;
use chronicler_engine::domain::model::agent::Role;
use chronicler_engine::domain::model::game::Game;
use chronicler_engine::domain::model::llm_message::LlmMessage;
use chronicler_engine::domain::model::prompt_preset::{PresetType, PromptPreset};
use chronicler_engine::domain::model::settings::{AppSettings, LlmConnection, NarratorMode};
use chronicler_engine::domain::model::world::WorldCard;
use chronicler_engine::domain::model::state::generation_status::{
    GenerationFailure, GenerationFailureKind, Step,
};
use chronicler_engine::domain::model::state::message_types::{StoryRow, MessageType};
use chronicler_engine::test_support::{default_story_row, TestPromptPreset};

use super::server::{get_config_port, release_port_lock};
use super::CONFIG_PATH;

const DASHBOARD_SHELL: &str = include_str!("../../assets/index.html");

const FIXTURE_VISUAL_SIDEBAR: &str = include_str!("stub_fixtures/visual_sidebar.html");

/// The one role health the stub knows: a degraded Quantifier, or none.
fn stub_roles(degraded: bool) -> Vec<RoleHealth> {
    if degraded {
        vec![RoleHealth {
            role: Role::Quantifier,
            backend_model: Some("Mock mock".to_string()),
            last_error: Some("Characters present kept".to_string()),
        }]
    } else {
        Vec::new()
    }
}

fn header_html() -> String {
    header_fragment_html("Test Realm_2026-09-17_1".to_string()).expect("render header")
}

fn failure_banner_html_for(degraded: bool) -> String {
    failure_banner_html(&stub_roles(degraded))
}

const NARRATIVE_TEXT: &str = "Welcome to the Test World, Test Player! This is a simple scenario for testing the starting scenarios feature. Feel free to explore and test the game engine.\n\nThe tavern around you is warm and inviting. Wooden beams stretch across the ceiling, and a crackling fire in the hearth casts dancing shadows on the walls. The smell of fresh bread and mulled cider fills the air.\n\nBehind the bar, the bartender wipes down a mug and glances your way. \"First time in the Test Realm?\" he asks with a knowing smile. \"Don't worry, everyone here is friendly. Mostly.\"\n\nA merchant in the corner adjusts her pack and catches your eye. \"If you're heading north to the village square, mind the cobblestones. They get slippery after dark,\" she advises.\n\nYou take a moment to gather your bearings. The road ahead promises adventure, but for now, the warmth of the tavern offers a brief respite.";

fn story_log_html(entries: &[StoryRow]) -> String {
    use askama::Template;
    use chronicler_engine::adapters::driving::http::templates::NarrationLogTemplate;
    use chronicler_engine::test_support::fixtures::window_entries;

    let mut window = window_entries(entries);
    if let Some(opening) = window.iter_mut().find(|entry| entry.row.id == 1) {
        opening.room_header = Some("Test Tavern".to_string());
    }
    NarrationLogTemplate::new(&window, true, false)
        .render()
        .expect("render story log")
}

/// The template renders swipe/retrigger controls only on the trailing entry, so
/// the order of this vec is load-bearing.
fn default_story_log_entries() -> Vec<StoryRow> {
    vec![
        StoryRow {
            id: 2,
            text: "look at the casle".to_string(),
            message_type: MessageType::Input,
            timestamp: Utc.with_ymd_and_hms(2026, 1, 1, 19, 7, 0).unwrap(),
            ..default_story_row(MessageType::Input)
        },
        StoryRow {
            id: 1,
            text: NARRATIVE_TEXT.to_string(),
            message_type: MessageType::Narration,
            timestamp: Utc.with_ymd_and_hms(2026, 1, 1, 19, 8, 0).unwrap(),
            ..default_story_row(MessageType::Narration)
        },
    ]
}

/// The LLM messages the stub shows, newest first: each carries prompts and raw
/// JSON, so the card holds every kind of disclosure the real panel renders.
fn llm_messages_html(count: usize) -> String {
    use askama::Template;
    use chronicler_engine::adapters::driving::http::templates::LlmMessagesTemplate;

    let messages: Vec<LlmMessage> = (1..=count as u64)
        .rev()
        .map(|id| LlmMessage {
            id: id as i64,
            agent_name: "narrator".to_string(),
            backend_name: "Mock".to_string(),
            model_name: "mock".to_string(),
            system_prompt: "the system prompt".to_string(),
            user_prompt: "the user prompt".to_string(),
            raw_request_json: "{\"request\":true}".to_string(),
            raw_response_json: "{\"response\":true}".to_string(),
            parsed_response: "canned narration".to_string(),
            error_message: None,
            created_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, id as u32).unwrap(),
        })
        .collect();
    LlmMessagesTemplate::new(&messages)
        .render()
        .expect("render LLM messages")
}

/// The Games panel, rendered by the engine's own template over the games the
/// stub holds: the active Game, and the saved Games that no delete removed.
fn games_html(games: &StubGames) -> String {
    use askama::Template;

    let preset = |id: &str| TestPromptPreset::system(id, "Default");
    let game = Game {
        id: 1,
        persona_name: "Test Player".to_string(),
        narrator_mode: NarratorMode::Novel,
        active_system_prompt_preset_id: "system_default".to_string(),
        active_quantifier_prompt_preset_id: "quantifier_default".to_string(),
        active_impersonate_prompt_preset_id: "impersonate_default".to_string(),
        active_options_prompt_preset_id: "options_default".to_string(),
        ..Game::default()
    };
    let posture = GamePostureTemplate::from_game(
        &game,
        &[preset("system_default")],
        &[preset("quantifier_default")],
        &[preset("impersonate_default")],
        &[TestPromptPreset::options("options_default", "Default")],
        false,
    )
    .render()
    .expect("render the posture controls");
    let row = |id: u64, display_name: &str| GameRowView {
        id,
        display_name: display_name.to_string(),
        world_name: "Test Realm".to_string(),
        persona_name: "Test Player".to_string(),
        run_held: false,
    };
    let world = |key: &str, name: &str, description: &str| WorldCard {
        key: key.to_string(),
        name: name.to_string(),
        description: description.to_string(),
        ..WorldCard::default()
    };
    GamesPanelTemplate {
        active_game: Some(row(1, &games.active_name)),
        saved_games: games
            .saved
            .iter()
            .map(|(id, name)| row(*id, name))
            .collect(),
        worlds: vec![
            world("redmist_estate", "Redmist Estate", "A coastal estate."),
            world("test", "Test Realm", "A small testing kingdom for UI tests"),
        ],
        personas: vec![
            PersonaRowView {
                key: "julian".to_string(),
                name: "Julian".to_string(),
            },
            PersonaRowView {
                key: "test_player".to_string(),
                name: "Test Player".to_string(),
            },
        ],
        posture_html: posture,
    }
    .render()
    .expect("render the Games panel")
}

fn settings_html(degraded: bool) -> String {
    use askama::Template;

    SettingsTemplate::from_settings(&AppSettings::default(), &stub_roles(degraded))
        .render()
        .expect("render settings")
}

fn connection_form_html(connection: Option<&LlmConnection>) -> String {
    use askama::Template;

    ConnectionFormTemplate::new(connection)
        .render()
        .expect("render connection form")
}

fn custom_preset(id: &str, name: &str) -> PromptPreset {
    PromptPreset {
        id: id.to_string(),
        name: name.to_string(),
        role: None,
        instructions: Some(format!("{name}, a custom system prompt for the stub.")),
        writing_style: None,
        output_format: None,
        allowed_modes: vec![NarratorMode::Novel],
        is_default: false,
        preset_type: PresetType::System,
    }
}

fn default_preset(
    id: &str,
    name: &str,
    preset_type: PresetType,
    mode: NarratorMode,
) -> PromptPreset {
    PromptPreset {
        id: id.to_string(),
        name: name.to_string(),
        role: None,
        instructions: Some(format!("The default {name} prompt of the stub.")),
        writing_style: None,
        output_format: None,
        allowed_modes: vec![mode],
        is_default: true,
        preset_type,
    }
}

/// The presets the stub lists: the defaults, and the custom System presets that
/// a delete removes, a duplicate adds and an Activate marks.
struct StubPresets {
    custom: Vec<PromptPreset>,
    active_system: String,
    copies: u32,
}

impl StubPresets {
    fn new() -> Self {
        Self {
            custom: vec![
                custom_preset("custom_ref", "My Custom Prompt"),
                custom_preset("custom_two", "Second Custom Prompt"),
            ],
            active_system: "system_default".to_string(),
            copies: 0,
        }
    }

    fn settings(&self) -> AppSettings {
        let mut settings = AppSettings::default();
        settings.set_active_preset(
            PresetType::System,
            NarratorMode::Novel,
            self.active_system.clone(),
        );
        settings
    }

    fn system_presets(&self) -> Vec<PromptPreset> {
        let mut presets = vec![
            default_preset(
                "system_default",
                "Default",
                PresetType::System,
                NarratorMode::Novel,
            ),
            default_preset(
                "system_if_default",
                "Interactive Fiction",
                PresetType::System,
                NarratorMode::InteractiveFiction,
            ),
        ];
        presets.extend(self.custom.iter().cloned());
        presets
    }

    fn find(&self, id: &str) -> Option<PromptPreset> {
        self.system_presets()
            .into_iter()
            .find(|preset| preset.id == id)
    }
}

fn presets_html(presets: &StubPresets) -> String {
    use askama::Template;

    PromptPresetsTemplate {
        system_presets: presets.system_presets(),
        quantifier_presets: vec![default_preset(
            "quantifier_default",
            "Default",
            PresetType::Quantifier,
            NarratorMode::Novel,
        )],
        impersonate_presets: vec![default_preset(
            "impersonate_default",
            "Default",
            PresetType::Impersonate,
            NarratorMode::Novel,
        )],
        options_presets: Vec::new(),
        settings: presets.settings(),
    }
    .render()
    .expect("render the Prompt Presets panel")
}

/// The Worlds the stub lists: a delete removes one and a save renames one.
struct StubWorlds {
    worlds: Vec<WorldCard>,
    posture_failing: bool,
}

impl StubWorlds {
    fn new() -> Self {
        Self {
            worlds: vec![
                WorldCard {
                    key: "redmist_estate".to_string(),
                    name: "Redmist Estate".to_string(),
                    description: "The Redmist Estate stands in Islaport, a large coastal city on the eastern seaboard.".to_string(),
                    ..Default::default()
                },
                WorldCard {
                    key: "test".to_string(),
                    name: "Test Realm".to_string(),
                    description: "A small testing kingdom for UI tests".to_string(),
                    ..Default::default()
                },
            ],
            posture_failing: false,
        }
    }
}

fn worlds_html(worlds: &StubWorlds) -> String {
    use askama::Template;

    let games_per_world = HashMap::from([("test".to_string(), 1usize)]);
    WorldsPanelTemplate::from_worlds(&worlds.worlds, &games_per_world)
        .render()
        .expect("render the Worlds panel")
}

fn world_form_html(world: Option<&WorldCard>) -> String {
    use askama::Template;
    use chronicler_engine::domain::model::map::MapDef;

    WorldFormTemplate::from_world_data(world, world.map(|_| MapDef::default()).as_ref(), &[])
        .render()
        .expect("render the World form")
}

const CANNED_OPTIONS: [&str; 3] = [
    "Ask the bartender about the Test Realm",
    "Examine the merchant's pack",
    "Step outside into the night air",
];

const CANNED_OPTIONS_ALT: [&str; 2] = ["Search the cellar", "Call for the innkeeper"];

fn options_dock_html(alternate: bool, failed: bool) -> String {
    use askama::Template;
    use chronicler_engine::adapters::driving::http::templates::OptionsDockTemplate;
    use chronicler_engine::adapters::driving::http::view_models::OptionsDockViewModel;
    use chronicler_engine::domain::model::dock::OptionsFailure;

    let options: Vec<String> = if alternate {
        CANNED_OPTIONS_ALT.iter().map(|o| o.to_string()).collect()
    } else {
        CANNED_OPTIONS.iter().map(|o| o.to_string()).collect()
    };
    let failure = failed.then(|| OptionsFailure {
        reason: "The model gave no usable Options.".to_string(),
        raw: "stub raw Options answer".to_string(),
    });
    let vm = OptionsDockViewModel::new(options, false, failure);
    OptionsDockTemplate::new(vm)
        .render()
        .expect("render options dock")
}

const TEXT_CHECK_TRIGGER: &str = "casle";

fn text_check_preview_html() -> String {
    use askama::Template;
    use chronicler_engine::adapters::driving::http::templates::TextCheckPreviewTemplate;

    TextCheckPreviewTemplate {
        original: "look at the casle".to_string(),
        corrected: "look at the castle".to_string(),
        issues: vec![],
    }
    .render()
    .expect("render text check preview")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StubActionOutcome {
    /// "Thinking..." acknowledgement — the shapes a test uses to observe a
    /// pending generation.
    #[default]
    Pending,
    /// A 500, as a failing engine action would produce. The body is the
    /// engine's own error render naming the command, so the disclosure's raw
    /// text is the server's response rather than the input echoed back.
    Error,
    /// A 200 that refuses the command: the engine answers the wait status
    /// because a generation is already in flight.
    Refused,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum StubStatus {
    /// The engine is idle; the status display answers Ready.
    #[default]
    Idle,
    /// A Job runs at this step; the status display answers the label of the step.
    Step(Step),
    /// An Autoplay run holds the Game: the cycle, the budget and the step of the
    /// run's Job, or none between two Jobs.
    Autoplay {
        done: u32,
        budget: u32,
        step: Option<Step>,
    },
    /// A failed generation; the status display answers the error fragment,
    /// clamped to the sentence the failure kind names, over this raw text.
    Error(GenerationFailureKind, String),
}

struct StubState {
    action_outcome: StubActionOutcome,
    status: std::sync::Mutex<StubStatus>,
    retrigger_requests: std::sync::atomic::AtomicUsize,
    action_requests: std::sync::atomic::AtomicUsize,
    two_swipes: std::sync::atomic::AtomicBool,
    switched: std::sync::atomic::AtomicBool,
    switch_fails: std::sync::atomic::AtomicBool,
    panel_loads_failing: std::sync::atomic::AtomicBool,
    dock_options_live: std::sync::atomic::AtomicBool,
    dock_options_alt: std::sync::atomic::AtomicBool,
    newest_narration: std::sync::Mutex<Option<String>>,
    extra_oldest: std::sync::atomic::AtomicBool,
    save_succeeds: std::sync::atomic::AtomicBool,
    story_writes_refused_as_busy: std::sync::atomic::AtomicBool,
    delete_succeeds: std::sync::atomic::AtomicBool,
    deleted: std::sync::atomic::AtomicBool,
    header_degraded: std::sync::atomic::AtomicBool,
    polls_failing: std::sync::atomic::AtomicBool,
    row_deletes_succeed: std::sync::atomic::AtomicBool,
    world_delete_refused: std::sync::atomic::AtomicBool,
    preset_card_refused: std::sync::atomic::AtomicBool,
    changes: std::sync::Mutex<StubChanges>,
    change_asks: std::sync::atomic::AtomicUsize,
    fragment_loads: std::sync::Mutex<HashMap<String, usize>>,
    loads_failing: std::sync::atomic::AtomicBool,
    story_log_delay_ms: std::sync::atomic::AtomicU64,
    options_failed: std::sync::atomic::AtomicBool,
    games: std::sync::Mutex<StubGames>,
    presets: std::sync::Mutex<StubPresets>,
    worlds: std::sync::Mutex<StubWorlds>,
    llm_message_count: std::sync::atomic::AtomicUsize,
}

/// The Games the stub lists: a rename changes a name and a delete removes a saved
/// Game, and each moves the `games` number, as the engine does.
struct StubGames {
    active_name: String,
    saved: Vec<(u64, String)>,
    rename_failing: bool,
}

impl StubGames {
    fn new() -> Self {
        Self {
            active_name: "Test Realm_2026-09-17_1".to_string(),
            saved: vec![
                (2, "Test Realm — 17 Sep 2026 (2)".to_string()),
                (3, "Test Realm — 17 Sep 2026 (3)".to_string()),
            ],
            rename_failing: false,
        }
    }
}

/// What `GET /changes` answers. A test moves a number through `StubChangesHandle`; the
/// setters of the other handles move the number of the data they change, so a
/// test that changes stub data still reaches the region without naming a number.
#[derive(Debug, Clone)]
struct StubChanges {
    server: String,
    game: Option<u64>,
    story: u64,
    options: u64,
    busy: u64,
    games: u64,
    worlds: u64,
    settings: u64,
    presets: u64,
    llm: u64,
}

impl StubChanges {
    fn new() -> Self {
        Self {
            server: "stub-server-1".to_string(),
            game: Some(1),
            story: 1,
            options: 1,
            busy: 1,
            games: 1,
            worlds: 1,
            settings: 1,
            presets: 1,
            llm: 1,
        }
    }

    fn number_mut(&mut self, kind: &str) -> &mut u64 {
        match kind {
            "story" => &mut self.story,
            "options" => &mut self.options,
            "busy" => &mut self.busy,
            "games" => &mut self.games,
            "worlds" => &mut self.worlds,
            "settings" => &mut self.settings,
            "presets" => &mut self.presets,
            "llm" => &mut self.llm,
            other => panic!("the stub has no change number named {other:?}"),
        }
    }

    fn answer(&self) -> serde_json::Value {
        let mut answer = serde_json::json!({
            "server": self.server,
            "game": self.game,
            "games": self.games,
            "worlds": self.worlds,
            "settings": self.settings,
            "presets": self.presets,
            "llm": self.llm,
        });
        if self.game.is_some() {
            answer["story"] = self.story.into();
            answer["options"] = self.options.into();
            answer["busy"] = self.busy.into();
        }
        answer
    }
}

impl StubState {
    fn bump(&self, kinds: &[&str]) {
        let mut changes = self.changes.lock().expect("stub changes lock poisoned");
        for kind in kinds {
            *changes.number_mut(kind) += 1;
        }
    }
}

pub struct StubServer {
    addr: SocketAddr,
    shutdown: Arc<std::sync::Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
    state: Arc<StubState>,
}

impl StubServer {
    /// The port comes from `tests/test_config.json` so the stub cannot silently
    /// collide with real engine servers if the range moves.
    pub async fn start(action_outcome: StubActionOutcome) -> Self {
        let port = get_config_port(CONFIG_PATH).expect("allocate a stub port");
        Self::start_on_port(port, action_outcome).await
    }

    async fn start_on_port(port: u16, action_outcome: StubActionOutcome) -> Self {
        let state = Arc::new(StubState {
            action_outcome,
            status: std::sync::Mutex::new(StubStatus::default()),
            retrigger_requests: std::sync::atomic::AtomicUsize::new(0),
            action_requests: std::sync::atomic::AtomicUsize::new(0),
            two_swipes: std::sync::atomic::AtomicBool::new(false),
            switched: std::sync::atomic::AtomicBool::new(false),
            switch_fails: std::sync::atomic::AtomicBool::new(false),
            panel_loads_failing: std::sync::atomic::AtomicBool::new(false),
            dock_options_live: std::sync::atomic::AtomicBool::new(true),
            dock_options_alt: std::sync::atomic::AtomicBool::new(false),
            newest_narration: std::sync::Mutex::new(None),
            extra_oldest: std::sync::atomic::AtomicBool::new(false),
            save_succeeds: std::sync::atomic::AtomicBool::new(false),
            story_writes_refused_as_busy: std::sync::atomic::AtomicBool::new(false),
            delete_succeeds: std::sync::atomic::AtomicBool::new(false),
            deleted: std::sync::atomic::AtomicBool::new(false),
            header_degraded: std::sync::atomic::AtomicBool::new(false),
            polls_failing: std::sync::atomic::AtomicBool::new(false),
            row_deletes_succeed: std::sync::atomic::AtomicBool::new(false),
            world_delete_refused: std::sync::atomic::AtomicBool::new(false),
            preset_card_refused: std::sync::atomic::AtomicBool::new(false),
            changes: std::sync::Mutex::new(StubChanges::new()),
            change_asks: std::sync::atomic::AtomicUsize::new(0),
            fragment_loads: std::sync::Mutex::new(HashMap::new()),
            loads_failing: std::sync::atomic::AtomicBool::new(false),
            story_log_delay_ms: std::sync::atomic::AtomicU64::new(0),
            options_failed: std::sync::atomic::AtomicBool::new(false),
            games: std::sync::Mutex::new(StubGames::new()),
            presets: std::sync::Mutex::new(StubPresets::new()),
            worlds: std::sync::Mutex::new(StubWorlds::new()),
            llm_message_count: std::sync::atomic::AtomicUsize::new(1),
        });
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
            .await
            .unwrap_or_else(|e| panic!("bind stub on port {port}: {e}"));
        let addr = listener.local_addr().expect("stub local addr");
        let tx = serve_stub(listener, Arc::clone(&state));
        Self {
            addr,
            shutdown: Arc::new(std::sync::Mutex::new(Some(tx))),
            state,
        }
    }

    pub fn stop(&self) {
        self.lifecycle_handle().stop();
    }

    pub fn lifecycle_handle(&self) -> StubLifecycleHandle {
        StubLifecycleHandle {
            addr: self.addr,
            shutdown: Arc::clone(&self.shutdown),
            state: Arc::clone(&self.state),
        }
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn url(&self) -> String {
        format!("http://{}", self.addr)
    }

    pub fn status_handle(&self) -> StubStatusHandle {
        StubStatusHandle {
            state: Arc::clone(&self.state),
        }
    }

    pub fn retrigger_handle(&self) -> StubRetriggerHandle {
        StubRetriggerHandle {
            state: Arc::clone(&self.state),
        }
    }

    pub fn action_handle(&self) -> StubActionHandle {
        StubActionHandle {
            state: Arc::clone(&self.state),
        }
    }

    pub fn swipe_handle(&self) -> StubSwipeHandle {
        StubSwipeHandle {
            state: Arc::clone(&self.state),
        }
    }

    pub fn story_log_handle(&self) -> StubStoryLogHandle {
        StubStoryLogHandle {
            state: Arc::clone(&self.state),
        }
    }

    pub fn options_handle(&self) -> StubOptionsHandle {
        StubOptionsHandle {
            state: Arc::clone(&self.state),
        }
    }

    pub fn save_handle(&self) -> StubSaveHandle {
        StubSaveHandle {
            state: Arc::clone(&self.state),
        }
    }

    pub fn failure_handle(&self) -> StubFailureHandle {
        StubFailureHandle {
            state: Arc::clone(&self.state),
        }
    }

    pub fn changes_handle(&self) -> StubChangesHandle {
        StubChangesHandle {
            state: Arc::clone(&self.state),
        }
    }

    pub fn games_handle(&self) -> StubGamesHandle {
        StubGamesHandle {
            state: Arc::clone(&self.state),
        }
    }

    pub fn llm_handle(&self) -> StubLlmHandle {
        StubLlmHandle {
            state: Arc::clone(&self.state),
        }
    }

    pub fn row_delete_handle(&self) -> StubRowDeleteHandle {
        StubRowDeleteHandle {
            state: Arc::clone(&self.state),
        }
    }
}

#[derive(Clone)]
pub struct StubLifecycleHandle {
    addr: SocketAddr,
    shutdown: Arc<std::sync::Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
    state: Arc<StubState>,
}

impl StubLifecycleHandle {
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn stop(&self) {
        if let Some(tx) = self
            .shutdown
            .lock()
            .expect("stub shutdown lock poisoned")
            .take()
        {
            let _ = tx.send(());
        }
    }

    pub async fn restart(&self) {
        let listener = tokio::net::TcpListener::bind(self.addr)
            .await
            .unwrap_or_else(|e| panic!("rebind stub on {}: {e}", self.addr));
        let tx = serve_stub(listener, Arc::clone(&self.state));
        *self.shutdown.lock().expect("stub shutdown lock poisoned") = Some(tx);
    }
}

/// The numbers of `GET /changes`, and what the page asked and loaded.
#[derive(Clone)]
pub struct StubChangesHandle {
    state: Arc<StubState>,
}

impl StubChangesHandle {
    /// Moves the number of one kind: `story`, `options`, `busy`, `games`,
    /// `worlds`, `settings`, `presets` or `llm`.
    pub fn bump(&self, kind: &str) {
        self.state.bump(&[kind]);
    }

    pub fn set_server(&self, server: &str) {
        self.state
            .changes
            .lock()
            .expect("stub changes lock poisoned")
            .server = server.to_string();
    }

    pub fn set_game(&self, game: Option<u64>) {
        self.state
            .changes
            .lock()
            .expect("stub changes lock poisoned")
            .game = game;
    }

    /// The `GET /changes` requests the stub answered or failed.
    pub fn asks(&self) -> usize {
        self.state
            .change_asks
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    /// The requests the stub received for a fragment path or `/status/busy`,
    /// failed ones included.
    pub fn loads(&self, path: &str) -> usize {
        self.state
            .fragment_loads
            .lock()
            .expect("stub load counts lock poisoned")
            .get(path)
            .copied()
            .unwrap_or(0)
    }

    /// Fails every fragment and `/status/busy` request with a 500 and leaves
    /// `GET /changes` working, so a region load fails after a good ask.
    pub fn set_loads_failing(&self, failing: bool) {
        self.state
            .loads_failing
            .store(failing, std::sync::atomic::Ordering::SeqCst);
    }

    /// Holds the story-log answer back, so a load is in flight for a while.
    pub fn set_story_log_delay_ms(&self, millis: u64) {
        self.state
            .story_log_delay_ms
            .store(millis, std::sync::atomic::Ordering::SeqCst);
    }
}

/// The Games the stub lists.
#[derive(Clone)]
pub struct StubGamesHandle {
    state: Arc<StubState>,
}

impl StubGamesHandle {
    /// A rename answers a 500 until this is turned off.
    pub fn set_rename_failing(&self, failing: bool) {
        self.state
            .games
            .lock()
            .expect("stub games lock poisoned")
            .rename_failing = failing;
    }

    /// The names of the saved Games the panel lists now, in order.
    pub fn saved_names(&self) -> Vec<String> {
        self.state
            .games
            .lock()
            .expect("stub games lock poisoned")
            .saved
            .iter()
            .map(|(_, name)| name.clone())
            .collect()
    }
}

/// The LLM messages the stub lists.
#[derive(Clone)]
pub struct StubLlmHandle {
    state: Arc<StubState>,
}

impl StubLlmHandle {
    /// A new LLM message is recorded: the next load of the panel lists it first.
    pub fn record_message(&self) {
        self.state
            .llm_message_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.state.bump(&["llm"]);
    }
}

#[derive(Clone)]
pub struct StubStoryLogHandle {
    state: Arc<StubState>,
}

impl StubStoryLogHandle {
    pub fn set_extra_oldest(&self, present: bool) {
        self.state
            .extra_oldest
            .store(present, std::sync::atomic::Ordering::SeqCst);
        self.state.bump(&["story"]);
    }

    pub fn append_narration(&self, text: &str) {
        *self
            .state
            .newest_narration
            .lock()
            .expect("stub narration lock poisoned") = Some(text.to_string());
        self.state.bump(&["story", "games"]);
    }

    /// The newest entry is gone from the next story-log answer.
    pub fn remove_newest(&self) {
        self.state
            .deleted
            .store(true, std::sync::atomic::Ordering::SeqCst);
        self.state.bump(&["story"]);
    }
}

#[derive(Clone)]
pub struct StubOptionsHandle {
    state: Arc<StubState>,
}

impl StubOptionsHandle {
    pub fn serve_alternate_set(&self, alternate: bool) {
        self.state
            .dock_options_alt
            .store(alternate, std::sync::atomic::Ordering::SeqCst);
        self.state.bump(&["options"]);
    }

    /// The dock shows an Options failure with its Details disclosure instead of
    /// the offered set.
    pub fn set_failure(&self, failed: bool) {
        self.state
            .options_failed
            .store(failed, std::sync::atomic::Ordering::SeqCst);
        self.state.bump(&["options"]);
    }
}

#[derive(Clone)]
pub struct StubSaveHandle {
    state: Arc<StubState>,
}

impl StubSaveHandle {
    pub fn set_save_succeeds(&self, succeeds: bool) {
        self.state
            .save_succeeds
            .store(succeeds, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn set_story_writes_refused_as_busy(&self, refused: bool) {
        self.state
            .story_writes_refused_as_busy
            .store(refused, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn set_delete_succeeds(&self, succeeds: bool) {
        self.state
            .delete_succeeds
            .store(succeeds, std::sync::atomic::Ordering::SeqCst);
    }
}

#[derive(Clone)]
pub struct StubRowDeleteHandle {
    state: Arc<StubState>,
}

impl StubRowDeleteHandle {
    /// A preset delete refuses and a world delete fails until armed, so the
    /// tests that drive those paths keep their outcome.
    pub fn set_row_deletes_succeed(&self, succeeds: bool) {
        self.state
            .row_deletes_succeed
            .store(succeeds, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn set_world_delete_refused(&self, refused: bool) {
        self.state
            .world_delete_refused
            .store(refused, std::sync::atomic::Ordering::SeqCst);
    }
}

#[derive(Clone)]
pub struct StubFailureHandle {
    state: Arc<StubState>,
}

impl StubFailureHandle {
    pub fn set_header_degraded(&self, degraded: bool) {
        self.state
            .header_degraded
            .store(degraded, std::sync::atomic::Ordering::SeqCst);
        self.state.bump(&["llm"]);
    }

    pub fn set_polls_failing(&self, failing: bool) {
        self.state
            .polls_failing
            .store(failing, std::sync::atomic::Ordering::SeqCst);
    }

    /// The Worlds and Games panels load once per page load, the Games panel when
    /// its section opens, so arming this has no effect on a panel that has loaded.
    pub fn set_panel_loads_failing(&self, failing: bool) {
        self.state
            .panel_loads_failing
            .store(failing, std::sync::atomic::Ordering::SeqCst);
    }

    /// A World posture auto-save answers a 400 refusal until this is turned off.
    pub fn set_world_posture_failing(&self, failing: bool) {
        self.state
            .worlds
            .lock()
            .expect("stub worlds lock poisoned")
            .posture_failing = failing;
    }

    pub fn set_preset_card_refused(&self, refused: bool) {
        self.state
            .preset_card_refused
            .store(refused, std::sync::atomic::Ordering::SeqCst);
    }
}

#[derive(Clone)]
pub struct StubSwipeHandle {
    state: Arc<StubState>,
}

impl StubSwipeHandle {
    /// The log's 2s poll renders the new shape on a later cycle.
    pub fn set_two_swipes(&self) {
        self.state
            .two_swipes
            .store(true, std::sync::atomic::Ordering::SeqCst);
        self.state.bump(&["story"]);
    }

    pub fn set_switch_failing(&self) {
        self.state
            .switch_fails
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

#[derive(Clone)]
pub struct StubStatusHandle {
    state: Arc<StubState>,
}

impl StubStatusHandle {
    /// Moves the busy number, so the status display loads on the next ask.
    pub fn set(&self, status: StubStatus) {
        *self.state.status.lock().expect("stub status lock poisoned") = status;
        self.state.bump(&["busy"]);
    }
}

#[derive(Clone)]
pub struct StubRetriggerHandle {
    state: Arc<StubState>,
}

#[derive(Clone)]
pub struct StubActionHandle {
    state: Arc<StubState>,
}

impl StubActionHandle {
    pub fn count(&self) -> usize {
        self.state
            .action_requests
            .load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl StubRetriggerHandle {
    pub fn count(&self) -> usize {
        self.state
            .retrigger_requests
            .load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl Drop for StubServer {
    fn drop(&mut self) {
        let sender = self
            .shutdown
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();
        if let Some(tx) = sender {
            let _ = tx.send(());
        }
        release_port_lock(self.addr.port());
    }
}

pub async fn wait_until_port_closed(addr: SocketAddr) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        if tokio::net::TcpStream::connect(addr).await.is_err() {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    panic!("{addr} still accepts connections 5s after its stub stopped");
}

fn serve_stub(
    listener: tokio::net::TcpListener,
    state: Arc<StubState>,
) -> tokio::sync::oneshot::Sender<()> {
    let app = stub_router(state);
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = rx.await;
            })
            .await;
    });
    tx
}

fn stub_router(state: Arc<StubState>) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/action/check", post(action_check))
        .route("/action/confirm", post(action_confirm))
        .route(
            "/connections/add",
            post(|| async { client_refusal("Unknown LLM backend 'bogus_provider'") }),
        )
        .route(
            "/connections/:id/edit",
            post(|| async { client_refusal("Unknown LLM backend 'bogus_provider'") }),
        )
        // A saved role change answers nothing, the way the engine answers it.
        .route(
            "/connections/set-quantifier",
            post(|| async { (StatusCode::OK, "").into_response() }),
        )
        .route(
            "/prompt-presets",
            post(|| async { client_refusal("Invalid preset type") }),
        )
        .route(
            "/prompt-presets/:id",
            post(|| async { client_failure("preset save failure") }),
        )
        .route("/prompt-presets/:id/delete", post(delete_preset))
        .route("/prompt-presets/:id/activate", post(activate_preset))
        .route("/prompt-presets/:id/duplicate", post(duplicate_preset))
        .route("/worlds", post(save_world))
        .route("/worlds/:key", post(save_world))
        .route("/worlds/:key/posture", post(save_world_posture))
        .route(
            "/settings/text-check",
            post(|| async { (StatusCode::OK, "").into_response() }),
        )
        .route("/games/:id/posture", post(failing_posture_save))
        // The real delete answers an empty body, so htmx removes the row the
        // control sits in.
        .route("/games/:id/delete", post(delete_game))
        .route("/games/:id/rename", post(rename_game))
        .route("/history/:id", post(save_message))
        // Scenario 16.34 drives /reset from the games row: a 500 here fills that
        // row's own error slot rather than the panel's.
        .route(
            "/reset",
            post(|| async { client_failure("Stub reset failure") }),
        )
        .route("/worlds/:key/delete", post(delete_world))
        .route("/history/delete", post(delete_message))
        .route("/swipe/new", post(retry_swipe))
        .route("/autoplay", post(start_autoplay))
        .route("/retrigger", post(record_retrigger))
        // The real switch makes another Swipe active; the stub scripts only the two
        // log shapes and the dock drop a switch test observes.
        .route("/message/:id/swipe/:index", post(switch_swipe))
        .route("/status/busy", get(status_busy))
        .route("/status/reset-busy", post(reset_busy))
        .route("/changes", get(changes_answer))
        .route("/fragment/header", get(header_fragment))
        .route("/fragment/failure-banner", get(failure_banner_fragment))
        .route("/fragment/story-log", get(story_log_fragment))
        .route(
            "/fragment/visual-sidebar",
            get(|| async { Html(FIXTURE_VISUAL_SIDEBAR) }),
        )
        .route("/fragment/options-dock", get(options_dock_fragment))
        .route("/fragment/llm-messages", get(llm_messages_fragment))
        .route("/fragment/settings", get(settings_fragment))
        .route(
            "/fragment/connections/new",
            get(|| async { Html(connection_form_html(None)) }),
        )
        .route(
            "/fragment/connections/:id/edit",
            get(|| async {
                let settings = AppSettings::default();
                Html(connection_form_html(Some(&settings.connections[0])))
            }),
        )
        .route("/fragment/prompt-presets", get(presets_fragment))
        .route("/fragment/prompt-presets/:id", get(preset_card_fragment))
        .route(
            "/fragment/prompt-presets/:id/edit",
            get(preset_edit_form_fragment),
        )
        .route("/fragment/worlds", get(worlds_fragment))
        .route(
            "/fragment/worlds/new",
            get(|| async { Html(world_form_html(None)) }),
        )
        .route("/worlds/:key/edit", get(edit_world_form_fragment))
        .route("/fragment/games", get(games_fragment))
        .nest_service("/assets", tower_http::services::ServeDir::new("assets"))
        .nest_service("/data", tower_http::services::ServeDir::new("data"))
        .layer(axum::middleware::from_fn_with_state(
            Arc::clone(&state),
            fail_polls_middleware,
        ))
        .with_state(state)
}

async fn games_fragment(State(state): State<Arc<StubState>>) -> Html<String> {
    let games = state.games.lock().expect("stub games lock poisoned");
    Html(games_html(&games))
}

async fn llm_messages_fragment(State(state): State<Arc<StubState>>) -> Html<String> {
    let count = state
        .llm_message_count
        .load(std::sync::atomic::Ordering::SeqCst);
    Html(llm_messages_html(count))
}

/// The real delete answers nothing: the next load of the panel no longer lists the
/// row.
async fn delete_game(
    State(state): State<Arc<StubState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
) -> Response<Body> {
    state
        .games
        .lock()
        .expect("stub games lock poisoned")
        .saved
        .retain(|(saved, _)| *saved != id);
    state.bump(&["games"]);
    (StatusCode::OK, "").into_response()
}

#[derive(serde::Deserialize)]
struct RenameForm {
    display_name: String,
}

/// The real rename answers nothing and moves `games`.
async fn rename_game(
    State(state): State<Arc<StubState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    Form(form): Form<RenameForm>,
) -> Response<Body> {
    {
        let mut games = state.games.lock().expect("stub games lock poisoned");
        if games.rename_failing {
            return client_failure("Stub rename failure");
        }
        if id == 1 {
            games.active_name = form.display_name;
        } else if let Some((_, name)) = games.saved.iter_mut().find(|(saved, _)| *saved == id) {
            *name = form.display_name;
        }
    }
    state.bump(&["games"]);
    (StatusCode::OK, "").into_response()
}

async fn story_log_fragment(State(state): State<Arc<StubState>>) -> Html<String> {
    use std::sync::atomic::Ordering::SeqCst;
    let mut entries = if state.two_swipes.load(SeqCst) {
        if state.switched.load(SeqCst) {
            restored_swipe_entries()
        } else {
            two_swipe_entries()
        }
    } else if state.extra_oldest.load(SeqCst) {
        story_log_entries_with_extra_oldest()
    } else {
        default_story_log_entries()
    };
    if let Some(text) = state
        .newest_narration
        .lock()
        .expect("stub narration lock poisoned")
        .clone()
    {
        entries.push(StoryRow {
            id: 4,
            text,
            message_type: MessageType::Narration,
            timestamp: Utc.with_ymd_and_hms(2026, 1, 1, 19, 9, 0).unwrap(),
            ..default_story_row(MessageType::Narration)
        });
    }
    if state.deleted.load(SeqCst) {
        entries.pop();
    }
    Html(story_log_html(&entries))
}

fn story_log_entries_with_extra_oldest() -> Vec<StoryRow> {
    let mut entries = default_story_log_entries();
    entries.insert(
        0,
        StoryRow {
            id: 3,
            text: "An older turn the poll will drop.".to_string(),
            message_type: MessageType::Narration,
            timestamp: Utc.with_ymd_and_hms(2026, 1, 1, 19, 6, 0).unwrap(),
            ..default_story_row(MessageType::Narration)
        },
    );
    entries
}

async fn fail_polls_middleware(
    State(state): State<Arc<StubState>>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response<Body> {
    use std::sync::atomic::Ordering::SeqCst;
    let path = request.uri().path().to_string();
    let is_load = path.starts_with("/fragment/") || path == "/status/busy";
    if is_load {
        *state
            .fragment_loads
            .lock()
            .expect("stub load counts lock poisoned")
            .entry(path.clone())
            .or_insert(0) += 1;
    }
    if path == "/changes" {
        state.change_asks.fetch_add(1, SeqCst);
    }
    let delay_ms = state.story_log_delay_ms.load(SeqCst);
    if path == "/fragment/story-log" && delay_ms > 0 {
        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
    }
    let is_panel_load = path == "/fragment/worlds" || path == "/fragment/games";
    if is_panel_load && state.panel_loads_failing.load(SeqCst) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            render_error("panel load failed"),
        )
            .into_response();
    }
    let is_poll = is_load || path == "/changes";
    let failing = state.polls_failing.load(SeqCst) || (is_load && state.loads_failing.load(SeqCst));
    if is_poll && failing {
        let mut response = (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            render_error("poll failed"),
        )
            .into_response();
        response
            .headers_mut()
            .insert("HX-Reswap", axum::http::HeaderValue::from_static("none"));
        return response;
    }
    next.run(request).await
}

async fn header_fragment() -> Html<String> {
    Html(header_html())
}

async fn failure_banner_fragment(State(state): State<Arc<StubState>>) -> Html<String> {
    use std::sync::atomic::Ordering::SeqCst;
    Html(failure_banner_html_for(state.header_degraded.load(SeqCst)))
}

async fn settings_fragment(State(state): State<Arc<StubState>>) -> Html<String> {
    use std::sync::atomic::Ordering::SeqCst;
    Html(settings_html(state.header_degraded.load(SeqCst)))
}

fn swipe_script_entries(text: &str, active_swipe_index: usize) -> Vec<StoryRow> {
    vec![StoryRow {
        id: 1,
        text: text.to_string(),
        message_type: MessageType::Narration,
        timestamp: Utc.with_ymd_and_hms(2026, 1, 1, 19, 7, 0).unwrap(),
        swipe_count: 2,
        active_swipe_index,
        ..default_story_row(MessageType::Narration)
    }]
}

fn two_swipe_entries() -> Vec<StoryRow> {
    swipe_script_entries("A tavern by night.", 1)
}

fn restored_swipe_entries() -> Vec<StoryRow> {
    swipe_script_entries("A road at dawn.", 0)
}

/// Mirrors the engine's settled-restore outcome: the restored log, no dock options.
async fn switch_swipe(State(state): State<Arc<StubState>>) -> Response<Body> {
    use std::sync::atomic::Ordering::SeqCst;
    if state.story_writes_refused_as_busy.load(SeqCst) {
        return (StatusCode::OK, Html(BUSY_STATUS)).into_response();
    }
    if state.switch_fails.load(SeqCst) {
        return client_failure("Stub swipe switch failure");
    }
    state.switched.store(true, SeqCst);
    state.dock_options_live.store(false, SeqCst);
    state.bump(&["story", "options"]);
    Html(story_log_html(&restored_swipe_entries())).into_response()
}

/// The real Stop answers a plain word and moves the busy state.
async fn reset_busy(State(state): State<Arc<StubState>>) -> Response<Body> {
    *state.status.lock().expect("stub status lock poisoned") = StubStatus::Idle;
    state.bump(&["busy"]);
    (StatusCode::OK, "reset").into_response()
}

async fn changes_answer(State(state): State<Arc<StubState>>) -> axum::Json<serde_json::Value> {
    let answer = state
        .changes
        .lock()
        .expect("stub changes lock poisoned")
        .answer();
    axum::Json(answer)
}

async fn options_dock_fragment(State(state): State<Arc<StubState>>) -> Html<String> {
    use std::sync::atomic::Ordering::SeqCst;
    if state.dock_options_live.load(SeqCst) {
        Html(options_dock_html(
            state.dock_options_alt.load(SeqCst),
            state.options_failed.load(SeqCst),
        ))
    } else {
        Html(String::new())
    }
}

async fn index() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        DASHBOARD_SHELL,
    )
}

async fn action_check(
    State(state): State<Arc<StubState>>,
    Form(form): Form<HashMap<String, String>>,
) -> Response<Body> {
    state
        .action_requests
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let command = form.get("command").map(String::as_str).unwrap_or_default();
    if command.contains(TEXT_CHECK_TRIGGER) {
        return (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            text_check_preview_html(),
        )
            .into_response();
    }
    match state.action_outcome {
        StubActionOutcome::Pending => dispatched(&state),
        StubActionOutcome::Refused => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            BUSY_STATUS,
        )
            .into_response(),
        StubActionOutcome::Error => action_failure(command),
    }
}

/// The engine answers nothing for a command it took, and its Job holds the
/// Game, so the busy state moves.
fn dispatched(state: &StubState) -> Response<Body> {
    *state.status.lock().expect("stub status lock poisoned") = StubStatus::Step(Step::Narrating);
    state.bump(&["busy"]);
    (StatusCode::OK, "").into_response()
}

/// The 500 a failing engine action produces: the engine's own error render
/// naming the command, so the disclosure's raw text is the server's response.
fn action_failure(command: &str) -> Response<Body> {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        render_error(&format!("Failed to process action: {command}")),
    )
        .into_response()
}

async fn status_busy(State(state): State<Arc<StubState>>) -> Response<Body> {
    let status = state
        .status
        .lock()
        .expect("stub status lock poisoned")
        .clone();
    let body = match status {
        StubStatus::Idle => idle_answer(),
        StubStatus::Step(step) => busy_answer(&Busy::Job(step)),
        StubStatus::Autoplay { done, budget, step } => busy_answer(&Busy::Autoplay {
            progress: AutoplayProgress { done, budget },
            step,
        }),
        StubStatus::Error(kind, raw) => failure_answer(&GenerationFailure {
            kind,
            named: raw.clone(),
            raw,
        }),
    };
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        body,
    )
        .into_response()
}

/// An error outcome answers the 500 a failing engine action produces, so a test
/// can drive a refused confirm. A refused outcome answers the wait status.
async fn action_confirm(
    State(state): State<Arc<StubState>>,
    Form(form): Form<HashMap<String, String>>,
) -> Response<Body> {
    let command = form.get("command").map(String::as_str).unwrap_or_default();
    match state.action_outcome {
        StubActionOutcome::Error => action_failure(command),
        StubActionOutcome::Refused => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            BUSY_STATUS,
        )
            .into_response(),
        StubActionOutcome::Pending => dispatched(&state),
    }
}

async fn failing_posture_save() -> Response<Body> {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        render_error("Failed to save posture"),
    )
        .into_response()
}

/// The retry route: a refusal while the Game is busy, else the 500 the retry
/// failure scenario drives.
async fn retry_swipe(State(state): State<Arc<StubState>>) -> Response<Body> {
    if state
        .story_writes_refused_as_busy
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        return (StatusCode::OK, Html(BUSY_STATUS)).into_response();
    }
    client_failure("Stub retry failure")
}

/// The Autoplay start answers by the action outcome: it takes the budget and the
/// run holds the Game, or the busy Game refuses it, or it fails.
async fn start_autoplay(State(state): State<Arc<StubState>>) -> Response<Body> {
    match state.action_outcome {
        StubActionOutcome::Pending => {
            *state.status.lock().expect("stub status lock poisoned") = StubStatus::Autoplay {
                done: 0,
                budget: 10,
                step: Some(Step::Narrating),
            };
            state.bump(&["busy"]);
            (StatusCode::OK, "").into_response()
        }
        StubActionOutcome::Refused => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            BUSY_STATUS,
        )
            .into_response(),
        StubActionOutcome::Error => client_failure("Stub autoplay failure"),
    }
}

async fn record_retrigger(State(state): State<Arc<StubState>>) -> Response<Body> {
    state
        .retrigger_requests
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    client_failure("Stub retrigger failure")
}

async fn presets_fragment(State(state): State<Arc<StubState>>) -> Html<String> {
    let presets = state.presets.lock().expect("stub presets lock poisoned");
    Html(presets_html(&presets))
}

/// A card's Cancel loads the card back in place of its editor.
async fn preset_card_fragment(
    State(state): State<Arc<StubState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Html<String> {
    use askama::Template;

    let presets = state.presets.lock().expect("stub presets lock poisoned");
    let Some(preset) = presets.find(&id) else {
        return Html(render_error("Preset not found"));
    };
    Html(
        PresetCardTemplate::from_settings(&preset, &presets.settings())
            .render()
            .expect("render the preset card"),
    )
}

/// The real preset delete answers nothing for a delete that went through: the
/// next load of the panel no longer lists the card.
async fn delete_preset(
    State(state): State<Arc<StubState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response<Body> {
    if state
        .row_deletes_succeed
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        state
            .presets
            .lock()
            .expect("stub presets lock poisoned")
            .custom
            .retain(|preset| preset.id != id);
        state.bump(&["presets"]);
        (StatusCode::OK, "").into_response()
    } else {
        client_refusal("Preset is a mode default; change the default before deleting")
    }
}

#[derive(serde::Deserialize)]
struct ActivateQuery {
    mode: Option<String>,
}

/// The real Activate answers nothing and moves the presets and settings numbers.
async fn activate_preset(
    State(state): State<Arc<StubState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
    axum::extract::Query(query): axum::extract::Query<ActivateQuery>,
) -> Response<Body> {
    if query.mode.as_deref().unwrap_or("novel") == "novel" {
        state
            .presets
            .lock()
            .expect("stub presets lock poisoned")
            .active_system = id;
    }
    state.bump(&["presets", "settings"]);
    (StatusCode::OK, "").into_response()
}

/// The real Duplicate answers nothing and moves the presets number.
async fn duplicate_preset(
    State(state): State<Arc<StubState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response<Body> {
    {
        let mut presets = state.presets.lock().expect("stub presets lock poisoned");
        let Some(source) = presets.find(&id) else {
            return client_refusal("Preset not found");
        };
        presets.copies += 1;
        let number = presets.copies;
        let mut copy = source.clone();
        copy.id = format!("{}_copy{number}", source.id);
        copy.name = format!("{} (Copy {number})", source.name);
        copy.is_default = false;
        presets.custom.push(copy);
    }
    state.bump(&["presets"]);
    (StatusCode::OK, "").into_response()
}

/// The card's Edit control: the engine answers a default preset's edit with a
/// 200 error fragment in place of the card.
async fn preset_edit_form_fragment(
    State(state): State<Arc<StubState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Html<String> {
    use std::sync::atomic::Ordering::SeqCst;
    if state.preset_card_refused.load(SeqCst) {
        return Html("<div class=\"error-message\">Cannot edit default presets</div>".to_string());
    }
    let presets = state.presets.lock().expect("stub presets lock poisoned");
    match presets.find(&id) {
        Some(preset) => Html(preset_edit_form_html(&preset, "system")),
        None => Html(render_error("Preset not found")),
    }
}

async fn worlds_fragment(State(state): State<Arc<StubState>>) -> Html<String> {
    let worlds = state.worlds.lock().expect("stub worlds lock poisoned");
    Html(worlds_html(&worlds))
}

async fn edit_world_form_fragment(
    State(state): State<Arc<StubState>>,
    axum::extract::Path(key): axum::extract::Path<String>,
) -> Response<Body> {
    let worlds = state.worlds.lock().expect("stub worlds lock poisoned");
    match worlds.worlds.iter().find(|world| world.key == key) {
        Some(world) => Html(world_form_html(Some(world))).into_response(),
        None => client_refusal("World not found"),
    }
}

/// The real World save answers nothing and moves the worlds number.
async fn save_world(
    State(state): State<Arc<StubState>>,
    Form(form): Form<HashMap<String, String>>,
) -> Response<Body> {
    {
        let mut worlds = state.worlds.lock().expect("stub worlds lock poisoned");
        let key = form.get("key").cloned().unwrap_or_default();
        let name = form.get("name").cloned().unwrap_or_default();
        match worlds.worlds.iter_mut().find(|world| world.key == key) {
            Some(world) => world.name = name,
            None => worlds.worlds.push(WorldCard {
                key,
                name,
                ..Default::default()
            }),
        }
    }
    state.bump(&["worlds"]);
    (StatusCode::OK, "").into_response()
}

/// The real posture auto-save answers nothing for a saved change, and a refusal
/// for a value it cannot take.
async fn save_world_posture(State(state): State<Arc<StubState>>) -> Response<Body> {
    if state
        .worlds
        .lock()
        .expect("stub worlds lock poisoned")
        .posture_failing
    {
        return client_refusal("Unknown narrator mode");
    }
    state.bump(&["worlds"]);
    (StatusCode::OK, "").into_response()
}

/// The real world delete answers nothing for a delete that went through, and a
/// 400 refusal for a world that still has games.
async fn delete_world(
    State(state): State<Arc<StubState>>,
    axum::extract::Path(key): axum::extract::Path<String>,
) -> Response<Body> {
    use std::sync::atomic::Ordering::SeqCst;
    if state.world_delete_refused.load(SeqCst) {
        return client_refusal("World still has games");
    }
    if state.row_deletes_succeed.load(SeqCst) {
        state
            .worlds
            .lock()
            .expect("stub worlds lock poisoned")
            .worlds
            .retain(|world| world.key != key);
        state.bump(&["worlds"]);
        (StatusCode::OK, "").into_response()
    } else {
        client_failure("Stub world delete failure")
    }
}

async fn save_message(State(state): State<Arc<StubState>>) -> Response<Body> {
    if state
        .story_writes_refused_as_busy
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        return (StatusCode::OK, Html(BUSY_STATUS)).into_response();
    }
    if state
        .save_succeeds
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        state.bump(&["story"]);
        (StatusCode::OK, "").into_response()
    } else {
        client_failure("Stub save failure")
    }
}

async fn delete_message(State(state): State<Arc<StubState>>) -> Response<Body> {
    use std::sync::atomic::Ordering::SeqCst;
    if state.story_writes_refused_as_busy.load(SeqCst) {
        return (StatusCode::OK, Html(BUSY_STATUS)).into_response();
    }
    if !state.delete_succeeds.load(SeqCst) {
        return client_failure("Stub delete failure");
    }
    state.deleted.store(true, SeqCst);
    state.bump(&["story"]);
    (StatusCode::OK, "").into_response()
}

fn client_failure(message: &str) -> Response<Body> {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        format!("<p>{message}</p>"),
    )
        .into_response()
}

fn client_refusal(message: &str) -> Response<Body> {
    (
        StatusCode::BAD_REQUEST,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        render_error(message),
    )
        .into_response()
}
