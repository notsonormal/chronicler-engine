//! [DOC: docs/system/game_flow.md]
//! Action execution pipeline and validation
use crate::engine::logic::{attempt_semantic_walk, create_dynamic_room};
use crate::engine::state_diagnostics::assert_state_consistency;
use crate::engine::trigger_eval::evaluate_triggers;
use crate::error::EngineError;
use crate::model::character::NpcCard;
use crate::model::quantifier::{NpcEvent, NpcTransitionType, QuantifierResult, compute_npc_events};
use crate::model::state::game_state::GameState;
use crate::model::state::message_types::MessageType;
use crate::model::state::trigger_context::StoredTriggerContext;
use crate::model::template::{render_template, TemplateVars};
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

pub fn attempt_movement(state: GameState, destination: &str) -> Result<GameState, EngineError> {
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

pub fn update_npc_encounters_on_room_change(
    mut state: GameState,
    previous_room_id: &str,
    new_npc_ids: &[String],
) -> GameState {
    if previous_room_id != state.movement.current_room_id {
        for npc_id in new_npc_ids {
            state.npc_encounter_log.set_currently_meeting(npc_id, true);
        }
    }
    state
}

pub fn log_movement_completion(state: GameState) -> GameState {
    let mut state = state;
    if let Some(current_room) = state.current_room() {
        state.narrative.pending_location = Some(current_room.name.clone());
    }
    state
}

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

pub fn apply_npc_events(state: GameState, events: &[NpcEvent]) -> Result<GameState, EngineError> {
    let mut state = state;
    for event in events {
        match event.event_type {
            NpcTransitionType::Entered => {
                state
                    .npc_encounter_log
                    .set_currently_meeting(&event.npc_id, true);
                state.npc_encounter_log.increment_times_met(&event.npc_id);
            }
            NpcTransitionType::Left => {
                state
                    .npc_encounter_log
                    .set_currently_meeting(&event.npc_id, false);
            }
        }
    }

    assert_state_consistency(&state)?;
    Ok(state)
}

pub fn commit_trigger_narration(
    state: GameState,
    trigger: &StoredTriggerContext,
    continuation_text: &str,
) -> Result<GameState, EngineError> {
    if continuation_text.trim().is_empty() {
        return Ok(state);
    }
    let mut state = state;
    state.narrative.last_trigger = Some(trigger.clone());
    state.narrative.pending_event = Some(trigger.trigger_name.clone());
    state.add_message(continuation_text.to_string(), None, MessageType::Narration);
    if !trigger.trigger_repeat {
        state
            .npc_encounter_log
            .mark_trigger_fired(&trigger.npc_id, trigger.trigger_idx);
    }

    assert_state_consistency(&state)?;
    Ok(state)
}

pub fn execute_freeaction_impl(
    state: &GameState,
    ctx: &FreeActionContext<'_>,
) -> Result<TurnResult, EngineError> {
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

    next_state.scene.npcs_in_area = ctx
        .quantifier_result
        .npcs
        .npc_ids
        .iter()
        .filter_map(|id| next_state.npcs.get(id).cloned())
        .collect();
    let current_npc_ids = ctx.quantifier_result.npcs.npc_ids.clone();

    let trigger_match =
        evaluate_triggers(&next_state)
            .into_iter()
            .next()
            .map(|(npc, trigger, idx)| TriggerMatch {
                npc_id: npc.id,
                trigger_idx: idx,
                trigger_name: trigger.narration.name,
                trigger_repeat: trigger.repeat,
                trigger_narration_prompt: render_template(
                    &trigger.narration.narration_prompt,
                    &TemplateVars::new(&state.player.sheet.name),
                ),
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
