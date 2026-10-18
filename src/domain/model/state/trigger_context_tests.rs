//! The trigger record a generation writes, one branch per test.

use crate::domain::model::utils::game_rules::TriggerMatch;

use super::trigger_context::{EventRecord, TriggerRef};

fn trigger_match() -> TriggerMatch {
    TriggerMatch {
        npc_id: "npc_1".to_string(),
        trigger_idx: 2,
        trigger_name: "The gate opens".to_string(),
        trigger_repeat: false,
        trigger_narration_prompt: "the gate swings open".to_string(),
    }
}

#[test]
fn trigger_ref_names_the_trigger_that_fired() {
    assert_eq!(
        TriggerRef::from(&trigger_match()),
        TriggerRef {
            npc_id: "npc_1".to_string(),
            trigger_idx: 2,
            trigger_name: "The gate opens".to_string(),
            trigger_repeat: false,
        }
    );
}

#[test]
fn event_record_carries_the_trigger_and_its_direction() {
    assert_eq!(
        EventRecord::from(&trigger_match()),
        EventRecord {
            trigger: TriggerRef::from(&trigger_match()),
            header_name: "The gate opens".to_string(),
            direction: "the gate swings open".to_string(),
        }
    );
}
