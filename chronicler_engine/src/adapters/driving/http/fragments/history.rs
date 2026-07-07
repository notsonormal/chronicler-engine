//! [DOC: docs/system/dashboard.md]
//! History fragment handlers

use axum::{
    extract::{Form, Path},
    response::Response,
};

use crate::application::application_service::ApplicationError;
use crate::application::context::OpContext;
use crate::application::message_editing;

use super::renderers::ok;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct EditHistoryForm {
    pub text: String,
}

pub async fn edit_history_handler(
    Path(id): Path<u64>,
    ctx: OpContext,
    Form(form): Form<EditHistoryForm>,
) -> Result<Response, ApplicationError> {
    message_editing::edit_history(ctx, id, form.text)?;
    Ok(ok("<span class=\"status ready\">Edited</span>"))
}

pub async fn delete_history_handler(ctx: OpContext) -> Result<Response, ApplicationError> {
    message_editing::delete_last(ctx)?;
    Ok(ok(""))
}
