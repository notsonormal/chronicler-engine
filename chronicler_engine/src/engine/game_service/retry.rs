use std::sync::Arc;

use crate::engine::logic::{find_room_in_map, get_current_room};
use crate::model::character::NpcCard;
use crate::model::state::{GameState, GenerationPhase, GenerationStatus};
use crate::narrative::prompt::make_prompt_context;

use super::context::GameServiceContext;
use super::helpers::{load_state, map_llm_error, save_state};
use super::service::DefaultGameService;

/// [DOC: docs/architecture/system.md]
pub fn retry_last_response_impl(service: &DefaultGameService, ctx: GameServiceContext) {
    let (
        input_text,
        snapshot_message_id,
        snapshot_swipe_index,
        world,
        map,
        player,
        all_npcs,
        room_npc_ids,
        history_for_retry,
        current_room_id,
    ) = {
        let snapshot = match ctx.snapshot_storage.load_latest(None) {
            Ok(Some(s)) => s,
            _ => {
                log::error!("No snapshot to retry");
                return;
            }
        };

        let guard = GameState::from_snapshot(
            &snapshot,
            Arc::clone(&ctx.world),
            Arc::clone(&ctx.map),
            Arc::clone(&ctx.player),
            (*ctx.npcs).clone(),
        );

        let input_text = match guard.get_last_input_text() {
            Some((_sender, text)) => text,
            None => {
                log::error!("No input to retry");
                return;
            }
        };

        let room_npc_ids = match get_current_room(&guard) {
            Ok(room) => room.npcs.clone(),
            Err(_) => vec![],
        };

        (
            input_text,
            snapshot.message_id,
            snapshot.swipe_index,
            Arc::clone(&guard.world),
            Arc::clone(&guard.map),
            Arc::clone(&guard.player),
            guard.npcs.values().cloned().collect::<Vec<_>>(),
            room_npc_ids,
            guard.get_history_context_for_retry(),
            guard.movement.current_room_id.clone(),
        )
    };

    let backend = Arc::clone(&service.llm_backend);

    let Some(room) = find_room_in_map(&map, &current_room_id) else {
        let mut state = load_state(&ctx);
        state.narrative.generation.status =
            GenerationStatus::Error("Retry failed: room not found".to_string());
        save_state(
            &ctx,
            &state,
            snapshot_message_id.clone(),
            snapshot_swipe_index + 1,
        );
        return;
    };

    let nearby_npcs: Vec<NpcCard> = all_npcs
        .iter()
        .filter(|npc| room_npc_ids.contains(&npc.id))
        .cloned()
        .collect();
    let context = make_prompt_context(
        &world,
        room,
        &all_npcs,
        &nearby_npcs,
        &player,
        &input_text,
        &history_for_retry,
    );

    let new_narration = match backend.narrate_action(&context) {
        Ok(t) => t,
        Err(e) => {
            let mut state = load_state(&ctx);
            state.narrative.generation.status = GenerationStatus::Error(map_llm_error(&e));
            save_state(
                &ctx,
                &state,
                snapshot_message_id.clone(),
                snapshot_swipe_index + 1,
            );
            return;
        }
    };

    if new_narration.trim().is_empty() {
        let mut state = load_state(&ctx);
        state.narrative.generation.status =
            GenerationStatus::Error("LLM Error: empty response".to_string());
        save_state(
            &ctx,
            &state,
            snapshot_message_id.clone(),
            snapshot_swipe_index + 1,
        );
        return;
    }

    let mut state = load_state(&ctx);
    if let Err(e) = state.replace_last_ai_response(new_narration) {
        state.narrative.generation.status = GenerationStatus::Error(format!("Retry failed: {e}"));
    } else {
        state.narrative.generation.status = GenerationStatus::Idle;
        state.narrative.generation.phase = GenerationPhase::default();
    }
    save_state(
        &ctx,
        &state,
        snapshot_message_id.clone(),
        snapshot_swipe_index + 1,
    );
}
