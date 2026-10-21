//! [DOC: docs/diataxis/reference/storage.md]
//! Test infrastructure types for storage failure injection

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::adapters::driven::storage::core::StorageCall;
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
    pub(crate) overrides: Arc<Mutex<HashMap<StorageCall, TestOverride>>>,
    pub(crate) writes: Arc<Mutex<HashMap<TxWrite, TestOverride>>>,
}

impl TestFailureHandle {
    pub fn set(&self, call: StorageCall, override_: TestOverride) {
        self.overrides
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(call, override_);
    }

    pub fn set_write(&self, write: TxWrite, override_: TestOverride) {
        self.writes
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(write, override_);
    }

    pub fn clear(&self, call: StorageCall) {
        self.overrides
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&call);
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
