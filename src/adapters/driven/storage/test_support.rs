//! [DOC: docs/diataxis/reference/storage.md]
//! Test infrastructure types for storage failure injection

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::error::{EngineError, InternalError};

pub struct TestOverride {
    kind: ErrorKind,
    message: String,
}

/// [TRIVIAL_ENUM]
#[derive(Clone, Copy)]
pub(crate) enum ErrorKind {
    Config,
    Internal,
}

pub struct TestFailureHandle {
    pub(crate) overrides: Arc<Mutex<HashMap<&'static str, TestOverride>>>,
}

impl TestFailureHandle {
    pub fn set(&self, method: &'static str, override_: TestOverride) {
        self.overrides
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(method, override_);
    }

    pub fn clear(&self, method: &'static str) {
        self.overrides
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(method);
    }

    pub fn clear_all(&self) {
        self.overrides
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
    }

    #[allow(clippy::panic)]
    pub fn assert_no_unconsumed(&self) {
        let map = self.overrides.lock().unwrap_or_else(|e| e.into_inner());
        if !map.is_empty() {
            let keys: Vec<_> = map.keys().cloned().collect();
            panic!("Unconsumed overrides remain: {keys:?}");
        }
    }
}

impl Drop for TestFailureHandle {
    fn drop(&mut self) {
        let map = self.overrides.lock().unwrap_or_else(|e| e.into_inner());
        if !map.is_empty() {
            let keys: Vec<_> = map.keys().cloned().collect();
            tracing::warn!("TestFailureHandle dropped with unconsumed overrides: {keys:?}");
        }
    }
}

impl TestOverride {
    pub fn config(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Config,
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Internal,
            message: message.into(),
        }
    }

    pub(crate) fn to_error(&self) -> EngineError {
        match self.kind {
            ErrorKind::Config => EngineError::Config(self.message.clone()),
            ErrorKind::Internal => EngineError::Internal(InternalError::new(self.message.clone())),
        }
    }
}
