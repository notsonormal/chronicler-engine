
use crate::test_support::forensics::ForensicsCollector;

#[test]
fn test_forensics_collector_creation() {
    let collector = ForensicsCollector::new();
    // Verify it constructs without panic
    drop(collector);
}

#[test]
fn test_set_test_name() {
    let collector = ForensicsCollector::new();
    collector.set_test_name("test_example");
    // No panic = success
    drop(collector);
}
