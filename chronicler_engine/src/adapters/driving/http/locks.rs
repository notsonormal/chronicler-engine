//! [DOC: docs/system/dashboard.md]
//! Shared poison-recovering lock helpers for the HTTP layer.

use std::sync::{RwLock, RwLockWriteGuard};

pub fn read_lock_or_recover<T: Clone>(lock: &RwLock<T>, name: &str) -> T {
    lock.read().map(|g| g.clone()).unwrap_or_else(|p| {
        tracing::warn!("Poisoned {name} read lock recovered");
        p.into_inner().clone()
    })
}

pub fn write_lock_or_recover<'a, T>(lock: &'a RwLock<T>, name: &str) -> RwLockWriteGuard<'a, T> {
    lock.write().unwrap_or_else(|p| {
        tracing::warn!("Poisoned {name} write lock recovered");
        p.into_inner()
    })
}
