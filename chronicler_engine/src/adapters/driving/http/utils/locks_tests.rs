//! Unit tests for poison-recovering lock helpers.

use std::sync::Arc;

use super::write_lock_or_recover;

#[test]
fn test_write_lock_recover_from_poisoned_rwlock() {
    let lock = Arc::new(std::sync::RwLock::new(0));

    let lock_clone = Arc::clone(&lock);
    let _ = std::thread::spawn(move || {
        let mut guard = lock_clone.write().unwrap();
        *guard = 42;
        panic!("intentional panic to poison lock");
    })
    .join();

    let recovered = write_lock_or_recover(&lock, "test");
    assert_eq!(*recovered, 42);
}
