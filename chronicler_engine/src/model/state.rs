use crate::model::character::{NpcCard, PlayerCard};
use crate::model::map::MapDef;
use crate::model::world::WorldCard;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogType {
    Narration,
    Dialogue,
    System,
    Input,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogEntry {
    pub sender: Option<String>,
    pub text: String,
    pub log_type: LogType,
}

#[derive(Debug, Default)]
pub struct TuiState {
    pub input: String,
    pub cursor_position: usize,
    pub scroll_offset: u16,
    pub is_generating: bool,
}

impl TuiState {
    pub fn push_char(&mut self, c: char) {
        self.input.push(c);
        self.cursor_position += 1;
    }

    pub fn pop_char(&mut self) {
        if !self.input.is_empty() {
            self.input.pop();
            self.cursor_position = self.cursor_position.saturating_sub(1);
        }
    }

    pub fn clear_input(&mut self) {
        self.input.clear();
        self.cursor_position = 0;
    }
}

use std::sync::Arc;

#[derive(Debug)]
pub struct GameState {
    pub world: Arc<WorldCard>,
    pub map: Arc<MapDef>,
    pub player: Arc<PlayerCard>,
    pub npcs: HashMap<String, NpcCard>,
    pub current_room_id: String,
    pub narration_history: Vec<LogEntry>,
    pub tui_state: TuiState,
}

impl GameState {
    pub fn new(
        world: Arc<WorldCard>,
        map: Arc<MapDef>,
        player: Arc<PlayerCard>,
        npcs: Vec<NpcCard>,
        starting_room: String,
    ) -> Self {
        let mut npcs_map = HashMap::new();
        for npc in npcs {
            npcs_map.insert(npc.id.clone(), npc);
        }
        Self {
            world,
            map,
            player,
            npcs: npcs_map,
            current_room_id: starting_room,
            narration_history: Vec::new(),
            tui_state: TuiState::default(),
        }
    }

    pub fn add_log(&mut self, text: String, sender: Option<String>, log_type: LogType) {
        self.narration_history.push(LogEntry {
            sender,
            text,
            log_type,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::character::{CharacterSheet, NpcCard, PlayerCard};
    use crate::model::world::WorldCard;
    use crate::model::map::MapDef;

    #[test]
    fn test_game_state_initialization() {
        let world = WorldCard {
            name: "W".into(),
            description: "D".into(),
            global_rules: vec![],
        };
        let map = MapDef {
            overworld: crate::model::map::Overworld {
                id: "ow".into(),
                name: "ow".into(),
                regions: vec![],
            },
        };
        let player = PlayerCard {
            sheet: CharacterSheet {
                name: "P".into(),
                description: "P".into(),
                personality: "P".into(),
                scenario: "S".into(),
                example_dialogue: "E".into(),
                image_path: None,
            },
            inventory: vec![],
        };
        let npc = NpcCard {
            id: "npc_1".into(),
            sheet: CharacterSheet {
                name: "N".into(),
                description: "D".into(),
                personality: "P".into(),
                scenario: "S".into(),
                example_dialogue: "E".into(),
                image_path: None,
            },
            inventory: vec![],
        };

        let state = GameState::new(
            Arc::new(world),
            Arc::new(map),
            Arc::new(player),
            vec![npc],
            "room_1".to_string()
        );

        assert_eq!(state.current_room_id, "room_1");
        assert_eq!(state.npcs.len(), 1);
        assert!(state.npcs.contains_key("npc_1"));
    }

    #[test]
    fn test_tui_state_input_robustness() {
        let mut tui = TuiState::default();
        
        // Test push
        tui.push_char('A');
        assert_eq!(tui.input, "A");
        assert_eq!(tui.cursor_position, 1);

        // Test pop
        tui.pop_char();
        assert_eq!(tui.input, "");
        assert_eq!(tui.cursor_position, 0);

        // Test underflow protection (Negative Case)
        tui.pop_char(); 
        assert_eq!(tui.cursor_position, 0); // Still 0, no panic
        
        // Test clear
        tui.push_char('h');
        tui.clear_input();
        assert_eq!(tui.input, "");
        assert_eq!(tui.cursor_position, 0);
    }

    #[test]
    fn test_log_ordering() {
        let mut state = GameState::new(
            Arc::new(WorldCard { name: "W".into(), description: "D".into(), global_rules: vec![] }),
            Arc::new(MapDef { overworld: crate::model::map::Overworld { id: "o".into(), name: "o".into(), regions: vec![] } }),
            Arc::new(PlayerCard { sheet: CharacterSheet { name: "P".into(), description: "D".into(), personality: "P".into(), scenario: "S".into(), example_dialogue: "E".into(), image_path: None }, inventory: vec![] }),
            vec![],
            "room1".into()
        );

        state.add_log("Message 1".into(), None, LogType::Narration);
        state.add_log("Message 2".into(), None, LogType::Narration);

        assert_eq!(state.narration_history.len(), 2);
        assert_eq!(state.narration_history[0].text, "Message 1");
        assert_eq!(state.narration_history[1].text, "Message 2");
    }
}
