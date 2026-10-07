---
diataxis: reference
title: UI Design
---

## Overview

The dashboard's visual language is defined by a small set of CSS custom properties (design tokens) and a structured set of component specifications. Tokens are the source of truth for colors, typography, spacing, sizing, and animation timings; components declare the token-derived styling for each dashboard region. The static stylesheet at `assets/styles.css` is the binding code that consumes both.

This doc carries the token tables verbatim because the tables are the curated token→usage reference — the `--color-accent-green` value itself lives in `assets/styles.css`. Component specs describe structure and visual state in prose; enforced interaction contracts live in the per-region browser specs (`docs/specs/browser_*.md`), and the CSS implementation lives in `assets/styles.css`.

## Design Tokens

### Colors

| Token | Value | Usage |
|-------|-------|-------|
| `--color-bg-primary` | `#0a0a0a` | Main background |
| `--color-bg-secondary` | `#111` | Story log background |
| `--color-bg-tertiary` | `#0f0f0f` | Visual sidebar background |
| `--color-bg-header` | `#1a1a1a` | Header and action area background |
| `--color-border` | `#333` | All borders |
| `--color-text-primary` | `#e0e0e0` | Main text |
| `--color-text-muted` | `#888` | Muted text, inactive tab, swap/swipe controls, NPC portrait labels |
| `--color-text-placeholder` | `#555` | Placeholder text |
| `--color-accent-green` | `#00ff00` | Ready status, focus states |
| `--color-accent-green-bright` | `#4ade80` | Location headers |
| `--color-accent-cyan` | `#00ffff` | Narration text, edit/save/cancel hover |
| `--color-accent-blue-cyan` | `#38bdf8` | Event headers, style issue tags |
| `--color-accent-orange` | `#ffb347` | Dialogue text, retry hover, quantifier badge |
| `--color-accent-yellow` | `#ffff00` | System text, Thinking status, capitalization tags |
| `--color-accent-red` | `#ff4444` | Error status, danger buttons |
| `--color-accent-pink` | `#ff6b6b` | Speaker names (default), delete hover, grammar tags |
| `--color-button-gradient-start` | `#2a2a2a` | Generic button gradient top (unused at runtime) |
| `--color-button-gradient-end` | `#1a1a1a` | Generic button gradient bottom (unused at runtime) |
| `--color-button-border` | `#555` | Command input border, custom checkbox border |
| `--color-log-input` | `#2a2a2a` | User input bubble background |
| `--color-log-narration` | `#1a3a3a` | Narration bubble background |
| `--color-log-dialogue` | `#3a2a1a` | Dialogue bubble background |
| `--color-log-system` | `#3a3a1a` | System message bubble background |
| `--color-error-gradient-start` | `#ff4444` | Error notification gradient top |
| `--color-error-gradient-end` | `#cc0000` | Error notification gradient bottom |

### Typography

| Token | Value | Usage |
|-------|-------|-------|
| `--font-family` | `-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif` | All text |
| `--font-size-base` | `14px` | Body text, input, buttons, action buttons |
| `--font-size-small` | `12px` | NPC labels, status, connection details |
| `--font-size-xs` | `11px` | Badges |
| `--font-size-sender` | `13px` | Speaker name above each log entry |

### Spacing

| Token | Value | Usage |
|-------|-------|-------|
| `--spacing-xs` | `4px` | Tight spacing |
| `--spacing-sm` | `8px` | Small gaps |
| `--spacing-md` | `16px` | Standard padding |
| `--spacing-lg` | `20px` | Larger spacing |

### Sizing

| Token | Value | Usage |
|-------|-------|-------|
| `--header-height` | `48px` | Header height |
| `--action-area-height` | `64px` | Action area height |
| `--input-height` | `40px` | Input and button height |
| `--button-min-width` | `100px` | Button minimum width |
| `--input-min-width` | `200px` | Input minimum width |

### Animation

| Token | Value | Usage |
|-------|-------|-------|
| `--transition-fast` | `0.2s` | Hover/focus transitions |

## Components

### Header Bar

- Height: `--header-height`
- Background: `--color-bg-header`
- Border-bottom: 1px solid `--color-border`
- Contains: the game title and the current game name
- Location is **not** in the header — it appears in the story log as the active-room location header

### Tab Bar

- Display: flex, positioned below the header
- Background: `--color-bg-secondary`
- Border-bottom: 1px solid `--color-border`
- Padding: `0 var(--spacing-md)` (16px horizontal)
- Gap: `var(--spacing-sm)` (8px between tabs)
- Active tab: green text (`--color-accent-green`), green bottom border (`2px solid`)
- Inactive tab: muted text (`--color-text-muted`), transparent border
- Hover: muted text brightens to primary (`--color-text-primary`)

### Tab Content

- Default: `display: none`
- Active: `display: flex` (replaces `none`)
- Always-on: `flex-direction: column; flex: 1; overflow: hidden`

### Game Title

- Color: `--color-text-muted`
- Margin-right: `var(--spacing-md)`

### Location Header (in story log)

- Color: `--color-accent-green-bright`
- Weight: bold
- Display: inline with timestamp

### Event Header (in story log)

- Color: `--color-accent-blue-cyan` (NOT `--color-accent-cyan`)
- Weight: bold
- Display: inline with timestamp

### Panel Save Model

Save timing is a per-region behavioural contract owned by the specs; this doc
carries only the visible state each region shows:

- **Instant controls** carry inline feedback and no Save button; the Text Check
  card's `Saved` label is the exemplar.
- **World posture** selects sit in a labelled `<fieldset>` ("Posture — saves
  automatically") with its own `#world-posture-status` status target.
- **On Save** regions (text fields and JSON blobs) use the standard Save /
  Cancel button pair.

The per-region contracts live in [`../../../specs/settings.md`](../../../specs/settings.md)
(Text Check auto-save; connection Add/Edit), [`../../../specs/games.md`](../../../specs/games.md)
(per-game posture, mode, and preset auto-save), and
[`../../../specs/worlds.md`](../../../specs/worlds.md) (world posture auto-save;
world details Save). [`../../../specs/browser_worlds.md`](../../../specs/browser_worlds.md)
checks the posture change wiring.

### Main Container (Game Tab)

- `flex: 1`
- `overflow: hidden`
- Hosts the story log (80% width) and visual sidebar (20% width) as horizontal siblings

### Story Log

- Width: 80%
- Background: `--color-bg-secondary`
- Border: 1px solid `--color-border`
- Padding: `var(--spacing-md)` (16px)
- `overflow-y: auto`
- Auto-scrolls to bottom on new content

### Visual Sidebar

- Width: 20%
- Background: `--color-bg-tertiary`
- Border: 1px solid `--color-border`
- Display: flex, flex-direction: column
- `overflow: hidden`
- Hosts the location-header bar (top) and NPC portraits row (bottom)

### Location Image Container

- Full width within the sidebar, `overflow: hidden`
- Image: `width: 100%; max-height: 200px; object-fit: contain`
- No-image state: "No Location Image" placeholder centered, color `--color-text-placeholder`

### NPC Portraits

- Flex row, `nowrap`, horizontal scroll (`overflow-x: auto`)
- Gap: `6px`
- Each portrait: fixed 80×80 square
- Image: `width: 80px; height: 80px; object-fit: cover`
- Shows NPCs currently in the room only

### Action Area

- Height: `--action-area-height`
- Background: `--color-bg-header`
- Border: 1px solid `--color-border` (top and sides only — no bottom border so it sits flush)
- Padding: `10px var(--spacing-md)`
- Display: flex, `align-items: center`, `gap: var(--spacing-md)`

### Command Input

- Background: `--color-bg-primary`
- Border: 1px solid `--color-button-border`
- Border-radius: `4px`
- Color: `--color-text-primary`
- Padding: `8px 14px`
- Font: inherit, `--font-size-base`
- Height: `--input-height`
- Min-width: `var(--input-min-width)`
- Flex: 1 (consumes remaining width in `#command-form`)
- Focus: border-color `--color-accent-green`, box-shadow `0 0 8px rgba(0, 255, 0, 0.2)`
- Placeholder color: `--color-text-placeholder`

### Slash-Command Auto-Suggestion Menu

A position-fixed palette that appears above the command input while the input value starts with `/`, listing the steering slash commands.

- Container: `.slash-menu`
  - `position: fixed`, `z-index: 1000`
  - Min-width: `280px`
  - Background: `--color-bg-secondary`
  - Border: `1px solid var(--color-border)`
  - Border-radius: `6px`
  - Box-shadow: `0 -4px 16px rgba(0, 0, 0, 0.5)`
  - Font: inherit, `--font-size-base`
- Item: `.slash-suggestion`
  - Display: flex, `justify-content: space-between`, `align-items: center`
  - Padding: `var(--spacing-sm) var(--spacing-md)`
  - Cursor: pointer
  - Gap: `var(--spacing-md)`
  - Active state (`.active`): background `--color-bg-header`
- Command text (`.slash-cmd`): `--color-accent-cyan`, `font-weight: 600`, `white-space: nowrap`
- Description text (`.slash-desc`): `--color-text-muted`, `--font-size-small`, `text-align: right`, `white-space: nowrap`

### Send Button

Gradients are hardcoded in `#command-form button` (not tokenized), since the send button has its own visual identity distinct from the `.btn-primary` utility class.

- Background: linear-gradient(180deg, `#00aa00` 0%, `#006600` 100%) (idle)
- Border: 1px solid `--color-accent-green`
- Border-radius: `4px`
- Color: `--color-accent-green`
- Padding: `8px var(--spacing-md)`
- Height: `--input-height`, min-width: `--button-min-width`
- Font: inherit, `--font-size-base`, bold
- Box-shadow: `0 0 8px rgba(0, 255, 0, 0.3)`
- Hover: linear-gradient(180deg, `#00cc00` 0%, `#008800` 100%), box-shadow `0 0 12px rgba(0, 255, 0, 0.5)`
- Active: linear-gradient(180deg, `#006600` 0%, `#004400` 100%), box-shadow `0 0 4px rgba(0, 255, 0, 0.3)`
- Disabled: `opacity: 0.5; cursor: not-allowed; box-shadow: none`

### Status Display

- Font size: `--font-size-small`
- `margin-left: auto`
- Min-width: `--button-min-width`
- Text-align: right
- States:
  - **Ready**: `--color-accent-green`
  - **Thinking**: `--color-accent-yellow`
  - **Error**: `--color-accent-red`

### Error Notification

- Position: fixed top, full width
- Background: linear-gradient(180deg, `--color-accent-red` 0%, `--color-error-gradient-end` 100%)
- Color: white
- Padding: `12px 20px`
- Box-shadow: `0 2px 8px rgba(0, 0, 0, 0.5)`
- `z-index: 1000`
- Hidden by default: `transform: translateY(-100%)`
- Visible state: `transform: translateY(0)`
- Auto-hide: 5 seconds

### Log Entry Bubbles

Per-`log_type` bubble styling, keyed by the `MessageType` enum (`Narration`, `Dialogue`, `System`, `Input`). Each bubble is a `max-width: 85%` rounded rect with `padding: 10px 14px`, `border-radius: 12px`, and a `4px` corner radius on the side opposite the alignment to suggest a chat-bubble tail.

| Bubble | Background | Text color | Sender color | Alignment |
|---|---|---|---|---|
| Input | `--color-log-input` | `#cccccc` (hardcoded) | `--color-text-muted` | right (`margin-left: auto`) |
| Narration | `--color-log-narration` | `--color-accent-cyan` | `#00cccc` (hardcoded) | left (`margin-right: auto`) |
| Dialogue | `--color-log-dialogue` | `--color-accent-orange` (italic) | `--color-accent-orange` | left (`margin-right: auto`) |
| System | `--color-log-system` | `--color-accent-yellow` | (no sender) | centered, max-width 70% |

The base `.sender` style is `display: block; font-size: var(--font-size-sender); font-weight: bold; color: var(--color-accent-pink); margin-bottom: var(--spacing-xs)`. Dialogue and narration override it to their own bubble colors; input overrides to muted.

The base `.text` style is `font-size: var(--font-size-base); line-height: 1.5; overflow-wrap: anywhere; word-wrap: break-word`. Narration, dialogue, system override the text color to their accent; input overrides to `#cccccc`. Quoted text (`<q>`) inside `.text` is `--color-accent-red` italic.

### Per-Entry Action Buttons

Three buttons rendered above each entry's text span. Conditional visibility rules live in `NarrativeLogTemplate::new` (templates.rs).

| Button | Glyph | Visibility rule |
|---|---|---|
| Edit | ✎ | always visible on every entry |
| Delete | 🗑 | last entry, only when more than one entry exists |
| Retrigger | ♻ | last entry, narration or dialogue, no event continuation, previous turn had a trigger |

Base `.action-btn` style: `background: rgba(255, 255, 255, 0.08); border: 1px solid rgba(255, 255, 255, 0.15); border-radius: 4px; color: var(--color-text-muted); cursor: pointer; font-size: 14px; padding: 2px 6px; min-width: 24px; height: 24px; display: inline-flex; align-items: center; justify-content: center; transition: background, border-color, color all on var(--transition-fast)`.

Default hover deepens the background to `rgba(255, 255, 255, 0.15)` and the border to `rgba(255, 255, 255, 0.25)`. Per-button hover colors override:

| Button | Hover color/border |
|---|---|
| Edit | `--color-accent-cyan` |
| Delete | `--color-accent-pink` |
| Retry | `--color-accent-orange` |

The retrigger button uses a separate `.retrigger-btn` class (see Swipe Controls below), not `.action-btn`.

### Swipe Controls

Rendered below the last entry's text when `swipe_count > 1`. Container: flex row, gap `8px`, `margin-top: 6px`, `padding-top: 6px`, border-top `1px solid var(--color-border)`, centered.

- **Left arrow (◀)**: `.swipe-btn`, switches to previous swipe; disabled on the first swipe (opacity 0.3)
- **Counter**: `.swipe-counter` — `font-size: 12px; color: var(--color-text-muted); font-variant-numeric: tabular-nums; min-width: 40px; text-align: center`
- **Right arrow (▶)**: `.swipe-btn`; when not on the latest swipe, switches to next; when on the latest swipe, generates a new swipe

`.swipe-btn` base: `background: transparent; border: 1px solid var(--color-border); color: var(--color-text-muted); padding: 2px 8px; border-radius: 4px; cursor: pointer; font-size: 12px; line-height: 1; transition: all 0.15s ease`.

Hover (when not disabled): `background: var(--color-bg-tertiary); color: var(--color-text-primary); border-color: var(--color-accent-cyan)`.

Disabled: `opacity: 0.3; cursor: not-allowed`.

### Retrigger Button

Uses its own `.retrigger-btn` class (not `.action-btn`), rendered next to the swipe controls when the retrigger visibility rule applies.

- Base: `background: transparent; border: 1px solid var(--color-accent-cyan); color: var(--color-accent-cyan); padding: 2px 8px; border-radius: 4px; cursor: pointer; font-size: 12px; line-height: 1; transition: all 0.15s ease`
- Hover: inverts — `background: var(--color-accent-cyan); color: var(--color-bg-primary)`

### Inline Edit Textarea

Replaces the entry's text span when the user clicks Edit.

- Width: 100%, `box-sizing: border-box`
- Background: `--color-bg-primary`
- Border: 1px solid `--color-border`
- Color: `--color-text-primary`
- Border-radius: `4px`
- Padding: `var(--spacing-xs)`
- Font: inherit, `--font-size-base`
- `resize: none`
- `display: block`
- Line-height: 1.5 (matches `.log-entry .text`)
- `margin: 0`
- Auto-grows to fit its content, capped at `max-height: 50vh` with `overflow-y: auto`
- Takes focus when it appears; Escape cancels, Ctrl/Cmd+Enter saves

### Save / Cancel Buttons (Edit Mode)

Replace the entry's action-button cluster while in edit mode. Both share the same base; only hover differs.

- Base: `background: rgba(255, 255, 255, 0.08); border: 1px solid rgba(255, 255, 255, 0.15); border-radius: 4px; cursor: pointer; font-size: 14px; padding: 2px 6px; min-width: 24px; height: 24px; display: inline-flex; align-items: center; justify-content: center; color: var(--color-text-muted)`
- Save hover: `background: rgba(0, 255, 0, 0.15); border-color: var(--color-accent-green); color: var(--color-accent-green)`
- Cancel hover: `background: rgba(255, 68, 68, 0.15); border-color: var(--color-accent-red); color: var(--color-accent-red)`

### Text Check Preview

Replaces the action area when text-check preflight surfaces issues.

- Background: `--color-bg-header`, border `1px solid var(--color-border)`, border-radius `8px`, padding `var(--spacing-md)`
- Max-width: `600px`
- Display: flex column, gap `var(--spacing-sm)`
- Original text (read-only): label uppercase muted, value strikethrough muted
- Corrected text (editable textarea): label uppercase muted, value green (`--color-accent-green`), word-break
- Issue tags: orange (spell), pink (grammar), yellow (capitalization), blue-cyan (style), muted (formatting/other)
- Three buttons: **Send Corrected**, **Send Original**, **Cancel**
- The check button itself (`.btn-check`): transparent background, cyan border+text, padding `8px 14px`, height `var(--input-height)`, bold; hover adds a cyan glow

When the action area contains a `.text-check-preview`, the parent `.action-area` expands: `height: auto; min-height: var(--action-area-height); align-items: flex-start; padding-top/bottom: var(--spacing-md)`.

### Panel Layout and Viewports

Every dashboard panel renders in one centered content column. The four
management panels do not scroll internally; the tab body supplies the scroll.

- Content column: `width: 100%`, `max-width: 960px`, `margin: 0 auto`,
  `padding: 24px`. Applies to `.settings-panel`, `.prompt-presets-panel`,
  `.worlds-panel`, and `.games-panel`.
- Scroll region: the active `.tab-content` (`#settings-tab`, `#worlds-tab`,
  `#prompt-presets-tab`, `#games-tab` use `overflow-y: auto`). The panels have
  no `overflow-y` of their own, so the scrollbar sits at the viewport edge.
- Tab bar: `.tab-bar` scrolls horizontally (`overflow-x: auto`) and each
  `.tab` does not shrink, so the tabs keep their width and the bar scrolls
  when they do not fit.
- Worlds: the edit form uses the same card frame as the world list, so the
  text inset does not change between the two views.

Supported viewports, desktop-first:

| Range | Support |
|---|---|
| ≥ 1024×700 | Fully supported |
| 768–1024 | Best-effort |
| Phone (< 768px) | Out of scope (the existing `@media (max-width: 768px)` rules stay) |

### Settings Panel

- Padding: `24px`
- Max-width: `960px`, centered (`margin: 0 auto`)
- Display: flex column, gap `var(--spacing-md)`
- Scrolls with the tab body (`#settings-tab`)

### Settings Sub-tabs

- Bar: flex row, gap `var(--spacing-sm)`, bottom border `var(--color-border)`
- Sub-tab: transparent, muted text, `2px` transparent bottom border; `.active` uses `--color-accent-green` text and bottom border
- Degraded marker (`.subtab-degraded-dot`): 8px orange circle beside the Connections label while a role is degraded
- Panel (`.settings-subtab-panel`): `display: none`; `.active` is `display: flex`, flex column, gap `var(--spacing-md)`

### Role Rows

- Row: flex, wrap, gap `var(--spacing-sm)`, `--color-bg-secondary` background, `1px solid var(--color-border)` border, `border-radius: 8px`, padding `var(--spacing-sm) var(--spacing-md)`
- Role name: bold, min-width `90px`
- Connection select: flex `1 1 220px`, min-width `160px`
- Health: `.role-health.healthy` green, `.role-health.degraded` red, `.role-health.unknown` muted; a degraded role carries the shared error disclosure

### Connection Rows

- List: flex column, gap `var(--spacing-sm)`
- Row (`.connection-row`): flex, wrap, gap `var(--spacing-md)`, `--color-bg-secondary` background, `1px solid var(--color-border)` border, `border-radius: 8px`, padding `var(--spacing-sm) var(--spacing-md)`
- Meta: flex column, name bold and provider/model small muted
- Role badges: flex row, gap `4px`
  - **Narrator badge**: green background `rgba(0, 255, 0, 0.12)`, green text, green border
  - **Quantifier badge**: orange background `rgba(255, 179, 71, 0.12)`, orange text, orange border
- Actions: flex row, gap `var(--spacing-sm)`, pushed right with `margin-left: auto`

### Connection Form Page

- The shared Add/Edit page is a `.settings-panel` with `.connection-form-page`
- Back link: transparent, cyan text, no border, aligned to the start
- Form fields inherit the `.settings-panel` input/select styling; actions are a flex row with gap `var(--spacing-sm)`

### Text Check Card

- The Text Check sub-tab keeps the `.connection-card` frame: `--color-bg-secondary` background, `1px solid var(--color-border)` border, `border-radius: 8px`, padding `var(--spacing-md)`
- Header: flex, space-between, wrap; title bold `1.05em`

### Button Utility Classes

Three utility classes provide the gradient+border+text styling for action buttons across the dashboard panels. Gradient hex values are hardcoded in the class definitions (NOT tokenized). Context-scoped selectors (`.settings-panel button`, `.prompt-presets-panel button`, `.games-panel button`) apply layout overrides only — gradients come from the utility classes.

| Class | Gradient | Text/border | Padding | Typical actions |
|---|---|---|---|---|
| `.btn-primary` | `#2a5a2a` → `#1a4a1a` (idle) / `#3a6a3a` → `#2a5a2a` (hover) | `--color-accent-green` | `8px 20px`, bold | Save, create, add-connection, submit |
| `.btn-cyan` | `#2a4a5a` → `#1a3a4a` (idle) / `#3a5a6a` → `#2a4a5a` (hover) | `--color-accent-cyan` | `4px 12px`, xs font | Edit, view, switch |
| `.btn-danger` | `#5a2a2a` → `#4a1a1a` (idle) / `#6a3a3a` → `#5a2a2a` (hover) | `--color-accent-red` | `4px 12px`, xs font | Delete, reset |

Hover state for each class also adds a colored glow box-shadow in the matching accent (rgba 0.25 alpha).

### LLM Messages Panel

- Panel: `flex: 1; overflow-y: auto; padding: var(--spacing-md); min-height: 0`
- List: flex column, gap `var(--spacing-sm)`
- Card: `--color-bg-secondary` background, `1px solid var(--color-border)` border, `border-radius: 6px`, `overflow: hidden`
- Header: flex row, `--color-bg-tertiary` background, gap `var(--spacing-sm)`, padding `var(--spacing-sm) var(--spacing-md)`, hover darkens to `#1f1f1f`
  - Agent: bold, `--color-accent-cyan`, `--font-size-small`, uppercase, min-width `80px`
  - Model: muted, `--font-size-xs`, flex 1
  - Timestamp: muted, `--font-size-xs`, monospace
  - Error badge (when present): red text, red border, `0 2px 6px`, `--font-size-xs`, bold
- Body: `display: none` by default; `.expanded` adds `display: block` and a top border
- Three prompt blocks per card: system prompt preview, user prompt preview (open by default), response preview; each rendered as `<details>` with the agent-color heading
- Two raw JSON blocks below the prompts: raw request JSON, raw response JSON; each in a `<details>` block
- Empty state: "No LLM messages yet" when no calls have been logged (muted, italic, centered, padded)
- Polling pauses while any card is expanded; on collapse-all, polling resumes

## Document References

- [`./dashboard.md`](./dashboard.md) — page layout, tabs, polling cadences, and the per-flow interactions whose visual appearance this doc specifies.
- [`../game_flow.md#text-check-branch`](../game_flow.md#text-check-branch) — text-check settings and the preview UI's data model.
- [`../narrative/narration_system.md#llm-call-logging--forensics`](../narrative/narration_system.md#llm-call-logging--forensics) — LLM Messages tab content and the 50-row `llm_messages` cap.
