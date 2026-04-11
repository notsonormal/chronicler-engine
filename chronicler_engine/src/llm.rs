use crate::character::{NpcCard, PlayerCard};
use crate::map::Room;
use crate::world::WorldCard;
use serde_json::json;

pub trait LlmBackend {
    fn generate_dialogue(
        &self,
        world: &WorldCard,
        room: &Room,
        npc: &NpcCard,
        user_message: &Option<String>,
    ) -> String;

    fn narrate_action(
        &self,
        world: &WorldCard,
        room: &Room,
        nearby_npcs: &[&NpcCard],
        player: &PlayerCard,
        player_input: &str,
    ) -> String;
}

pub struct OpenRouterBackend;

impl LlmBackend for OpenRouterBackend {
    fn generate_dialogue(
        &self,
        world: &WorldCard,
        room: &Room,
        npc: &NpcCard,
        user_message: &Option<String>,
    ) -> String {
        let api_key = std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");

        let mut system_prompt = format!(
            "You are a character in a text adventure game. Your name is {}.\n\
        Personality: {}\n\
        Scenario/Background: {}\n\
        \n",
            npc.name, npc.personality, npc.scenario
        );

        system_prompt.push_str(&format!(
            "Current Room: {} - {}\n\n",
            room.name, room.description
        ));

        system_prompt.push_str("World Rules:\n");
        for rule in &world.global_rules {
            system_prompt.push_str(&format!("- {}\n", rule));
        }

        system_prompt.push_str("\nInstructions: Roleplay as your character and respond to the player's action. Reply primarily with dialogue, and do not act or speak on behalf of the player.");

        let user_text = match user_message {
            Some(msg) => format!("The player says: \"{}\"", msg),
            None => "The player approaches you in silence, waiting for you to speak.".to_string(),
        };

        call_openrouter(&api_key, &system_prompt, &user_text)
    }

    fn narrate_action(
        &self,
        world: &WorldCard,
        room: &Room,
        nearby_npcs: &[&NpcCard],
        player: &PlayerCard,
        player_input: &str,
    ) -> String {
        let api_key = std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");

        let mut system_prompt = String::from(
            "You are the Game Master of a text adventure game. \
Narrate what happens in response to the player's action. \
Voice any NPCs present if they would logically react. \
Keep responses immersive, concise, and in the style of literary fiction. \
Never act or speak on behalf of the player.\n\n",
        );

        system_prompt.push_str(&format!(
            "Player: {} — {}\n\n",
            player.name, player.description
        ));

        system_prompt.push_str(&format!(
            "Current Location: {} — {}\n\n",
            room.name, room.description
        ));

        if !nearby_npcs.is_empty() {
            system_prompt.push_str("Characters present:\n");
            for npc in nearby_npcs {
                system_prompt.push_str(&format!(
                    "- {} ({}): {} Background: {}\n",
                    npc.name, npc.personality, npc.description, npc.scenario
                ));
            }
            system_prompt.push('\n');
        }

        system_prompt.push_str("World Lore:\n");
        for rule in &world.global_rules {
            system_prompt.push_str(&format!("- {}\n", rule));
        }

        let user_text = format!("The player does the following: {}", player_input);

        call_openrouter(&api_key, &system_prompt, &user_text)
    }
}

fn call_openrouter(api_key: &str, system_prompt: &str, user_text: &str) -> String {
    let client = reqwest::blocking::Client::new();

    let payload = json!({
        "model": "sao10k/l3.3-euryale-70b",
        "messages": [
            {
                "role": "system",
                "content": system_prompt
            },
            {
                "role": "user",
                "content": user_text
            }
        ]
    });

    let res = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&payload)
        .send();

    match res {
        Ok(response) => {
            if response.status().is_success() {
                if let Ok(json_response) = response.json::<serde_json::Value>() {
                    if let Some(content) =
                        json_response["choices"][0]["message"]["content"].as_str()
                    {
                        return content.to_string();
                    }
                }
                "The world seems to hold its breath (parse error).".to_string()
            } else {
                format!("Error communicating with OpenRouter: {}", response.status())
            }
        }
        Err(e) => format!("Request failed: {}", e),
    }
}

pub struct MockBackend;

impl LlmBackend for MockBackend {
    fn generate_dialogue(
        &self,
        _world: &WorldCard,
        _room: &Room,
        _npc: &NpcCard,
        user_message: &Option<String>,
    ) -> String {
        match user_message {
            Some(msg) => format!("[MockGenerated] Replying to: {}", msg),
            None => "[MockGenerated] Standard greeting.".to_string(),
        }
    }

    fn narrate_action(
        &self,
        _world: &WorldCard,
        _room: &Room,
        _nearby_npcs: &[&NpcCard],
        _player: &PlayerCard,
        player_input: &str,
    ) -> String {
        format!("[MockNarration] {}", player_input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::Room;
    use std::collections::HashMap;

    fn make_test_room() -> Room {
        Room {
            id: "room1".to_string(),
            name: "Test Room".to_string(),
            description: "A plain room.".to_string(),
            exits: HashMap::new(),
            items: vec![],
            npcs: vec![],
        }
    }

    fn make_test_world() -> WorldCard {
        WorldCard {
            name: "Test World".to_string(),
            description: "Testing.".to_string(),
            global_rules: vec!["Rule 1".to_string()],
        }
    }

    fn make_test_player() -> PlayerCard {
        PlayerCard {
            name: "Hero".to_string(),
            description: "The protagonist.".to_string(),
            inventory: vec![],
        }
    }

    #[test]
    fn test_mock_narrate_action() {
        let backend = MockBackend;
        let world = make_test_world();
        let room = make_test_room();
        let player = make_test_player();

        let result =
            backend.narrate_action(&world, &room, &[], &player, "I look around carefully.");
        assert_eq!(result, "[MockNarration] I look around carefully.");
    }

    #[test]
    fn test_mock_narrate_action_with_npcs() {
        let backend = MockBackend;
        let world = make_test_world();
        let room = make_test_room();
        let player = make_test_player();
        let npc = NpcCard {
            id: "carla".to_string(),
            name: "Carla".to_string(),
            description: "A bodyguard.".to_string(),
            personality: "Strict".to_string(),
            scenario: "Guards the estate.".to_string(),
            example_dialogue: String::new(),
            inventory: vec![],
        };

        let result = backend.narrate_action(&world, &room, &[&npc], &player, "Hello Carla!");
        assert_eq!(result, "[MockNarration] Hello Carla!");
    }
}
