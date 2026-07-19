---
diataxis: reference
title: UI Design
---

> **Diátaxis mode:** Reference. The dashboard's design tokens and component specs as they are: the CSS custom properties for colors, typography, spacing, sizing, and animation, plus the structural and visual specifications for every dashboard component. The problem it solves for the reader is *look-up* — what token or selector to use for a given visual effect, and how each component is composed.

## Overview

The dashboard's visual language is defined by a small set of CSS custom properties (design tokens) and a structured set of component specifications. Tokens are the source of truth for colors, typography, spacing, sizing, and animation timings; components declare the token-derived styling for each dashboard surface. The static stylesheet at `assets/styles.css` is the binding code that consumes both.

This doc carries the token tables verbatim because the tokens ARE the reference — there is no single source in code that an LLM can grep to recover the `--color-accent-green` value. Component specs describe structure and behavior in prose, with the CSS implementation deferred to `assets/styles.css`.

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
| `--color-accent-red` | `#ff4444` | Error status, Disconnected status, danger buttons |
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
| `--font-size-xs` | `11px` | Connection status, badges |
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
- Contains: game title (left), current game name, location, connection status, and reset button (right)
- Location is **not** in the header — it appears in the story log as the active-room location header
- Reset button (`.reset-btn`): margin-left auto, margin-right `var(--spacing-sm)`, font-size `--font-size-xs`, padding `4px var(--spacing-sm)`, red gradient with `--color-accent-red` border and text; hover deepens gradient and adds a red glow

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

### Connection Status

- Font size: `--font-size-xs`
- Padding: `2px var(--spacing-sm)`
- Border radius: `3px`
- States:
  - **Connected**: `--color-accent-green` text, `rgba(0, 255, 0, 0.1)` background
  - **Disconnected**: `--color-accent-red` text, `rgba(255, 68, 68, 0.1)` background

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

Four buttons rendered above each entry's text span. Conditional visibility rules live in `NarrativeLogTemplate::new` (templates.rs).

| Button | Glyph | Visibility rule |
|---|---|---|
| Edit | ✎ | always visible on every entry |
| Delete | 🗑 | last entry, only when more than one entry exists |
| Check | ✓ | input entries only (spellcheck the user's text) |
| Retrigger | ♻ | last entry, narration or dialogue, no event continuation, previous turn had a trigger |

Base `.action-btn` style: `background: rgba(255, 255, 255, 0.08); border: 1px solid rgba(255, 255, 255, 0.15); border-radius: 4px; color: var(--color-text-muted); cursor: pointer; font-size: 14px; padding: 2px 6px; min-width: 24px; height: 24px; display: inline-flex; align-items: center; justify-content: center; transition: background, border-color, color all on var(--transition-fast)`.

Default hover deepens the background to `rgba(255, 255, 255, 0.15)` and the border to `rgba(255, 255, 255, 0.25)`. Per-button hover colors override:

| Button | Hover color/border |
|---|---|
| Edit | `--color-accent-cyan` |
| Delete | `--color-accent-pink` |
| Check | `--color-accent-green` |
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
- Three buttons:
  - **Send Corrected** — submits the corrected text
  - **Send Original** — submits the original text
  - **Cancel** — restores the action area from `data-original-html`
- The check button itself (`.btn-check`): transparent background, cyan border+text, padding `8px 14px`, height `var(--input-height)`, bold; hover adds a cyan glow

When the action area contains a `.text-check-preview`, the parent `.action-area` expands: `height: auto; min-height: var(--action-area-height); align-items: flex-start; padding-top/bottom: var(--spacing-md)`.

### Settings Panel

- Padding: `var(--spacing-md)` (16px)
- Max-width: `800px`
- Display: flex column, gap `var(--spacing-md)`
- `overflow-y: auto`

### Connection Cards

- Background: `--color-bg-secondary`
- Border: 1px solid `--color-border`
- Border-radius: `8px`
- Padding: `var(--spacing-md)`
- Margin-bottom: `var(--spacing-md)`
- Header: flex, space-between, wrap
  - Title: bold, `1.05em`
  - Badges: flex row, gap `4px`
    - **Narrator badge**: green background `rgba(0, 255, 0, 0.12)`, green text, green border
    - **Quantifier badge**: orange background `rgba(255, 179, 71, 0.12)`, orange text, orange border
- Details: small text, muted color, line-height 1.5
- Actions: flex row, gap `var(--spacing-sm)`, wrap

### Connection Edit Form

- Background: `--color-bg-secondary`
- Border: 1px solid `--color-accent-cyan` (cyan accent)
- Border-radius: `8px`
- Padding: `var(--spacing-md)`
- Form groups: flex column, gap `4px`
- Labels: small text, muted color
- Inputs / selects: same styling as command input
- Focus: cyan border, cyan box-shadow

### Button Utility Classes

Three utility classes provide the gradient+border+text styling for action buttons across the dashboard panels. Gradient hex values are hardcoded in the class definitions (NOT tokenized). Context-scoped selectors (`.settings-panel button`, `.prompt-presets-panel button`, `.games-panel button`) apply layout overrides only — gradients come from the utility classes.

| Class | Gradient | Text/border | Padding | Typical actions |
|---|---|---|---|---|
| `.btn-primary` | `#2a5a2a` → `#1a4a1a` (idle) / `#3a6a3a` → `#2a5a2a` (hover) | `--color-accent-green` | `8px 20px`, bold | Save, create, set-narrator, submit |
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

- [`./dashboard.md`](./dashboard.md) — page layout, tabs, polling cadences, and the per-flow interactions whose visual surface this doc specifies.
- [`../game_flow.md#text-check-branch`](../game_flow.md#text-check-branch) — text-check settings and the preview UI's data model.
- [`../narrative/narration_system.md#llm-call-logging--forensics`](../narrative/narration_system.md#llm-call-logging--forensics) — LLM Messages tab content and the 50-row `llm_messages` cap.
- [ADR-001: HTMX Web Dashboard Architecture](../../../docs/adr/adr-001-htmx-web-dashboard.md) — HTMX + server-rendered HTML architecture.
- [ADR-003: Askama Template Engine](../../../docs/adr/adr-003-askama-templates.md) — compile-time template validation.
