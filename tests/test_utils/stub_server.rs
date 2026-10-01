//! Stub-browser server: the real dashboard shell plus canned fragments, with no engine behind it.

// What is real: `assets/index.html` (the shipped shell, `include_str!`-ed) and
// the static assets, served through `ServeDir` exactly as the engine routes
// them.
//
// What is canned: every fragment the shell loads or polls, under
// `tests/test_utils/stub_fixtures/` — except the options dock, which is
// rendered through the engine's own `OptionsDockTemplate` (a pure vm → HTML
// render, so the drift tax there is avoidable). The dynamic endpoints answer
// scripted outcomes a test names up front: `POST /action/check`, `POST
// /action/confirm`, `POST /check-text`, and `GET /status/generating` (whose
// `StubStatus` a live test may change between polls). `POST /history/:id` and
// `POST /swipe/new` always answer 500.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Form, State};
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;

use chronicler_engine::adapters::driving::http::builders::headers::add_status_swap_headers;
use chronicler_engine::adapters::driving::http::utils::error::render_error;

use super::server::{get_config_port, release_port_lock};
use super::CONFIG_PATH;

/// The shipped dashboard shell, served verbatim so the stub exercises the real
/// htmx wiring.
const DASHBOARD_SHELL: &str = include_str!("../../assets/index.html");

const FIXTURE_STORY_LOG: &str = include_str!("stub_fixtures/story_log.html");
const FIXTURE_VISUAL_SIDEBAR: &str = include_str!("stub_fixtures/visual_sidebar.html");
const FIXTURE_LLM_MESSAGES: &str = include_str!("stub_fixtures/llm_messages.html");
const FIXTURE_HEADER: &str = include_str!("stub_fixtures/header.html");
const FIXTURE_SETTINGS: &str = include_str!("stub_fixtures/settings.html");
const FIXTURE_PROMPT_PRESETS: &str = include_str!("stub_fixtures/prompt_presets.html");
const FIXTURE_WORLDS: &str = include_str!("stub_fixtures/worlds.html");
const FIXTURE_GAMES: &str = include_str!("stub_fixtures/games.html");
const FIXTURE_ACTION_AREA: &str = include_str!("stub_fixtures/action_area.html");

/// Canned options the rendered dock offers — the three texts the old fixture
/// carried, which the stub-tier tests interact with.
const CANNED_OPTIONS: [&str; 3] = [
    "Ask the bartender about the Test Realm",
    "Examine the merchant's pack",
    "Step outside into the night air",
];

/// Render the options dock through the engine's own template — the dock is a
/// pure `vm → HTML` render needing no `AppState`, so serving a hand-copy
/// fixture would be drift we pay for nothing. Askama escapes the texts.
fn options_dock_html() -> String {
    use askama::Template;
    use chronicler_engine::adapters::driving::http::templates::OptionsDockTemplate;
    use chronicler_engine::adapters::driving::http::view_models::OptionsDockViewModel;

    let vm = OptionsDockViewModel::new(
        CANNED_OPTIONS.iter().map(|o| o.to_string()).collect(),
        false,
    );
    OptionsDockTemplate::new(vm)
        .render()
        .expect("render options dock")
}

/// The command the stub treats as a text check with issues. The real
/// `POST /action/check` runs the text check before dispatch and returns the
/// preview for a misspelling; the stub keys on this one token so a test can
/// drive the preview → confirm swap without a real text-check engine.
const TEXT_CHECK_TRIGGER: &str = "casle";

/// Render the text-check preview through the engine's own template — a pure
/// `vm → HTML` render, so the canned preview cannot drift from the shipped
/// `TextCheckPreviewTemplate`.
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
/// "idle", a phase name, or the error span a failed generation produces.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum StubStatus {
    /// The engine is idle; the poll answers "idle".
    #[default]
    Idle,
    /// A failed generation; the poll answers the error span.
    Error(String),
}

#[derive(Clone)]
struct StubState {
    action_outcome: StubActionOutcome,
    status: Arc<std::sync::Mutex<StubStatus>>,
}

/// A running stub server. Dropping it shuts the server down and releases
/// the port lock.
pub struct StubServer {
    addr: SocketAddr,
    shutdown: Option<tokio::sync::oneshot::Sender<()>>,
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

    /// Start the stub on a named port.
    async fn start_on_port(port: u16, action_outcome: StubActionOutcome) -> Self {
        let state = Arc::new(StubState {
            action_outcome,
            status: Arc::new(std::sync::Mutex::new(StubStatus::default())),
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
            shutdown: Some(tx),
            state,
        }
    }

    /// Base URL, e.g. `http://127.0.0.1:3011`.
    pub fn url(&self) -> String {
        format!("http://{}", self.addr)
    }

    /// A cloneable handle to the scripted status, so a test closure can change
    /// what `/status/generating` answers without borrowing the server.
    pub fn status_handle(&self) -> StubStatusHandle {
        StubStatusHandle {
            state: Arc::clone(&self.state),
        }
    }
}

/// A cloneable handle to the stub's scripted status.
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

impl Drop for StubServer {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown.take() {
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
        .route("/check-text", post(check_text))
        // Client-side failure recovery: the save and retry paths are raw
        // `fetch`, so their failure handling is stub-tier behaviour with no
        // htmx swap. These routes always answer 500; no stub-tier test drives
        // a successful save/retry through the client JS.
        .route(
            "/history/:id",
            post(|| async { client_failure("Stub save failure") }),
        )
        .route(
            "/swipe/new",
            post(|| async { client_failure("Stub retry failure") }),
        )
        .route("/status/generating", get(status_generating))
        .route("/fragment/header", get(|| async { Html(FIXTURE_HEADER) }))
        .route(
            "/fragment/story-log",
            get(|| async { Html(FIXTURE_STORY_LOG) }),
        )
        .route(
            "/fragment/visual-sidebar",
            get(|| async { Html(FIXTURE_VISUAL_SIDEBAR) }),
        )
        .route(
            "/fragment/options-dock",
            get(|| async { Html(options_dock_html()) }),
        )
        .route(
            "/fragment/llm-messages",
            get(|| async { Html(FIXTURE_LLM_MESSAGES) }),
        )
        .route(
            "/fragment/settings",
            get(|| async { Html(FIXTURE_SETTINGS) }),
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
        .with_state(state)
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
        StubActionOutcome::Error => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            render_error(&format!("Failed to process action: {command}")),
        )
            .into_response(),
    }
}

/// The real `/status/generating` poll answer: idle text, a phase name, or the
/// error span a failed generation renders.
async fn status_generating(State(state): State<Arc<StubState>>) -> Response<Body> {
    let status = state
        .status
        .lock()
        .expect("stub status lock poisoned")
        .clone();
    let body = match status {
        StubStatus::Idle => "idle".to_string(),
        StubStatus::Error(message) => {
            format!(r#"<span class="status error">Error: {message}</span>"#)
        }
    };
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        body,
    )
        .into_response()
}

/// The real `POST /action/confirm` swaps a fresh `#action-area` back in
/// (`hx-swap="outerHTML"`); the stub serves the canned action area.
async fn action_confirm() -> Html<&'static str> {
    Html(FIXTURE_ACTION_AREA)
}

/// The real `POST /check-text` renders its result into the shell's
/// `#text-check-result` element; the stub serves the disabled-mode body.
async fn check_text() -> Response<Body> {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        r#"<span class="status ready">Text check is disabled</span>"#,
    )
        .into_response()
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
