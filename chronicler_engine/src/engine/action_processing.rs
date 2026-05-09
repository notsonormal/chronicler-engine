//! [DOC: docs/architecture/system.md]

use crate::engine::logic::{attempt_semantic_walk, create_dynamic_room, get_current_room};
use crate::engine::trigger_eval::{
    evaluate_triggers, increment_times_met, mark_trigger_fired, set_currently_meeting,
};
use crate::error::EngineError;
use crate::model::character::NpcCard;

use crate::model::character::PlayerCard;
use crate::model::map::Room;
use crate::model::state::{GameState, LogEntry, LogType};
use crate::model::world::WorldCard;
use crate::narrative::prompt::{PromptBuilder, PromptContext};
use crate::narrative::quantifier::{NpcEvent, QuantifierResult, compute_npc_events};

/// [DOC: docs/architecture/system.md]
pub struct FreeActionContext<'a> {
    pub narration_text: &'a str,
    pub user_input: &'a str,
    pub quantifier_result: &'a QuantifierResult,
    pub world: &'a WorldCard,
    pub player: &'a crate::model::character::PlayerCard,
    pub all_npcs: &'a [NpcCard],
    pub history: &'a [crate::model::state::LogEntry],
    pub llm_backend: &'a dyn crate::narrative::llm::LlmBackend,
}

/// The LLM call itself happens outside the state lock so the frontend can poll the main narration.
pub struct TriggerContinuationRequest {
    pub npc_id: String,
    pub trigger_idx: usize,
    pub trigger_name: String,
    pub trigger_repeat: bool,
    pub system_prompt: String,
    pub user_prompt: String,
    pub max_tokens: Option<u32>,
}

/// [DOC: docs/architecture/system.md]
pub fn get_static_npcs(state: &GameState, room_npc_ids: &[String]) -> Vec<NpcCard> {
    room_npc_ids
        .iter()
        .filter_map(|id| state.npcs.get(id).cloned())
        .collect()
}

/// [DOC: docs/architecture/system.md]
pub fn handle_movement(state: &mut GameState, destination: Option<&str>, new_npc_ids: &[String]) {
    let Some(trigger) = destination else {
        return;
    };

    let previous_room_id = state.current_room_id.clone();

    let success = match attempt_semantic_walk(state, trigger) {
        Ok(_) => true,
        Err(_) => {
            let dynamic_room = create_dynamic_room(trigger, "A place you have never seen before.");
            state.add_log(
                format!("[System] Entered unknown location: {}", dynamic_room.id),
                None,
                LogType::System,
            );
            state
                .dynamic_rooms
                .insert(dynamic_room.id.clone(), dynamic_room.clone());
            state.current_room_id = dynamic_room.id.clone();
            true
        }
    };

    if !success {
        return;
    }

    if previous_room_id != state.current_room_id {
        for npc_id in new_npc_ids {
            set_currently_meeting(&mut state.character_state, npc_id, true);
        }
    }

    if let Ok(current_room) = get_current_room(state) {
        state.add_log(
            String::new(),
            Some(current_room.name.clone()),
            LogType::Narration,
        );
    }
}

/// [DOC: docs/architecture/system.md]
pub fn apply_npc_events(state: &mut GameState, events: &[NpcEvent]) {
    for event in events {
        match event.event_type {
            crate::narrative::quantifier::NpcEventType::Entered => {
                set_currently_meeting(&mut state.character_state, &event.npc_id, true);
                increment_times_met(&mut state.character_state, &event.npc_id);
            }
            crate::narrative::quantifier::NpcEventType::Left => {
                set_currently_meeting(&mut state.character_state, &event.npc_id, false);
            }
        }
    }
}

/// Called after the trigger continuation LLM call completes.
/// [DOC: docs/system/triggers.md]
pub fn commit_trigger_narration(
    state: &mut GameState,
    request: &TriggerContinuationRequest,
    continuation_text: &str,
) {
    if continuation_text.trim().is_empty() {
        return;
    }
    state.add_log(
        String::new(),
        Some(request.trigger_name.clone()),
        LogType::Event,
    );
    state.add_log(continuation_text.to_string(), None, LogType::Narration);
    if !request.trigger_repeat {
        mark_trigger_fired(
            &mut state.character_state,
            &request.npc_id,
            request.trigger_idx,
        );
    }
}

/// Shared by `build_trigger_request` and `evaluate_and_narrate_triggers`.
fn build_trigger_prompt_parts(
    world: &WorldCard,
    room: &Room,
    all_npcs: &[NpcCard],
    npcs_in_area: &[NpcCard],
    player: &PlayerCard,
    user_message: &str,
    history: &[LogEntry],
) -> Option<(String, String, u32)> {
    let settings = crate::settings::load_settings().unwrap_or_default();
    let narration_conn = settings.get_narration_connection();
    let max_context = narration_conn
        .map(|c| c.resolve_max_context_tokens())
        .unwrap_or(crate::narrative::prompt::budget::MAX_CONTEXT_TOKENS);
    let max_tokens = narration_conn.and_then(|c| c.max_tokens);

    let trigger_ctx = PromptContext {
        world,
        room,
        all_npcs,
        npcs_in_area,
        player,
        user_message,
        history,
    };

    let mut pb = PromptBuilder::from_context(&trigger_ctx);
    pb.max_context_tokens = Some(max_context);
    pb.requested_max_tokens = max_tokens;
    pb.response_length = Some(&settings.response_length);

    match pb.build_split() {
        Ok(parts) => Some(parts),
        Err(e) => {
            log::error!("Failed to build trigger continuation prompt: {e}");
            None
        }
    }
}

fn build_trigger_request(
    state: &GameState,
    ctx: &FreeActionContext<'_>,
    room_data: &Room,
) -> Option<TriggerContinuationRequest> {
    let matching_triggers = evaluate_triggers(state);

    let (trigger_idx, (npc, trigger)) = matching_triggers.iter().enumerate().next()?;

    let continuation_user_msg = format!(
        "Previous narration:\n{}\n\nTrigger event: {}\n\n\
         Continue the scene naturally, incorporating the trigger event into the narrative. \
         Do NOT repeat or contradict what was already described. Build naturally on the existing scene.",
        ctx.narration_text, trigger.action.narration_prompt
    );

    let (system_prompt, user_prompt, fitted_max_tokens) = build_trigger_prompt_parts(
        ctx.world,
        room_data,
        ctx.all_npcs,
        &state.npcs_in_area,
        ctx.player,
        &continuation_user_msg,
        &state.narration_history,
    )?;

    Some(TriggerContinuationRequest {
        npc_id: npc.id.clone(),
        trigger_idx,
        trigger_name: trigger.action.name.clone(),
        trigger_repeat: trigger.repeat,
        system_prompt,
        user_prompt,
        max_tokens: Some(fitted_max_tokens),
    })
}

/// [DOC: docs/system/triggers.md]
pub fn evaluate_and_narrate_triggers(
    state: &mut GameState,
    narration_text: &str,
    trigger_context: &PromptContext<'_>,
    llm_backend: &dyn crate::narrative::llm::LlmBackend,
) {
    let matching_triggers = evaluate_triggers(state);

    let Some((trigger_idx, (npc, trigger))) = matching_triggers.iter().enumerate().next() else {
        return;
    };

    state.generation_state.phase = crate::model::state::GenerationPhase::GeneratingEvent;

    let continuation_user_msg = format!(
        "Previous narration:\n{narration_text}\n\nTrigger event: {}\n\n\
         Continue the scene naturally, incorporating the trigger event into the narrative. \
         Do NOT repeat or contradict what was already described. Build naturally on the existing scene.",
        trigger.action.narration_prompt
    );

    let (system_prompt, user_prompt, fitted_max_tokens) = match build_trigger_prompt_parts(
        trigger_context.world,
        trigger_context.room,
        trigger_context.all_npcs,
        &state.npcs_in_area,
        trigger_context.player,
        &continuation_user_msg,
        &state.narration_history,
    ) {
        Some(parts) => parts,
        None => return,
    };

    let continuation_text = match llm_backend.narrate_action_from_prompt(
        &system_prompt,
        &user_prompt,
        Some(fitted_max_tokens),
    ) {
        Ok(text) => text,
        Err(e) => {
            log::error!("Trigger narration failed: {e}");
            state.add_log(
                format!("[Trigger narration failed: {e}]"),
                None,
                LogType::System,
            );
            return;
        }
    };

    if continuation_text.trim().is_empty() {
        return;
    }
    state.add_log(
        String::new(),
        Some(trigger.action.name.clone()),
        LogType::Event,
    );
    state.add_log(continuation_text, None, LogType::Narration);
    if !trigger.repeat {
        mark_trigger_fired(&mut state.character_state, &npc.id, trigger_idx);
    }
}

/// [DOC: docs/architecture/system.md]
pub fn execute_freeaction_impl(
    state: &mut GameState,
    ctx: &FreeActionContext<'_>,
) -> Result<Option<TriggerContinuationRequest>, EngineError> {
    let previous_room_npcs: Vec<NpcCard> = state.npcs_in_area.clone();
    let previous_npc_ids: Vec<String> = previous_room_npcs.iter().map(|n| n.id.clone()).collect();

    handle_movement(
        state,
        ctx.quantifier_result.movement.destination.as_deref(),
        &ctx.quantifier_result.npcs.npc_ids,
    );

    let current_npcs: Vec<NpcCard> = ctx
        .quantifier_result
        .npcs
        .npc_ids
        .iter()
        .filter_map(|id| state.npcs.get(id).cloned())
        .collect();
    let current_npc_ids: Vec<String> = current_npcs.iter().map(|n| n.id.clone()).collect();

    let room_data = get_current_room(state)
        .map_err(|_| EngineError::RoomNotFound("current room not found".to_string()))?
        .clone();

    // [DOC: docs/system/triggers.md Â§Mutation Order Invariant]
    // Order is load-bearing: narration logged first (step 1), then triggers evaluated
    // which read history for context (step 2), then NPC events applied (step 3).
    state.add_log(ctx.narration_text.to_string(), None, LogType::Narration);
    state.npcs_in_area = current_npcs.clone();

    let trigger_request = build_trigger_request(state, ctx, &room_data);

    let events = compute_npc_events(&previous_npc_ids, &current_npc_ids);
    apply_npc_events(state, &events.events);

    Ok(trigger_request)
}
