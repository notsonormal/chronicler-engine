//! [DOC: docs/system/triggers.md]
//! Trigger evaluation and condition checking

use crate::model::character::NpcCard;
use crate::model::state::GameState;
use crate::model::trigger::{ComparisonOperator, NpcEncounterLog, Trigger, TriggerRequirement};
pub fn evaluate_triggers(state: &GameState) -> Vec<(NpcCard, Trigger, usize)> {
    let current_room_id = &state.movement.current_room_id;

    let mut results = Vec::new();

    for npc in state.npcs.values() {
        for (index, trigger) in npc.triggers.iter().enumerate() {
            if let Some(room_id) = &trigger.room_id {
                if room_id != current_room_id {
                    tracing::debug!(
                        npc_id = %npc.id,
                        trigger = %trigger.narration.name,
                        reason = "room_mismatch",
                        "Trigger skipped"
                    );
                    continue;
                }
            }

            if !check_condition(&state.npc_encounter_log, &npc.id, &trigger.requirement) {
                tracing::debug!(
                    npc_id = %npc.id,
                    trigger = %trigger.narration.name,
                    reason = "condition_not_met",
                    "Trigger skipped"
                );
                continue;
            }

            if !trigger.repeat && is_trigger_fired(&state.npc_encounter_log, &npc.id, index) {
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

pub fn check_condition(
    npc_encounter_log: &crate::model::trigger::NpcEncounterLog,
    npc_id: &str,
    condition: &TriggerRequirement,
) -> bool {
    let TriggerRequirement {
        operator,
        threshold,
    } = condition;
    let times_met = get_times_met(npc_encounter_log, npc_id);
    match operator {
        ComparisonOperator::Eq => times_met == *threshold,
        ComparisonOperator::Lt => times_met < *threshold,
        ComparisonOperator::Gte => times_met >= *threshold,
    }
}

pub fn is_currently_meeting(npc_encounter_log: &NpcEncounterLog, npc_id: &str) -> bool {
    npc_encounter_log
        .npcs
        .get(npc_id)
        .map(|s| s.currently_meeting)
        .unwrap_or(false)
}

pub fn increment_times_met(npc_encounter_log: &mut NpcEncounterLog, npc_id: &str) {
    let entry = npc_encounter_log
        .npcs
        .entry(npc_id.to_string())
        .or_default();
    entry.times_met += 1;
}

pub fn mark_trigger_fired(
    npc_encounter_log: &mut NpcEncounterLog,
    npc_id: &str,
    trigger_index: usize,
) {
    let entry = npc_encounter_log
        .npcs
        .entry(npc_id.to_string())
        .or_default();
    entry.trigger_fired.insert(trigger_index, true);
}

pub fn set_currently_meeting(npc_encounter_log: &mut NpcEncounterLog, npc_id: &str, meeting: bool) {
    let entry = npc_encounter_log
        .npcs
        .entry(npc_id.to_string())
        .or_default();
    entry.currently_meeting = meeting;
}

pub fn get_times_met(npc_encounter_log: &NpcEncounterLog, npc_id: &str) -> u32 {
    npc_encounter_log
        .npcs
        .get(npc_id)
        .map(|s| s.times_met)
        .unwrap_or(0)
}

pub fn is_trigger_fired(
    npc_encounter_log: &NpcEncounterLog,
    npc_id: &str,
    trigger_index: usize,
) -> bool {
    npc_encounter_log
        .npcs
        .get(npc_id)
        .and_then(|s| s.trigger_fired.get(&trigger_index))
        .copied()
        .unwrap_or(false)
}
