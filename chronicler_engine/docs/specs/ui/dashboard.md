# Specification: TUI Dashboard & Components

## Overview
The Chronicler Engine utilizes a sophisticated Terminal User Interface (TUI) powered by **Ratatui**. The interface is designed as a domain-driven dashboard that provides separate contexts for narrative immersion, visual grounding, and user intent.

## The Dashboard Layout
The UI is divided into three primary vertical chunks:

### 1. Header (Length: 3)
Displays system-level context using the `Header` component.
- **Style**: Green Bold text for the current location name.
- **Element**: Standard block with all-borders.

### 2. Main Body (Min: 0)
Splits horizontally into the story context and the visual context:
- **Story Log (70%)**: A scrollable history using the `draw_story_log` component.
    - **Styles**: 
        - Narration: Cyan.
        - Dialogue: White.
        - System: Yellow.
        - Inputs: Gray.
- **Visual Sidebar (30%)**: A vertical stack using the `draw_sidebar` component.
    - **Location (40%)**: Landscape render of the current room.
    - **NPC Portraits (Min: 0)**: Vertical grid of present NPCs.

### 3. Action Area (Length: 5)
Interactive zone for user roleplay via the `draw_input` component.
- **Responsiveness**: Displays a "Thinking..." status in DarkGray while the background LLM worker is active. 
- **Cursor**: The cursor is only shown and active when the system is ready for input.

## Visual Protocols
- **Rendering**: Prioritize the `half-block` protocol (via `ratatui-image`) for universal compatibility in containerized shell environments.
- **Bust Cropping**: Character portraits are automatically cropped to the top 40% (Head/Shoulders) to maintain pixel density in the vertical sidebar.
- **Caching**: Images are processed once and cached in a thread-safe `PROTOCOL_CACHE` (LazyStatic Mutex) to ensure smooth re-renders during text input.
