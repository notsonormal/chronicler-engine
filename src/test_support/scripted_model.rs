//! `ScriptedModel`: the strict, holdable LLM port fake for tier-1 tests.
//!
//! A call the test did not script fails the test: the fake answers it with an
//! error, records it in its stray-call list, and panics when it drops with one
//! on record.
//!
//! `MockBackend` stays the lenient runtime provider of `LlmBackendType::Mock`.
#![allow(clippy::expect_used)]

use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use crate::application::ports::llm_provider::{
    LlmCallResult, LlmProvider, AGENT_DIALOGUE, AGENT_NARRATOR, AGENT_OPTIONS, AGENT_QUANTIFIER,
    AGENT_TRIGGER,
};
use crate::error::{EngineError, NarrativeFailure};

/// The words a stray call's failure carries, so an endpoint that cannot reach
/// the fake (a support helper holding only the app) can still spot one.
pub const STRAY_CALL_MARKER: &str = "did not script";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ModelRole {
    /// The turn's narration (`narrator`, `dialogue`).
    Narration,
    /// An event's narration (`trigger`).
    Event,
    /// The analysis of the narration (`quantifier`).
    Analysis,
    /// The offered-set generation (`options`).
    Options,
}

impl ModelRole {
    pub fn of_agent_name(agent_name: &str) -> Option<Self> {
        match agent_name {
            AGENT_NARRATOR | AGENT_DIALOGUE => Some(Self::Narration),
            AGENT_TRIGGER => Some(Self::Event),
            AGENT_QUANTIFIER => Some(Self::Analysis),
            AGENT_OPTIONS => Some(Self::Options),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Narration => "narration",
            Self::Event => "event narration",
            Self::Analysis => "analysis",
            Self::Options => "options",
        }
    }
}

#[derive(Clone, Debug)]
pub struct ModelCall {
    pub role: ModelRole,
    pub system_prompt: String,
    pub user_prompt: String,
    pub max_tokens: Option<u32>,
}

#[derive(Clone, Debug)]
enum Reply {
    /// The answer the call returns.
    Text(String),
    /// A failure at the transport seam.
    Failed,
}

#[derive(Default)]
struct Inner {
    replies: HashMap<ModelRole, VecDeque<Reply>>,
    calls: Vec<ModelCall>,
    stray: Vec<ModelRole>,
}

pub struct ScriptedModel {
    inner: Arc<Mutex<Inner>>,
    hold: Mutex<Option<(ModelRole, Arc<HoldState>)>>,
}

impl Default for ScriptedModel {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptedModel {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner::default())),
            hold: Mutex::new(None),
        }
    }

    #[must_use]
    pub fn turn(self, narration: impl Into<String>) -> Self {
        self.narration(narration)
            .analysis(analysis_report(&[], None))
    }

    #[must_use]
    pub fn narration(self, text: impl Into<String>) -> Self {
        self.reply(ModelRole::Narration, text)
    }

    #[must_use]
    pub fn event_narration(self, text: impl Into<String>) -> Self {
        self.reply(ModelRole::Event, text)
    }

    #[must_use]
    pub fn analysis(self, text: impl Into<String>) -> Self {
        self.reply(ModelRole::Analysis, text)
    }

    #[must_use]
    pub fn options(self, text: impl Into<String>) -> Self {
        self.reply(ModelRole::Options, text)
    }

    #[must_use]
    pub fn reply(self, role: ModelRole, text: impl Into<String>) -> Self {
        self.push(role, Reply::Text(text.into()));
        self
    }

    #[must_use]
    pub fn empty(self, role: ModelRole) -> Self {
        self.push(role, Reply::Text(String::new()));
        self
    }

    #[must_use]
    pub fn failed(self, role: ModelRole) -> Self {
        self.push(role, Reply::Failed);
        self
    }

    fn push(&self, role: ModelRole, reply: Reply) {
        // arch-lint: allow(no-unwrap-expect) reason="test fake: a poisoned script lock is a broken test"
        let mut inner = self.inner.lock().expect("scripted model script lock");
        inner.replies.entry(role).or_default().push_back(reply);
    }

    pub fn calls(&self) -> Vec<ModelCall> {
        // arch-lint: allow(no-unwrap-expect) reason="test fake: a poisoned script lock is a broken test"
        let inner = self.inner.lock().expect("scripted model call log lock");
        inner.calls.clone()
    }

    pub fn assert_no_unscripted_calls(&self) {
        assert_no_strays(&self.inner);
    }

    /// Holds the next `role` call until the returned hold releases it; calls the
    /// test scripts after that one pass through.
    #[must_use]
    pub fn hold(&self, role: ModelRole) -> ModelHold {
        let state = Arc::new(HoldState {
            script: Arc::clone(&self.inner),
            sync: Mutex::new(HoldSync::default()),
            cv: Condvar::new(),
        });
        // arch-lint: allow(no-unwrap-expect) reason="test fake: a poisoned hold lock is a broken test"
        let mut hold = self.hold.lock().expect("scripted model hold lock");
        *hold = Some((role, Arc::clone(&state)));
        ModelHold { state }
    }

    fn block_if_held(&self, role: ModelRole) {
        let state = {
            // arch-lint: allow(no-unwrap-expect) reason="test fake: a poisoned hold lock is a broken test"
            let mut hold = self.hold.lock().expect("scripted model hold lock");
            match hold.as_ref() {
                Some((held_role, state)) if *held_role == role => {
                    let state = Arc::clone(state);
                    *hold = None;
                    state
                }
                _ => return,
            }
        };
        // arch-lint: allow(no-unwrap-expect) reason="test fake: a poisoned hold lock is a broken test"
        let mut sync = state.sync.lock().expect("scripted model hold wait lock");
        sync.started = true;
        state.cv.notify_all();
        while !sync.released {
            // arch-lint: allow(no-unwrap-expect) reason="test fake: a poisoned hold lock is a broken test"
            sync = state.cv.wait(sync).expect("scripted model hold wait");
        }
    }

    fn next_reply(&self, call: &ModelCall) -> Result<Reply, EngineError> {
        // arch-lint: allow(no-unwrap-expect) reason="test fake: a poisoned script lock is a broken test"
        let mut inner = self.inner.lock().expect("scripted model script lock");
        inner.calls.push(call.clone());
        let reply = inner
            .replies
            .get_mut(&call.role)
            .and_then(VecDeque::pop_front);
        match reply {
            Some(reply) => Ok(reply),
            None => {
                inner.stray.push(call.role);
                let seen = inner
                    .calls
                    .iter()
                    .map(|call| call.role.label())
                    .collect::<Vec<_>>();
                Err(EngineError::Config(format!(
                    "ScriptedModel: the test {STRAY_CALL_MARKER} a {} call; its calls so far: {seen:?}",
                    call.role.label()
                )))
            }
        }
    }
}

impl LlmProvider for ScriptedModel {
    fn model(&self) -> &str {
        "scripted"
    }

    fn name(&self) -> &str {
        "Scripted"
    }

    fn complete(
        &self,
        agent_name: &str,
        system_prompt: &str,
        user_prompt: &str,
        max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        let role = ModelRole::of_agent_name(agent_name)
            // arch-lint: allow(no-unwrap-expect) reason="test fake: an unknown agent name is a wiring bug"
            .expect("ScriptedModel: a call names one of the four model roles");
        let call = ModelCall {
            role,
            system_prompt: system_prompt.to_string(),
            user_prompt: user_prompt.to_string(),
            max_tokens,
        };
        self.block_if_held(role);
        match self.next_reply(&call)? {
            Reply::Text(text) => Ok(LlmCallResult {
                text: text.clone(),
                raw_request_json: String::new(),
                raw_response_json: format!("{{\"content\":\"{text}\"}}"),
                backend_name: self.name().to_string(),
                model_name: self.model().to_string(),
                agent_name: agent_name.to_string(),
            }),
            Reply::Failed => Err(EngineError::Narrative(NarrativeFailure::Generation {
                stage: "scripted",
                reason: "configured_failure",
            })),
        }
    }
}

/// A test that leaves a stray call on record fails when the fake drops, the way
/// a panic inside the call itself would fail it — that panic dies in the
/// blocking thread the Job runs on.
impl Drop for ScriptedModel {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            self.assert_no_unscripted_calls();
        }
    }
}

#[derive(Default)]
struct HoldSync {
    started: bool,
    released: bool,
}

struct HoldState {
    /// The fake's record, so the hold's drop can fail a stray call too.
    script: Arc<Mutex<Inner>>,
    sync: Mutex<HoldSync>,
    cv: Condvar,
}

fn assert_no_strays(inner: &Mutex<Inner>) {
    // arch-lint: allow(no-unwrap-expect) reason="test fake: a poisoned script lock is a broken test"
    let inner = inner.lock().expect("scripted model stray lock");
    assert!(
        inner.stray.is_empty(),
        "ScriptedModel: the test did not script {} call(s): {:?}",
        inner.stray.len(),
        inner.stray
    );
}

pub struct ModelHold {
    state: Arc<HoldState>,
}

impl ModelHold {
    /// Waits until the held call reached the fake. Blocks the calling thread,
    /// because the held call runs on the blocking pool and never on a runtime
    /// worker. `false` on timeout.
    pub fn wait_started(&self, timeout_ms: u64) -> bool {
        // arch-lint: allow(no-unwrap-expect) reason="test fake: a poisoned hold lock is a broken test"
        let sync = self.state.sync.lock().expect("scripted model hold lock");
        let (sync, _) = self
            .state
            .cv
            .wait_timeout_while(sync, Duration::from_millis(timeout_ms), |sync| {
                !sync.started
            })
            // arch-lint: allow(no-unwrap-expect) reason="test fake: a poisoned hold lock is a broken test"
            .expect("scripted model hold wait");
        sync.started
    }

    pub fn release(&self) {
        // arch-lint: allow(no-unwrap-expect) reason="test fake: a poisoned hold lock is a broken test"
        let mut sync = self.state.sync.lock().expect("scripted model hold lock");
        sync.released = true;
        self.state.cv.notify_all();
    }
}

/// Releases the hold so a test that panics before `release` cannot strand the
/// held call, and reports a stray call the test made while it held one.
impl Drop for ModelHold {
    fn drop(&mut self) {
        self.release();
        if !std::thread::panicking() {
            assert_no_strays(&self.state.script);
        }
    }
}

/// A parsed report is a High-confidence analysis, so the turn reports no
/// uncertain-detection notice.
pub fn analysis_report(npcs_in_room: &[&str], movement_destination: Option<&str>) -> String {
    let npcs = npcs_in_room
        .iter()
        .map(|id| format!("\"{id}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let movement = match movement_destination {
        Some(destination) => {
            format!("{{\"type\": \"Entering\", \"destination\": \"{destination}\"}}")
        }
        None => "null".to_string(),
    };
    format!("{{\"npcs_in_room\": [{npcs}], \"movement\": {movement}}}")
}
