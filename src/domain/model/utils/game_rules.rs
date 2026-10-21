//! [DOC: docs/diataxis/reference/game_flow.md]
//! The game rules: movement, the Characters present and the Triggers, as a pure function of the Swipe state

use std::collections::HashMap;

use crate::domain::model::agent::Kind;
use crate::domain::model::character::NpcCard;
use crate::domain::model::map::{MapDef, Room};
use crate::domain::model::quantifier::{NpcEvent, NpcEventList, NpcTransitionType, QuantifierResult};
use crate::domain::model::state::swipe_state::SwipeState;
use crate::domain::model::state::trigger_context::{EventRecord, TriggerRef};
use crate::domain::model::template::TemplateVars;
use crate::domain::model::trigger::Trigger;
use crate::error::{EngineError, Result};
#[cfg(feature = "diagnostics")]
use crate::error::InternalError;

/// What the rules read of the World: the map, the Characters and the Persona's name.
#[derive(Debug, Clone, Copy)]
pub struct GameWorld<'a> {
    pub map: &'a MapDef,
    pub characters: &'a HashMap<String, NpcCard>,
    pub persona_name: &'a str,
}

/// The rules write no story: the caller saves `state` and, when `fired` is set,
/// orders the event generation.
#[derive(Debug, Clone, PartialEq)]
pub struct RulesOutcome {
    pub state: SwipeState,
    /// The Trigger that fired, with its direction rendered. An event generation fires none.
    pub fired: Option<EventRecord>,
}

/// An event generation moves nobody and fires no Trigger; every other kind runs
/// movement, the Characters present, the Trigger and the encounters. `Err` is the
/// contradiction of the consistency check, on the state the rules leave.
///
/// The Trigger reads `times_met` before the Character events advance it; swapping the
/// two suppresses every first-encounter Trigger.
pub fn apply(
    mut state: SwipeState,
    result: &QuantifierResult,
    world: &GameWorld<'_>,
    kind: Kind,
) -> Result<RulesOutcome> {
    let is_event = kind == Kind::Event;
    let previous_npc_ids: Vec<String> = state
        .scene
        .npcs_in_area
        .iter()
        .map(|npc| npc.id.clone())
        .collect();

    if !is_event {
        movement(
            &mut state,
            result.destination.as_deref(),
            &result.present,
            world.map,
        );
    }
    assert_state_consistency(&state, world.map, world.characters)?;

    state.scene.npcs_in_area = result
        .present
        .iter()
        .filter_map(|id| world.characters.get(id).cloned())
        .collect();

    let fired = if is_event {
        None
    } else {
        fired_trigger(&state, world)
    };
    state.trigger_context = fired.clone();

    let events = NpcEventList::from_diff(&previous_npc_ids, &result.present);
    apply_npc_events(&mut state, &events.events);
    assert_state_consistency(&state, world.map, world.characters)?;

    Ok(RulesOutcome { state, fired })
}

fn fired_trigger(state: &SwipeState, world: &GameWorld<'_>) -> Option<EventRecord> {
    evaluate_triggers(state, world.characters)
        .into_iter()
        .next()
        .map(|(npc, trigger, index)| EventRecord {
            trigger: TriggerRef {
                npc_id: npc.id,
                trigger_idx: index,
                trigger_name: trigger.narration.name.clone(),
                trigger_repeat: trigger.repeat,
            },
            header_name: trigger.narration.name,
            direction: TemplateVars::new(world.persona_name)
                .render(&trigger.narration.narration_prompt),
        })
}

/// No destination moves nobody.
fn movement(
    state: &mut SwipeState,
    destination: Option<&str>,
    new_npc_ids: &[String],
    map: &MapDef,
) {
    let Some(destination) = destination else {
        return;
    };

    let previous_room_id = state.movement.current_room_id.clone();
    attempt_movement(state, destination, map);
    if previous_room_id != state.movement.current_room_id {
        for npc_id in new_npc_ids {
            state.npc_encounter_log.set_currently_meeting(npc_id, true);
        }
    }
}

/// A walk the map does not know becomes a room of its own.
fn attempt_movement(state: &mut SwipeState, destination: &str, map: &MapDef) {
    if let Err(e) = walk(state, map, destination) {
        tracing::debug!("Semantic walk failed for '{destination}': {e}");
        let dynamic_room = Room::new_dynamic(destination, "A place you have never seen before.");
        state.movement.current_room_id = dynamic_room.id.clone();
        state
            .movement
            .dynamic_rooms
            .insert(dynamic_room.id.clone(), dynamic_room);
    }
}

fn walk(state: &mut SwipeState, map: &MapDef, room_id: &str) -> Result<()> {
    if state.room_by_id(map, room_id).is_none() {
        return Err(EngineError::Navigation(
            "You don't see a way to go there.".to_string(),
        ));
    }

    state.movement.current_room_id = room_id.to_string();
    Ok(())
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
    if state.room(map).is_none() {
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
