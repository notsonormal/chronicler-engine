use std::io::{self, Write};

use chronicler_engine::action::Action;
use chronicler_engine::character::{NpcCard, PlayerCard};
use chronicler_engine::llm::LlmBackend;
use chronicler_engine::map::MapDef;
use chronicler_engine::parser::parse_command;
use chronicler_engine::state::GameState;
use chronicler_engine::world::WorldCard;

use std::fs;
use std::path::Path;

fn main() {
    dotenv::dotenv().ok();
    println!("Loading Chronicler Engine...");

    let world_json =
        fs::read_to_string("data/world/redmist_estate.json").expect("Failed to read world.json");
    let world: WorldCard = serde_json::from_str(&world_json).expect("Failed to parse world.json");

    let map_json =
        fs::read_to_string("data/maps/redmist_estate.json").expect("Failed to read map.json");
    let map: MapDef = serde_json::from_str(&map_json).expect("Failed to parse map.json");

    let mut npcs = Vec::new();
    let chars_dir = Path::new("data/characters");
    if chars_dir.is_dir() {
        for entry in fs::read_dir(chars_dir).expect("Failed to read characters dir") {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().unwrap_or_default() == "json" {
                    let char_json = fs::read_to_string(&path).expect("Failed to read char json");
                    if let Ok(npc) = serde_json::from_str::<NpcCard>(&char_json) {
                        npcs.push(npc);
                    }
                }
            }
        }
    }

    let player = PlayerCard {
        name: "Hero".to_string(),
        description: "You are the protagonist.".to_string(),
        inventory: vec![],
    };

    let mut state = GameState::new(world, map, player, npcs, "front_gates".to_string());

    let llm_backend = chronicler_engine::llm::OpenRouterBackend;

    // 2. Start REPL
    println!("Welcome to {}.", state.world.name);
    println!("You can type 'look', 'go <dir>', 'talk <npc>', or 'quit'.\n");

    print_room(&state);

    loop {
        print_prompt(&state);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() || input.is_empty() {
            break;
        }

        let action = parse_command(&input);

        match action {
            Action::Quit => {
                println!("Goodbye!");
                break;
            }
            Action::Look => {
                print_room(&state);
            }
            Action::WalkTo(target) => {
                match chronicler_engine::engine::attempt_walk(&mut state, &target) {
                    Ok(msg) => {
                        println!("{}", msg);
                        print_room(&state);
                    }
                    Err(err_msg) => {
                        println!("{}", err_msg);
                    }
                }
            }
            Action::Talk(name, message) => {
                let current_room = chronicler_engine::engine::get_current_room(&state);
                let lower_name = name.to_lowercase();

                let mut found = false;
                for npc_id in &current_room.npcs {
                    if let Some(npc) = state.npcs.get(npc_id) {
                        if npc.name.to_lowercase() == lower_name {
                            println!("Talking to {}... (Generating reply)", npc.name);
                            let reply = chronicler_engine::llm::LlmBackend::generate_dialogue(
                                &llm_backend,
                                &state.world,
                                current_room,
                                npc,
                                &message,
                            );
                            println!("{}: \"{}\"", npc.name, reply.trim());
                            found = true;
                            break;
                        }
                    }
                }

                if !found {
                    println!("There is no one here by that name.");
                }
            }
            Action::Inventory => {
                println!("Your inventory is empty.");
            }
            Action::FreeAction(text) => {
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let current_room = chronicler_engine::engine::get_current_room(&state);
                let npc_refs: Vec<&NpcCard> = current_room
                    .npcs
                    .iter()
                    .filter_map(|id| state.npcs.get(id))
                    .collect();
                let narration = llm_backend.narrate_action(
                    &state.world,
                    current_room,
                    &npc_refs,
                    &state.player,
                    trimmed,
                );
                println!("{}", narration.trim());
            }
        }
    }
}

fn print_room(state: &GameState) {
    let room = chronicler_engine::engine::get_current_room(state);
    println!("=== {} ===", room.name);
    println!("{}", room.description);

    if !room.npcs.is_empty() {
        print!("You see: ");
        for npc_id in &room.npcs {
            if let Some(npc) = state.npcs.get(npc_id) {
                print!("{} ", npc.name);
            }
        }
        println!();
    }

    print!("Exits: ");
    for (dir, _) in &room.exits {
        print!("{:?} ", dir);
    }
    println!();
}

fn print_prompt(state: &GameState) {
    let room = chronicler_engine::engine::get_current_room(state);
    print!("\n");
    for (dir, _) in &room.exits {
        print!("[Move {:?}] ", dir);
    }
    print!("[Look] [Inventory] [Quit]");
    println!();
    print!("> ");
}
