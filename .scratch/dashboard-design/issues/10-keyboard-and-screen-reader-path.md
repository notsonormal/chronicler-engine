# Decide the keyboard and screen-reader path through the story log

Type: grilling
Status: open
Blocked by: 03

## Question

How does a keyboard or screen-reader player move through the story log and reach the Action input, and which keys swipe and Retry?

## Context

- The critique's persona findings ([the Game tab critique](../assets/game-tab-critique/report.md)):
  - A keyboard player passes 41 buttons, all named "Edit", before the Action input (verified: 44 focusable elements first).
  - An entry does not name its speaker. An Input and a Narration differ only by side and shade.
  - The critique reported that `#command-form input:focus` sets `outline: none`. The plan review found that the same rule adds a focus ring (`box-shadow: var(--shadow-focus-ring)`), and that `assets/styles.css` has a global `:focus-visible` rule. Check that the ring is visible enough, instead of adding a second one.
  - SillyTavern players expect the arrow keys to swipe on the last Message. There is no key to focus the Action input.
- What works now and must stay: the live regions, the slash menu combobox, the skip link, the `aria-label` on every icon button.
- The story-log structure follows the chosen direction ([Choose the direction of the Game tab](03-choose-the-direction.md)).
- The specs `docs/specs/browser_story_log.md`, `browser_swipes.md` and `browser_slash_menu.md` hold the current selectors.

Done when the answer records the tab order, the speaker naming, the keys, the focus style, and the build tickets they need.
