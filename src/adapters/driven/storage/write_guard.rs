//! [DOC: docs/diataxis/reference/storage.md]
//! The write guard: the one place a `Tx` write checks its injected failure and records the kinds it changed

use crate::adapters::driven::storage::changes::ChangeCounters;
use crate::adapters::driven::storage::core::WriteOverrides;
use crate::adapters::driven::storage::transaction::TxWrite;
use crate::domain::model::change::ChangeKind;
use crate::error::EngineError;

/// Its fields are private to this file, so a write reaches the failure injection and
/// the record of kinds only through `write`. A write that skips `write` cannot compile,
/// and a write that goes through it names its `TxWrite` once, for both.
pub(super) struct WriteGuard {
    writes: WriteOverrides,
    changed: Vec<ChangeKind>,
}

impl WriteGuard {
    pub(super) fn new(writes: WriteOverrides) -> Self {
        Self {
            writes,
            changed: Vec::new(),
        }
    }

    /// Fails with the injected failure of `write` before `body` runs. When `body`
    /// succeeds, the kinds of `write` join the kinds of the unit, each once.
    pub(super) fn write<T>(
        &mut self,
        write: TxWrite,
        body: impl FnOnce() -> Result<T, EngineError>,
    ) -> Result<T, EngineError> {
        self.writes.fail_if_injected(write)?;
        let value = body()?;
        for &kind in ChangeCounters::kinds_of_write(write) {
            if !self.changed.contains(&kind) {
                self.changed.push(kind);
            }
        }
        Ok(value)
    }

    /// The kinds of the writes that succeeded.
    pub(super) fn into_changed(self) -> Vec<ChangeKind> {
        self.changed
    }
}
