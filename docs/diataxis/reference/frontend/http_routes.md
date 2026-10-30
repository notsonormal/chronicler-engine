---
diataxis: reference
title: HTTP Routes
---

## Overview

Generated from `src/adapters/driving/http/builders/router.rs` — re-run `python scripts/extract_http_routes.py` after any `router.rs` change. Covers `.route()` calls only; `.nest_service` (assets/data) and `fallback_service` live in `router.rs` but aren't enumerated.

## Core

| Method | Path | Handler |
|--------|------|---------|
| POST | `/swipe/new` | `retry_handler` |
| POST | `/message/:id/swipe/:index` | `switch_swipe_handler` |
| POST | `/retrigger` | `retrigger_handler` |
| POST | `/reset` | `reset_handler` |
| GET | `/` | `index_handler` |
| GET | `/debug/state` | `debug_state_handler` |
| GET | `/debug/is_busy` | `debug_is_busy_handler` |
| GET | `/debug/backend` | `debug_backend_handler` |

## Action

| Method | Path | Handler |
|--------|------|---------|
| POST | `/action` | `action_handler` |
| POST | `/action/check` | `action_check_handler` |

## Autoplay

| Method | Path | Handler |
|--------|------|---------|
| POST | `/autoplay` | `start_autoplay_handler` |

## History

| Method | Path | Handler |
|--------|------|---------|
| POST | `/history/:id` | `edit_history_handler` |
| POST | `/history/delete` | `delete_history_handler` |

## Layout fragments

| Method | Path | Handler |
|--------|------|---------|
| POST | `/status/reset-busy` | `reset_busy_handler` |
| GET | `/changes` | `changes_handler` |
| GET | `/fragment/llm-messages/:id` | `llm_message_record_fragment` |

## Games

| Method | Path | Handler |
|--------|------|---------|
| POST | `/games` | `create_game_handler` |
| POST | `/games/:id/switch` | `switch_game_handler` |
| POST | `/games/:id/rename` | `rename_game_handler` |
| POST | `/games/:id/delete` | `delete_game_handler` |
| POST | `/games/:id/mode` | `switch_game_mode_handler` |
| POST | `/games/:id/posture` | `update_game_posture_handler` |
| POST | `/games/:id/presets` | `update_game_presets_handler` |
| GET | `/fragment/games/new` | `new_game_form_handler` |
| GET | `/fragment/games/:id/rename` | `rename_game_form_handler` |

## Worlds

| Method | Path | Handler |
|--------|------|---------|
| POST | `/worlds` | `create_world_handler` |
| POST | `/worlds/:key` | `update_world_handler` |
| POST | `/worlds/:key/posture` | `update_world_posture_handler` |
| GET | `/fragment/worlds/new` | `new_world_form_handler` |
| GET | `/worlds/:key/edit` | `edit_world_form_handler` |
| POST | `/worlds/:key/delete` | `delete_world_handler` |

## Settings & connections

| Method | Path | Handler |
|--------|------|---------|
| POST | `/connections/add` | `add_connection_handler` |
| POST | `/connections/test` | `test_form_connection_handler` |
| POST | `/connections/:id/test` | `test_saved_connection_handler` |
| GET | `/fragment/connections/new` | `new_connection_form` |
| GET | `/fragment/connections/:id/edit` | `edit_connection_form` |
| POST | `/connections/:id/edit` | `edit_connection_handler` |
| POST | `/connections/:id/delete` | `delete_connection_handler` |
| POST | `/connections/set-narrator` | `set_narrator_handler` |
| POST | `/connections/set-quantifier` | `set_quantifier_handler` |
| POST | `/settings/text-check` | `save_text_check_handler` |

## Prompt presets

| Method | Path | Handler |
|--------|------|---------|
| POST | `/prompt-presets` | `save_preset_handler` |
| GET | `/fragment/prompt-presets/new` | `new_preset_form_handler` |
| GET | `/fragment/prompt-presets/:id/edit` | `edit_preset_form_handler` |
| GET | `/fragment/prompt-presets/:id/view` | `view_preset_form_handler` |
| POST | `/prompt-presets/:id` | `update_preset_handler` |
| POST | `/prompt-presets/:id/delete` | `delete_preset_handler` |
| POST | `/prompt-presets/:id/duplicate` | `duplicate_preset_handler` |
| POST | `/prompt-presets/:id/activate` | `activate_preset_handler` |

## Unknown areas

The following routes have handler-module prefixes not listed in the script's `AREA_GROUPS`. Add a new area (or fold them into an existing one) in `scripts/extract_http_routes.py`:

| Method | Path | Handler |
|--------|------|---------|
| GET | `/screen` | `screen_handler` |
