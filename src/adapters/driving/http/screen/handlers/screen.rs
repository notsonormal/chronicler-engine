//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! The screen load: one request that renders the parts the page names.

use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use super::parts::{PartRenderer, SCREEN_PARTS};
use crate::adapters::driving::http::utils::error::render_error;
use crate::adapters::driving::http::utils::response::{bad_request, internal_error};
use crate::adapters::driving::http::AppState;

const REVISION_HEADER: &str = "x-revision";
const GAME_HEADER: &str = "x-game";

#[derive(Debug, Deserialize)]
pub struct ScreenQuery {
    parts: Option<String>,
}

/// The renderers of the named parts, in the order of the request. A missing,
/// unknown or repeated name is a refusal that names it.
fn renderers_named(parts: Option<&str>) -> Result<Vec<(&'static str, PartRenderer)>, String> {
    let names: Vec<&str> = parts
        .map(|list| list.split(',').map(str::trim).collect())
        .unwrap_or_default();
    if names.iter().all(|name| name.is_empty()) {
        return Err("The parts query names no part".to_string());
    }
    let mut chosen: Vec<(&'static str, PartRenderer)> = Vec::new();
    for name in names {
        let Some(&(part, renderer)) = SCREEN_PARTS.iter().find(|(part, _)| *part == name) else {
            return Err(format!("Unknown screen part \"{name}\""));
        };
        if chosen.iter().any(|(chosen_part, _)| *chosen_part == part) {
            return Err(format!("Screen part \"{name}\" is named twice"));
        }
        chosen.push((part, renderer));
    }
    Ok(chosen)
}

/// `GET /screen?parts=a,b`: one `<template data-part>` for each named part, with the
/// stamp of the revision read before the render and the Game the parts show. One
/// part that fails to render fails the whole answer.
pub async fn screen_handler(
    State(state): State<AppState>,
    Query(query): Query<ScreenQuery>,
) -> Response<Body> {
    let renderers = match renderers_named(query.parts.as_deref()) {
        Ok(renderers) => renderers,
        Err(message) => return bad_request(render_error(&message)),
    };
    let stamp = state.game_view_query.change_versions(None).stamp();
    let game = state.game_catalogue.active_game_id();

    let mut body = String::new();
    for (name, render) in renderers {
        match render(&state, game) {
            Ok(html) => body.push_str(&format!("<template data-part=\"{name}\">{html}</template>")),
            Err(error) => {
                tracing::error!("screen part {name} failed: {error}");
                return internal_error(render_error(&format!("Failed to render {name}: {error}")));
            }
        }
    }

    let mut response = (StatusCode::OK, axum::response::Html(body)).into_response();
    let headers = response.headers_mut();
    if let Ok(value) = HeaderValue::from_str(&stamp) {
        headers.insert(REVISION_HEADER, value);
    }
    let game_id = game.map_or_else(String::new, |game_id| game_id.to_string());
    if let Ok(value) = HeaderValue::from_str(&game_id) {
        headers.insert(GAME_HEADER, value);
    }
    response
}
