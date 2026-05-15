use crate::model::character::NpcCard;
use crate::model::state::GameState;
use crate::model::trigger::{CharacterState, ComparisonOperator, Trigger, TriggerCondition};

/// [DOC: docs/system/triggers.md]
pub fn evaluate_triggers(state: &GameState) -> Vec<(NpcCard, Trigger, usize)> {
    let current_room_id = &state.movement.current_room_id;

    let mut results = Vec::new();

    for npc in state.npcs.values() {
        for (index, trigger) in npc.triggers.iter().enumerate() {
            if let Some(room_id) = &trigger.room_id {
                if room_id != current_room_id {
                    log::debug!(
                        "[Trigger] '{}' skipped: room_id mismatch (expected '{}', current '{}')",
                        trigger.action.name,
                        room_id,
                        current_room_id
                    );
                    continue;
                }
            }

            if check_condition(&state.character_state, &npc.id, &trigger.condition) {
                if !trigger.repeat && is_trigger_fired(&state.character_state, &npc.id, index) {
                    log::debug!(
                        "[Trigger] '{}' skipped: already fired (non-repeatable)",
                        trigger.action.name
                    );
                    continue;
                }
                results.push((npc.clone(), trigger.clone(), index));
            } else {
                log::debug!(
                    "[Trigger] '{}' skipped: condition not met for NPC '{}'",
                    trigger.action.name,
                    npc.id
                );
            }
        }
    }

    results
}

/// [DOC: docs/system/triggers.md]
pub fn check_condition(
    character_state: &crate::model::trigger::CharacterState,
    npc_id: &str,
    condition: &TriggerCondition,
) -> bool {
    match condition {
        TriggerCondition::TimesMet(op, threshold) => {
            let times_met = get_times_met(character_state, npc_id);
            match op {
                ComparisonOperator::Eq => times_met == *threshold,
                ComparisonOperator::Lt => times_met < *threshold,
                ComparisonOperator::Gte => times_met >= *threshold,
            }
        }
    }
}

pub fn is_currently_meeting(character_state: &CharacterState, npc_id: &str) -> bool {
    character_state
        .npcs
        .get(npc_id)
        .map(|s| s.currently_meeting)
        .unwrap_or(false)
}

pub fn increment_times_met(character_state: &mut CharacterState, npc_id: &str) {
    let entry = character_state.npcs.entry(npc_id.to_string()).or_default();
    entry.times_met += 1;
}

pub fn mark_trigger_fired(
    character_state: &mut CharacterState,
    npc_id: &str,
    trigger_index: usize,
) {
    let entry = character_state.npcs.entry(npc_id.to_string()).or_default();
    entry.trigger_fired.insert(trigger_index, true);
}

pub fn set_currently_meeting(character_state: &mut CharacterState, npc_id: &str, meeting: bool) {
    let entry = character_state.npcs.entry(npc_id.to_string()).or_default();
    entry.currently_meeting = meeting;
}

pub fn get_times_met(character_state: &CharacterState, npc_id: &str) -> u32 {
    character_state
        .npcs
        .get(npc_id)
        .map(|s| s.times_met)
        .unwrap_or(0)
}

pub fn is_trigger_fired(
    character_state: &CharacterState,
    npc_id: &str,
    trigger_index: usize,
) -> bool {
    character_state
        .npcs
        .get(npc_id)
        .and_then(|s| s.trigger_fired.get(&trigger_index))
        .copied()
        .unwrap_or(false)
}
