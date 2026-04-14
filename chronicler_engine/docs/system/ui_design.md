# Specification: UI Design

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
| `--color-accent-green` | #00ff00 | Location, Ready status, focus states |
| `--color-accent-cyan` | #00ffff | Narration text |
| `--color-accent-orange` | #ffb347 | Dialogue text |
| `--color-accent-yellow` | #ffff00 | System text, Thinking status |
| `--color-accent-red` | #ff4444 | Error status, Disconnected status |
| `--color-accent-pink` | #ff6b6b | Speaker names |
| `--color-button-gradient-start` | #2a2a2a | Button gradient top |
| `--color-button-gradient-end` | #1a1a1a | Button gradient bottom |
| `--color-button-border` | #555 | Button border |

### Typography

| Token | Value | Usage |
|-------|-------|-------|
| `--font-family` | 'Courier New', monospace | All text |
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
| `--button-min-width` | 90px | Button minimum width |
| `--input-min-width` | 200px | Input minimum width |

### Animation

| Token | Value | Usage |
|-------|-------|-------|
| `--transition-fast` | 0.2s | Hover/focus transitions |

---

## Components

### Header
- Height: 48px
- Background: #1a1a1a
- Border: 1px solid #333
- Contains: Game title, location, connection status

### Game Title
- Color: #888
- Text: "Chronicler Engine"

### Location
- Color: #00ff00
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

### Image Container
- Max height: 200px (180px for image)
- Object-fit: contain
- Contains: Location image with label
- Placeholder state when no image: center-aligned "No image" text in #555

### NPC Portraits
- Layout: flex-wrap, gap 8px
- Each portrait: 45% width, min 100px, max 150px

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
- Color: #e0e0e0
- Padding: 8px 20px
- Height: 40px, min-width: 90px
- Font: inherit, 14px, bold
- Box-shadow: 0 0 8px rgba(0, 255, 0, 0.3)
- Hover: background linear-gradient(180deg, #00cc00 0%, #008800 100%), border-color #00ff00, box-shadow 0 0 12px rgba(0, 255, 0, 0.4)
- Active: gradient #008800-#006600
- Disabled: opacity 0.5, cursor not-allowed, box-shadow none

### Action Hints
- Font size: 12px
- Color: #888

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