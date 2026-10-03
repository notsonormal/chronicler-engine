//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Trim-and-compare name uniqueness shared by the settings and prompt-preset services.

/// Whether `candidate` is unused among the existing `(id, name)` entries,
/// comparing trimmed, case-insensitive names. `except_id` names the entry the
/// candidate belongs to, so renaming an entry in place never collides with
/// itself.
pub(crate) fn name_is_available<'a>(
    existing: impl IntoIterator<Item = (&'a str, &'a str)>,
    candidate: &str,
    except_id: Option<&str>,
) -> bool {
    let normalized = candidate.trim().to_lowercase();
    existing
        .into_iter()
        .all(|(id, name)| except_id == Some(id) || name.trim().to_lowercase() != normalized)
}
