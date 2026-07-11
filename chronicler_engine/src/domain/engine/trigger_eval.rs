//! [DOC: docs/system/triggers.md]
//! Trigger evaluation and condition checking

use crate::domain::model::character::NpcCard;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::trigger::{ComparisonOperator, Trigger, TriggerRequirement};
pub fn evaluate_triggers(
    state: &GameState,
    npcs: &std::collections::HashMap<String, NpcCard>,
) -> Vec<(NpcCard, Trigger, usize)> {
    let current_room_id = &state.movement.current_room_id;

    let mut results = Vec::new();

    for npc in npcs.values() {
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

pub fn check_condition(
    npc_encounter_log: &crate::domain::model::trigger::NpcEncounterLog,
    npc_id: &str,
    condition: &TriggerRequirement,
) -> bool {
    let TriggerRequirement {
        operator,
        threshold,
    } = condition;
    let times_met = npc_encounter_log.get_times_met(npc_id);
    match operator {
        ComparisonOperator::Eq => times_met == *threshold,
        ComparisonOperator::Lt => times_met < *threshold,
        ComparisonOperator::Gte => times_met >= *threshold,
    }
}
