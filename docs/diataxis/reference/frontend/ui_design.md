---
diataxis: reference
title: UI Design
---

## Overview

The dashboard's visual language is defined by a small set of CSS custom properties (design tokens) and a structured set of component specifications. Tokens are the source of truth for colors, typography, spacing, sizing, and animation timings; components declare the token-derived styling for each dashboard region. The static stylesheet at `assets/styles.css` is the binding code that consumes both.

The token tables list each token's value and usage. The values live in `assets/styles.css`. Component specs describe structure and visual state. The interaction contracts live in the browser specs (`docs/specs/browser_*.md`).

## Design Tokens

### Colors

| Token | Value | Usage |
|-------|-------|-------|
| `--color-bg-primary` | `#121212` | Main background |
| `--color-bg-secondary` | `#191919` | Story log background |
| `--color-bg-tertiary` | `#161616` | Visual sidebar background |
| `--color-bg-header` | `#202020` | Header, action area, LLM card header hover |
| `--color-border` | `#343434` | All borders |
| `--color-text-primary` | `#e6e6e6` | Main text, narration and input bubble text |
| `--color-text-muted` | `#a3a3a3` | Muted text, inactive tab, swipe controls, NPC portrait labels |
| `--color-text-placeholder` | `#858585` | Placeholder text |
| `--color-accent-ok` | `#8ab4f8` | Ready status, active tab, Healthy role, primary/Send buttons, focus, Narrator badge |
| `--color-accent-green-bright` | `#a6d6a0` | Location headers, LLM prompt emphasis |
| `--color-accent-cyan` | `#9cc9d6` | Option rows, slash commands, back link, retrigger, LLM agent label |
| `--color-accent-blue-cyan` | `#8ab4e0` | Event headers, style issue tags |
| `--color-accent-orange` | `#e8b878` | Quoted dialogue, degraded marker, quantifier badge, spell issue tags |
| `--color-accent-yellow` | `#dcc888` | System text, Thinking status, capitalization tags |
| `--color-accent-red` | `#ea8080` | Error status, danger buttons, cancel hover |
| `--color-accent-pink` | `#e8a0a0` | Delete hover, grammar issue tags |
| `--color-button-border` | `#4a4a4a` | Command input border, custom checkbox border |
| `--color-button-primary-start` | `#283a54` | `.btn-primary` gradient top |
| `--color-button-primary-end` | `#202f45` | `.btn-primary` gradient bottom |
| `--color-button-cyan-start` | `#283338` | `.btn-cyan` gradient top |
| `--color-button-cyan-end` | `#20292d` | `.btn-cyan` gradient bottom |
| `--color-button-danger-start` | `#402626` | `.btn-danger` gradient top |
| `--color-button-danger-end` | `#321e1e` | `.btn-danger` gradient bottom |
| `--color-button-send-start` | `#2c4466` | Send button gradient top |
| `--color-button-send-end` | `#243650` | Send button gradient bottom |
| `--color-log-input` | `#272727` | User input bubble background |
| `--color-log-narration` | `#1f1f1f` | Narration bubble background |
| `--color-log-system` | `#27251c` | System message bubble background |
| `--color-error-gradient-end` | `#963c3c` | Unreachable failure banner background |
| `--color-tint-ok` | `color-mix(in srgb, var(--color-accent-ok) 12%, transparent)` | OK badges, save hover |
| `--color-tint-ok-edge` | `color-mix(in srgb, var(--color-accent-ok) 28%, transparent)` | OK badge border |
| `--color-tint-orange` | `color-mix(in srgb, var(--color-accent-orange) 12%, transparent)` | Quantifier badge, spell issue tags |
| `--color-tint-orange-edge` | `color-mix(in srgb, var(--color-accent-orange) 28%, transparent)` | Orange tag border |
| `--color-tint-cyan` | `color-mix(in srgb, var(--color-accent-cyan) 7%, transparent)` | Option rows |
| `--color-tint-cyan-strong` | `color-mix(in srgb, var(--color-accent-cyan) 15%, transparent)` | Option row hover |
| `--color-tint-red` | `color-mix(in srgb, var(--color-accent-red) 12%, transparent)` | Error message, cancel hover |
| `--shadow-focus-ring` | `0 0 0 2px color-mix(in srgb, var(--color-accent-ok) 30%, transparent)` | Every focus ring |

Every text token clears 4.5:1 (WCAG AA) against the backgrounds it is used on.

### Typography

| Token | Value | Usage |
|-------|-------|-------|
| `--font-family` | `-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif` | All text |
| `--font-size-base` | `14px` | Body text, input, buttons, action buttons |
| `--font-size-small` | `12px` | NPC labels, status, connection details |
| `--font-size-xs` | `11px` | Badges |

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
- Contains: the game title and the current display name

### Tab Bar

- Display: flex, positioned below the header
- Background: `--color-bg-secondary`
- Border-bottom: 1px solid `--color-border`
- Padding: `0 var(--spacing-md)` (16px horizontal)
- Gap: `var(--spacing-sm)` (8px between tabs)
- Active tab: `--color-accent-ok` text and bottom border (`2px solid`)
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

- Color: `--color-accent-blue-cyan`
- Weight: bold
- Display: inline with timestamp

### Panel Save Model

The visible state each region shows:

- **Instant controls** carry inline feedback and no Save button; the Text Check
  card's `Saved` label is the exemplar.
- **World posture** selects sit in a labelled `<fieldset>` ("Posture — saves
  automatically") with its own `#world-posture-status` status target.
- **On Save** regions (text fields and JSON blobs) use the standard Save /
  Cancel button pair.

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

### Visual Sidebar

- Width: 20%
- Background: `--color-bg-tertiary`
- Border: 1px solid `--color-border`
- Display: flex, flex-direction: column
- `overflow: hidden`
- Hosts the location image (top) and the NPC portraits row (bottom)

### Location Image Container

- Full width within the sidebar, `overflow: hidden`
- Image: `width: 100%; max-height: 200px; object-fit: contain`
- No-image state: "No Location Image" placeholder centered, color `--color-text-placeholder`

### NPC Portraits

- Flex row, `nowrap`, horizontal scroll (`overflow-x: auto`)
- Gap: `6px`
- Each portrait: fixed 80×80 square
- Image: `width: 80px; height: 80px; object-fit: cover`

### Action Area

- Height: `--action-area-height`
- Background: `--color-bg-header`
- Border: 1px solid `--color-border` (top and sides only — no bottom border so it sits flush)
- Padding: `10px var(--spacing-md)`
- Display: flex, `align-items: center`, `gap: var(--spacing-md)`

The action area expands past its fixed height while the preview is open — see Text Check Preview.

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
- Focus: border-color `--color-accent-ok`, box-shadow `var(--shadow-focus-ring)`
- Placeholder color: `--color-text-placeholder`

### Slash-Command Auto-Suggestion Menu

A position-fixed palette above the command input that lists the steering slash commands.

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

`#command-form button` carries its own gradient tokens, since the send button has a visual identity distinct from the `.btn-primary` utility class.

- Background: linear-gradient(180deg, `--color-button-send-start` 0%, `--color-button-send-end` 100%) (idle)
- Border: 1px solid `--color-accent-ok`
- Border-radius: `4px`
- Color: `--color-accent-ok`
- Padding: `8px var(--spacing-md)`
- Height: `--input-height`, fixed width
- Font: inherit, `--font-size-base`, bold
- Icon: a 16×16 `.icon` from the sprite — a paper-plane while idle, a spinning `loader-circle` while a turn runs
- Hover: background `--color-button-send-start`, `filter: brightness(1.15)`
- Active: background `--color-button-send-end`
- Disabled: `opacity: 0.5; cursor: not-allowed`

### Status Display

- Font size: `--font-size-small`
- `margin-left: auto`
- Min-width: `--button-min-width`
- Text-align: right
- `#status-display`: a fixed-width column; an error line wraps onto at most two lines
- States:
  - **Ready**: `--color-accent-ok`
  - **Thinking**: `--color-accent-yellow`
  - **Still thinking**: `--color-text-primary`
  - **Error**: `--color-accent-red`

### Log Entry Bubbles

Per-`log_type` bubble styling, keyed by the `MessageType` enum (`Narration`, `System`, `Input`). Each bubble is a `max-width: 85%` rounded rect with `padding: 10px 14px`, `border-radius: 12px`, and a `4px` corner radius on the side opposite the alignment to suggest a chat-bubble tail.

| Bubble | Background | Text color | Alignment |
|---|---|---|---|
| Input | `--color-log-input` | `--color-text-primary` | right (`margin-left: auto`) |
| Narration | `--color-log-narration` | `--color-text-primary` | left (`margin-right: auto`) |
| System | `--color-log-system` | `--color-accent-yellow` | centered, max-width 70% |

The base `.text` style is `font-size: var(--font-size-base); line-height: 1.5; overflow-wrap: anywhere; word-wrap: break-word`. Narration and input text use `--color-text-primary`; system uses `--color-accent-yellow`. Quoted text (`<q>`) inside `.text` is `--color-accent-orange` italic.

### Per-Entry Action Buttons

Three icon-only buttons rendered above each entry's text span. Each draws a `.icon` from the shell sprite and carries both an `aria-label` and a matching `title`.

| Button | Icon |
|---|---|
| Edit | `#i-pencil` |
| Delete | `#i-trash` |
| Retrigger | `#i-zap` |

Base `.action-btn` style: `background: rgba(255, 255, 255, 0.08); border: 1px solid rgba(255, 255, 255, 0.15); border-radius: 4px; color: var(--color-text-muted); cursor: pointer; padding: 0; width: 24px; height: 24px; display: inline-flex; align-items: center; justify-content: center; transition: background, border-color, color all on var(--transition-fast)`.

Default hover deepens the background to `rgba(255, 255, 255, 0.15)` and the border to `rgba(255, 255, 255, 0.25)`. Per-button hover colors override:

| Button | Hover color/border |
|---|---|
| Edit | `--color-accent-cyan` |
| Delete | `--color-accent-pink` |

The retrigger button carries `.action-btn` for its 24×24 box plus `.retrigger-btn` for its cyan colour (see Swipe Controls below).

### Swipe Controls

Rendered below the last entry's text. Container: flex row, gap `8px`, `margin-top: 6px`, `padding-top: 6px`, border-top `1px solid var(--color-border)`, centered.

- **Previous**: `.swipe-btn` with `#i-chevron-left`
- **Counter**: `.swipe-counter` — `font-size: 12px; color: var(--color-text-muted); font-variant-numeric: tabular-nums; min-width: 40px; text-align: center`
- **Forward**: one `.swipe-btn`, either `#i-chevron-right` (`Next swipe`) or `#i-refresh-cw` (`Retry`)

A swipe button carries `.action-btn` for its 24×24 box and `.swipe-btn` for its transparent ground and border. `.swipe-btn` base: `background: transparent; border: 1px solid var(--color-border); color: var(--color-text-muted); border-radius: 4px; cursor: pointer; transition: all 0.15s ease`.

Hover (when not disabled): `background: var(--color-bg-tertiary); color: var(--color-text-primary); border-color: var(--color-accent-cyan)`.

Disabled: `opacity: 0.3; cursor: not-allowed`.

### Retrigger Button

Carries `.retrigger-btn` alongside `.action-btn`, in the entry's action cluster above the text.

- Base: `background: transparent; border: 1px solid var(--color-accent-cyan); color: var(--color-accent-cyan); border-radius: 4px; cursor: pointer; transition: all 0.15s ease`
- Hover: inverts — `background: var(--color-accent-cyan); color: var(--color-bg-primary)`

### Inline Edit Textarea

Takes the place of the entry's text span in edit mode.

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

### Save / Cancel Buttons (Edit Mode)

Replace the entry's action-button cluster while in edit mode. Both share the same base; only hover differs.

- Base: `background: rgba(255, 255, 255, 0.08); border: 1px solid rgba(255, 255, 255, 0.15); border-radius: 4px; cursor: pointer; padding: 0; width: 24px; height: 24px; display: inline-flex; align-items: center; justify-content: center; color: var(--color-text-muted)`
- Save hover: `background: var(--color-tint-ok); border-color: var(--color-accent-ok); color: var(--color-accent-ok)`
- Cancel hover: `background: var(--color-tint-red); border-color: var(--color-accent-red); color: var(--color-accent-red)`
- Save and Cancel draw `#i-check` and `#i-x` and carry an `aria-label`

### Text Check Preview

Renders inside `#action-preview`, above the command form.

- Background: `--color-bg-header`, border `1px solid var(--color-border)`, border-radius `8px`, padding `var(--spacing-md)`
- Max-width: `600px`
- Display: flex column, gap `var(--spacing-sm)`
- Original text (read-only): label uppercase muted, value strikethrough muted
- Corrected text (editable textarea): label uppercase muted, value `--color-text-primary`
- Issue tags: orange (spell), pink (grammar), yellow (capitalization), blue-cyan (style), muted (formatting/other)
- Three controls in one row: **Send with edits** (`.btn-cyan`), **Send Original** (`.btn-primary.btn-original`), **Cancel** (`.btn-cyan.preview-cancel`). All three render at one size (see Button Utility Classes).
- Header icon (`.preview-icon`): an 18×18 `#i-spell-check` sprite icon beside the title

`#action-preview:empty` is `display: none`. While it holds the preview, the parent `.action-area` expands past its fixed height and the preview takes the full row (`.action-area:has(#action-preview:not(:empty))`).

### Panel Layout and Viewports

Every dashboard panel renders in one centered content column. The four
management panels do not scroll internally; the tab body supplies the scroll.

- Content column: `width: 100%`, `max-width: 960px`, `margin: 0 auto`,
  `padding: 24px`. Applies to `.settings-panel`, `.prompt-presets-panel`,
  `.worlds-panel`, and `.games-panel`.
- Scroll region: the active `.tab-content` (`#settings-tab`, `#worlds-tab`,
  `#prompt-presets-tab`, `#games-tab` use `overflow-y: auto`). The scrollbar
  sits at the viewport edge.
- Tab bar: `.tab-bar` scrolls horizontally (`overflow-x: auto`), and each
  `.tab` keeps its width (`flex: 0 0 auto`).
- Worlds: the edit form uses the same card frame as the world list.

Supported viewports, desktop-first:

| Range | Support |
|---|---|
| ≥ 1024×700 | Fully supported |
| 768–1024 | Best-effort |
| Phone (< 768px) | Out of scope (the existing `@media (max-width: 768px)` rules stay) |

### Settings Panel

- Content column: as Panel Layout
- Display: flex column, gap `var(--spacing-md)`

### Settings Sub-tabs

- Bar: flex row, gap `var(--spacing-sm)`, bottom border `var(--color-border)`
- Sub-tab: transparent, muted text, `2px` transparent bottom border; `.active` uses `--color-accent-ok` text and bottom border
- Degraded marker (`.subtab-degraded-marker`): a `#i-triangle-alert` sprite icon in `--color-accent-orange` beside the Connections label while a role is degraded, with a `title` tooltip that reads "A role is degraded (engine-wide role health)"
- Panel (`.settings-subtab-panel`): `display: none`; `.active` is `display: flex`, flex column, gap `var(--spacing-md)`

### Role Rows

- Row: flex, wrap, gap `var(--spacing-sm)`, `--color-bg-secondary` background, `1px solid var(--color-border)` border, `border-radius: 8px`, padding `var(--spacing-sm) var(--spacing-md)`
- Role name: bold, min-width `90px`
- Connection select: flex `1 1 220px`, min-width `160px`
- Health: `.role-health.healthy` `--color-accent-ok`, `.role-health.degraded` `--color-accent-red`, `.role-health.unknown` muted; a degraded role carries the shared error disclosure

### Connection Rows

- List: flex column, gap `var(--spacing-sm)`
- Row (`.connection-row`): flex, wrap, gap `var(--spacing-md)`, `--color-bg-secondary` background, `1px solid var(--color-border)` border, `border-radius: 8px`, padding `var(--spacing-sm) var(--spacing-md)`
- Meta: flex column, name bold and provider/model small muted
- Role badges: flex row, gap `4px`
  - **Narrator badge**: `--color-tint-ok` background, `--color-accent-ok` text, `--color-tint-ok-edge` border
  - **Quantifier badge**: `--color-tint-orange` background, `--color-accent-orange` text, `--color-tint-orange-edge` border
- Actions: flex row, gap `var(--spacing-sm)`, pushed right with `margin-left: auto`
- Test control: `.btn-cyan`; its result goes into the row's `.connection-test-slot`
- Result slot (`.connection-test-slot`): full-width, hidden while empty; a passing result is `.connection-test-result.success` in `--color-accent-ok`, and a failure carries the shared error disclosure

### Connection Form Page

- The shared Add/Edit page is a `.settings-panel` with `.connection-form-page`
- Back link: transparent, cyan text, no border, aligned to the start, with a `#i-chevron-left` sprite icon
- Form fields inherit the `.settings-panel` input/select styling; actions are a flex row with gap `var(--spacing-sm)`
- Test control: its result goes into the form's `.connection-test-slot`, with the same styles as a connection row

### Text Check Card

- The Text Check sub-tab keeps the `.connection-card` frame: `--color-bg-secondary` background, `1px solid var(--color-border)` border, `border-radius: 8px`, padding `var(--spacing-md)`
- Header: flex, space-between, wrap; title bold `1.05em`

### Button Utility Classes

Three utility classes provide the gradient+border+text styling for action buttons across the dashboard panels. Each gradient comes from its own token pair. Context-scoped selectors (`.settings-panel button`, `.prompt-presets-panel button`, `.games-panel button`) apply layout overrides only — gradients come from the utility classes.

| Class | Gradient | Text/border | Padding | Typical actions |
|---|---|---|---|---|
| `.btn-primary` | `--color-button-primary-start` → `--color-button-primary-end` | `--color-accent-ok` | `8px 20px`, bold | Save, create, add-connection, submit |
| `.btn-cyan` | `--color-button-cyan-start` → `--color-button-cyan-end` | `--color-accent-cyan` | `4px 12px`, xs font | Edit, view |
| `.btn-danger` | `--color-button-danger-start` → `--color-button-danger-end` | `--color-accent-red` | `4px 12px`, xs font | Delete, reset |

`.btn-primary`, `.btn-cyan` and `.btn-danger` buttons in `.form-actions`, `.card-actions` and `.game-actions` render at `--font-size-base` and `8px 20px`. Buttons in other rows (a world card's Edit/Delete, a connection row's Edit/Test/Delete) keep the compact xs size.

Hover state for each class swaps to the start colour, brightened with `filter: brightness(1.15)`, and adds no glow.

### LLM Messages Panel

- Panel: `flex: 1; overflow-y: auto; padding: var(--spacing-md); min-height: 0`
- List: flex column, gap `var(--spacing-sm)`
- Card: `--color-bg-secondary` background, `1px solid var(--color-border)` border, `border-radius: 6px`, `overflow: hidden`
- Header: flex row, `--color-bg-tertiary` background, gap `var(--spacing-sm)`, padding `var(--spacing-sm) var(--spacing-md)`, hover uses `--color-bg-header`
  - Agent: bold, `--color-accent-cyan`, `--font-size-small`, uppercase, min-width `80px`
  - Model: muted, `--font-size-xs`, flex 1
  - Timestamp: muted, `--font-size-xs`, monospace
  - Error badge (when present): red text, red border, `0 2px 6px`, `--font-size-xs`, bold
- Body: `display: none` by default; `.expanded` adds `display: block` and a top border
- Three prompt blocks per card: system prompt preview, user prompt preview (open by default), response preview; each rendered as `<details>` with the agent-color heading
- Two raw JSON blocks below the prompts: raw request JSON, raw response JSON; each in a `<details>` block
- Empty state: "No LLM messages yet" (muted, italic, centered, padded)

## Document References

- [`./dashboard.md`](./dashboard.md) — page layout, tabs, polling cadences, and the story-log entry controls.
- [`../game_flow.md#text-check-branch`](../game_flow.md#text-check-branch) — text-check settings and the preview UI's data model.
- [`../../../specs/llm_messages.md`](../../../specs/llm_messages.md) — LLM Messages tab content.
