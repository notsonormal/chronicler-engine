//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
//! HTTP router composition.

use axum::{
    Router,
    routing::{get, post},
};
use tower_http::services::ServeDir;

use crate::adapters::driving::http::AppState;
use crate::adapters::driving::http::action;
use crate::adapters::driving::http::core;
use crate::adapters::driving::http::games;
use crate::adapters::driving::http::history;
use crate::adapters::driving::http::layout;
use crate::adapters::driving::http::prompt_presets;
use crate::adapters::driving::http::settings;
use crate::adapters::driving::http::worlds;

pub(crate) fn build_router(app_state: AppState) -> Router {
    Router::new()
        // --- Core ---
        .route("/", get(core::handlers::index_handler))
        .route("/check-text", post(core::handlers::check_text_handler))
        .route("/swipe/new", post(core::handlers::retry_handler))
        .route(
            "/message/:id/swipe/:index",
            post(core::handlers::switch_swipe_handler),
        )
        .route("/retrigger", post(core::handlers::retrigger_handler))
        .route("/reset", post(core::handlers::reset_handler))
        .route("/debug/state", get(core::handlers::debug_state_handler))
        .route(
            "/debug/is_generating",
            get(core::handlers::debug_is_generating_handler),
        )
        .route("/debug/backend", get(core::handlers::debug_backend_handler))
        // --- Action ---
        .route("/action", post(action::handlers::action_handler))
        .route(
            "/action/check",
            post(action::handlers::action_check_handler),
        )
        .route(
            "/action/confirm",
            post(action::handlers::action_confirm_handler),
        )
        // --- History ---
        .route(
            "/history/:id",
            post(history::handlers::edit_history_handler),
        )
        .route(
            "/history/delete",
            post(history::handlers::delete_history_handler),
        )
        // --- Layout fragments ---
        .route("/fragment/header", get(layout::handlers::header_fragment))
        .route(
            "/fragment/story-log",
            get(layout::handlers::story_log_fragment),
        )
        .route(
            "/fragment/visual-sidebar",
            get(layout::handlers::visual_sidebar_fragment),
        )
        .route(
            "/fragment/action-area",
            get(layout::handlers::action_area_fragment),
        )
        .route(
            "/fragment/character-headshots",
            get(layout::handlers::character_headshots_fragment),
        )
        .route("/status/ready", get(layout::handlers::status_ready_handler))
        .route(
            "/status/generating",
            get(layout::handlers::generating_status_handler),
        )
        .route(
            "/status/reset-generating",
            post(layout::handlers::reset_generating_handler),
        )
        .route(
            "/fragment/llm-messages",
            get(layout::handlers::llm_messages_fragment),
        )
        // --- Games ---
        .route("/games", post(games::handlers::create_game_handler))
        .route(
            "/games/:id/switch",
            post(games::handlers::switch_game_handler),
        )
        .route(
            "/games/:id/delete",
            post(games::handlers::delete_game_handler),
        )
        .route("/fragment/games", get(games::handlers::list_games_fragment))
        // --- Worlds ---
        .route(
            "/fragment/worlds",
            get(worlds::handlers::list_worlds_fragment),
        )
        .route("/worlds", post(worlds::handlers::create_world_handler))
        .route("/worlds/:key", post(worlds::handlers::update_world_handler))
        .route(
            "/fragment/worlds/new",
            get(worlds::handlers::new_world_form_handler),
        )
        .route(
            "/worlds/:key/edit",
            get(worlds::handlers::edit_world_form_handler),
        )
        .route(
            "/worlds/:key/delete",
            post(worlds::handlers::delete_world_handler),
        )
        // --- Settings & connections ---
        .route(
            "/fragment/settings",
            get(settings::handlers::settings_panel),
        )
        .route("/settings", post(settings::handlers::save_settings_handler))
        .route(
            "/connections/add",
            post(settings::handlers::add_connection_handler),
        )
        .route(
            "/fragment/connections/:id",
            get(settings::handlers::connection_card_fragment),
        )
        .route(
            "/fragment/connections/:id/edit",
            get(settings::handlers::edit_connection_form),
        )
        .route(
            "/connections/:id/edit",
            post(settings::handlers::edit_connection_handler),
        )
        .route(
            "/connections/:id/delete",
            post(settings::handlers::delete_connection_handler),
        )
        .route(
            "/connections/:id/set-narrator",
            post(settings::handlers::set_narrator_handler),
        )
        .route(
            "/connections/:id/set-quantifier",
            post(settings::handlers::set_quantifier_handler),
        )
        .route(
            "/settings/text-check",
            post(settings::handlers::save_text_check_handler),
        )
        // --- Prompt presets ---
        .route(
            "/fragment/prompt-presets",
            get(prompt_presets::handlers::panel_handler),
        )
        .route(
            "/prompt-presets",
            post(prompt_presets::handlers::save_preset_handler),
        )
        .route(
            "/fragment/prompt-presets/:id",
            get(prompt_presets::handlers::preset_card_handler),
        )
        .route(
            "/fragment/prompt-presets/:id/edit",
            get(prompt_presets::handlers::edit_preset_form_handler),
        )
        .route(
            "/fragment/prompt-presets/:id/view",
            get(prompt_presets::handlers::view_preset_form_handler),
        )
        .route(
            "/prompt-presets/:id",
            post(prompt_presets::handlers::update_preset_handler),
        )
        .route(
            "/prompt-presets/:id/delete",
            post(prompt_presets::handlers::delete_preset_handler),
        )
        .route(
            "/prompt-presets/:id/duplicate",
            post(prompt_presets::handlers::duplicate_preset_handler),
        )
        .route(
            "/prompt-presets/:id/activate",
            post(prompt_presets::handlers::activate_preset_handler),
        )
        .nest_service("/assets", ServeDir::new("assets"))
        .nest_service("/data", ServeDir::new("data"))
        .fallback_service(ServeDir::new("assets"))
        .with_state(app_state)
}
