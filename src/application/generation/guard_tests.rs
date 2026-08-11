use std::collections::HashMap;
use std::panic::{self, AssertUnwindSafe};
use std::sync::Arc;
use std::sync::RwLock;

use crate::application::generation::slot::GenerationSlot;

use crate::application::generation::guard::GenerationGuard;

#[test]
fn test_generation_guard_clears_on_drop() {
    let registry = Arc::new(RwLock::new(HashMap::new()));
    registry
        .write()
        .unwrap()
        .insert(1, GenerationSlot::Generating { generation_id: 1 });
    {
        let _guard = GenerationGuard::new(1, 1, Arc::clone(&registry));
        assert!(
            registry.read().unwrap().get(&1).unwrap().is_generating(),
            "slot should be generating while guard lives"
        );
    }
    assert!(
        !registry.read().unwrap().get(&1).unwrap().is_generating(),
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
    let registry_for_thread = Arc::clone(&registry);

    let result = panic::catch_unwind(AssertUnwindSafe(move || {
        let _guard = GenerationGuard::new(1, 1, registry_for_thread);
        panic!("intentional panic to test guard drop");
    }));

    assert!(result.is_err(), "panic should have occurred");
    assert!(
        !registry.read().unwrap().get(&1).unwrap().is_generating(),
        "GenerationGuard did not clear registry slot on panic"
    );
}
