//! [DOC: docs/architecture/system.md]
use crate::engine::logic::{attempt_semantic_walk, create_dynamic_room};
use crate::engine::state_diagnostics::assert_state_consistency;
use crate::engine::trigger_eval::{
    evaluate_triggers, increment_times_met, mark_trigger_fired, set_currently_meeting,
};
use crate::error::EngineError;
use crate::model::character::NpcCard;
use crate::model::quantifier::{NpcEvent, NpcTransitionType, QuantifierResult, compute_npc_events};
use crate::model::state::{GameState, MessageType, StoredTriggerContext};
/// [DOC: docs/architecture/system.md]
pub struct FreeActionContext<'a> {
    pub narration_text: &'a str,
    pub quantifier_result: &'a QuantifierResult,
}

pub struct TriggerMatch {
    pub npc_id: String,
    pub trigger_idx: usize,
    pub trigger_name: String,
    pub trigger_repeat: bool,
    pub trigger_narration_prompt: String,
}
pub struct TurnResult {
    pub next_state: GameState,
    pub narration: String,
    pub trigger_match: Option<TriggerMatch>,
}
/// Request type for trigger continuation narration.
pub struct TriggerContinuationRequest {
    pub stored: StoredTriggerContext,
}

/// Attempts movement to a destination, creating a dynamic room on failure.
///
/// Returns Ok(state) in both success and error cases — errors result in dynamic room creation.
pub fn attempt_movement(state: GameState, destination: &str) -> Result<GameState, EngineError> {
    // [DOC: docs/architecture/system.md]
    let mut state = state;
    match attempt_semantic_walk(&mut state, destination) {
        Ok(_) => Ok(state),
        Err(e) => {
            tracing::debug!("Semantic walk failed for '{destination}': {e}");
            let dynamic_room =
                create_dynamic_room(destination, "A place you have never seen before.");
            state.add_message(
                format!("[System] Entered unknown location: {}", dynamic_room.id),
                None,
                MessageType::System,
            );
            state
                .movement
                .dynamic_rooms
                .insert(dynamic_room.id.clone(), dynamic_room.clone());
            state.movement.current_room_id = dynamic_room.id.clone();
            Ok(state)
        }
    }
}

/// Updates NPC encounter log if room changed.
///
/// Pure function — no I/O or logging.
pub fn update_npc_encounters_on_room_change(
    mut state: GameState,
    previous_room_id: &str,
    new_npc_ids: &[String],
) -> GameState {
    // [DOC: docs/architecture/system.md]
    if previous_room_id != state.movement.current_room_id {
        for npc_id in new_npc_ids {
            set_currently_meeting(&mut state.npc_encounter_log, npc_id, true);
        }
    }
    state
}

/// Updates narrative state after movement completion.
///
/// Pure function — sets pending location based on current room.
pub fn log_movement_completion(state: GameState) -> GameState {
    // [DOC: docs/architecture/system.md]
    let mut state = state;
    if let Some(current_room) = state.current_room() {
        state.narrative.pending_location = Some(current_room.name.clone());
    }
    state
}

/// [DOC: docs/architecture/system.md]
pub fn handle_movement(
    state: GameState,
    destination: Option<&str>,
    new_npc_ids: &[String],
) -> Result<GameState, EngineError> {
    let Some(destination) = destination else {
        return Ok(state);
    };

    let previous_room_id = state.movement.current_room_id.clone();

    let state = attempt_movement(state, destination)?;
    let state = update_npc_encounters_on_room_change(state, &previous_room_id, new_npc_ids);
    let state = log_movement_completion(state);

    assert_state_consistency(&state)?;
    Ok(state)
}

/// [DOC: docs/architecture/system.md]
pub fn apply_npc_events(state: GameState, events: &[NpcEvent]) -> Result<GameState, EngineError> {
    let mut state = state;
    for event in events {
        match event.event_type {
            NpcTransitionType::Entered => {
                set_currently_meeting(&mut state.npc_encounter_log, &event.npc_id, true);
                increment_times_met(&mut state.npc_encounter_log, &event.npc_id);
            }
            NpcTransitionType::Left => {
                set_currently_meeting(&mut state.npc_encounter_log, &event.npc_id, false);
            }
        }
    }

    assert_state_consistency(&state)?;
    Ok(state)
}

/// Commits trigger continuation narration to state.
// [DOC: docs/architecture/system.md]
pub fn commit_trigger_narration(
    state: GameState,
    request: &TriggerContinuationRequest,
    continuation_text: &str,
) -> Result<GameState, EngineError> {
    // [DOC: docs/architecture/system.md]
    if continuation_text.trim().is_empty() {
        return Ok(state);
    }
    let mut state = state;
    state.narrative.last_trigger = Some(request.stored.clone());
    state.narrative.pending_event = Some(request.stored.trigger_name.clone());
    state.add_message(continuation_text.to_string(), None, MessageType::Narration);
    if !request.stored.trigger_repeat {
        mark_trigger_fired(
            &mut state.npc_encounter_log,
            &request.stored.npc_id,
            request.stored.trigger_idx,
        );
    }

    assert_state_consistency(&state)?;
    Ok(state)
}

/// Executes free action processing with quantifier result. Mutation order is load-bearing — see inline comment.
// [DOC: docs/architecture/system.md]
pub fn execute_freeaction_impl(
    state: &GameState,
    ctx: &FreeActionContext<'_>,
) -> Result<TurnResult, EngineError> {
    // [DOC: docs/architecture/system.md]
    // Mutation order: 1.handle_movement 2.resolve NPCs 3.add_message 4.evaluate_triggers 5.apply_npc_events
    // Swapping 3&4 or 4&5 breaks trigger firing.
    let previous_room_npcs: Vec<NpcCard> = state.scene.npcs_in_area.clone();
    let previous_npc_ids: Vec<String> = previous_room_npcs.iter().map(|n| n.id.clone()).collect();

    let mut next_state = handle_movement(
        state.clone(),
        ctx.quantifier_result.movement.destination.as_deref(),
        &ctx.quantifier_result.npcs.npc_ids,
    )?;
    assert_state_consistency(&next_state)?;

    // [DOC: docs/system/triggers.md section: Mutation Order Invariant]
    next_state.add_message(ctx.narration_text.to_string(), None, MessageType::Narration);
    next_state.scene.npcs_in_area = ctx
        .quantifier_result
        .npcs
        .npc_ids
        .iter()
        .filter_map(|id| next_state.npcs.get(id).cloned())
        .collect();
    let current_npc_ids = ctx.quantifier_result.npcs.npc_ids.clone();

    // Evaluate triggers BEFORE applying NPC events so that trigger conditions
    // (e.g., times_met) are checked against the pre-event state.
    let trigger_match =
        evaluate_triggers(&next_state)
            .into_iter()
            .next()
            .map(|(npc, trigger, idx)| TriggerMatch {
                npc_id: npc.id,
                trigger_idx: idx,
                trigger_name: trigger.narration.name,
                trigger_repeat: trigger.repeat,
                trigger_narration_prompt: trigger.narration.narration_prompt,
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
