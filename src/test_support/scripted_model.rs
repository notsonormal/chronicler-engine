//! `ScriptedModel`: the strict, holdable LLM port fake for tier-1 tests.
//!
//! A call the test did not script fails the test: the fake answers it with an
//! error, records it in its stray-call list, and panics when it drops with one
//! on record. A scripted reply that no call used fails the test too, when the
//! fake drops.
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
use crate::domain::model::agent::Role;
use crate::error::{EngineError, NarrationFailure};

/// The words a stray call's failure carries, so an endpoint that cannot reach
/// the fake (a support helper holding only the app) can still spot one.
pub const STRAY_CALL_MARKER: &str = "did not script";

/// The model role a call's agent name stands for: the narrator and the dialogue
/// agent share the narrator's script.
fn role_of_agent_name(agent_name: &str) -> Option<Role> {
    match agent_name {
        AGENT_NARRATOR | AGENT_DIALOGUE => Some(Role::Narrator),
        AGENT_TRIGGER => Some(Role::Trigger),
        AGENT_QUANTIFIER => Some(Role::Quantifier),
        AGENT_OPTIONS => Some(Role::Options),
        _ => None,
    }
}

#[derive(Clone, Debug)]
pub struct ModelCall {
    pub role: Role,
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

struct Scripted {
    position: usize,
    reply: Reply,
    optional: bool,
}

#[derive(Default)]
struct Inner {
    replies: HashMap<Role, VecDeque<Scripted>>,
    scripted: HashMap<Role, usize>,
    /// The role of the reply pushed last, which `optional` marks.
    last_pushed: Option<Role>,
    calls: Vec<ModelCall>,
    stray: Vec<Role>,
}

pub struct ScriptedModel {
    inner: Arc<Mutex<Inner>>,
    hold: Mutex<Option<(Role, Arc<HoldState>)>>,
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
            .after_narration(analysis_report(&[], None))
    }

    #[must_use]
    pub fn narration(self, text: impl Into<String>) -> Self {
        self.reply(Role::Narrator, text)
    }

    #[must_use]
    pub fn event_narration(self, text: impl Into<String>) -> Self {
        self.reply(Role::Trigger, text)
    }

    #[must_use]
    pub fn after_narration(self, text: impl Into<String>) -> Self {
        self.reply(Role::Quantifier, text)
    }

    #[must_use]
    pub fn options(self, text: impl Into<String>) -> Self {
        self.reply(Role::Options, text)
    }

    #[must_use]
    pub fn reply(self, role: Role, text: impl Into<String>) -> Self {
        self.push(role, Reply::Text(text.into()));
        self
    }

    #[must_use]
    pub fn empty(self, role: Role) -> Self {
        self.push(role, Reply::Text(String::new()));
        self
    }

    #[must_use]
    pub fn failed(self, role: Role) -> Self {
        self.push(role, Reply::Failed);
        self
    }

    /// Lets the reply scripted last stay unused. For a reply that a stop or a
    /// cancel can keep a call from taking.
    #[must_use]
    pub fn optional(self) -> Self {
        {
            // arch-lint: allow(no-unwrap-expect) reason="test fake: a poisoned script lock is a broken test"
            let mut inner = self.inner.lock().expect("scripted model script lock");
            let role = inner
                .last_pushed
                // arch-lint: allow(no-unwrap-expect) reason="test fake: `optional` with no reply scripted is a broken test"
                .expect("ScriptedModel: `optional` follows a scripted reply");
            if let Some(last) = inner
                .replies
                .get_mut(&role)
                .and_then(|replies| replies.back_mut())
            {
                last.optional = true;
            }
        }
        self
    }

    fn push(&self, role: Role, reply: Reply) {
        // arch-lint: allow(no-unwrap-expect) reason="test fake: a poisoned script lock is a broken test"
        let mut inner = self.inner.lock().expect("scripted model script lock");
        let count = inner.scripted.entry(role).or_default();
        let position = *count;
        *count += 1;
        inner.last_pushed = Some(role);
        inner.replies.entry(role).or_default().push_back(Scripted {
            position,
            reply,
            optional: false,
        });
    }

    pub fn calls(&self) -> Vec<ModelCall> {
        // arch-lint: allow(no-unwrap-expect) reason="test fake: a poisoned script lock is a broken test"
        let inner = self.inner.lock().expect("scripted model call log lock");
        inner.calls.clone()
    }

    pub fn assert_no_unscripted_calls(&self) {
        assert_no_strays(&self.inner);
    }

    pub fn assert_no_unused_replies(&self) {
        // arch-lint: allow(no-unwrap-expect) reason="test fake: a poisoned script lock is a broken test"
        let inner = self.inner.lock().expect("scripted model unused lock");
        let mut unused = inner
            .replies
            .iter()
            .flat_map(|(role, replies)| {
                replies
                    .iter()
                    .filter(|scripted| !scripted.optional)
                    .map(move |scripted| {
                        let kind = match &scripted.reply {
                            Reply::Text(text) if text.is_empty() => "empty",
                            Reply::Text(_) => "text",
                            Reply::Failed => "failed",
                        };
                        (role.agent_name(), scripted.position, kind)
                    })
            })
            .collect::<Vec<_>>();
        unused.sort_unstable();
        let seen = inner
            .calls
            .iter()
            .map(|call| call.role.agent_name())
            .collect::<Vec<_>>();
        assert!(
            unused.is_empty(),
            "ScriptedModel: no call used {} scripted repl(ies), as (role, position in the script of the role, kind): {unused:?}; its calls: {seen:?}",
            unused.len()
        );
    }

    /// Holds the next `role` call until the returned hold releases it; calls the
    /// test scripts after that one pass through.
    #[must_use]
    pub fn hold(&self, role: Role) -> ModelHold {
        let state = Arc::new(HoldState {
            script: Arc::clone(&self.inner),
            sync: Mutex::new(HoldSync::default()),
            cv: Condvar::new(),
        });
        // arch-lint: allow(no-unwrap-expect) reason="test fake: a poisoned hold lock is a broken test"
        let mut hold = self.hold.lock().expect("scripted model hold lock");
        assert!(
            hold.is_none(),
            "ScriptedModel: a hold is already in place; release it before holding again"
        );
        *hold = Some((role, Arc::clone(&state)));
        ModelHold { state }
    }

    /// Blocks a held call, and answers the hold it fired so the caller can
    /// report that the call took its answer.
    fn block_if_held(&self, role: Role) -> Option<Arc<HoldState>> {
        let state = {
            // arch-lint: allow(no-unwrap-expect) reason="test fake: a poisoned hold lock is a broken test"
            let mut hold = self.hold.lock().expect("scripted model hold lock");
            match hold.as_ref() {
                Some((held_role, state)) if *held_role == role => {
                    let state = Arc::clone(state);
                    *hold = None;
                    state
                }
                _ => return None,
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
        drop(sync);
        Some(state)
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
            Some(scripted) => Ok(scripted.reply),
            None => {
                inner.stray.push(call.role);
                let seen = inner
                    .calls
                    .iter()
                    .map(|call| call.role.agent_name())
                    .collect::<Vec<_>>();
                Err(EngineError::Config(format!(
                    "ScriptedModel: the test {STRAY_CALL_MARKER} a {} call; its calls so far: {seen:?}",
                    call.role.agent_name()
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
        let role = role_of_agent_name(agent_name)
            // arch-lint: allow(no-unwrap-expect) reason="test fake: an unknown agent name is a wiring bug"
            .expect("ScriptedModel: a call names one of the four model roles");
        let call = ModelCall {
            role,
            system_prompt: system_prompt.to_string(),
            user_prompt: user_prompt.to_string(),
            max_tokens,
        };
        let fired = self.block_if_held(role);
        let reply = self.next_reply(&call);
        if let Some(state) = fired {
            state.mark_taken();
        }
        match reply? {
            Reply::Text(text) => Ok(LlmCallResult {
                text: text.clone(),
                raw_request_json: String::new(),
                raw_response_json: format!("{{\"content\":\"{text}\"}}"),
                backend_name: self.name().to_string(),
                model_name: self.model().to_string(),
                agent_name: agent_name.to_string(),
            }),
            Reply::Failed => Err(EngineError::Narration(NarrationFailure::Generation {
                stage: "scripted",
                reason: "configured_failure",
            })),
        }
    }
}

/// A test fails when the fake drops with a stray call on record, the way a panic
/// inside the call itself would fail it — that panic dies in the blocking thread
/// the Job runs on. It fails too when a scripted reply stays unused, unless the
/// test marked that reply `optional`.
impl Drop for ScriptedModel {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            self.assert_no_unscripted_calls();
            self.assert_no_unused_replies();
        }
    }
}

#[derive(Default)]
struct HoldSync {
    started: bool,
    released: bool,
    /// Whether the released call has taken its scripted answer, so a test can
    /// order two held calls: the call released first takes the first answer.
    taken: bool,
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

    /// Waits until the released call has taken its scripted answer. Blocks the
    /// calling thread, like `wait_started`, and answers `false` on timeout.
    pub fn wait_taken(&self, timeout_ms: u64) -> bool {
        // arch-lint: allow(no-unwrap-expect) reason="test fake: a poisoned hold lock is a broken test"
        let sync = self.state.sync.lock().expect("scripted model hold lock");
        let (sync, _) = self
            .state
            .cv
            .wait_timeout_while(sync, Duration::from_millis(timeout_ms), |sync| !sync.taken)
            // arch-lint: allow(no-unwrap-expect) reason="test fake: a poisoned hold lock is a broken test"
            .expect("scripted model hold wait");
        sync.taken
    }

    pub fn release(&self) {
        // arch-lint: allow(no-unwrap-expect) reason="test fake: a poisoned hold lock is a broken test"
        let mut sync = self.state.sync.lock().expect("scripted model hold lock");
        sync.released = true;
        self.state.cv.notify_all();
    }
}

impl HoldState {
    /// Records that the released call has taken its scripted answer.
    fn mark_taken(&self) {
        // arch-lint: allow(no-unwrap-expect) reason="test fake: a poisoned hold lock is a broken test"
        let mut sync = self.sync.lock().expect("scripted model hold lock");
        sync.taken = true;
        self.cv.notify_all();
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

/// A parsed report is a High-confidence analysis, so the Job reports no
/// uncertain-detection notice.
pub fn analysis_report(characters_present: &[&str], movement_destination: Option<&str>) -> String {
    let characters = characters_present
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
    format!("{{\"characters_present\": [{characters}], \"movement\": {movement}}}")
}
