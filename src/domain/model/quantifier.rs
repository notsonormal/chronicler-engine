//! [DOC: docs/diataxis/reference/game_flow.md]
//! The Quantifier result and the Character transitions the Game rules derive from it

/// What the Quantifier found in a generation's new text. The Game rules apply it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct QuantifierResult {
    pub present: Vec<String>,
    /// The room the player walks to, by id or name.
    pub destination: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CharacterTransitionType {
    /// The Character was not among the Characters present and now is.
    Entered,
    /// The Character was among the Characters present and now is not.
    Left,
}

/// The engine computes a transition from the set difference of the Characters present before
/// and after; the Quantifier result does not name transitions.
#[derive(Debug, Clone)]
pub struct CharacterTransition {
    pub character_id: String,
    pub event_type: CharacterTransitionType,
}

#[derive(Debug, Clone, Default)]
pub struct CharacterTransitions {
    pub events: Vec<CharacterTransition>,
}

impl CharacterTransitions {
    pub fn from_diff(previous_character_ids: &[String], current_character_ids: &[String]) -> Self {
        let previous_set: std::collections::HashSet<_> = previous_character_ids.iter().collect();
        let current_set: std::collections::HashSet<_> = current_character_ids.iter().collect();

        let mut events = Vec::new();

        for character_id in current_character_ids {
            if !previous_set.contains(character_id) {
                events.push(CharacterTransition {
                    character_id: character_id.clone(),
                    event_type: CharacterTransitionType::Entered,
                });
            }
        }
        for character_id in previous_character_ids {
            if !current_set.contains(character_id) {
                events.push(CharacterTransition {
                    character_id: character_id.clone(),
                    event_type: CharacterTransitionType::Left,
                });
            }
        }

        Self { events }
    }
}
