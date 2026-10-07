//! Stub-browser server: the real dashboard shell plus canned fragments, with no engine behind it.

// The shell and static assets are real; the story log, the LLM Messages
// panel, the options dock and the text-check preview render through the
// engine's own templates, so a hook change in a shipped template reaches the
// served fragment with no fixture to keep in step. The remaining polled
// fragments are canned under `tests/test_utils/stub_fixtures/`. The dynamic
// endpoints answer scripted outcomes a test names up front; `/history/:id`,
// `/swipe/new` and `/retrigger` always answer 500.

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

use chronicler_engine::adapters::driving::http::builders::headers::add_status_swap_headers;
use chronicler_engine::adapters::driving::http::settings::templates::{
    ConnectionFormTemplate, SettingsTemplate,
};
use chronicler_engine::adapters::driving::http::utils::error::{
    error_disclosure, generation_error_summary, raw_error_detail, render_error,
};
use chronicler_engine::domain::model::llm_message::LlmMessage;
use chronicler_engine::domain::model::settings::{AppSettings, LlmProviderConfig};
use chronicler_engine::domain::model::state::message_types::{MessageEntry, MessageType};

use super::server::{get_config_port, release_port_lock};
use super::CONFIG_PATH;

/// The shipped dashboard shell, served verbatim so the stub exercises the real
/// htmx wiring.
const DASHBOARD_SHELL: &str = include_str!("../../assets/index.html");

const FIXTURE_VISUAL_SIDEBAR: &str = include_str!("stub_fixtures/visual_sidebar.html");
const FIXTURE_PROMPT_PRESETS: &str = include_str!("stub_fixtures/prompt_presets.html");
const FIXTURE_WORLDS: &str = include_str!("stub_fixtures/worlds.html");
const FIXTURE_GAMES: &str = include_str!("stub_fixtures/games.html");
const FIXTURE_ACTION_AREA: &str = include_str!("stub_fixtures/action_area.html");

fn header_html(degraded: bool) -> String {
    use askama::Template;
    use chronicler_engine::adapters::driving::http::templates::HeaderTemplate;

    let header = HeaderTemplate {
        game_name: "Test Realm_2026-09-17_1".to_string(),
    }
    .render()
    .expect("render header");

    let banner = if degraded {
        error_disclosure(
            "failure-banner-popover",
            "Quantifier failed — using fallback NPC IDs",
            "<div class=\"error-role degraded\"><span class=\"error-role-name\">Quantifier</span>
             <span class=\"error-role-backend\">Mock mock</span></div>",
        )
    } else {
        String::new()
    };
    format!("{header}<div id=\"failure-banner-degraded\" hx-swap-oob=\"true\">{banner}</div>")
}

const NARRATIVE_TEXT: &str = "Welcome to the Test World, Test Player! This is a simple scenario for testing the starting scenarios feature. Feel free to explore and test the game engine.\n\nThe tavern around you is warm and inviting. Wooden beams stretch across the ceiling, and a crackling fire in the hearth casts dancing shadows on the walls. The smell of fresh bread and mulled cider fills the air.\n\nBehind the bar, the bartender wipes down a mug and glances your way. \"First time in the Test Realm?\" he asks with a knowing smile. \"Don't worry, everyone here is friendly. Mostly.\"\n\nA merchant in the corner adjusts her pack and catches your eye. \"If you're heading north to the village square, mind the cobblestones. They get slippery after dark,\" she advises.\n\nYou take a moment to gather your bearings. The road ahead promises adventure, but for now, the warmth of the tavern offers a brief respite.";

fn story_log_html(entries: &[MessageEntry]) -> String {
    use askama::Template;
    use chronicler_engine::adapters::driving::http::templates::NarrativeLogTemplate;

    NarrativeLogTemplate::new(entries, true)
        .render()
        .expect("render story log")
}

/// Player input first, narration last: the template renders swipe/retrigger
/// controls only on the trailing narration.
fn default_story_log_entries() -> Vec<MessageEntry> {
    vec![
        MessageEntry {
            id: 2,
            text: "look at the casle".to_string(),
            message_type: MessageType::Input,
            timestamp: Utc.with_ymd_and_hms(2026, 1, 1, 19, 7, 0).unwrap(),
            ..Default::default()
        },
        MessageEntry {
            id: 1,
            text: NARRATIVE_TEXT.to_string(),
            message_type: MessageType::Narration,
            timestamp: Utc.with_ymd_and_hms(2026, 1, 1, 19, 8, 0).unwrap(),
            location_header: Some("Test Tavern".to_string()),
            ..Default::default()
        },
    ]
}

fn llm_messages_html() -> String {
    use askama::Template;
    use chronicler_engine::adapters::driving::http::templates::LlmMessagesTemplate;

    let messages = vec![LlmMessage {
        id: 1,
        agent_name: "narrator".to_string(),
        backend_name: "Mock".to_string(),
        model_name: "mock".to_string(),
        system_prompt: String::new(),
        user_prompt: String::new(),
        raw_request_json: String::new(),
        raw_response_json: String::new(),
        parsed_response: "canned narration".to_string(),
        error_message: None,
        created_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
    }];
    LlmMessagesTemplate::new(&messages)
        .render()
        .expect("render LLM messages")
}

/// Served through the shipped `SettingsTemplate` so a sub-tab or role-row hook
/// change reaches the fragment.
fn settings_html() -> String {
    use askama::Template;

    SettingsTemplate::from_settings(&AppSettings::default(), &[], None)
        .render()
        .expect("render settings")
}

fn connection_form_html(connection: Option<&LlmProviderConfig>) -> String {
    use askama::Template;

    ConnectionFormTemplate::new(connection)
        .render()
        .expect("render connection form")
}

const CANNED_OPTIONS: [&str; 3] = [
    "Ask the bartender about the Test Realm",
    "Examine the merchant's pack",
    "Step outside into the night air",
];

const CANNED_OPTIONS_ALT: [&str; 2] = ["Search the cellar", "Call for the innkeeper"];

fn options_dock_html(alternate: bool) -> String {
    use askama::Template;
    use chronicler_engine::adapters::driving::http::templates::OptionsDockTemplate;
    use chronicler_engine::adapters::driving::http::view_models::OptionsDockViewModel;

    let options: Vec<String> = if alternate {
        CANNED_OPTIONS_ALT.iter().map(|o| o.to_string()).collect()
    } else {
        CANNED_OPTIONS.iter().map(|o| o.to_string()).collect()
    };
    let vm = OptionsDockViewModel::new(options, false);
    OptionsDockTemplate::new(vm)
        .render()
        .expect("render options dock")
}

/// The command the stub treats as a text check with issues. The real
/// `POST /action/check` runs the text check before dispatch and returns the
/// preview for a misspelling; the stub keys on this one token so a test can
/// drive the preview → confirm swap without a real text-check engine.
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

/// What `POST /action/check` should answer with. A test names the outcome it
/// needs; the stub runs no pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StubActionOutcome {
    /// "Thinking..." acknowledgement — the shapes a test uses to observe a
    /// pending generation.
    #[default]
    Pending,
    /// A 500, as a failing engine action would produce. The body is the
    /// engine's own error render naming the command, so the toast text is the
    /// server's response rather than the input echoed back.
    Error,
}

/// What the stub's `GET /status/generating` answers. The real endpoint returns
/// "idle", a phase name, or the error fragment a failed generation produces.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum StubStatus {
    /// The engine is idle; the poll answers "idle".
    #[default]
    Idle,
    /// The engine is generating; the poll answers the phase name.
    Phase(String),
    /// A failed generation; the poll answers the error fragment.
    Error(String),
}

struct StubState {
    action_outcome: StubActionOutcome,
    status: std::sync::Mutex<StubStatus>,
    retrigger_requests: std::sync::atomic::AtomicUsize,
    action_requests: std::sync::atomic::AtomicUsize,
    /// The story log is scripted through two variants a swipe-switch test
    /// drives: the default single-swipe fixture, then a two-swipe one.
    two_swipes: std::sync::atomic::AtomicBool,
    /// Set once `POST /message/:id/swipe/:index` has been answered; the story
    /// log and dock then render the post-switch shapes.
    switched: std::sync::atomic::AtomicBool,
    /// The dock carries the canned set until a switch restores a Swipe with
    /// no set of its own.
    dock_options_live: std::sync::atomic::AtomicBool,
    dock_options_alt: std::sync::atomic::AtomicBool,
    newest_narration: std::sync::Mutex<Option<String>>,
    extra_oldest: std::sync::atomic::AtomicBool,
    save_succeeds: std::sync::atomic::AtomicBool,
    header_degraded: std::sync::atomic::AtomicBool,
    polls_failing: std::sync::atomic::AtomicBool,
}

/// A running stub server. Dropping it shuts the server down and releases
/// the port lock.
pub struct StubServer {
    addr: SocketAddr,
    shutdown: Arc<std::sync::Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
    state: Arc<StubState>,
}

impl StubServer {
    /// Start the stub on a free port from the shared test port range, resolved
    /// through `tests/test_config.json` so the stub cannot silently collide
    /// with real engine servers if the range moves.
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
            dock_options_live: std::sync::atomic::AtomicBool::new(true),
            dock_options_alt: std::sync::atomic::AtomicBool::new(false),
            newest_narration: std::sync::Mutex::new(None),
            extra_oldest: std::sync::atomic::AtomicBool::new(false),
            save_succeeds: std::sync::atomic::AtomicBool::new(false),
            header_degraded: std::sync::atomic::AtomicBool::new(false),
            polls_failing: std::sync::atomic::AtomicBool::new(false),
        });
        let app = stub_router(Arc::clone(&state));
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
            .await
            .unwrap_or_else(|e| panic!("bind stub on port {port}: {e}"));
        let addr = listener.local_addr().expect("stub local addr");
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        tokio::spawn(async move {
            let _ = axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    let _ = rx.await;
                })
                .await;
        });
        Self {
            addr,
            shutdown: Arc::new(std::sync::Mutex::new(Some(tx))),
            state,
        }
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
    }

    pub fn append_narration(&self, text: &str) {
        *self
            .state
            .newest_narration
            .lock()
            .expect("stub narration lock poisoned") = Some(text.to_string());
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
    }

    pub fn set_polls_failing(&self, failing: bool) {
        self.state
            .polls_failing
            .store(failing, std::sync::atomic::Ordering::SeqCst);
    }
}

#[derive(Clone)]
pub struct StubSwipeHandle {
    state: Arc<StubState>,
}

impl StubSwipeHandle {
    /// Script the story log as a Message with two Swipes. The log's 2s poll
    /// renders it on a later cycle.
    pub fn set_two_swipes(&self) {
        self.state
            .two_swipes
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

#[derive(Clone)]
pub struct StubStatusHandle {
    state: Arc<StubState>,
}

impl StubStatusHandle {
    /// Set what the next `GET /status/generating` poll answers. The status
    /// display polls every 5s, so the change lands on a later poll cycle.
    pub fn set(&self, status: StubStatus) {
        *self.state.status.lock().expect("stub status lock poisoned") = status;
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

fn stub_router(state: Arc<StubState>) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/action/check", post(action_check))
        .route("/action/confirm", post(action_confirm))
        .route("/games/:id/posture", post(failing_posture_save))
        .route("/history/:id", post(save_message))
        .route(
            "/swipe/new",
            post(|| async { client_failure("Stub retry failure") }),
        )
        .route("/retrigger", post(record_retrigger))
        // The real switch restores a Snapshot; the stub scripts only the two
        // log shapes and the dock drop a switch test observes.
        .route("/message/:id/swipe/:index", post(switch_swipe))
        .route("/status/generating", get(status_generating))
        .route("/fragment/header", get(header_fragment))
        .route("/fragment/story-log", get(story_log_fragment))
        .route(
            "/fragment/visual-sidebar",
            get(|| async { Html(FIXTURE_VISUAL_SIDEBAR) }),
        )
        .route("/fragment/options-dock", get(options_dock_fragment))
        .route(
            "/fragment/llm-messages",
            get(|| async { Html(llm_messages_html()) }),
        )
        .route(
            "/fragment/settings",
            get(|| async { Html(settings_html()) }),
        )
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
        .route(
            "/fragment/prompt-presets",
            get(|| async { Html(FIXTURE_PROMPT_PRESETS) }),
        )
        .route("/fragment/worlds", get(|| async { Html(FIXTURE_WORLDS) }))
        .route("/fragment/games", get(|| async { Html(FIXTURE_GAMES) }))
        .route(
            "/fragment/action-area",
            get(|| async { Html(FIXTURE_ACTION_AREA) }),
        )
        // Static assets are served from the real `assets/` and `data/`
        // directories, nested exactly as the engine routes them, so the shell's
        // script and stylesheet hrefs and the fragment images resolve as in
        // production.
        .nest_service("/assets", tower_http::services::ServeDir::new("assets"))
        .nest_service("/data", tower_http::services::ServeDir::new("data"))
        .layer(axum::middleware::from_fn_with_state(
            Arc::clone(&state),
            fail_polls_middleware,
        ))
        .with_state(state)
}

/// The scripted story log: the default entry pair, the two-Swipe shape a
/// switch test enables, or the settled shape a switch restores.
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
        entries.push(MessageEntry {
            id: 4,
            text,
            message_type: MessageType::Narration,
            timestamp: Utc.with_ymd_and_hms(2026, 1, 1, 19, 9, 0).unwrap(),
            ..Default::default()
        });
    }
    Html(story_log_html(&entries))
}

fn story_log_entries_with_extra_oldest() -> Vec<MessageEntry> {
    let mut entries = default_story_log_entries();
    entries.insert(
        0,
        MessageEntry {
            id: 3,
            text: "An older turn the poll will drop.".to_string(),
            message_type: MessageType::Narration,
            timestamp: Utc.with_ymd_and_hms(2026, 1, 1, 19, 6, 0).unwrap(),
            ..Default::default()
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
    let is_poll = path.starts_with("/fragment/") || path == "/status/generating";
    if is_poll && state.polls_failing.load(SeqCst) {
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

async fn header_fragment(State(state): State<Arc<StubState>>) -> Html<String> {
    use std::sync::atomic::Ordering::SeqCst;
    Html(header_html(state.header_degraded.load(SeqCst)))
}

fn swipe_script_entries(text: &str, active_swipe_index: usize) -> Vec<MessageEntry> {
    vec![MessageEntry {
        id: 1,
        text: text.to_string(),
        message_type: MessageType::Narration,
        timestamp: Utc.with_ymd_and_hms(2026, 1, 1, 19, 7, 0).unwrap(),
        swipe_count: 2,
        active_swipe_index,
        ..Default::default()
    }]
}

/// The scripted Message on its second Swipe: the shape a switch test starts
/// from, whose Previous control switches to the first.
fn two_swipe_entries() -> Vec<MessageEntry> {
    swipe_script_entries("A tavern by night.", 1)
}

fn restored_swipe_entries() -> Vec<MessageEntry> {
    swipe_script_entries("A road at dawn.", 0)
}

/// The stub answers a swipe switch with the restored log and drops the dock's
/// option set, mirroring the engine's settled-restore outcome.
async fn switch_swipe(State(state): State<Arc<StubState>>) -> Html<String> {
    use std::sync::atomic::Ordering::SeqCst;
    state.switched.store(true, SeqCst);
    state.dock_options_live.store(false, SeqCst);
    Html(story_log_html(&restored_swipe_entries()))
}

/// The canned dock set until a switch has restored a set-less Swipe.
async fn options_dock_fragment(State(state): State<Arc<StubState>>) -> Html<String> {
    use std::sync::atomic::Ordering::SeqCst;
    if state.dock_options_live.load(SeqCst) {
        Html(options_dock_html(state.dock_options_alt.load(SeqCst)))
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
    // The real handler runs the text check before dispatch; the stub keys on a
    // canned misspelling so a test can drive the preview → confirm swap.
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
        // The real `POST /action/check` acknowledgement retargets the status
        // span rather than replacing the action area: consume the engine's
        // `add_status_swap_headers` builder so the canned shape cannot drift
        // from it.
        StubActionOutcome::Pending => {
            let mut response = (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                r#"<span class="status thinking">Thinking...</span>"#,
            )
                .into_response();
            add_status_swap_headers(&mut response);
            response
        }
        StubActionOutcome::Error => {
            let mut response = (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                render_error(&format!("Failed to process action: {command}")),
            )
                .into_response();
            add_status_swap_headers(&mut response);
            response
        }
    }
}

/// The real `/status/generating` poll answer: idle text, a phase name, or the
/// error fragment a failed generation renders.
async fn status_generating(State(state): State<Arc<StubState>>) -> Response<Body> {
    let status = state
        .status
        .lock()
        .expect("stub status lock poisoned")
        .clone();
    let body = match status {
        StubStatus::Idle => "idle".to_string(),
        StubStatus::Phase(phase) => phase,
        StubStatus::Error(message) => error_disclosure(
            "status-error-popover",
            &generation_error_summary(&message),
            &raw_error_detail(&message),
        ),
    };
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        body,
    )
        .into_response()
}

/// The real `POST /action/confirm` retargets its status fragment at
/// `#status-display`; the stub consumes the engine's `add_status_swap_headers`
/// so the canned shape cannot drift from it.
async fn action_confirm() -> Response<Body> {
    let mut response = (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        r#"<span class="status thinking">Thinking...</span>"#,
    )
        .into_response();
    add_status_swap_headers(&mut response);
    response
}

async fn failing_posture_save() -> Response<Body> {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        render_error("Failed to save posture"),
    )
        .into_response()
}

/// Counts the request before answering 500, so a test can assert the
/// retrigger control fired exactly once.
async fn record_retrigger(State(state): State<Arc<StubState>>) -> Response<Body> {
    state
        .retrigger_requests
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    client_failure("Stub retrigger failure")
}

async fn save_message(State(state): State<Arc<StubState>>) -> Response<Body> {
    if state
        .save_succeeds
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        (StatusCode::OK, "").into_response()
    } else {
        client_failure("Stub save failure")
    }
}

/// A canned 500 for a raw-fetch route whose only stub-tier use is the client
/// failure-recovery path.
fn client_failure(message: &str) -> Response<Body> {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        format!("<p>{message}</p>"),
    )
        .into_response()
}
