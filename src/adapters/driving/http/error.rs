//! [DOC: docs/diataxis/reference/architecture_system.md]
//! HTTP driving adapter — maps application `ApplicationError` to axum `Response`.

use axum::response::{IntoResponse, Response};

use super::utils::error::{AnswerPolicy, answer_for_error};

use crate::application::errors::ApplicationError;
use crate::application::generation::gate::NotAccepted;

impl IntoResponse for ApplicationError {
    fn into_response(self) -> Response {
        answer_for_error(self, None, AnswerPolicy::GenericText)
    }
}

impl IntoResponse for NotAccepted {
    fn into_response(self) -> Response {
        ApplicationError::NotAccepted(self).into_response()
    }
}
