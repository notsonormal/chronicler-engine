# Expose the dashboard to assistive technology

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

The dashboard's structure is invisible to assistive technology:

- The six tab buttons are plain `<button>`s with no `role="tablist"/"tab"`, no `aria-selected` and no `aria-controls`, and the active panel is indicated by colour and a border only, so a screen reader hears six unrelated buttons and cannot tell which panel is showing or that it changed.
- The slash menu is a `<div>` of `.slash-suggestion` divs with no `role="listbox"/"option"`, and the input has no `aria-expanded`/`aria-controls`/`aria-activedescendant`, so the keyboard-driven suggestions are invisible.
- There are no `<main>` or `<nav>` landmarks, no page heading and no skip link, so the header and the six-button tab bar are re-read after every panel change.
- The command input's only accessible name is its placeholder (`[textbox] Enter command...` in the accessibility tree).
- Fields set `outline: none`, so under forced colors the command input shows no focus indicator while buttons keep the user-agent ring.

Add the tab and combobox/listbox patterns, landmarks with a skip-to-content link, a real label for the command input, and a `:focus-visible` outline that survives forced colors — without changing the visual or keyboard behaviour.

## Context

- Findings K5 (P2) and K8, K9, K10, K11 (P3) of [Review keyboard use and screen-reader output](07-review-keyboard-screen-reader.md).
- Evidence (local only): `tmp/a11y-game.txt` (`[button] Game`, `[button] Settings`, …), `tmp/a11y-*.txt`, `tmp/ui-review/A4-slash-after-enter.png`, `B4-forced-colors-input-realkey.png` (real Tab under emulated `forced-colors: active`), `B2-forced-colors-tab-focus.png`.
- What must not change: Enter/Space activation, Arrow keys in the slash menu, Enter to insert, Escape to close and return focus to the input. The panels already hide correctly — `display: none` keeps them out of the tab order and the accessibility tree.
- Focus rings in normal rendering are already fine (user-agent ring on buttons, a green/cyan border and glow on inputs and selects); scope the CSS change to forced colors and verify it with real key input, not a synthetic event.
- Turning the slash suggestions into a listbox means moving the active-item highlight from a class to `aria-activedescendant`. Add `aria-label`s to icon-only controls only if [Decide the icon-button approach](13-decide-icon-buttons.md) has landed.
- Announcing dynamic state is [Announce dynamic state changes to assistive technology](47-announce-state-changes-to-at.md) (blocked by 08); focus restoration after swaps is [Restore keyboard focus after in-place htmx swaps](45-restore-focus-after-inline-swaps.md). Neither belongs here.

## Done when

- The accessibility tree exposes a tablist with a selected tab and its panel, and a listbox with the active option, on every tab and in the slash menu.
- The page has a main landmark and a skip link, and the command input has a real accessible name.
- The command input shows a visible focus ring under forced colors.
- A test covers the tab-panel relationship and the slash menu's active option.
- Test placement follows `tests/STRATEGY.md`, and the answer names the tier. Until [Decide what a tier-1 test may observe](31-decide-tier-1-observations.md) resolves, write new spec Givens and Thens in `CONTEXT.md` terms rather than field names.
- `python build.py` is green, the user reviews the diff, and the work is committed through `/commit-and-push`.

## Answer

The dashboard now exposes its structure to assistive technology, with no change to the visual or keyboard behaviour.

- **Landmarks:** a `.skip-link` ("Skip to content") targets `<main id="main-content" class="main-content" tabindex="-1">`, which now wraps all six tab panels; `.main-content` carries the body's column layout so the panels fill the space as before.
- **Tabs:** `.tab-bar` is `role="tablist"` with `aria-label="Dashboard sections"`; each tab is `role="tab"` with a stable id, `aria-controls` its panel and `aria-selected`; each panel is `role="tabpanel"` with `aria-labelledby` its tab. The tab click handler is now the single writer of `aria-selected`, keeping it in step with the `.active` class.
- **Slash menu:** `#slash-menu` is `role="listbox"` with an `aria-label`; each `.slash-suggestion` gets a stable `id="slash-option-N"`, `role="option"` and `aria-selected`. The command input becomes `role="combobox"` with `aria-expanded`, `aria-controls="slash-menu"` and `aria-autocomplete="list"`, and a new `syncSlashActiveDescendant` keeps `aria-activedescendant` on the highlighted option as the arrow keys move it. The `.active` class stays as the visual highlight, mirrored into `aria-selected` rather than replaced — the ticket allowed either, and keeping both leaves spec 31.3's wording intact.
- **Command input name:** a `.visually-hidden` `<label for="command-input">Command</label>` plus `id="command-input"` gives the input a real accessible name instead of its placeholder.
- **Forced colours:** `@media (forced-colors: active) { :focus-visible { outline: 2px solid Highlight !important; outline-offset: 2px; } }` restores the focus ring that the fields' `outline: none` suppressed, while normal rendering keeps its existing rings. Added the `.visually-hidden` and `.skip-link` helpers.

Untouched: Enter/Space activation, Arrow keys in the slash menu, Enter to insert, Escape to close and return focus, and the `display: none` panel hiding. No `aria-label`s were added to icon-only controls: [Decide the icon-button approach](13-decide-icon-buttons.md) has not landed.

Tests — **tier 2** (`tests/browser/stub/dashboard.rs`, `tests/browser/stub/slash_menu.rs`), per `tests/STRATEGY.md`: this is the shipped shell's client behaviour over canned fragments, so faking the engine changes nothing.

- 16.16 (new, `browser_dashboard.md`) — the tab bar exposes a tablist, every tab controls a labelled tabpanel, Game starts selected, and activating Settings moves the selection and shows its panel.
- 16.17 (new, `browser_dashboard.md`) — a skip link targets the main landmark, and a label names the command input "Command" (distinct from the placeholder).
- 31.1 extended (`browser_slash_menu.md`) — the open menu is a listbox whose first option is active, with the input expanded and pointing at it.
- 31.3 extended (`browser_slash_menu.md`) — ArrowDown/ArrowUp move `aria-activedescendant` and the selected option.

Spec Givens and Thens are in user-facing terms, not field names. The forced-colours ring is CSS and gets no test, per the map's rule that pure CSS fixes need none.

Validation: full gate `python build.py` green — 1532 integration passed / 2 skipped, 43 browser, 137 guardrails, 1 architecture; clippy/fmt/spec-coverage clean. No commit made (awaiting the user's review, then `/commit-and-push`).
