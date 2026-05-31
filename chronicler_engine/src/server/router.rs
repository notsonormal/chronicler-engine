use axum::{Router, routing::{get, post}};
use tower_http::services::ServeDir;

use super::AppState;
use super::handlers::index_handler;
use super::fragments;
use super::settings_fragment;
use super::prompt_presets_fragment;
use super::debug;

/// Builds the Axum router with all routes configured.
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
        // History edit, delete & retry endpoints
        .route("/history/:id", post(fragments::edit_history_handler))
        .route("/history/delete", post(fragments::delete_history_handler))
        .route("/swipe/new", post(fragments::retry_handler))
        .route(
            "/message/:id/swipe/:index",
            post(fragments::switch_swipe_handler),
        )
        .route("/retrigger", post(fragments::retrigger_handler))
        .route("/reset", post(fragments::reset_handler))
        .route("/games", post(fragments::create_game_handler))
        .route("/games/:id/switch", post(fragments::switch_game_handler))
        .route("/games/:id/delete", post(fragments::delete_game_handler))
        .route("/fragment/games", get(fragments::list_games_fragment))
        .route(
            "/fragment/llm-messages",
            get(fragments::llm_messages_fragment),
        )
        // Settings endpoints
        .route(
            "/fragment/settings",
            get(settings_fragment::settings_panel),
        )
        .route(
            "/settings",
            post(settings_fragment::save_settings_handler),
        )
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
        // Prompt Presets endpoints
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
        // NOTE: dev-only diagnostic endpoint
        .route("/debug/state", get(debug::debug_state_handler))
        .nest_service("/assets", ServeDir::new("assets"))
        .nest_service("/data", ServeDir::new("data"))
        .fallback_service(ServeDir::new("assets"))
        .with_state(app_state)
}

/// Creates a new Axum app with the given state (public wrapper around `build_router`).
pub fn create_app_with_state(app_state: AppState) -> Router {
    build_router(app_state)
}
