# A Playwright click that scrolls closes the error popover it opens

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

`assets/dashboard-errors.js:247` closes every open error popover on any `scroll` (capture phase). A Playwright `click` scrolls its target into view first, and the browser fires that `scroll` event a frame later, after the click has opened the popover. So a stub check that clicks a Details toggle below the fold reads the popover as closed. A real user scrolls first, so the product is not affected. Any change that makes a dialog taller can break such a check with no product change.

Should the tests click through a shared helper that scrolls and waits a frame, or should the script ignore the scroll that a click causes, or neither?

## Context

- Source: `/retro` 2026-10-31, session `01a122c2-4e16-73db-89b0-50a2465e456b` (branch `opencode-go-provider`).
- Two new fields on the connection form made the dialog taller. `check_escape_closes_an_open_error_detail_before_it_closes_the_dialog` (`tests/browser/stub/dialog.rs`) failed at `Details opens` in 3 of 3 runs, and passed with the two fields removed. The fix in that branch is local to the check: `scrollIntoView`, two `requestAnimationFrame`s, then the click.
- I don't know why the Phase 4 full gate of the same branch (`logs/build_20261031_010740.log`) passed with the same template.
- `tests/test_utils` has no scroll or frame-wait helper; other checks click toggles with plain `locator.click` (`rg -n "error-details-toggle" tests/browser`).
