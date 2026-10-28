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

use chronicler_engine::domain::model::change::{
    ActiveGameVersions, ChangeKind, ChangeVersions, EngineKind, GameKind, GameVersions,
};
use chronicler_engine::domain::model::game::GameId;
use chronicler_engine::adapters::driving::http::builders::headers::{
    failure_banner_html, header_fragment_html,
};
use chronicler_engine::adapters::driving::http::builders::presets::{
    preset_edit_form_html, preset_view_form_html,
};
use chronicler_engine::adapters::driving::http::prompt_presets::templates::{
    PresetAddFormTemplate, PromptPresetsTemplate,
};
use chronicler_engine::adapters::driving::http::worlds::templates::worlds::{
    WorldFormTemplate, WorldsPanelTemplate,
};
use chronicler_engine::adapters::driving::http::games::templates::games::{
    GamePostureTemplate, GameRenameFormTemplate, GameRowView, GamesPagerView, GamesPanelTemplate,
    NewGameFormTemplate, PersonaRowView,
};
use chronicler_engine::domain::model::saved_games_page::{SavedGamesPage, SAVED_GAMES_PAGE_SIZE};
use chronicler_engine::adapters::driving::http::settings::templates::{
    ConnectionFormTemplate, ConnectionTestDialogTemplate, ConnectionTestResultTemplate,
    SettingsTemplate,
};
use chronicler_engine::adapters::driving::http::layout::handlers::{
    autoplay_controls_answer, busy_answer, failure_answer, idle_answer, send_control_answer,
};
use chronicler_engine::adapters::driving::http::utils::answer_words::{
    describe_answers, item_named, naming_item, RefusalKind,
};
use chronicler_engine::adapters::driving::http::utils::error::{
    action_failure_response, action_invalid_response, action_not_found_response,
};
use chronicler_engine::adapters::driving::http::utils::response::refused;
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
use super::stub_action_layers::StubActionLayers;
use super::CONFIG_PATH;

mod handles;

pub use handles::*;

const DASHBOARD_SHELL: &str = include_str!("../../assets/index.html");

const FIXTURE_VISUAL_SIDEBAR: &str = include_str!("stub_fixtures/visual_sidebar.html");

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

/// The LLM messages the stub holds, newest first: each carries prompts and raw
/// JSON, so its record holds every kind of disclosure the real dialog renders.
fn canned_llm_messages(count: usize) -> Vec<LlmMessage> {
    (1..=count as u64)
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
        .collect()
}

/// The list the real panel renders: one header for each message.
fn llm_messages_html(count: usize) -> String {
    use askama::Template;
    use chronicler_engine::adapters::driving::http::templates::LlmMessagesTemplate;

    LlmMessagesTemplate::new(&canned_llm_messages(count))
        .render()
        .expect("render LLM messages")
}

/// The Games panel, rendered by the engine's own template over the games the
/// stub holds: the active Game, and the page of saved Games that no delete removed.
fn games_html(games: &StubGames, requested_page: u64) -> String {
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
    let total = games.saved.len() as u64;
    let page = SavedGamesPage::clamp_page(requested_page, total);
    GamesPanelTemplate {
        active_game: Some(row(1, &games.active_name)),
        saved_games: games
            .saved
            .iter()
            .skip(((page - 1) * SAVED_GAMES_PAGE_SIZE) as usize)
            .take(SAVED_GAMES_PAGE_SIZE as usize)
            .map(|(id, name)| row(*id, name))
            .collect(),
        pager: GamesPagerView::new(page, SavedGamesPage::page_count_of(total)),
        worlds: new_game_worlds(),
        posture_html: posture,
    }
    .render()
    .expect("render the Games panel")
}

fn new_game_worlds() -> Vec<WorldCard> {
    let world = |key: &str, name: &str, description: &str| WorldCard {
        key: key.to_string(),
        name: name.to_string(),
        description: description.to_string(),
        ..WorldCard::default()
    };
    vec![
        world("redmist_estate", "Redmist Estate", "A coastal estate."),
        world("test", "Test Realm", "A small testing kingdom for UI tests"),
    ]
}

fn new_game_personas() -> Vec<PersonaRowView> {
    vec![
        PersonaRowView {
            key: "julian".to_string(),
            name: "Julian".to_string(),
        },
        PersonaRowView {
            key: "test_player".to_string(),
            name: "Test Player".to_string(),
        },
    ]
}

fn settings_html(settings: &AppSettings, degraded: bool) -> String {
    use askama::Template;

    SettingsTemplate::from_settings(settings, &stub_roles(degraded))
        .render()
        .expect("render settings")
}

fn connection_form_html(connection: Option<&LlmConnection>) -> String {
    use askama::Template;

    ConnectionFormTemplate::new(connection)
        .render()
        .expect("render connection form")
}

fn connection_test_result_html(model: &str) -> String {
    use askama::Template;

    ConnectionTestResultTemplate {
        backend_name: "Mock".to_string(),
        model_name: model.to_string(),
        elapsed_ms: 7,
    }
    .render()
    .expect("render connection test result")
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

    fn quantifier_presets(&self) -> Vec<PromptPreset> {
        vec![default_preset(
            "quantifier_default",
            "Default",
            PresetType::Quantifier,
            NarratorMode::Novel,
        )]
    }

    fn impersonate_presets(&self) -> Vec<PromptPreset> {
        vec![default_preset(
            "impersonate_default",
            "Default",
            PresetType::Impersonate,
            NarratorMode::Novel,
        )]
    }

    fn find(&self, id: &str) -> Option<PromptPreset> {
        self.system_presets()
            .into_iter()
            .chain(self.quantifier_presets())
            .chain(self.impersonate_presets())
            .find(|preset| preset.id == id)
    }
}

fn presets_html(presets: &StubPresets) -> String {
    use askama::Template;

    PromptPresetsTemplate {
        system_presets: presets.system_presets(),
        quantifier_presets: presets.quantifier_presets(),
        impersonate_presets: presets.impersonate_presets(),
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
    /// A 409 that refuses the command: the engine answers its busy refusal
    /// because a generation is already in flight.
    Refused,
    /// A 400 for a value the engine cannot use, as the Action input and the
    /// Autoplay cycles receive it.
    Invalid,
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
    save_invalid: std::sync::atomic::AtomicBool,
    saved_texts: std::sync::Mutex<Vec<String>>,
    connection_saves_succeed: std::sync::atomic::AtomicBool,
    role_saves_keep_stored_value: std::sync::atomic::AtomicBool,
    role_save_answers_held: std::sync::atomic::AtomicBool,
    role_save_answer_permits: tokio::sync::Semaphore,
    story_writes_refused_as_busy: std::sync::atomic::AtomicBool,
    generation_requests_accepted: std::sync::atomic::AtomicBool,
    action_layers: StubActionLayers,
    delete_succeeds: std::sync::atomic::AtomicBool,
    deleted: std::sync::atomic::AtomicBool,
    header_degraded: std::sync::atomic::AtomicBool,
    asks_and_loads_failing: std::sync::atomic::AtomicBool,
    row_deletes_succeed: std::sync::atomic::AtomicBool,
    world_delete_refused: std::sync::atomic::AtomicBool,
    preset_card_refused: std::sync::atomic::AtomicBool,
    changes: std::sync::Mutex<StubChanges>,
    settings: std::sync::Mutex<AppSettings>,
    change_asks: std::sync::atomic::AtomicUsize,
    fragment_loads: std::sync::Mutex<HashMap<String, usize>>,
    screen_requests: std::sync::Mutex<Vec<Vec<String>>>,
    loads_failing: std::sync::atomic::AtomicBool,
    story_log_delay_ms: std::sync::atomic::AtomicU64,
    options_failed: std::sync::atomic::AtomicBool,
    games: std::sync::Mutex<StubGames>,
    presets: std::sync::Mutex<StubPresets>,
    worlds: std::sync::Mutex<StubWorlds>,
    llm_message_count: std::sync::atomic::AtomicUsize,
    llm_records_pruned: std::sync::atomic::AtomicBool,
}

/// The Games the stub lists: a rename changes a name and a delete removes a saved
/// Game, and each moves the `games` number, as the engine does.
struct StubGames {
    active_name: String,
    saved: Vec<(u64, String)>,
    rename_failing: bool,
    rename_refused: bool,
    posture_saves_succeed: bool,
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
            rename_refused: false,
            posture_saves_succeed: false,
        }
    }
}

/// What `GET /changes` answers: a real `ChangeVersions`, so the stub answers the
/// fields of the engine's own snapshot. A test moves a number through
/// `StubChangesHandle`; the setters of the other handles move the number of the
/// data they change, so a test that changes stub data still reaches the screen
/// without naming a number.
#[derive(Debug, Clone)]
struct StubChanges {
    versions: ChangeVersions,
}

impl StubChanges {
    fn new() -> Self {
        let one = GameVersions {
            story: 1,
            options: 1,
            busy_state: 1,
        };
        Self {
            versions: ChangeVersions {
                server: "stub-server-1".to_string(),
                game: Some(ActiveGameVersions {
                    id: GameId(1),
                    numbers: one,
                }),
                games: 1,
                worlds: 1,
                settings: 1,
                prompt_presets: 1,
                llm_messages: 1,
                revision: 1,
            },
        }
    }

    fn bump(&mut self, kind: ChangeKind) {
        let versions = &mut self.versions;
        versions.revision += 1;
        match kind {
            ChangeKind::Game(kind) => {
                let game = versions
                    .game
                    .as_mut()
                    .unwrap_or_else(|| panic!("the stub has no active Game to move {kind:?}"));
                match kind {
                    GameKind::Story => game.numbers.story += 1,
                    GameKind::Options => game.numbers.options += 1,
                    GameKind::BusyState => game.numbers.busy_state += 1,
                }
            }
            ChangeKind::Engine(kind) => match kind {
                EngineKind::Games => versions.games += 1,
                EngineKind::Worlds => versions.worlds += 1,
                EngineKind::Settings => versions.settings += 1,
                EngineKind::PromptPresets => versions.prompt_presets += 1,
                EngineKind::LlmMessages => versions.llm_messages += 1,
            },
        }
    }

    /// A Game that was active before keeps its numbers when it becomes active again.
    fn set_game(&mut self, game: Option<u64>) {
        let kept = self.versions.game.map(|active| active.numbers);
        self.versions.game = game.map(|id| ActiveGameVersions {
            id: GameId(id),
            numbers: kept.unwrap_or_default(),
        });
    }

    fn answer(&self) -> serde_json::Value {
        serde_json::to_value(&self.versions).expect("the stub snapshot serializes")
    }
}

impl StubState {
    fn bump(&self, kinds: &[ChangeKind]) {
        let mut changes = self.changes.lock().expect("stub changes lock poisoned");
        for &kind in kinds {
            changes.bump(kind);
        }
    }
}

pub struct StubServer {
    addr: SocketAddr,
    shutdown: Arc<std::sync::Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
    state: Arc<StubState>,
    owns_server: bool,
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
            save_invalid: std::sync::atomic::AtomicBool::new(false),
            saved_texts: std::sync::Mutex::new(Vec::new()),
            connection_saves_succeed: std::sync::atomic::AtomicBool::new(false),
            role_saves_keep_stored_value: std::sync::atomic::AtomicBool::new(false),
            role_save_answers_held: std::sync::atomic::AtomicBool::new(false),
            role_save_answer_permits: tokio::sync::Semaphore::new(0),
            story_writes_refused_as_busy: std::sync::atomic::AtomicBool::new(false),
            generation_requests_accepted: std::sync::atomic::AtomicBool::new(false),
            action_layers: StubActionLayers::default(),
            delete_succeeds: std::sync::atomic::AtomicBool::new(false),
            deleted: std::sync::atomic::AtomicBool::new(false),
            header_degraded: std::sync::atomic::AtomicBool::new(false),
            asks_and_loads_failing: std::sync::atomic::AtomicBool::new(false),
            row_deletes_succeed: std::sync::atomic::AtomicBool::new(false),
            world_delete_refused: std::sync::atomic::AtomicBool::new(false),
            preset_card_refused: std::sync::atomic::AtomicBool::new(false),
            changes: std::sync::Mutex::new(StubChanges::new()),
            settings: std::sync::Mutex::new(AppSettings::default()),
            change_asks: std::sync::atomic::AtomicUsize::new(0),
            fragment_loads: std::sync::Mutex::new(HashMap::new()),
            screen_requests: std::sync::Mutex::new(Vec::new()),
            loads_failing: std::sync::atomic::AtomicBool::new(false),
            story_log_delay_ms: std::sync::atomic::AtomicU64::new(0),
            options_failed: std::sync::atomic::AtomicBool::new(false),
            games: std::sync::Mutex::new(StubGames::new()),
            presets: std::sync::Mutex::new(StubPresets::new()),
            worlds: std::sync::Mutex::new(StubWorlds::new()),
            llm_message_count: std::sync::atomic::AtomicUsize::new(1),
            llm_records_pruned: std::sync::atomic::AtomicBool::new(false),
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
            owns_server: true,
        }
    }

    /// A second value for the same server that does not stop it when dropped, so
    /// the runner can still read the page after a check has consumed its value.
    pub(crate) fn lent(&self) -> StubServer {
        Self {
            addr: self.addr,
            shutdown: Arc::clone(&self.shutdown),
            state: Arc::clone(&self.state),
            owns_server: false,
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

impl Drop for StubServer {
    fn drop(&mut self) {
        if !self.owns_server {
            return;
        }
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
        .route("/connections/add", post(save_connection))
        .route("/connections/:id/edit", post(save_connection))
        .route("/connections/set-quantifier", post(set_quantifier))
        .route("/connections/test", post(test_connection_form))
        .route("/connections/:id/test", post(test_saved_connection))
        .route(
            "/prompt-presets",
            post(|| async { action_invalid_response("Invalid preset type") }),
        )
        .route(
            "/prompt-presets/:id",
            post(|| async { action_failure_response("preset save failure") }),
        )
        .route("/prompt-presets/:id/delete", post(delete_preset))
        .route("/prompt-presets/:id/activate", post(activate_preset))
        .route("/prompt-presets/:id/duplicate", post(duplicate_preset))
        .route("/worlds", post(save_world))
        .route("/worlds/:key", post(save_world))
        .route("/worlds/:key/posture", post(save_world_posture))
        .route("/settings/text-check", post(save_text_check))
        .route("/games/:id/posture", post(game_posture_save))
        // The real delete answers an empty body, so htmx removes the row the
        // control sits in.
        .route("/games/:id/delete", post(delete_game))
        .route("/games/:id/rename", post(rename_game))
        .route("/games/:id/switch", post(switch_game))
        .route("/history/:id", post(save_message))
        // Scenario 16.34 drives /reset from the games row: a 500 here fills that
        // row's own error slot rather than the panel's.
        .route(
            "/reset",
            post(|| async { action_failure_response("Stub reset failure") }),
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
        .route("/screen", get(screen_answer))
        .route("/stub/touch", get(touch_revision))
        // A button that a check adds to a part posts here, so its request ends with 200.
        .route("/probe/:name", post(|| async { StatusCode::OK }))
        .route("/fragment/header", get(header_fragment))
        .route("/fragment/failure-banner", get(failure_banner_fragment))
        .route("/fragment/story-log", get(story_log_fragment))
        .route(
            "/fragment/visual-sidebar",
            get(|| async { Html(FIXTURE_VISUAL_SIDEBAR) }),
        )
        .route("/fragment/options-dock", get(options_dock_fragment))
        .route("/fragment/llm-messages", get(llm_messages_fragment))
        .route(
            "/fragment/llm-messages/:id",
            get(llm_message_record_fragment),
        )
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
        .route(
            "/fragment/prompt-presets/new",
            get(preset_add_form_fragment),
        )
        .route(
            "/fragment/prompt-presets/:id/edit",
            get(preset_edit_form_fragment),
        )
        .route(
            "/fragment/prompt-presets/:id/view",
            get(preset_view_form_fragment),
        )
        .route("/fragment/worlds", get(worlds_fragment))
        .route(
            "/fragment/worlds/new",
            get(|| async { Html(world_form_html(None)) }),
        )
        .route("/worlds/:key/edit", get(edit_world_form_fragment))
        .route("/fragment/games", get(games_fragment))
        .route("/fragment/games/new", get(new_game_form_fragment))
        .route("/fragment/games/:id/rename", get(rename_game_form_fragment))
        .nest_service("/assets", tower_http::services::ServeDir::new("assets"))
        .nest_service("/data", tower_http::services::ServeDir::new("data"))
        .layer(axum::middleware::from_fn_with_state(
            Arc::clone(&state),
            count_and_fail_reads_middleware,
        ))
        .layer(axum::middleware::from_fn_with_state(
            Arc::clone(&state),
            action_answers_middleware,
        ))
        .layer(axum::middleware::from_fn(describe_answers))
        .with_state(state)
}

/// The switch makes the Game active for `GET /changes`, as the engine does.
async fn switch_game(
    State(state): State<Arc<StubState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
) -> Response<Body> {
    state
        .changes
        .lock()
        .expect("stub changes lock poisoned")
        .set_game(Some(id));
    state.bump(&[ChangeKind::GAMES]);
    (StatusCode::OK, "").into_response()
}

async fn games_fragment(State(state): State<Arc<StubState>>) -> Html<String> {
    let games = state.games.lock().expect("stub games lock poisoned");
    Html(games_html(&games, 1))
}

async fn new_game_form_fragment() -> Html<String> {
    use askama::Template;

    Html(
        NewGameFormTemplate {
            worlds: new_game_worlds(),
            personas: new_game_personas(),
        }
        .render()
        .expect("render the New Game form"),
    )
}

async fn rename_game_form_fragment(
    State(state): State<Arc<StubState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
) -> Response<Body> {
    use askama::Template;

    let games = state.games.lock().expect("stub games lock poisoned");
    let display_name = if id == 1 {
        Some(games.active_name.clone())
    } else {
        games
            .saved
            .iter()
            .find(|(saved, _)| *saved == id)
            .map(|(_, name)| name.clone())
    };
    match display_name {
        Some(display_name) => Html(
            GameRenameFormTemplate { id, display_name }
                .render()
                .expect("render the Rename form"),
        )
        .into_response(),
        None => action_not_found_response("Game not found"),
    }
}

async fn llm_messages_fragment(State(state): State<Arc<StubState>>) -> Html<String> {
    let count = state
        .llm_message_count
        .load(std::sync::atomic::Ordering::SeqCst);
    Html(llm_messages_html(count))
}

/// The record of one message, rendered by the engine's own template. A pruned
/// record answers the engine's 404.
async fn llm_message_record_fragment(
    State(state): State<Arc<StubState>>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Response<Body> {
    use askama::Template;
    use chronicler_engine::adapters::driving::http::templates::LlmMessageRecordTemplate;
    use std::sync::atomic::Ordering::SeqCst;

    let count = state.llm_message_count.load(SeqCst);
    let record = canned_llm_messages(count)
        .into_iter()
        .find(|message| message.id == id)
        .filter(|_| !state.llm_records_pruned.load(SeqCst));
    match record {
        Some(message) => Html(
            LlmMessageRecordTemplate::new(&message)
                .render()
                .expect("render the LLM message record"),
        )
        .into_response(),
        None => action_not_found_response("LLM message not found"),
    }
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
    state.bump(&[ChangeKind::GAMES]);
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
        if games.rename_refused {
            return refused(RefusalKind::Busy);
        }
        if games.rename_failing {
            return action_failure_response("Stub rename failure");
        }
        if id == 1 {
            games.active_name = form.display_name;
        } else if let Some((_, name)) = games.saved.iter_mut().find(|(saved, _)| *saved == id) {
            *name = form.display_name;
        }
    }
    state.bump(&[ChangeKind::GAMES]);
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
            text: "An older turn the next load will drop.".to_string(),
            message_type: MessageType::Narration,
            timestamp: Utc.with_ymd_and_hms(2026, 1, 1, 19, 6, 0).unwrap(),
            ..default_story_row(MessageType::Narration)
        },
    );
    entries
}

const PANEL_LOAD_PATHS: [&str; 4] = [
    "/fragment/worlds",
    "/fragment/games",
    "/fragment/settings",
    "/fragment/prompt-presets",
];

async fn action_answers_middleware(
    State(state): State<Arc<StubState>>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response<Body> {
    let read_changes = || {
        let changes = state.changes.lock().expect("stub changes lock poisoned");
        (
            changes.versions.stamp(),
            changes.versions.game.map(|game| game.id.0),
        )
    };
    state
        .action_layers
        .answer(request, next, read_changes)
        .await
}

async fn count_and_fail_reads_middleware(
    State(state): State<Arc<StubState>>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response<Body> {
    use std::sync::atomic::Ordering::SeqCst;
    let path = request.uri().path().to_string();
    let query = request.uri().query().unwrap_or_default().to_string();
    let is_load = path.starts_with("/fragment/") || path == "/status/busy" || path == "/screen";
    let names_part = |part: &str| query.split(['=', ',', '&']).any(|name| name == part);
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
    let loads_story_log =
        path == "/fragment/story-log" || (path == "/screen" && names_part("story-log"));
    if loads_story_log && delay_ms > 0 {
        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
    }
    let is_panel_load = PANEL_LOAD_PATHS.contains(&path.as_str())
        || (path == "/screen"
            && ["worlds", "games", "settings", "prompt-presets"]
                .iter()
                .any(|part| names_part(part)));
    if is_panel_load && state.panel_loads_failing.load(SeqCst) {
        return action_failure_response("panel load failed");
    }
    let is_read = is_load || path == "/changes";
    let failing =
        state.asks_and_loads_failing.load(SeqCst) || (is_load && state.loads_failing.load(SeqCst));
    if is_read && failing {
        return action_failure_response("read failed");
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
    let settings = state.settings.lock().expect("stub settings lock poisoned");
    Html(settings_html(&settings, state.header_degraded.load(SeqCst)))
}

/// A saved role change answers nothing, the way the engine answers it, and keeps
/// the value, so the next render of Settings shows it.
async fn set_quantifier(
    State(state): State<Arc<StubState>>,
    Form(form): Form<HashMap<String, String>>,
) -> Response<Body> {
    let keeps = state
        .role_saves_keep_stored_value
        .load(std::sync::atomic::Ordering::SeqCst);
    if let (Some(id), false) = (form.get("connection_id"), keeps) {
        state
            .settings
            .lock()
            .expect("stub settings lock poisoned")
            .quantifier_connection_id = id.clone();
    }
    state.bump(&[ChangeKind::SETTINGS]);
    if state
        .role_save_answers_held
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        state
            .role_save_answer_permits
            .acquire()
            .await
            .expect("role save answer gate closed")
            .forget();
    }
    (StatusCode::OK, "").into_response()
}

/// The real handler wraps the result of a saved connection's test in the dialog.
async fn test_saved_connection(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Html<String> {
    use askama::Template;

    Html(
        ConnectionTestDialogTemplate::new(id.clone(), connection_test_result_html(&id))
            .render()
            .expect("render connection test dialog"),
    )
}

async fn test_connection_form() -> Html<String> {
    Html(connection_test_result_html("typed-model"))
}

/// A connection save refuses the typed provider until a test arms it.
async fn save_connection(State(state): State<Arc<StubState>>) -> Response<Body> {
    if state
        .connection_saves_succeed
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        return (StatusCode::OK, "").into_response();
    }
    action_invalid_response("Unknown LLM backend 'bogus_provider'")
}

async fn save_text_check(
    State(state): State<Arc<StubState>>,
    Form(form): Form<HashMap<String, String>>,
) -> Response<Body> {
    use chronicler_engine::domain::model::settings::TextCheckMode;
    let mode = match form.get("check_mode").map(String::as_str) {
        Some("spell") => Some(TextCheckMode::Spell),
        Some("grammar") => Some(TextCheckMode::Grammar),
        Some("spell_grammar") => Some(TextCheckMode::SpellGrammar),
        Some("disabled") => Some(TextCheckMode::Disabled),
        _ => None,
    };
    if let Some(mode) = mode {
        state
            .settings
            .lock()
            .expect("stub settings lock poisoned")
            .text_check
            .mode = mode;
    }
    state.bump(&[ChangeKind::SETTINGS]);
    (StatusCode::OK, "").into_response()
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
        return refused(RefusalKind::Busy);
    }
    if state.switch_fails.load(SeqCst) {
        return action_failure_response("Stub swipe switch failure");
    }
    state.switched.store(true, SeqCst);
    state.dock_options_live.store(false, SeqCst);
    state.bump(&[ChangeKind::STORY, ChangeKind::OPTIONS]);
    Html(story_log_html(&restored_swipe_entries())).into_response()
}

/// The real Stop answers a plain word and moves the busy state.
async fn reset_busy(State(state): State<Arc<StubState>>) -> Response<Body> {
    *state.status.lock().expect("stub status lock poisoned") = StubStatus::Idle;
    state.bump(&[ChangeKind::BUSY_STATE]);
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

/// The shell asks every `STUB_POLL_MS`, so a stub check waits for a load for a
/// tenth of the time that the real shell takes.
const STUB_POLL_MS: &str = "100";

async fn index() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        DASHBOARD_SHELL.replace(
            "data-poll-ms=\"1000\"",
            &format!("data-poll-ms=\"{STUB_POLL_MS}\""),
        ),
    )
}

async fn screen_part_html(state: &Arc<StubState>, name: &str, games_page: u64) -> Option<String> {
    let state = || State(Arc::clone(state));
    Some(match name {
        "header" => header_html(),
        "failure-banner" => failure_banner_fragment(state()).await.0,
        "story-log" => story_log_fragment(state()).await.0,
        "visual-sidebar" => FIXTURE_VISUAL_SIDEBAR.to_string(),
        "options-dock" => options_dock_fragment(state()).await.0,
        "send-control" => send_control_answer(stub_busy(&state().0).as_ref()),
        "autoplay-controls" => autoplay_controls_answer(stub_busy(&state().0).as_ref()),
        "status-display" => {
            let answer = status_busy(state()).await;
            let bytes = axum::body::to_bytes(answer.into_body(), usize::MAX)
                .await
                .expect("read the stub status answer");
            String::from_utf8_lossy(&bytes).to_string()
        }
        "settings" => settings_fragment(state()).await.0,
        "prompt-presets" => presets_fragment(state()).await.0,
        "worlds" => worlds_fragment(state()).await.0,
        "games" => games_html(
            &state().0.games.lock().expect("stub games lock poisoned"),
            games_page,
        ),
        "llm-messages" => llm_messages_fragment(state()).await.0,
        _ => return None,
    })
}

/// `GET /screen?parts=`: the canned parts, each answered as the engine answers it.
/// The request is recorded, and each part counts as a load of its own route.
async fn screen_answer(
    State(state): State<Arc<StubState>>,
    axum::extract::Query(query): axum::extract::Query<HashMap<String, String>>,
) -> Response<Body> {
    let names: Vec<String> = query
        .get("parts")
        .map(|parts| parts.split(',').map(str::to_string).collect())
        .unwrap_or_default();
    let games_page = query
        .get("games_page")
        .and_then(|page| page.parse().ok())
        .unwrap_or(1);
    let (stamp, game) = {
        let changes = state.changes.lock().expect("stub changes lock poisoned");
        (
            changes.versions.stamp(),
            changes.versions.game.map(|game| game.id.0),
        )
    };
    state
        .screen_requests
        .lock()
        .expect("stub screen requests lock poisoned")
        .push(names.clone());
    let mut body = String::new();
    for name in &names {
        let Some(html) = screen_part_html(&state, name, games_page).await else {
            return (StatusCode::BAD_REQUEST, format!("unknown part {name}")).into_response();
        };
        if let Some(part) = super::screen_part_table::PARTS
            .iter()
            .find(|part| part.name == name)
        {
            *state
                .fragment_loads
                .lock()
                .expect("stub load counts lock poisoned")
                .entry(part.name.to_string())
                .or_insert(0) += 1;
        }
        body.push_str(&format!("<template data-part=\"{name}\">{html}</template>"));
    }
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8".to_string()),
            (header::HeaderName::from_static("x-revision"), stamp),
            (
                header::HeaderName::from_static("x-game"),
                game.map(|id| id.to_string()).unwrap_or_default(),
            ),
        ],
        body,
    )
        .into_response()
}

/// Moves the revision and nothing else, so the page loads its screen once more.
async fn touch_revision(State(state): State<Arc<StubState>>) -> Response<Body> {
    state
        .changes
        .lock()
        .expect("stub changes lock poisoned")
        .versions
        .revision += 1;
    (StatusCode::OK, "").into_response()
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
        StubActionOutcome::Refused => refused(RefusalKind::Busy),
        StubActionOutcome::Error => action_failure(command),
        StubActionOutcome::Invalid => action_invalid_response(INVALID_ACTION_MESSAGE),
    }
}

/// The engine's message for a value that an invalid outcome rejects.
pub const INVALID_ACTION_MESSAGE: &str = "The value is not allowed";

/// The engine answers nothing for a command it took, and its Job holds the
/// Game, so the busy state moves.
fn dispatched(state: &StubState) -> Response<Body> {
    *state.status.lock().expect("stub status lock poisoned") = StubStatus::Step(Step::Narrating);
    state.bump(&[ChangeKind::BUSY_STATE]);
    (StatusCode::OK, "").into_response()
}

/// The 500 a failing engine action produces: the engine's own error render
/// naming the command, so the disclosure's raw text is the server's response.
fn action_failure(command: &str) -> Response<Body> {
    action_failure_response(format!("Failed to process action: {command}"))
}

/// The busy state that a stub status stands for: the status display and the
/// action controls both read it. A failure line is not a busy state.
fn busy_of(status: &StubStatus) -> Option<Busy> {
    match status.clone() {
        StubStatus::Step(step) => Some(Busy::Job(step)),
        StubStatus::Autoplay { done, budget, step } => Some(Busy::Autoplay {
            progress: AutoplayProgress { done, budget },
            step,
        }),
        StubStatus::Idle | StubStatus::Error(..) => None,
    }
}

fn stub_busy(state: &StubState) -> Option<Busy> {
    busy_of(&state.status.lock().expect("stub status lock poisoned"))
}

async fn status_busy(State(state): State<Arc<StubState>>) -> Response<Body> {
    let status = state
        .status
        .lock()
        .expect("stub status lock poisoned")
        .clone();
    let body = match (busy_of(&status), status) {
        (Some(busy), _) => busy_answer(&busy),
        (None, StubStatus::Error(kind, raw)) => failure_answer(&GenerationFailure {
            kind,
            named: raw.clone(),
            raw,
        }),
        (None, _) => idle_answer(),
    };
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        body,
    )
        .into_response()
}

/// An error outcome answers the 500 a failing engine action produces, so a test
/// can drive a refused confirm. A refused outcome answers the busy refusal.
async fn action_confirm(
    State(state): State<Arc<StubState>>,
    Form(form): Form<HashMap<String, String>>,
) -> Response<Body> {
    let command = form.get("command").map(String::as_str).unwrap_or_default();
    match state.action_outcome {
        StubActionOutcome::Error => action_failure(command),
        StubActionOutcome::Refused => refused(RefusalKind::Busy),
        StubActionOutcome::Pending => dispatched(&state),
        StubActionOutcome::Invalid => action_invalid_response(INVALID_ACTION_MESSAGE),
    }
}

async fn game_posture_save(State(state): State<Arc<StubState>>) -> Response<Body> {
    if state
        .games
        .lock()
        .expect("stub games lock poisoned")
        .posture_saves_succeed
    {
        return (StatusCode::OK, "").into_response();
    }
    action_failure_response("Failed to save posture")
}

/// The retry route: a refusal while the Game is busy, else the 500 the retry
/// failure scenario drives.
async fn retry_swipe(State(state): State<Arc<StubState>>) -> Response<Body> {
    if state
        .story_writes_refused_as_busy
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        return refused(RefusalKind::Busy);
    }
    if state
        .generation_requests_accepted
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        return dispatched(&state);
    }
    action_failure_response("Stub retry failure")
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
            state.bump(&[ChangeKind::BUSY_STATE]);
            (StatusCode::OK, "").into_response()
        }
        StubActionOutcome::Refused => refused(RefusalKind::Busy),
        StubActionOutcome::Error => action_failure_response("Stub autoplay failure"),
        StubActionOutcome::Invalid => action_invalid_response(INVALID_ACTION_MESSAGE),
    }
}

async fn record_retrigger(State(state): State<Arc<StubState>>) -> Response<Body> {
    state
        .retrigger_requests
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    if state
        .story_writes_refused_as_busy
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        return refused(RefusalKind::Busy);
    }
    if state
        .generation_requests_accepted
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        return dispatched(&state);
    }
    action_failure_response("Stub retrigger failure")
}

async fn presets_fragment(State(state): State<Arc<StubState>>) -> Html<String> {
    let presets = state.presets.lock().expect("stub presets lock poisoned");
    Html(presets_html(&presets))
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
        state.bump(&[ChangeKind::PROMPT_PRESETS]);
        (StatusCode::OK, "").into_response()
    } else {
        action_invalid_response("Preset is a mode default; change the default before deleting")
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
    state.bump(&[ChangeKind::PROMPT_PRESETS, ChangeKind::SETTINGS]);
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
            return action_invalid_response("Preset not found");
        };
        presets.copies += 1;
        let number = presets.copies;
        let mut copy = source.clone();
        copy.id = format!("{}_copy{number}", source.id);
        copy.name = format!("{} (Copy {number})", source.name);
        copy.is_default = false;
        presets.custom.push(copy);
    }
    state.bump(&[ChangeKind::PROMPT_PRESETS]);
    (StatusCode::OK, "").into_response()
}

/// The Add control of a preset type.
async fn preset_add_form_fragment(
    axum::extract::Query(query): axum::extract::Query<HashMap<String, String>>,
) -> Response<Body> {
    let preset_type = query
        .get("type")
        .and_then(|value| PresetType::try_from(value.as_str()).ok());
    match preset_type {
        Some(preset_type) => {
            use askama::Template;
            Html(
                PresetAddFormTemplate::for_type(preset_type)
                    .render()
                    .expect("render the preset Add form"),
            )
            .into_response()
        }
        None => action_invalid_response("Invalid preset type"),
    }
}

/// The card's Edit control: the engine refuses the edit of a default preset with
/// a non-2xx answer, and the card reports it in its own slot.
async fn preset_edit_form_fragment(
    State(state): State<Arc<StubState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response<Body> {
    use std::sync::atomic::Ordering::SeqCst;
    if state.preset_card_refused.load(SeqCst) {
        return action_invalid_response("Cannot edit default presets");
    }
    let presets = state.presets.lock().expect("stub presets lock poisoned");
    match presets.find(&id) {
        Some(preset) => Html(preset_edit_form_html(&preset, "system")).into_response(),
        None => action_not_found_response("Preset not found"),
    }
}

async fn preset_view_form_fragment(
    State(state): State<Arc<StubState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response<Body> {
    let presets = state.presets.lock().expect("stub presets lock poisoned");
    match presets.find(&id) {
        Some(preset) => Html(preset_view_form_html(&preset)).into_response(),
        None => action_not_found_response("Preset not found"),
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
        None => action_not_found_response("World not found"),
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
    state.bump(&[ChangeKind::WORLDS]);
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
        return action_invalid_response("Unknown narrator mode");
    }
    state.bump(&[ChangeKind::WORLDS]);
    (StatusCode::OK, "").into_response()
}

/// The real world delete answers nothing for a delete that went through, and a
/// 400 refusal for a world that still has games.
async fn delete_world(
    State(state): State<Arc<StubState>>,
    axum::extract::Path(key): axum::extract::Path<String>,
) -> Response<Body> {
    use std::sync::atomic::Ordering::SeqCst;
    let name = state
        .worlds
        .lock()
        .expect("stub worlds lock poisoned")
        .worlds
        .iter()
        .find(|world| world.key == key)
        .and_then(|world| item_named(&world.name));
    if state.world_delete_refused.load(SeqCst) {
        return naming_item(action_invalid_response("World still has games"), name);
    }
    if state.row_deletes_succeed.load(SeqCst) {
        state
            .worlds
            .lock()
            .expect("stub worlds lock poisoned")
            .worlds
            .retain(|world| world.key != key);
        state.bump(&[ChangeKind::WORLDS]);
        (StatusCode::OK, "").into_response()
    } else {
        naming_item(action_failure_response("Stub world delete failure"), name)
    }
}

#[derive(serde::Deserialize)]
struct SaveMessageForm {
    text: String,
}

/// Records the text of each save that carries one, so a test can check what the page sent.
async fn save_message(
    State(state): State<Arc<StubState>>,
    Form(form): Form<SaveMessageForm>,
) -> Response<Body> {
    state
        .saved_texts
        .lock()
        .expect("stub saved texts lock poisoned")
        .push(form.text);
    if state
        .story_writes_refused_as_busy
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        return refused(RefusalKind::Busy);
    }
    if state.save_invalid.load(std::sync::atomic::Ordering::SeqCst) {
        return action_invalid_response(INVALID_ACTION_MESSAGE);
    }
    if state
        .save_succeeds
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        state.bump(&[ChangeKind::STORY]);
        (StatusCode::OK, "").into_response()
    } else {
        action_failure_response("Stub save failure")
    }
}

async fn delete_message(State(state): State<Arc<StubState>>) -> Response<Body> {
    use std::sync::atomic::Ordering::SeqCst;
    if state.story_writes_refused_as_busy.load(SeqCst) {
        return refused(RefusalKind::Busy);
    }
    if !state.delete_succeeds.load(SeqCst) {
        return action_failure_response("Stub delete failure");
    }
    state.deleted.store(true, SeqCst);
    state.bump(&[ChangeKind::STORY]);
    (StatusCode::OK, "").into_response()
}
