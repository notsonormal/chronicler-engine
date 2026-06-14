//! [DOC: docs/system/dashboard.md]
//! Router configuration

use axum::{
    Router,
    routing::{get, post},
};
use tower_http::services::ServeDir;

use super::AppState;
use super::handlers::index_handler;
use super::fragments;
use super::games_fragment;
use super::settings_fragment;
use super::prompt_presets_fragment;
use super::worlds_fragment;
use super::debug;

pub(crate) fn build_router(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/fragment/header", get(fragments::header_fragment))
        .route("/fragment/story-log", get(fragments::story_log_fragment))
        .route(
            "/fragment/visual-sidebar",
            get(fragments::visual_sidebar_fragment),
        )
        .route(
            "/fragment/action-area",
            get(fragments::action_area_fragment),
        )
        .route(
            "/fragment/character-headshots",
            get(fragments::character_headshots_fragment),
        )
        .route("/action", post(fragments::action_handler))
        .route("/action/check", post(fragments::action_check_handler))
        .route("/action/confirm", post(fragments::action_confirm_handler))
        .route("/check-text", post(fragments::check_text_handler))
        .route("/hints", get(fragments::hints_handler))
        .route("/status/ready", get(fragments::status_ready_handler))
        .route(
            "/status/generating",
            get(fragments::generating_status_handler),
        )
        .route(
            "/status/reset-generating",
            post(fragments::reset_generating_handler),
        )
        .route("/history/:id", post(fragments::edit_history_handler))
        .route("/history/delete", post(fragments::delete_history_handler))
        .route("/swipe/new", post(fragments::retry_handler))
        .route(
            "/message/:id/swipe/:index",
            post(fragments::switch_swipe_handler),
        )
        .route("/retrigger", post(fragments::retrigger_handler))
        .route("/reset", post(fragments::reset_handler))
        .route("/games", post(games_fragment::create_game_handler))
        .route(
            "/games/:id/switch",
            post(games_fragment::switch_game_handler),
        )
        .route(
            "/games/:id/delete",
            post(games_fragment::delete_game_handler),
        )
        .route("/fragment/games", get(games_fragment::list_games_fragment))
        .route(
            "/fragment/llm-messages",
            get(fragments::llm_messages_fragment),
        )
        .route(
            "/fragment/worlds",
            get(worlds_fragment::list_worlds_fragment),
        )
        .route("/worlds", post(worlds_fragment::create_world_handler))
        .route("/worlds/:key", post(worlds_fragment::update_world_handler))
        .route(
            "/fragment/worlds/new",
            get(worlds_fragment::new_world_form_handler),
        )
        .route(
            "/worlds/:key/edit",
            get(worlds_fragment::edit_world_form_handler),
        )
        .route(
            "/worlds/:key/delete",
            post(worlds_fragment::delete_world_handler),
        )
        .route("/fragment/settings", get(settings_fragment::settings_panel))
        .route("/settings", post(settings_fragment::save_settings_handler))
        .route(
            "/connections/add",
            post(settings_fragment::add_connection_handler),
        )
        .route(
            "/fragment/connections/:id",
            get(settings_fragment::connection_card_fragment),
        )
        .route(
            "/fragment/connections/:id/edit",
            get(settings_fragment::edit_connection_form),
        )
        .route(
            "/connections/:id/edit",
            post(settings_fragment::edit_connection_handler),
        )
        .route(
            "/connections/:id/delete",
            post(settings_fragment::delete_connection_handler),
        )
        .route(
            "/connections/:id/set-narrator",
            post(settings_fragment::set_narrator_handler),
        )
        .route(
            "/connections/:id/set-quantifier",
            post(settings_fragment::set_quantifier_handler),
        )
        .route(
            "/settings/text-check",
            post(settings_fragment::save_text_check_handler),
        )
        .route(
            "/fragment/prompt-presets",
            get(prompt_presets_fragment::panel_handler),
        )
        .route(
            "/prompt-presets",
            post(prompt_presets_fragment::save_preset_handler),
        )
        .route(
            "/fragment/prompt-presets/:id",
            get(prompt_presets_fragment::preset_card_handler),
        )
        .route(
            "/fragment/prompt-presets/:id/edit",
            get(prompt_presets_fragment::edit_preset_form_handler),
        )
        .route(
            "/fragment/prompt-presets/:id/view",
            get(prompt_presets_fragment::view_preset_form_handler),
        )
        .route(
            "/prompt-presets/:id",
            post(prompt_presets_fragment::update_preset_handler),
        )
        .route(
            "/prompt-presets/:id/delete",
            post(prompt_presets_fragment::delete_preset_handler),
        )
        .route(
            "/prompt-presets/:id/duplicate",
            post(prompt_presets_fragment::duplicate_preset_handler),
        )
        .route(
            "/prompt-presets/:id/activate",
            post(prompt_presets_fragment::activate_preset_handler),
        )
        .route("/debug/state", get(debug::debug_state_handler))
        .route(
            "/debug/is_generating",
            get(debug::debug_is_generating_handler),
        )
        .route("/debug/backend", get(debug::debug_backend_handler))
        .nest_service("/assets", ServeDir::new("assets"))
        .nest_service("/data", ServeDir::new("data"))
        .fallback_service(ServeDir::new("assets"))
        .with_state(app_state)
}
