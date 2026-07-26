use std::collections::HashMap;
use std::panic::{self, AssertUnwindSafe};
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::application::utils::slot::GenerationSlot;

use super::GenerationGuard;

#[test]
fn test_generation_guard_clears_on_drop() {
    let registry = Arc::new(RwLock::new(HashMap::new()));
    registry
        .write()
        .unwrap()
        .insert(1, GenerationSlot::Generating { generation_id: 1 });
    let flag = Arc::new(AtomicBool::new(true));
    {
        let _guard = GenerationGuard::new(1, 1, Arc::clone(&registry), Arc::clone(&flag));
        assert!(
            flag.load(Ordering::SeqCst),
            "flag should be true while guard lives"
        );
    }
    assert!(
        !flag.load(Ordering::SeqCst),
        "GenerationGuard did not clear flag on drop"
    );
    assert_eq!(
        *registry.read().unwrap().get(&1).unwrap(),
        GenerationSlot::Idle,
        "GenerationGuard did not clear registry slot on drop"
    );
}

#[test]
fn test_generation_guard_clears_on_panic() {
    let registry = Arc::new(RwLock::new(HashMap::new()));
    registry
        .write()
        .unwrap()
        .insert(1, GenerationSlot::Generating { generation_id: 1 });
    let flag = Arc::new(AtomicBool::new(true));
    let registry_for_thread = Arc::clone(&registry);
    let flag_clone = Arc::clone(&flag);

    let result = panic::catch_unwind(AssertUnwindSafe(move || {
        let _guard = GenerationGuard::new(1, 1, registry_for_thread, flag_clone);
        panic!("intentional panic to test guard drop");
    }));

    assert!(result.is_err(), "panic should have occurred");
    assert!(
        !flag.load(Ordering::SeqCst),
        "GenerationGuard did not clear flag on panic"
    );
    assert_eq!(
        *registry.read().unwrap().get(&1).unwrap(),
        GenerationSlot::Idle,
        "GenerationGuard did not clear registry slot on panic"
    );
}
