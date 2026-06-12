use std::panic::{self, AssertUnwindSafe};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use super::GenerationGuard;

#[test]
fn test_generation_guard_clears_on_drop() {
    let flag = Arc::new(AtomicBool::new(true));
    {
        let _guard = GenerationGuard(Arc::clone(&flag));
        assert!(
            flag.load(Ordering::SeqCst),
            "flag should be true while guard lives"
        );
    }
    assert!(
        !flag.load(Ordering::SeqCst),
        "GenerationGuard did not clear flag on drop"
    );
}

#[test]
fn test_generation_guard_clears_on_panic() {
    let flag = Arc::new(AtomicBool::new(true));
    let flag_clone = Arc::clone(&flag);

    let result = panic::catch_unwind(AssertUnwindSafe(move || {
        let _guard = GenerationGuard(flag_clone);
        panic!("intentional panic to test guard drop");
    }));

    assert!(result.is_err(), "panic should have occurred");
    assert!(
        !flag.load(Ordering::SeqCst),
        "GenerationGuard did not clear flag on panic"
    );
}
