//! [DOC: docs/diataxis/reference/storage.md]
//! Trim-and-compare name uniqueness shared by the settings/preset services and preset storage.

/// `except_id` exempts the entry being renamed so it never collides with itself.
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
