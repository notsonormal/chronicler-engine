//! [DOC: docs/system/dashboard.md]
//! Generation guard logic

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// RAII guard that clears the `is_generating` flag on drop.
/// Used in `spawn_blocking` to ensure the flag is always released
/// even if the background task panics.
pub struct GenerationGuard(pub Arc<AtomicBool>);

impl Drop for GenerationGuard {
    fn drop(&mut self) {
        tracing::debug!("GenerationGuard: dropping, setting is_generating to false");
        self.0.store(false, Ordering::SeqCst);
    }
}
