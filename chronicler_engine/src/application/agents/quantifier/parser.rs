//! [DOC: chronicler_engine/docs/diataxis/reference/narrative/agent_system.md]
//! Quantifier parse entry points (`QuantifierParseResult::parse`, `QuantifierResult::parse_with_movement`).

use crate::domain::model::quantifier::QuantifierParseResult;

use super::utils::parser::{extract_npcs, parse_with_movement};

impl QuantifierParseResult {
    pub fn parse(response: &str, known_npc_ids: &[String]) -> Self {
        extract_npcs(response, known_npc_ids)
    }
}

impl crate::domain::model::quantifier::QuantifierResult {
    pub fn parse_with_movement(response: &str, known_npc_ids: &[String]) -> Self {
        parse_with_movement(response, known_npc_ids)
    }
}
