use std::io::stdout;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use chronicler_engine::engine::action::Action;
use chronicler_engine::model::character::{NpcCard, PlayerCard};
use chronicler_engine::narrative::llm::{LlmBackend, OpenRouterBackend};
use chronicler_engine::model::map::MapDef;
use chronicler_engine::engine::parser::parse_command;
use chronicler_engine::model::state::{GameState, LogType};
use chronicler_engine::model::world::WorldCard;
use chronicler_engine::error::Result;
use std::sync::Arc;

use std::fs;
use std::path::Path;

enum Message {
    LlmResponse(String, String, LogType), // (SenderName, Text, Type)
    SystemMessage(String),
}

fn main() -> Result<()> {
    dotenv::dotenv().ok();

    // 1. Data Loading
    let world_json = fs::read_to_string("data/world/redmist_estate.json")?;
    let world: WorldCard = serde_json::from_str(&world_json)?;

    let map_json = fs::read_to_string("data/maps/redmist_estate.json")?;
    let map: MapDef = serde_json::from_str(&map_json)?;

    let mut npcs = Vec::new();
    let chars_dir = Path::new("data/characters");
    if chars_dir.is_dir() {
        for entry in fs::read_dir(chars_dir)? {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().unwrap_or_default() == "json" {
                    let char_json = fs::read_to_string(&path)?;
                    if let Ok(npc) = serde_json::from_str::<NpcCard>(&char_json) {
                        npcs.push(npc);
                    }
                }
            }
        }
    }

    let player_json = fs::read_to_string("data/personas/julian.json")?;
    let player: PlayerCard = serde_json::from_str(&player_json)?;

    let mut state = GameState::new(
        Arc::new(world),
        Arc::new(map),
        Arc::new(player),
        npcs,
        "front_gates".to_string()
    );
    let llm_backend = OpenRouterBackend;

    // Initial greeting and room description
    state.add_log(format!("Welcome to {}.", state.world.name), None, LogType::System);
    state.add_log(format!("Logged in as: {}", state.player.sheet.name), None, LogType::System);
    
    let current_room = chronicler_engine::engine::logic::get_current_room(&state);
    state.add_log(current_room.description.clone(), Some(current_room.name.clone()), LogType::Narration);

    // 2. TUI Setup
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 3. Channels for Background GM
    let (tx, rx): (Sender<Message>, Receiver<Message>) = mpsc::channel();

    // 4. Main Event Loop
    loop {
        terminal.draw(|f| chronicler_engine::ui::dashboard::draw(f, &state))?;

        // Check for background messages
        while let Ok(msg) = rx.try_recv() {
            match msg {
                Message::LlmResponse(sender, text, log_type) => {
                    state.add_log(text, Some(sender), log_type);
                    state.tui_state.is_generating = false;
                }
                Message::SystemMessage(text) => {
                    state.add_log(text, None, LogType::System);
                    state.tui_state.is_generating = false;
                }
            }
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if state.tui_state.is_generating {
                        // Prevent input while generating
                        continue;
                    }

                    match key.code {
                        KeyCode::Char(c) => {
                            state.tui_state.push_char(c);
                        }
                        KeyCode::Backspace => {
                            state.tui_state.pop_char();
                        }
                        KeyCode::Enter => {
                            let input = state.tui_state.input.clone();
                            state.tui_state.clear_input();
                            
                            if !input.trim().is_empty() {
                                handle_action(&mut state, input, llm_backend.clone(), tx.clone());
                            }
                        }
                        KeyCode::Esc => break,
                        KeyCode::Up => {
                            if state.tui_state.scroll_offset > 0 {
                                state.tui_state.scroll_offset -= 1;
                            }
                        }
                        KeyCode::Down => {
                            state.tui_state.scroll_offset += 1;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn handle_action(
    state: &mut GameState,
    input: String,
    llm: OpenRouterBackend,
    tx: Sender<Message>,
) {
    state.add_log(input.clone(), Some(state.player.sheet.name.clone()), LogType::Input);
    
    let action = parse_command(&input);
    match action {
        Action::Quit => {
            let _ = tx.send(Message::SystemMessage("Goodbye!".to_string()));
        }
        Action::Look => {
            let room = chronicler_engine::engine::logic::get_current_room(state);
            state.add_log(room.description.clone(), Some(room.name.clone()), LogType::Narration);
        }
        Action::WalkTo(target) => {
            match chronicler_engine::engine::logic::attempt_walk(state, &target) {
                Ok(_) => {
                    let room = chronicler_engine::engine::logic::get_current_room(state).clone();
                    let npc_refs: Vec<NpcCard> = room.npcs.iter()
                        .filter_map(|id| state.npcs.get(id).cloned())
                        .collect();
                    let world = Arc::clone(&state.world);
                    let player = Arc::clone(&state.player);
                    let llm = llm; // No clone needed for ZST Copy type, but Arc would handle it if not
                    let tx = tx.clone();
                    
                    state.tui_state.is_generating = true;
                    state.add_log(room.description.clone(), Some(room.name.clone()), LogType::Narration);

                    thread::spawn(move || {
                        let npc_ptrs: Vec<&NpcCard> = npc_refs.iter().collect();
                        let narration = llm.narrate_arrival(&world, &room, &npc_ptrs, &player);
                        let _ = tx.send(Message::LlmResponse("Game Master".to_string(), narration, LogType::Narration));
                    });
                }
                Err(e) => {
                    state.add_log(e.to_string(), None, LogType::System);
                }
            }
        }
        Action::Talk(name, msg) => {
            let current_room = chronicler_engine::engine::logic::get_current_room(state);
            let lower_name = name.to_lowercase();
            
            let npc = current_room.npcs.iter()
                .filter_map(|id| state.npcs.get(id))
                .find(|n| n.sheet.name.to_lowercase() == lower_name)
                .cloned();

            if let Some(npc) = npc {
                let llm = llm;
                let tx = tx.clone();
                let world = Arc::clone(&state.world);
                let room = current_room.clone();
                
                state.tui_state.is_generating = true;
                thread::spawn(move || {
                    let reply = llm.generate_dialogue(&world, &room, &npc, &msg);
                    let _ = tx.send(Message::LlmResponse(npc.sheet.name.clone(), reply, LogType::Dialogue));
                });
            } else {
                state.add_log("There is no one here by that name.".to_string(), None, LogType::System);
            }
        }
        Action::Inventory => {
            state.add_log("Your inventory is empty.".to_string(), None, LogType::System);
        }
        Action::FreeAction(text) => {
            let room = chronicler_engine::engine::logic::get_current_room(state).clone();
            let npc_refs: Vec<NpcCard> = room.npcs.iter()
                .filter_map(|id| state.npcs.get(id).cloned())
                .collect();
            let world = Arc::clone(&state.world);
            let player = Arc::clone(&state.player);
            let llm = llm;
            let tx = tx.clone();
            
            state.tui_state.is_generating = true;
            thread::spawn(move || {
                let npc_ptrs: Vec<&NpcCard> = npc_refs.iter().collect();
                let narration = llm.narrate_action(&world, &room, &npc_ptrs, &player, &text);
                let _ = tx.send(Message::LlmResponse("Game Master".to_string(), narration, LogType::Narration));
            });
        }
    }
}
