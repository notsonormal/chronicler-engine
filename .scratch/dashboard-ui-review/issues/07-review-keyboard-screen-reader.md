# Review keyboard use and screen-reader output

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

Can every dashboard task be done with the keyboard alone? What does the accessibility tree expose for each tab?

## Context

- The first review found icon buttons with only `title` and no `aria-label` (finding 3.3), and New Game selects with no labels (finding 5.5). Do not re-report these. Look for what is still unknown: tab order, focus visibility, focus after swaps, and live regions for status changes.
- `chrome_devtools_*` has no accessibility snapshot. `node scripts/cdp.mjs snap <target>` gives the accessibility tree.

## Work

- Use `/chronicler-ui-investigator`. Use the same method and P1–P3 scale as the [first review](../review-2026-09-29.md).

## Done when

- The findings are in the ticket answer.
- Each finding is a new ticket in this map or is added to an existing one. New tickets are also added to the final re-review's `Blocked by:`.

## Answer

### Method

Server `target/debug/chronicler_engine --world redmist_estate --persona julian --port 3333` on an isolated copy of the DB (6 story-log entries; options generation returns 401 in this DB). Shared headless Chrome, my own tab only. Real key input via a throwaway CDP driver, `tmp/ui-key.mjs`, which sends `Input.dispatchKeyEvent` (not synthetic JS events); a11y trees via `node scripts/cdp.mjs snap`. Every tab visited; trees saved as `tmp/a11y-<tab>.txt`. Screenshots `tmp/ui-review/90`–`99`, `A0`–`A9`, `B0`–`B4`. Data touched: one turn added to the isolated game ("I check whether the study door is locked"), the active game's Mode toggled to Interactive Fiction and back to Novel, the worlds edit form opened and cancelled without saving, a preset View opened. Nothing persisted outside the isolated DB.

`[known]` = produced by real `Input.dispatchKeyEvent`/forced-colors emulation and confirmed in a screenshot I looked at. `[inferred]` = read from code or from the template's swap target, not exercised.

### Findings

| # | P | Finding | Evidence |
|---|---|---|---|
| K1 | P1 | **The 2s story-log poll ejects keyboard focus to `<body>`.** Focus a `.log-entry` button and wait for the poll to swap `#story-log`; a MutationObserver recorded `activeElement` as BODY on the same mutation as the swap, and Tab traces repeatedly land on BODY in the middle of the entry sequence. Focus can only be held inside the log for under 2s, so reading or stepping through the narrative with the keyboard is unreliable. Root cause is finding 2.4 / ticket 10; the keyboard consequence is new. | [known] `96-focus-body-after-poll.png`; observer log (swaps at ~538ms and ~2534ms, `activeTag: BODY`); Tab trace `92`→`93` |
| K2 | P1 | **LLM Messages cannot be expanded by the keyboard at all.** Each row header is `<div class="llm-message-header" onclick="toggleLlmMessage(this)">` (`templates.rs:159`) with no `role`, `tabindex` or key handler, and the panel contains zero focusable elements — the a11y tree shows only `StaticText`, and Tab leaves the panel straight to `<body>`. The System / User / Response / raw JSON bodies (the panel's only content) are mouse-only. The mouse path works (click → `.llm-message-card.expanded`). | [known] `99-llm-messages-keyboard.png`, `tmp/a11y-llm-messages.txt`, `focusables: []` eval |
| K3 | P2 | **Changing a Games-tab posture/preset select with the keyboard drops focus to `<body>`.** ArrowDown on `select[name=narrator_mode]` changes the value, posts to `#game-posture-controls` with `hx-swap="outerHTML"`, and focus is lost; the select has no `id`, so htmx's id-based focus restore does nothing. Same for Perspective/Tense and the three preset pickers (same container/target). | [known] `A1-games-posture-focus.png`; `focus-mode {value: novel}` then `ArrowDown` → `active: BODY` |
| K4 | P2 | **There are no live regions anywhere.** No element in the dashboard has `aria-live` or `role` (only one `aria-hidden="true"` on the NPC name label). The status display, generation errors, the toast, new narration, the options dock and text-check results are all silent for a screen reader. A real keyboard turn produced `Ready → Thinking... → Generating narration... → Quantifying scene... → Ready`; none of it is announced, and the toast enters the tree as a bare `StaticText` at the top of the page with no `role="alert"`. | [known] attribute scan (all null), `tmp/a11y-*.txt`, `A7-toast-no-live-region.png`, status history log |
| K5 | P2 | **The tab bar has no tab semantics.** The six tabs are plain `<button>`s — no `role="tablist"/"tab"`, no `aria-selected`, no `aria-controls`, no `role="tabpanel"` on the panels — and the active tab is indicated by colour and a border only. A screen reader hears six unrelated buttons and gets no indication of which panel is showing or that a panel changed. | [known] `tmp/a11y-game.txt` (`[button] Game`, `[button] Settings`, …), DOM attribute scan |
| K6 | P2 | **In-place swaps drop focus to `<body>` with no focus management.** Worlds Edit and Cancel, preset View (and by the same `hx-target="closest .preset-card"` / `.prompt-presets-panel` outerHTML pattern: preset Close, Cancel, Set Active, Duplicate), and the edit-mode Escape/Save path all replace the container that held focus and leave focus on body. A keyboard user must re-Tab from the top of the page after each. | [known] `A2-worlds-edit-focus.png`, `A3-worlds-cancel-focus.png`, `A6-presets-view-focus.png`, edit-mode trace (`after-escape {focus: BODY}`); [inferred] preset Close/Cancel/Set Active/Duplicate |
| K7 | P2 | **The story log is not keyboard-scrollable on its own.** `#story-log` is the only scroller (`overflow-y: auto`, 931 > 707px) and has `tabIndex -1`, so it never receives focus. Arrow keys scroll it only while a descendant button has focus (0 → 120px), and the poll then removes that focus. A keyboard-only user cannot scroll the narrative deliberately. | [known] overflow/tabIndex eval; `plan-log-scroll` trace |
| K8 | P3 | **The slash menu is invisible to assistive technology.** The suggestions are `<div class="slash-suggestion">` with no `role="listbox"/"option"`, and the input has no `aria-expanded`/`aria-controls`/`aria-activedescendant`. The keyboard flow works (ArrowDown moves the active item, Enter inserts `/guide `), but nothing is announced and the active option is not exposed. | [known] DOM eval, `A4-slash-after-enter.png` |
| K9 | P3 | **No landmarks, page heading or skip link.** Every tab's a11y tree is `[RootWebArea]` → header `StaticText` → six buttons → panel; there is no `<main>`/`<nav>`, no page-level heading, and no way to skip the header and tab bar, which are re-read on every panel change. | [known] `tmp/a11y-*.txt` |
| K10 | P3 | **No visible focus indicator under forced colors (Windows High Contrast).** With `forced-colors: active` emulated and focus reached by real Tab (`:focus-visible` true), the command input computes `outline: none`, `box-shadow: none` and a single forced border colour — no ring is visible in the shot. Buttons keep the UA ring, so only the fields that set `outline: none` are affected. | [known] `B4-forced-colors-input-realkey.png`, `B2-forced-colors-tab-focus.png` |
| K11 | P3 | **The command input's only accessible name is its placeholder.** The a11y tree exposes `[textbox] Enter command...`; there is no `<label>` and no `aria-label`. | [known] `tmp/a11y-game.txt` |

### What works (so the negative findings are scoped)

- **Tab order is sane in every tab.** Game: tab bar → log-entry buttons → command input → Send. Settings: tab bar → connection buttons (focus scrolls the panel into view, `scrollTop` 0 → 1107) → Add-Connection fields → Text Check select/checkbox/Save. Hidden panels are `display: none` and correctly absent from the keyboard order and the a11y tree.
- **Focus rings are visible in normal rendering.** The UA ring paints on buttons (`auto 1px`), and inputs/selects get a green/cyan border plus glow. Verified on tab, log button, command input, Send, Settings button, preset field.
- **Keyboard activation works.** Enter activates tabs, log-entry buttons, Worlds Edit, preset View, preset `<details>` summary; Enter in the command input submits a turn and **focus stays in the input** (`HX-Retarget: #status-display`), so repeated turns are one keystroke apart.
- **Slash menu is keyboard-driven** (ArrowUp/Down, Enter, Escape) and Escape returns focus to the input.
- **Edit mode is keyboard-operable**: Enter opens it, focus moves into `#edit-textarea`, polling pauses, Escape cancels and resumes polling.
- **Native `confirm()` guards** the destructive actions (delete world/game/message, reset game), so they are keyboard-usable; all Games/Worlds/Presets/Settings controls are reachable buttons, selects, checkboxes or disclosures with labels (`for`/`id` present on the Settings and Presets forms).

### Tasks that cannot be completed by keyboard alone

1. **Reading an LLM message** — expanding a row to see System / User / Response / raw JSON has no keyboard path (K2). This is the one task that is fully blocked.
2. **Reading or navigating the story log** — not blocked outright but severely impaired: focus is ejected every 2s (K1) and the log cannot be focused to scroll (K7).

Everything else on the six tabs was completed with real key events.

### Proposed tickets

**Title:** Make LLM Messages rows keyboard-operable
**Type:** task
**Blocked by:** —
**Question:** The LLM Messages panel has zero focusable elements. Each row header is `<div class="llm-message-header" onclick="toggleLlmMessage(this)">` with no `role`, `tabindex` or key handler, so the System/User/Response/raw-JSON bodies cannot be opened without a mouse (finding K2, `tmp/ui-review/99-llm-messages-keyboard.png`). Make each row expandable with Enter and Space and expose the expanded state to assistive technology — for example a `<button aria-expanded>` header with the body as a labelled region, or a native `<details>`/`<summary>`. Keep the existing toggle behaviour and the JS-held expansion state that survives the panel's htmx refresh, and add browser coverage for the keyboard path.

**Title:** Restore keyboard focus after in-place htmx swaps
**Type:** task
**Blocked by:** —
**Question:** Keyboard focus falls to `<body>` whenever a swap replaces the element holding it (findings K3, K6). Reproduce: Games tab, focus a posture or preset select, press ArrowDown — `#game-posture-controls` is swapped and focus is lost; likewise Worlds Edit and Cancel, Prompt Presets View/Close/Cancel/Set Active/Duplicate, and cancelling edit mode with Escape. Choose one mechanism — give each replaced control a stable `id` so htmx's built-in focus restore works, or move focus to the replacement (or a sensible target) after each swap — and apply it consistently. Cover at least the posture-select and Worlds-Cancel paths with a browser test.

**Title:** Announce dynamic state changes to assistive technology
**Type:** task
**Blocked by:** 08
**Question:** No element in the dashboard carries `role` or `aria-live` (finding K4), so the status display (Ready → Thinking → Generating narration → Quantifying → Ready), generation errors, the toast, new narration, options and text-check results are all silent for a screen reader. Add live regions where they belong, choosing politeness per region. Two constraints: `#story-log` is replaced wholesale every 2s, so a naive `aria-live` on it would re-announce the entire log each poll — the announcement must be scoped to the changed entry or wait for ticket 10; and wait for 08 so the health/status model settles before regions are wired to it.

**Title:** Give the tab bar and slash menu real ARIA semantics
**Type:** task
**Blocked by:** —
**Question:** The six tab buttons are plain `<button>`s with no `role="tablist"/"tab"`, `aria-selected` or `aria-controls`, and the active panel is colour-only (finding K5), so a screen reader cannot tell which panel is shown or that it changed. The slash menu is a `<div>` of `.slash-suggestion` divs with no `role="listbox"/"option"` and no `aria-expanded`/`aria-activedescendant` on the input (finding K8), so the keyboard-driven suggestions are invisible to assistive tech. Add the ARIA tab pattern and combobox/listbox semantics without changing visual or keyboard behaviour (Enter/Space activation, Arrow keys, Escape).

**Title:** Landmarks, a command-input label and a forced-colors focus ring
**Type:** task
**Blocked by:** —
**Question:** The page has no `<main>`/`<nav>` landmarks, no page heading and no skip link (finding K9), so a screen-reader user re-hears the header and the six-tab bar after every panel change; the command input's only accessible name is its placeholder (finding K11); and fields set `outline: none`, so under forced colors there is no visible focus indicator on the command input (finding K10, `tmp/ui-review/B4-forced-colors-input-realkey.png`). Add landmarks plus a skip-to-content link, a real label for the command input, and a `:focus-visible` outline that survives forced colors.

**Existing ticket: 10 — why:** findings K1 and K7 are the keyboard consequences of the story-log poll. The chosen poll strategy (unchanged / morph / pause) should also be evaluated against focus retention inside `#story-log` and against making the scroller itself focusable (`tabindex="0"`), since a keyboard user cannot scroll the narrative otherwise.

### Not checked

- **Options dock with options present** — options generation returns HTTP 401 in this DB, so no options existed to drive. (Ticket 06 covers this state; the option buttons are `<button>`s and would be keyboard-reachable.)
- **The text-check-enabled preview/Confirm/Cancel path** — text check is disabled in this DB and I did not change the user's setting; the preview replaces `#action-area`, and `restoreActionArea()` does not restore focus, but I did not exercise it. Ticket 04 owns that flow.
- **Create/delete flows' dialogs** — I confirmed the buttons are keyboard-activatable and the guards are native `confirm()`, but did not complete a create or a delete; ticket 05 owns those flows.
- **Thinking/error announcements during a real failure** — the observed turn succeeded; the 401 options failure renders as a system log entry rather than a live error, so I did not measure announcement of a mid-generation failure.
- **Small viewport / mobile** — desktop 1280x800 only, matching the first review's primary target.
