// DEAD CODE: TUI removed in favor of HTMX web dashboard (see server/ module)
// This file is kept for reference only - not compiled

#![allow(dead_code)]

use crate::engine::logic::get_current_room;
use crate::model::map::Room;
use crate::model::state::{GameState, LogType};
use image::GenericImageView;
use lazy_static::lazy_static;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use ratatui_image::protocol::Protocol;
use ratatui_image::{Image, Resize};
use std::sync::Mutex;

lazy_static! {
    static ref PROTOCOL_CACHE: Mutex<HashMap<String, Protocol>> = Mutex::new(HashMap::new());
}

/// Main entry point for TUI rendering
pub fn draw(f: &mut Frame, state: &GameState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main Body
            Constraint::Length(5), // Input Area
        ])
        .split(f.area());

    // Get current room, handle error gracefully
    let Ok(room) = get_current_room(state) else {
        // If no valid room, just render header and input areas
        draw_input(f, chunks[2], state);
        return;
    };

    draw_header(f, chunks[0], room);

    // Split Body
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70), // Narration Log
            Constraint::Percentage(30), // Visual Sidebar
        ])
        .split(chunks[1]);

    draw_story_log(f, body_chunks[0], state);
    draw_sidebar(f, body_chunks[1], state, room);
    draw_input(f, chunks[2], state);
}

fn draw_header(f: &mut Frame, area: Rect, room: &Room) {
    let header = Paragraph::new(Line::from(vec![
        Span::raw(" Chronicler Engine | "),
        Span::styled(
            &room.name,
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, area);
}

fn draw_story_log(f: &mut Frame, area: Rect, state: &GameState) {
    let history: Vec<Line> = state
        .narration_history
        .iter()
        .map(|entry| {
            let mut spans = Vec::new();
            if let Some(sender) = &entry.sender {
                spans.push(Span::styled(
                    format!("{}: ", sender),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ));
            }

            let color = match entry.log_type {
                LogType::Narration => Color::Cyan,
                LogType::Dialogue => Color::White,
                LogType::System => Color::Yellow,
                LogType::Input => Color::Gray,
            };

            spans.push(Span::styled(&entry.text, Style::default().fg(color)));
            Line::from(spans)
        })
        .collect();

    let logs = Paragraph::new(history)
        .block(Block::default().borders(Borders::ALL).title(" Story Log "))
        .wrap(Wrap { trim: true })
        .scroll((state.tui_state.scroll_offset, 0));
    f.render_widget(logs, area);
}

fn draw_sidebar(f: &mut Frame, area: Rect, state: &GameState, room: &Room) {
    let sidebar_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40), // Room Visual
            Constraint::Min(0),         // NPCs & Portraits
        ])
        .split(area);

    // Room Visual
    if let Some(path) = &room.image_path {
        render_sidebar_image(f, sidebar_chunks[0], path, " Location ", false);
    } else {
        f.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .title(" Location (No Image) "),
            sidebar_chunks[0],
        );
    }

    // NPC List Area
    let npc_count = room.npcs.len().max(1) as u16;
    let npc_sub_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage(100 / npc_count);
            npc_count as usize
        ])
        .split(sidebar_chunks[1]);

    for (i, npc_id) in room.npcs.iter().enumerate() {
        if let (Some(npc), true) = (state.npcs.get(npc_id), i < npc_sub_chunks.len()) {
            if let Some(path) = &npc.sheet.image_path {
                render_sidebar_image(
                    f,
                    npc_sub_chunks[i],
                    path,
                    &format!(" {} ", npc.sheet.name),
                    true,
                );
            } else {
                f.render_widget(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!(" {} ", npc.sheet.name)),
                    npc_sub_chunks[i],
                );
            }
        }
    }
}

fn draw_input(f: &mut Frame, area: Rect, state: &GameState) {
    let mut input_text = state.tui_state.input.clone();
    if state.tui_state.is_generating {
        input_text = "...The Game Master is thinking...".to_string();
    }

    let input = Paragraph::new(input_text)
        .style(if state.tui_state.is_generating {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Action (Enter command) "),
        );
    f.render_widget(input, area);

    if !state.tui_state.is_generating {
        f.set_cursor_position((
            area.x + 1 + state.tui_state.cursor_position as u16,
            area.y + 1,
        ));
    }
}

fn render_sidebar_image(f: &mut Frame, area: Rect, path: &str, title: &str, is_portrait: bool) {
    let block = Block::default().borders(Borders::ALL).title(title);
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // Try to load and cache protocol
    let Ok(mut cache) = (*PROTOCOL_CACHE).lock() else {
        return;
    };
    if !cache.contains_key(path)
        && let Ok(mut raw_img) = image::open(path)
        && is_portrait
    {
        // Bust crop: top 40%
        let (w, h) = raw_img.dimensions();
        raw_img = raw_img.crop_imm(0, 0, w, (h as f32 * 0.4) as u32);

        // Create a picker. Since we are in a container, use Halfblocks as safe default.
        let picker = ratatui_image::picker::Picker::halfblocks();

        let protocol = picker.new_protocol(raw_img, inner_area, Resize::Fit(None));
        if let Ok(proto) = protocol {
            cache.insert(path.to_string(), proto);
        }
    }

    if let Some(protocol) = cache.get_mut(path) {
        let image_widget = Image::new(protocol);
        f.render_widget(image_widget, inner_area);
    }
}
