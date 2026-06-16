# Specification: UI Design

> **Related Decisions**: [ADR-001](../adr/adr-001-htmx-web-dashboard.md), [ADR-003](../adr/adr-003-askama-templates.md)

## Design Tokens

### Colors

| Token | Value | Usage |
|-------|-------|-------|
| `--color-bg-primary` | #0a0a0a | Main background |
| `--color-bg-secondary` | #111 | Story log background |
| `--color-bg-tertiary` | #0f0f0f | Visual sidebar background |
| `--color-bg-header` | #1a1a1a | Header and action area background |
| `--color-border` | #333 | All borders |
| `--color-text-primary` | #e0e0e0 | Main text |
| `--color-text-muted` | #888 | Muted text, input text |
| `--color-text-placeholder` | #555 | Placeholder text |
| `--color-accent-green` | #00ff00 | Ready status, focus states |
| `--color-accent-green-bright` | #4ade80 | Location headers |
| `--color-accent-cyan` | #00ffff | Narration text |
| `--color-accent-orange` | #ffb347 | Dialogue text |
| `--color-accent-yellow` | #ffff00 | System text, Thinking status |
| `--color-accent-red` | #ff4444 | Error status, Disconnected status |
| `--color-accent-pink` | #ff6b6b | Speaker names |
| `--color-button-gradient-start` | #2a2a2a | Button gradient top |
| `--color-button-gradient-end` | #1a1a1a | Button gradient bottom |
| `--color-button-border` | #555 | Button border |
| `--color-log-input` | #2a2a2a | User input bubble background |
| `--color-log-narration` | #1a3a3a | Narration bubble background |
| `--color-log-dialogue` | #3a2a1a | Dialogue bubble background |
| `--color-log-system` | #3a3a1a | System message bubble background |
| `--color-error-gradient-start` | #ff4444 | Error notification gradient top |
| `--color-error-gradient-end` | #cc0000 | Error notification gradient bottom |

### Typography

| Token | Value | Usage |
|-------|-------|-------|
| `--font-family` | -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif | All text |
| `--font-size-base` | 14px | Body text, input, buttons |
| `--font-size-small` | 12px | NPC labels, action hints, status |
| `--font-size-xs` | 11px | Connection status |

### Spacing

| Token | Value | Usage |
|-------|-------|-------|
| `--spacing-xs` | 4px | Tight spacing |
| `--spacing-sm` | 8px | Small gaps |
| `--spacing-md` | 16px | Standard padding |
| `--spacing-lg` | 20px | Larger spacing |

### Sizing

| Token | Value | Usage |
|-------|-------|-------|
| `--header-height` | 48px | Header height |
| `--action-area-height` | 64px | Action area height |
| `--input-height` | 40px | Input and button height |
| `--button-min-width` | 100px | Button minimum width |
| `--input-min-width` | 200px | Input minimum width |

### Animation

| Token | Value | Usage |
|-------|-------|-------|
| `--transition-fast` | 0.2s | Hover/focus transitions |

---

## Components

### Header Bar
- Height: 48px
- Background: #1a1a1a
- Border-bottom: 1px solid #333
- Contains: Game title (left), connection status (right)
- Location is displayed in the story log, not the header

### Tab Bar
- Display: flex, below header
- Background: #111
- Border-bottom: 1px solid #333
- Padding: 0 16px
- Tabs: Game | Settings
- Active tab: green text (#00ff00), green bottom border
- Inactive tab: muted text (#888), transparent border
- Hover: muted text brightens to primary

### Tab Content
- Default: hidden (`display: none`)
- Active: flex column (`display: flex`)
- Flex: 1, overflow hidden
- Game tab contains main container + action area
- Settings tab contains connections panel (scrollable)

### Game Title
- Color: #888
- Text: "Chronicler Engine"

### Location Header
- Color: #4ade80
- Weight: bold

### Connection Status
- Font size: 11px
- Border radius: 3px
- States:
  - Connected: #00ff00 text, rgba(0,255,0,0.1) background
  - Disconnected: #ff4444 text, rgba(255,68,68,0.1) background

### Main Container
- Flex: 1
- Overflow: hidden

### Story Log
- Width: 80%
- Background: #111
- Border: 1px solid #333
- Padding: 16px
- Overflow-y: auto
- Auto-scrolls to bottom on new content

### Visual Sidebar
- Width: 20%
- Background: #0f0f0f
- Border: 1px solid #333
- Display: flex, flex-direction: column
- Overflow: hidden

### Location Header Bar
- Width: 100% (full width, above sidebar)
- Separator: 1px border bottom
- Contains: Location image or "No Location Image" placeholder
- No image state: center-aligned "No Location Image" text in #555

### Location Image
- Container: full width, overflow hidden
- Image: width 100%, max-height 200px, object-fit contain
- Auto-scales to fit container without cropping

### NPC Portraits
- Layout: Flex row, nowrap, horizontal scroll (overflow-x: auto)
- Gap: 6px
- Each portrait: fixed 80×80px square
- Image: width 80px, height 80px, object-fit: cover
- Shows present NPCs only
- Scrollable when multiple NPCs exceed sidebar width

### Action Area
- Height: 64px
- Background: #1a1a1a
- Border: 1px solid #333 (top and sides only)
- Padding: 10px 16px
- Display: flex, align-items center, gap 16px

### Command Input
- Background: #0d0d0d
- Border: 1px solid #444
- Border radius: 4px
- Color: #e0e0e0
- Padding: 8px 14px
- Font: inherit, 14px
- Height: 40px
- Focus state: border-color #00ff00, box-shadow 0 0 8px rgba(0,255,0,0.2)
- Placeholder color: #555

### Send Button
- Background: linear-gradient(180deg, #00aa00 0%, #006600 100%)
- Border: 1px solid #00ff00
- Border radius: 4px
- Color: #00ff00 (green text)
- Padding: 8px 16px
- Height: 40px, min-width: 100px
- Font: inherit, 14px, bold
- Box-shadow: 0 0 8px rgba(0, 255, 0, 0.3)
- Hover: background linear-gradient(180deg, #00cc00 0%, #008800 100%), box-shadow 0 0 12px rgba(0, 255, 0, 0.5)
- Active: background linear-gradient(180deg, #006600 0%, #004400 100%)
- Disabled: opacity 0.5, cursor not-allowed, box-shadow none

### Action Hints
- Font size: 12px
- Color: #888

### Game Selector Button
- Styled as a small button next to the game name in the header
- Triggers `GET /fragment/games` to load the game dropdown
- Dropdown is rendered into `#games-dropdown` below the button

### Status Display
- Font size: 12px
- Margin-left: auto
- Min-width: 100px
- Text-align: right
- States:
  - Ready: #00ff00
  - Thinking: #ffff00
  - Error: #ff4444

### Error Notification
- Position: fixed top, full width
- Background: linear-gradient(180deg, #ff4444 0%, #cc0000 100%)
- Color: white
- Padding: 12px 20px
- Box-shadow: 0 2px 8px rgba(0,0,0,0.5)
- Z-index: 1000
- Transform: translateY(-100%) (hidden by default)
- Visible state: transform translateY(0)
- Auto-hide: 5 seconds

### Log Entries

#### Location Header
- Color: #4ade80
- Font-size: 1.1em
- Weight: bold
- Display: inline with timestamp

#### Narration
- Color: #00ffff

#### Dialogue
- Color: #ffb347
- Font-style: italic
- Speaker name: #ff6b6b, bold

#### System
- Color: #ffff00

#### Input
- Color: #888888

### Edit, Retrigger, and Swipe Controls

#### Edit Button (✏️)
- Always visible (opacity: 1)
- Background: transparent, no border
- Color: muted (#888), cyan on hover (#00ffff)
- Font size: 18px
- Padding: 4px 8px
- Margin-left: 8px
- Transition: opacity 0.2s, color 0.2s

#### Retrigger Button (♻)
- Same styling as edit button
- Only appears on the last narration message when `last_trigger` is present in state
- Not shown on event continuations or user input
- Calls `submitRetrigger()` → `POST /retrigger`

#### Swipe Controls
- Container: flex row, gap 8px, centered below message text
- Only appears on the last message when `swipe_count > 1`
- **Left arrow (◀)**: switches to previous swipe. Disabled on first swipe.
- **Counter**: `active_index + 1 / swipe_count` (e.g., "2 / 3")
- **Right arrow (▶)**: if not on latest swipe, switches to next swipe. If on latest swipe, triggers new generation (`submitNewSwipe()` → `POST /swipe/new`).
- Swipe buttons share `.swipe-btn` styling with muted color, cyan on hover

#### Inline Edit Textarea
- Width: 100%, box-sizing: border-box
- Background: #0a0a0a
- Border: 1px solid #333
- Border-radius: 4px
- Color: #e0e0e0
- Padding: 4px
- Font: inherit, 14px
- Resize: none
- Line-height: 1.5 (matches `.log-entry .text`)
- Height: matches original text height + 10px padding/border compensation
- Auto-resizes on input to grow with content

#### Save/Cancel Buttons
- Background: transparent, no border
- Font size: 14px
- Padding: 2px 6px
- Margin-left: 4px
- Save: green on hover (#00ff00)
- Cancel: red on hover (#ff4444)

#### Polling Behavior
- During edit mode, story-log polling is paused via `hx-trigger: none`
- `htmx.process()` called to force HTMX to re-read the trigger attribute
- Polling resumes on save or cancel

### Game Dropdown
- Container: absolute positioned below the Games button, flex column, gap 8px, max-height 300px, overflow-y: auto
- **Game item**: flex row, align-items center, gap 12px, padding 8px 12px
  - Background: #111, border: 1px solid #333, border-radius: 4px
  - **Name**: primary text, flex 1
  - **Meta**: muted text, "Game {id}"
  - **Switch button**: cyan border, cyan text, `hx-post="/games/{id}/switch"`, `hx-swap="none"`, triggers full page refresh
  - **Delete button**: "×" text, red on hover, `hx-post="/games/{id}/delete"`, `hx-target="closest .game-item"`, `hx-swap="outerHTML"`
- **Create new game button**: at the bottom of the dropdown, `POST /games`, triggers full page refresh

### Settings Panel
- Padding: 16px
- Max-width: 800px
- Display: flex column, gap 16px
- Overflow-y: auto (settings tab scrolling)

### Connection Cards
- Background: #111
- Border: 1px solid #333
- Border-radius: 8px
- Padding: 16px
- Margin-bottom: 16px
- Header: flex, space-between, wrap
  - Title: bold, 1.05em
  - Badges: flex row, gap 4px
    - Narrator badge: green background rgba(0,255,0,0.12), green text, green border
    - Quantifier badge: orange background rgba(255,179,71,0.12), orange text, orange border
- Details: small text, muted color, line-height 1.5
- Actions: flex row, gap 8px, wrap

### Connection Edit Form
- Background: #111
- Border: 1px solid #00ffff (cyan accent)
- Border-radius: 8px
- Padding: 16px
- Form groups: flex column, gap 4px
- Labels: small text, muted color
- Inputs/selects: same styling as command input (dark background, #333 border, primary text)
- Focus: cyan border, cyan box-shadow

### Settings Buttons
- Primary (save/add): `.btn-primary` utility class — green gradient (#2a5a2a → #1a4a1a), green border, green text
- Danger (delete): `.btn-danger` utility class — red gradient (#5a2a2a → #4a1a1a), red border, red text
- Default/Edit/View: `.btn-cyan` utility class — cyan gradient (#2a4a5a → #1a3a4a), cyan border, cyan text

### Button Utility Classes
Shared gradient button classes defined in `assets/styles.css` after the design tokens, used across all panels:
- `.btn-primary` — Green gradient, green text/border (save, create, set-narrator, submit actions)
- `.btn-cyan` — Cyan gradient, cyan text/border (edit, view, switch actions)
- `.btn-danger` — Red gradient, red text/border (delete, reset actions)
- Context-scoped selectors (`.settings-panel button`, `.prompt-presets-panel button`, `.games-panel button`) apply layout overrides only — gradients come from utility classes

---

## JavaScript Features

### Status Polling
- Polls `/status/generating` every 5 seconds
- Updates button state based on response ("generating" vs "idle")

### Button State Management
- Ready: Shows "▶ Send", enabled
- Thinking: Shows "■ Stop", disabled, green gradient
- Uses MutationObserver to watch status changes

### Error Notification System
- Shows slide-down banner for LLM errors
- Auto-hides after 5 seconds
- Z-index above all content

## Implementation

### CSS Custom Properties

The design tokens above are implemented as CSS custom properties (CSS variables) defined in a `:root` block.

- **File**: `assets/styles.css`
- **Approach**: All tokens are defined in the `:root` pseudo-class for global scope
- **Usage**: Reference via `var(--token-name)` throughout stylesheets

### Responsive Breakpoints

Media queries handle responsive behavior:

| Breakpoint | Width | Adjustments |
|-----------|-------|-------------|
| Tablet | ≤ 768px | Stack sidebar below story log, sidebar 100% width |
| Mobile | ≤ 480px | Wrap header elements, stack action area vertically |

- **Mobile-first**: Base styles target smallest screens, `@media (max-width: ...)` adds larger layouts
- **Flexibility**: CSS variables enable theme changes without modifying component styles

### Reference Implementation

See `assets/styles.css` for the `:root` token definitions, button utility classes (`.btn-primary`, `.btn-cyan`, `.btn-danger`), and component styles. See `assets/worlds.css` for worlds-specific layout overrides.