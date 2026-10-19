//! [DOC: docs/diataxis/reference/game_flow.md]
//! The game rules: movement, the Characters present and the Triggers, as a pure function of the Swipe state

use std::collections::HashMap;

use crate::domain::model::character::NpcCard;
use crate::domain::model::map::{MapDef, Room};
use crate::domain::model::quantifier::{NpcEvent, NpcEventList, NpcTransitionType, QuantifierResult};
use crate::domain::model::state::swipe_state::SwipeState;
use crate::domain::model::state::trigger_context::TriggerRef;
use crate::domain::model::template::TemplateVars;
use crate::domain::model::trigger::Trigger;
use crate::error::{EngineError, Result};
#[cfg(feature = "diagnostics")]
use crate::error::InternalError;

#[derive(Debug, Clone, PartialEq)]
pub struct TriggerMatch {
    pub npc_id: String,
    pub trigger_idx: usize,
    pub trigger_name: String,
    pub trigger_repeat: bool,
    pub trigger_narration_prompt: String,
}

pub enum GameRuleRequest<'a> {
    /// The whole rule step of one Narration: movement, Characters and Trigger.
    Analysis {
        quantifier_result: &'a QuantifierResult,
        persona_name: &'a str,
    },
    /// The Characters that entered or left.
    NpcEvents(&'a [NpcEvent]),
    /// The Trigger narration that landed: which Trigger it was, and the text.
    TriggerNarration {
        trigger: &'a TriggerRef,
        text: &'a str,
    },
}

/// The rules write no story: the caller turns these into writes.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct GameRulesOutcome {
    pub arrived_room_name: Option<String>,
    /// The room the rules added, because no room known to the map had the destination.
    pub unknown_location: Option<Room>,
    /// The Trigger that matched, read before the Character events advanced `times_met`.
    pub trigger_match: Option<TriggerMatch>,
    /// The Trigger narration landed; an empty text lands nothing.
    pub trigger_narration_landed: bool,
}

/// Except for `TriggerNarration`, the consistency check is the returned `Err`: it runs on the
/// state the step left. A landed Trigger narration is the caller's to check, because the caller
/// reports that failure with a state that already holds the event Narration.
pub fn apply_game_rules(
    state: &mut SwipeState,
    request: GameRuleRequest<'_>,
    map: &MapDef,
    npcs: &HashMap<String, NpcCard>,
) -> Result<GameRulesOutcome> {
    match request {
        GameRuleRequest::Analysis {
            quantifier_result,
            persona_name,
        } => analysis(state, quantifier_result, persona_name, map, npcs),
        GameRuleRequest::NpcEvents(events) => {
            apply_npc_events(state, events);
            assert_state_consistency(state, map, npcs)?;
            Ok(GameRulesOutcome::default())
        }
        GameRuleRequest::TriggerNarration { trigger, text } => {
            Ok(trigger_narration(state, trigger, text))
        }
    }
}

/// Triggers read `times_met` before the Character events below advance it; swapping the two
/// suppresses every first-encounter Trigger.
fn analysis(
    state: &mut SwipeState,
    quantifier_result: &QuantifierResult,
    persona_name: &str,
    map: &MapDef,
    npcs: &HashMap<String, NpcCard>,
) -> Result<GameRulesOutcome> {
    let previous_npc_ids: Vec<String> = state
        .scene
        .npcs_in_area
        .iter()
        .map(|npc| npc.id.clone())
        .collect();

    let mut outcome = movement(
        state,
        quantifier_result.movement.destination.as_deref(),
        &quantifier_result.npcs.npc_ids,
        map,
    );
    assert_state_consistency(state, map, npcs)?;

    state.scene.npcs_in_area = quantifier_result
        .npcs
        .npc_ids
        .iter()
        .filter_map(|id| npcs.get(id).cloned())
        .collect();
    let current_npc_ids = quantifier_result.npcs.npc_ids.clone();

    outcome.trigger_match =
        evaluate_triggers(state, npcs)
            .into_iter()
            .next()
            .map(|(npc, trigger, index)| TriggerMatch {
                npc_id: npc.id,
                trigger_idx: index,
                trigger_name: trigger.narration.name,
                trigger_repeat: trigger.repeat,
                trigger_narration_prompt: TemplateVars::new(persona_name)
                    .render(&trigger.narration.narration_prompt),
            });

    let events = NpcEventList::from_diff(&previous_npc_ids, &current_npc_ids);
    apply_npc_events(state, &events.events);
    assert_state_consistency(state, map, npcs)?;

    Ok(outcome)
}

/// No destination moves nobody and reports no room.
fn movement(
    state: &mut SwipeState,
    destination: Option<&str>,
    new_npc_ids: &[String],
    map: &MapDef,
) -> GameRulesOutcome {
    let Some(destination) = destination else {
        return GameRulesOutcome::default();
    };

    let previous_room_id = state.movement.current_room_id.clone();
    let unknown_location = attempt_movement(state, destination, map);
    if previous_room_id != state.movement.current_room_id {
        for npc_id in new_npc_ids {
            state.npc_encounter_log.set_currently_meeting(npc_id, true);
        }
    }
    GameRulesOutcome {
        arrived_room_name: room_name(state, map),
        unknown_location,
        ..Default::default()
    }
}

/// A walk the map does not know becomes a room of its own.
fn attempt_movement(state: &mut SwipeState, destination: &str, map: &MapDef) -> Option<Room> {
    match walk(state, map, destination) {
        Ok(()) => None,
        Err(e) => {
            tracing::debug!("Semantic walk failed for '{destination}': {e}");
            let dynamic_room =
                Room::new_dynamic(destination, "A place you have never seen before.");
            state
                .movement
                .dynamic_rooms
                .insert(dynamic_room.id.clone(), dynamic_room.clone());
            state.movement.current_room_id = dynamic_room.id.clone();
            Some(dynamic_room)
        }
    }
}

fn walk(state: &mut SwipeState, map: &MapDef, room_id: &str) -> Result<()> {
    if room_of(state, map, room_id).is_none() {
        return Err(EngineError::Navigation(
            "You don't see a way to go there.".to_string(),
        ));
    }

    state.movement.current_room_id = room_id.to_string();
    Ok(())
}

fn room_of<'a>(state: &'a SwipeState, map: &'a MapDef, room_id: &str) -> Option<&'a Room> {
    map.get_room_by_id(room_id)
        .or_else(|| state.movement.dynamic_rooms.get(room_id))
}

fn room_name(state: &SwipeState, map: &MapDef) -> Option<String> {
    room_of(state, map, &state.movement.current_room_id).map(|room| room.name.clone())
}

fn apply_npc_events(state: &mut SwipeState, events: &[NpcEvent]) {
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
}

fn trigger_narration(state: &mut SwipeState, trigger: &TriggerRef, text: &str) -> GameRulesOutcome {
    if text.trim().is_empty() {
        return GameRulesOutcome::default();
    }

    if !trigger.trigger_repeat {
        state
            .npc_encounter_log
            .mark_trigger_fired(&trigger.npc_id, trigger.trigger_idx);
    }

    GameRulesOutcome {
        trigger_narration_landed: true,
        ..Default::default()
    }
}

/// Each result carries the Trigger's index on its Character's Trigger list.
fn evaluate_triggers(
    state: &SwipeState,
    npcs: &HashMap<String, NpcCard>,
) -> Vec<(NpcCard, Trigger, usize)> {
    let current_room_id = &state.movement.current_room_id;
    let mut results = Vec::new();

    for npc in npcs.values() {
        for (index, trigger) in npc.triggers.iter().enumerate() {
            if trigger
                .room_id
                .as_deref()
                .is_some_and(|room_id| room_id != current_room_id)
            {
                tracing::debug!(
                    npc_id = %npc.id,
                    trigger = %trigger.narration.name,
                    reason = "room_mismatch",
                    "Trigger skipped"
                );
                continue;
            }

            if !state
                .npc_encounter_log
                .check_condition(&npc.id, &trigger.requirement)
            {
                tracing::debug!(
                    npc_id = %npc.id,
                    trigger = %trigger.narration.name,
                    reason = "condition_not_met",
                    "Trigger skipped"
                );
                continue;
            }

            if !trigger.repeat && state.npc_encounter_log.is_trigger_fired(&npc.id, index) {
                tracing::debug!(
                    npc_id = %npc.id,
                    trigger = %trigger.narration.name,
                    reason = "already_fired",
                    "Trigger skipped"
                );
                continue;
            }

            results.push((npc.clone(), trigger.clone(), index));
        }
    }

    results
}

/// The "last Narration after last player Input" ordering over the story is not
/// a global invariant: `POST /history/delete` legitimately pops a trailing
/// Narration, so a state whose last Message is an Input is a valid rest state.
/// This check therefore omits it.
#[cfg(feature = "diagnostics")]
pub(crate) fn assert_state_consistency(
    state: &SwipeState,
    map: &MapDef,
    npcs: &HashMap<String, NpcCard>,
) -> Result<()> {
    assert_room_exists(state, map)?;
    assert_npc_consistency(state, npcs)?;
    assert_npc_encounter_log_consistency(state, npcs)?;
    Ok(())
}

#[cfg(not(feature = "diagnostics"))]
pub(crate) fn assert_state_consistency(
    _state: &SwipeState,
    _map: &MapDef,
    _npcs: &HashMap<String, NpcCard>,
) -> Result<()> {
    Ok(())
}

#[cfg(feature = "diagnostics")]
fn assert_room_exists(state: &SwipeState, map: &MapDef) -> Result<()> {
    if room_of(state, map, &state.movement.current_room_id).is_none() {
        return Err(EngineError::Internal(InternalError::new(format!(
            "current_room_id '{}' not found in map or dynamic_rooms",
            state.movement.current_room_id
        ))));
    }
    Ok(())
}

#[cfg(feature = "diagnostics")]
fn assert_npc_consistency(state: &SwipeState, npcs: &HashMap<String, NpcCard>) -> Result<()> {
    for npc in &state.scene.npcs_in_area {
        if !npcs.contains_key(&npc.id) {
            return Err(EngineError::Internal(InternalError::new(format!(
                "npcs_in_area contains NPC '{}' which is not in the npcs map",
                npc.id
            ))));
        }
    }
    Ok(())
}

#[cfg(feature = "diagnostics")]
fn assert_npc_encounter_log_consistency(
    state: &SwipeState,
    npcs: &HashMap<String, NpcCard>,
) -> Result<()> {
    for npc_id in state.npc_encounter_log.npcs.keys() {
        if !npcs.contains_key(npc_id) {
            return Err(EngineError::Internal(InternalError::new(format!(
                "npc_encounter_log references unknown NPC '{npc_id}'"
            ))));
        }
    }
    Ok(())
}
