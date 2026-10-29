//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Handler-level utilities: option string and storage-id helpers, and the form of a control that names the last Message.

use axum::body::Body;
use axum::response::Response;
use uuid::Uuid;

use super::error::invalid_answer;

/// An omitted id is an invalid answer, not a form rejection: the field is optional here.
#[derive(serde::Deserialize)]
pub struct LastMessageForm {
    #[serde(default)]
    message: Option<u64>,
}

impl LastMessageForm {
    pub fn named(self) -> Option<u64> {
        self.message
    }
}

pub fn names_no_message() -> Response<Body> {
    invalid_answer("The request names no Message")
}

pub(crate) fn opt_string(value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

/// Storage-key ids: a bare wall-clock id collides when two entries are created
/// in the same millisecond, and the loser is silently overwritten (create →
/// duplicate is the exact flow the UI offers). The uuid-v4 suffix makes
/// same-millisecond creations distinct (residual ~2⁻³² per pair).
pub(crate) fn generate_storage_id(prefix: &str) -> String {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!(
        "{prefix}-{millis}-{}",
        &Uuid::new_v4().simple().to_string()[..8]
    )
}
