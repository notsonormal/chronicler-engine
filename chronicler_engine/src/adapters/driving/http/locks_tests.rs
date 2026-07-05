use std::sync::{Arc, RwLock};
use std::thread;

use super::locks::{read_lock_or_recover, write_lock_or_recover};

fn poison<T: Send + Sync + 'static>(lock: &Arc<RwLock<T>>) {
    let clone = lock.clone();
    let _ = thread::spawn(move || {
        let _guard = clone.write().unwrap();
        panic!("intentional panic to poison the lock");
    })
    .join();
}

#[test]
fn test_read_lock_or_recover_returns_clone() {
    let lock = RwLock::new(vec![1, 2, 3]);
    assert_eq!(read_lock_or_recover(&lock, "test"), vec![1, 2, 3]);
}

#[test]
fn test_write_lock_or_recover_allows_mutation() {
    let lock = RwLock::new(10u32);
    {
        let mut guard = write_lock_or_recover(&lock, "test");
        *guard += 5;
    }
    assert_eq!(*lock.read().unwrap(), 15);
}

#[test]
fn test_read_lock_recovers_from_poison() {
    let lock = Arc::new(RwLock::new(vec![7, 8]));
    poison(&lock);
    assert!(lock.read().is_err(), "lock should be poisoned");

    assert_eq!(read_lock_or_recover(&lock, "test"), vec![7, 8]);
}

#[test]
fn test_write_lock_recovers_from_poison() {
    let lock = Arc::new(RwLock::new(3u32));
    poison(&lock);
    assert!(lock.write().is_err(), "lock should be poisoned");

    let mut guard = write_lock_or_recover(&lock, "test");
    *guard += 1;
    assert_eq!(*guard, 4);
}
