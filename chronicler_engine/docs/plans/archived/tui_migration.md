# Specification: TUI Dashboard Migration (SUPERSEDED)

> This spec has been superseded by `system/dashboard.md` (HTMX web dashboard).

## Objective
Transform the Chronicler Engine from a line-buffered REPL into a structured Terminal User Interface (TUI) Dashboard. This will provide a fixed layout for visuals, narration history, and character status, significantly improving immersion and portability.

## UI Layout
The dashboard will use a three-tier vertical layout:
1. **Header**: Game title and current location name.
2. **Main Body**: 
   - **Narration Log (Left/70%)**: A scrollable history of GM narration, dialogue, and system messages.
   - **Visual Sidebar (Right/30%)**: 
     - **Location Visual**: A landscape crop of the room image.
     - **NPC List**: A vertical stack of cropped character portraits for everyone in the room.
3. **Footer**:
   - **Status Bar**: Player name and current health/status (if applicable).
   - **Action Bar**: Dynamic action hints (e.g., `[Look] [Go North]`).
   - **Input Box**: The active text input field.

## State Management
To support TUI re-rendering, `GameState` must track:
- **Narration History**: A buffer of past messages (system messages, GM narration, and player input).
- **UI State**: The current cursor position in the input box and the scroll offset for the log.

## Visual Rendering
- **Portraits**: Characters will now be rendered as "Portrait Crops" (top 40-50% of source images) to ensure sharpness in the terminal grid.
- **Protocols**: The engine will use `ratatui-image` to automatically detect and utilize the best available graphics protocol (Sixel, Kitty, or Half-block fallback).

## Interaction Loop
- Input is collected character-by-character (via `crossterm`) instead of `stdin.read_line`.
- Pressing `Enter` triggers the action parser and (if narration is needed) displays a "The Game Master is thinking..." state in the log until the LLM responds.
