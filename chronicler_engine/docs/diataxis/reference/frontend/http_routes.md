---
diataxis: reference
title: HTTP Routes
---

## Overview

This doc is generated from `src/adapters/driving/http/router.rs` and is the canonical map of HTTP route to handler for the engine's HTTP server. Re-run `python scripts/extract_http_routes.py` after any change to `router.rs`. The seven areas below match the seven handler-module prefixes discovered from the router file (one prefix per source-tree module under `src/adapters/driving/http/`). Static-asset behaviour — `.nest_service` for `/assets` and `/data`, plus the `fallback_service` for unmatched paths — lives in `router.rs` but is not enumerated here; the generator handles `.route()` calls only.

## Index

| Method | Path | Handler |
|--------|------|---------|
| GET | `/` | `handlers::index_handler` |

## Action / Status / History / Lifecycle

| Method | Path | Handler |
|--------|------|---------|
| GET | `/fragment/header` | `fragments::header_fragment` |
| GET | `/fragment/story-log` | `fragments::story_log_fragment` |
| GET | `/fragment/visual-sidebar` | `fragments::visual_sidebar_fragment` |
| GET | `/fragment/action-area` | `fragments::action_area_fragment` |
| GET | `/fragment/character-headshots` | `fragments::character_headshots_fragment` |
| POST | `/action` | `fragments::action_handler` |
| POST | `/action/check` | `fragments::action_check_handler` |
| POST | `/action/confirm` | `fragments::action_confirm_handler` |
| POST | `/check-text` | `fragments::check_text_handler` |
| GET | `/status/ready` | `fragments::status_ready_handler` |
| GET | `/status/generating` | `fragments::generating_status_handler` |
| POST | `/status/reset-generating` | `fragments::reset_generating_handler` |
| POST | `/history/:id` | `fragments::edit_history_handler` |
| POST | `/history/delete` | `fragments::delete_history_handler` |
| POST | `/swipe/new` | `fragments::retry_handler` |
| POST | `/message/:id/swipe/:index` | `fragments::switch_swipe_handler` |
| POST | `/retrigger` | `fragments::retrigger_handler` |
| POST | `/reset` | `fragments::reset_handler` |
| GET | `/fragment/llm-messages` | `fragments::llm_messages_fragment` |

## Games

| Method | Path | Handler |
|--------|------|---------|
| POST | `/games` | `games_fragment::create_game_handler` |
| POST | `/games/:id/switch` | `games_fragment::switch_game_handler` |
| POST | `/games/:id/delete` | `games_fragment::delete_game_handler` |
| GET | `/fragment/games` | `games_fragment::list_games_fragment` |

## Worlds

| Method | Path | Handler |
|--------|------|---------|
| GET | `/fragment/worlds` | `worlds_fragment::list_worlds_fragment` |
| POST | `/worlds` | `worlds_fragment::create_world_handler` |
| POST | `/worlds/:key` | `worlds_fragment::update_world_handler` |
| GET | `/fragment/worlds/new` | `worlds_fragment::new_world_form_handler` |
| GET | `/worlds/:key/edit` | `worlds_fragment::edit_world_form_handler` |
| POST | `/worlds/:key/delete` | `worlds_fragment::delete_world_handler` |

## Settings & connections

| Method | Path | Handler |
|--------|------|---------|
| GET | `/fragment/settings` | `settings_fragment::settings_panel` |
| POST | `/settings` | `settings_fragment::save_settings_handler` |
| POST | `/connections/add` | `settings_fragment::add_connection_handler` |
| GET | `/fragment/connections/:id` | `settings_fragment::connection_card_fragment` |
| GET | `/fragment/connections/:id/edit` | `settings_fragment::edit_connection_form` |
| POST | `/connections/:id/edit` | `settings_fragment::edit_connection_handler` |
| POST | `/connections/:id/delete` | `settings_fragment::delete_connection_handler` |
| POST | `/connections/:id/set-narrator` | `settings_fragment::set_narrator_handler` |
| POST | `/connections/:id/set-quantifier` | `settings_fragment::set_quantifier_handler` |
| POST | `/settings/text-check` | `settings_fragment::save_text_check_handler` |

## Prompt presets

| Method | Path | Handler |
|--------|------|---------|
| GET | `/fragment/prompt-presets` | `prompt_presets_fragment::panel_handler` |
| POST | `/prompt-presets` | `prompt_presets_fragment::save_preset_handler` |
| GET | `/fragment/prompt-presets/:id` | `prompt_presets_fragment::preset_card_handler` |
| GET | `/fragment/prompt-presets/:id/edit` | `prompt_presets_fragment::edit_preset_form_handler` |
| GET | `/fragment/prompt-presets/:id/view` | `prompt_presets_fragment::view_preset_form_handler` |
| POST | `/prompt-presets/:id` | `prompt_presets_fragment::update_preset_handler` |
| POST | `/prompt-presets/:id/delete` | `prompt_presets_fragment::delete_preset_handler` |
| POST | `/prompt-presets/:id/duplicate` | `prompt_presets_fragment::duplicate_preset_handler` |
| POST | `/prompt-presets/:id/activate` | `prompt_presets_fragment::activate_preset_handler` |

## Debug

| Method | Path | Handler |
|--------|------|---------|
| GET | `/debug/state` | `debug::debug_state_handler` |
| GET | `/debug/is_generating` | `debug::debug_is_generating_handler` |
| GET | `/debug/backend` | `debug::debug_backend_handler` |
