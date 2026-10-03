# Expose the dashboard to assistive technology

Type: task (AFK)
Status: open
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
