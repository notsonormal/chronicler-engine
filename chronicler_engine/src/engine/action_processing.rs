//! [DOC: docs/architecture/system.md]

use crate::engine::logic::{attempt_semantic_walk, create_dynamic_room};
use crate::engine::state_diagnostics::assert_state_consistency;
use crate::engine::trigger_eval::{
    evaluate_triggers, increment_times_met, mark_trigger_fired, set_currently_meeting,
};
use crate::error::EngineError;
use crate::model::character::NpcCard;
use crate::model::quantifier::{NpcEvent, NpcEventType, QuantifierResult, compute_npc_events};
use crate::model::state::{GameState, LogType};

/// [DOC: docs/architecture/system.md]
pub struct FreeActionContext<'a> {
    pub narration_text: &'a str,
    pub quantifier_result: &'a QuantifierResult,
}

/// The LLM call itself happens outside the state lock so the frontend can poll the main narration.
pub struct TriggerContinuationRequest {
    pub stored: crate::model::state::StoredTriggerContext,
}

/// Raw trigger match data for the application tier to build continuation prompts.
pub struct TriggerMatch {
    pub npc_id: String,
    pub trigger_idx: usize,
    pub trigger_name: String,
    pub trigger_repeat: bool,
    pub trigger_narration_prompt: String,
}

/// Result of processing a single free action turn.
pub struct TurnResult {
    pub next_state: GameState,
    pub narration: String,
    pub trigger_match: Option<TriggerMatch>,
}

/// [DOC: docs/architecture/system.md]
pub fn handle_movement(
    state: GameState,
    destination: Option<&str>,
    new_npc_ids: &[String],
) -> Result<GameState, EngineError> {
    let Some(trigger) = destination else {
        return Ok(state);
    };

    let mut state = state;
    let previous_room_id = state.movement.current_room_id.clone();

    if let Err(e) = attempt_semantic_walk(&mut state, trigger) {
        log::debug!("Semantic walk failed for '{trigger}': {e}");
        let dynamic_room = create_dynamic_room(trigger, "A place you have never seen before.");
        state.add_log(
            format!("[System] Entered unknown location: {}", dynamic_room.id),
            None,
            LogType::System,
        );
        state
            .movement
            .dynamic_rooms
            .insert(dynamic_room.id.clone(), dynamic_room.clone());
        state.movement.current_room_id = dynamic_room.id.clone();
    }

    if previous_room_id != state.movement.current_room_id {
        for npc_id in new_npc_ids {
            set_currently_meeting(&mut state.character_state, npc_id, true);
        }
    }

    if let Some(current_room) = state.current_room() {
        state.narrative.pending_location = Some(current_room.name.clone());
    }

    assert_state_consistency(&state)?;
    Ok(state)
}

/// [DOC: docs/architecture/system.md]
pub fn apply_npc_events(state: GameState, events: &[NpcEvent]) -> Result<GameState, EngineError> {
    let mut state = state;
    for event in events {
        match event.event_type {
            NpcEventType::Entered => {
                set_currently_meeting(&mut state.character_state, &event.npc_id, true);
                increment_times_met(&mut state.character_state, &event.npc_id);
            }
            NpcEventType::Left => {
                set_currently_meeting(&mut state.character_state, &event.npc_id, false);
            }
        }
    }

    assert_state_consistency(&state)?;
    Ok(state)
}

/// Called after the trigger continuation LLM call completes.
/// [DOC: docs/system/triggers.md]
pub fn commit_trigger_narration(
    state: GameState,
    request: &TriggerContinuationRequest,
    continuation_text: &str,
) -> Result<GameState, EngineError> {
    if continuation_text.trim().is_empty() {
        return Ok(state);
    }
    let mut state = state;
    state.narrative.last_trigger = Some(request.stored.clone());
    state.narrative.pending_event = Some(request.stored.trigger_name.clone());
    state.add_log(continuation_text.to_string(), None, LogType::Narration);
    if !request.stored.trigger_repeat {
        mark_trigger_fired(
            &mut state.character_state,
            &request.stored.npc_id,
            request.stored.trigger_idx,
        );
    }

    assert_state_consistency(&state)?;
    Ok(state)
}

/// [DOC: docs/architecture/system.md]
pub fn execute_freeaction_impl(
    state: &GameState,
    ctx: &FreeActionContext<'_>,
) -> Result<TurnResult, EngineError> {
    let previous_room_npcs: Vec<NpcCard> = state.scene.npcs_in_area.clone();
    let previous_npc_ids: Vec<String> = previous_room_npcs.iter().map(|n| n.id.clone()).collect();

    let mut next_state = handle_movement(
        state.clone(),
        ctx.quantifier_result.movement.destination.as_deref(),
        &ctx.quantifier_result.npcs.npc_ids,
    )?;
    assert_state_consistency(&next_state)?;

    let current_npcs: Vec<NpcCard> = ctx
        .quantifier_result
        .npcs
        .npc_ids
        .iter()
        .filter_map(|id| next_state.npcs.get(id).cloned())
        .collect();
    let current_npc_ids: Vec<String> = current_npcs.iter().map(|n| n.id.clone()).collect();

    // [DOC: docs/system/triggers.md section: Mutation Order Invariant]
    // Order is load-bearing: narration logged first (step 1), then triggers evaluated
    // which read history for context (step 2), then NPC events applied (step 3).
    next_state.add_log(ctx.narration_text.to_string(), None, LogType::Narration);
    next_state.scene.npcs_in_area = current_npcs.clone();

    // Evaluate triggers BEFORE applying NPC events so that trigger conditions
    // (e.g., times_met) are checked against the pre-event state.
    let trigger_match =
        evaluate_triggers(&next_state)
            .into_iter()
            .next()
            .map(|(npc, trigger, idx)| TriggerMatch {
                npc_id: npc.id,
                trigger_idx: idx,
                trigger_name: trigger.action.name,
                trigger_repeat: trigger.repeat,
                trigger_narration_prompt: trigger.action.narration_prompt,
            });

    let events = compute_npc_events(&previous_npc_ids, &current_npc_ids);
    next_state = apply_npc_events(next_state, &events.events)?;
    assert_state_consistency(&next_state)?;

    Ok(TurnResult {
        next_state,
        narration: ctx.narration_text.to_string(),
        trigger_match,
    })
}
