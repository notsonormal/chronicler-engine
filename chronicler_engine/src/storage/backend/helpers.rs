//! [DOC: docs/system/storage.md]
//! Shared helper functions for storage backend operations

/// Helper: convert empty string to None for optional fields
pub fn empty_to_none(s: &str) -> Option<&str> {
    if s.is_empty() { None } else { Some(s) }
}
