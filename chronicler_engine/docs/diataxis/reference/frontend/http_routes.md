---
diataxis: reference
title: HTTP Routes
---

## Overview

Generated from `src/adapters/driving/http/builders/router.rs` — re-run `python scripts/extract_http_routes.py` after any `router.rs` change. Covers `.route()` calls only; `.nest_service` (assets/data) and `fallback_service` live in `router.rs` but aren't enumerated.

## Core

| Method | Path | Handler |
|--------|------|---------|
| GET | `/` | `index_handler` |
| POST | `/check-text` | `check_text_handler` |
| POST | `/swipe/new` | `retry_handler` |
| POST | `/message/:id/swipe/:index` | `switch_swipe_handler` |
| POST | `/retrigger` | `retrigger_handler` |
| POST | `/reset` | `reset_handler` |
| GET | `/debug/state` | `debug_state_handler` |
| GET | `/debug/is_generating` | `debug_is_generating_handler` |
| GET | `/debug/backend` | `debug_backend_handler` |

## Action

| Method | Path | Handler |
|--------|------|---------|
| POST | `/action` | `action_handler` |
| POST | `/action/check` | `action_check_handler` |
| POST | `/action/confirm` | `action_confirm_handler` |

## History

| Method | Path | Handler |
|--------|------|---------|
| POST | `/history/:id` | `edit_history_handler` |
| POST | `/history/delete` | `delete_history_handler` |

## Layout fragments

| Method | Path | Handler |
|--------|------|---------|
| GET | `/fragment/header` | `header_fragment` |
| GET | `/fragment/story-log` | `story_log_fragment` |
| GET | `/fragment/visual-sidebar` | `visual_sidebar_fragment` |
| GET | `/fragment/action-area` | `action_area_fragment` |
| GET | `/fragment/character-headshots` | `character_headshots_fragment` |
| GET | `/status/ready` | `status_ready_handler` |
| GET | `/status/generating` | `generating_status_handler` |
| POST | `/status/reset-generating` | `reset_generating_handler` |
| GET | `/fragment/llm-messages` | `llm_messages_fragment` |

## Games

| Method | Path | Handler |
|--------|------|---------|
| POST | `/games` | `create_game_handler` |
| POST | `/games/:id/switch` | `switch_game_handler` |
| POST | `/games/:id/delete` | `delete_game_handler` |
| GET | `/fragment/games` | `list_games_fragment` |

## Worlds

| Method | Path | Handler |
|--------|------|---------|
| GET | `/fragment/worlds` | `list_worlds_fragment` |
| POST | `/worlds` | `create_world_handler` |
| POST | `/worlds/:key` | `update_world_handler` |
| GET | `/fragment/worlds/new` | `new_world_form_handler` |
| GET | `/worlds/:key/edit` | `edit_world_form_handler` |
| POST | `/worlds/:key/delete` | `delete_world_handler` |

## Settings & connections

| Method | Path | Handler |
|--------|------|---------|
| GET | `/fragment/settings` | `settings_panel` |
| POST | `/settings` | `save_settings_handler` |
| POST | `/connections/add` | `add_connection_handler` |
| GET | `/fragment/connections/:id` | `connection_card_fragment` |
| GET | `/fragment/connections/:id/edit` | `edit_connection_form` |
| POST | `/connections/:id/edit` | `edit_connection_handler` |
| POST | `/connections/:id/delete` | `delete_connection_handler` |
| POST | `/connections/:id/set-narrator` | `set_narrator_handler` |
| POST | `/connections/:id/set-quantifier` | `set_quantifier_handler` |
| POST | `/settings/text-check` | `save_text_check_handler` |

## Prompt presets

| Method | Path | Handler |
|--------|------|---------|
| GET | `/fragment/prompt-presets` | `panel_handler` |
| POST | `/prompt-presets` | `save_preset_handler` |
| GET | `/fragment/prompt-presets/:id` | `preset_card_handler` |
| GET | `/fragment/prompt-presets/:id/edit` | `edit_preset_form_handler` |
| GET | `/fragment/prompt-presets/:id/view` | `view_preset_form_handler` |
| POST | `/prompt-presets/:id` | `update_preset_handler` |
| POST | `/prompt-presets/:id/delete` | `delete_preset_handler` |
| POST | `/prompt-presets/:id/duplicate` | `duplicate_preset_handler` |
| POST | `/prompt-presets/:id/activate` | `activate_preset_handler` |
