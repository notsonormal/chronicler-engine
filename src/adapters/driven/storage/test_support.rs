//! [DOC: docs/diataxis/reference/storage.md]
//! Test infrastructure types for storage failure injection

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::adapters::driven::storage::transaction::TxWrite;
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
    pub(crate) writes: Arc<Mutex<HashMap<TxWrite, TestOverride>>>,
}

impl TestFailureHandle {
    pub fn set(&self, method: &'static str, override_: TestOverride) {
        self.overrides
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(method, override_);
    }

    pub fn set_write(&self, write: TxWrite, override_: TestOverride) {
        self.writes
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(write, override_);
    }

    pub fn clear(&self, method: &'static str) {
        self.overrides
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(method);
    }

    pub fn clear_write(&self, write: TxWrite) {
        self.writes
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&write);
    }

    pub fn clear_all(&self) {
        self.overrides
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
        self.writes
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
    }

    /// The keys still standing in the call and record-write override maps.
    fn unconsumed_overrides(&self) -> (Vec<&'static str>, Vec<TxWrite>) {
        let calls = self
            .overrides
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .keys()
            .cloned()
            .collect();
        let writes = self
            .writes
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .keys()
            .cloned()
            .collect();
        (calls, writes)
    }

    #[allow(clippy::panic)]
    pub fn assert_no_unconsumed(&self) {
        let (calls, writes) = self.unconsumed_overrides();
        if !calls.is_empty() || !writes.is_empty() {
            panic!("Unconsumed overrides remain: calls={calls:?}, writes={writes:?}");
        }
    }
}

impl Drop for TestFailureHandle {
    fn drop(&mut self) {
        let (calls, writes) = self.unconsumed_overrides();
        if !calls.is_empty() || !writes.is_empty() {
            tracing::warn!(
                "TestFailureHandle dropped with unconsumed overrides: calls={calls:?}, writes={writes:?}"
            );
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
