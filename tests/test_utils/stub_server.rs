//! Stub-browser server: the real dashboard shell plus canned fragments, with no engine behind it.

// What is real: `assets/index.html` (the shipped shell, `include_str!`-ed) and
// the static assets, served through `ServeDir` exactly as the engine routes
// them.
//
// What is canned: every fragment the shell loads or polls, under
// `tests/test_utils/stub_fixtures/`. The one dynamic endpoint is
// `POST /action/check`, which answers with a scripted outcome the test names up
// front.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Form, State};
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;

use super::server::{get_available_port, release_port_lock};

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
const FIXTURE_OPTIONS_DOCK: &str = include_str!("stub_fixtures/options_dock.html");

/// What `POST /action/check` should answer with. A test names the outcome it
/// needs; the stub runs no pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StubActionOutcome {
    /// "Thinking..." acknowledgement — the shapes a test uses to observe a
    /// pending generation.
    #[default]
    Pending,
    /// A 500 error, as a failing engine action would produce.
    Error,
    /// The plain idle action area, no work started.
    Idle,
}

#[derive(Clone)]
struct StubState {
    action_outcome: StubActionOutcome,
}

/// A running stub server. Dropping it shuts the server down and releases
/// the port lock.
pub struct StubServer {
    addr: SocketAddr,
    shutdown: Option<tokio::sync::oneshot::Sender<()>>,
}

impl StubServer {
    /// Start the stub on a free port from the shared test port range.
    pub async fn start(action_outcome: StubActionOutcome) -> Self {
        let port = get_available_port(3010, 3050).expect("allocate a stub port");
        Self::start_on_port(port, action_outcome).await
    }

    /// Start the stub on a named port.
    async fn start_on_port(port: u16, action_outcome: StubActionOutcome) -> Self {
        let state = Arc::new(StubState { action_outcome });
        let app = stub_router(state);
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
        }
    }

    /// Base URL, e.g. `http://127.0.0.1:3011`.
    pub fn url(&self) -> String {
        format!("http://{}", self.addr)
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
        .route("/status/generating", get(|| async { "idle" }))
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
            get(|| async { Html(FIXTURE_OPTIONS_DOCK) }),
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
    Form(_form): Form<HashMap<String, String>>,
) -> Response<Body> {
    match state.action_outcome {
        // The real `POST /action/check` acknowledgement retargets the status
        // span rather than replacing the action area (`add_status_swap_headers`
        // in the engine). Mirroring both the body and the retarget headers keeps
        // the canned response shape from drifting.
        StubActionOutcome::Pending => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "text/html; charset=utf-8"),
                (
                    header::HeaderName::from_static("hx-retarget"),
                    "#status-display",
                ),
                (header::HeaderName::from_static("hx-reswap"), "innerHTML"),
            ],
            r#"<span class="status thinking">Thinking...</span>"#,
        )
            .into_response(),
        StubActionOutcome::Idle => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            FIXTURE_ACTION_AREA,
        )
            .into_response(),
        StubActionOutcome::Error => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            "<p>Stub action failure</p>",
        )
            .into_response(),
    }
}
