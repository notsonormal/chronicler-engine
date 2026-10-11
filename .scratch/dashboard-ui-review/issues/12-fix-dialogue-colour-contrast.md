# Fix dialogue colour and small-text contrast

Type: task (AFK)
Status: resolved
Blocked by: 57

## Question

Two colour problems:

- Quoted dialogue renders in `--color-accent-red` (`rgb(255,68,68)`, 3.6:1 on the narration bubble). `ui_design.md` specifies orange `#ffb347`. Red speech also reads as an error. (Finding 3.1.)
- The swipe counter and timestamps use `#888` on `#1a3a3a`: 3.46:1 at 11–12px. (Finding 3.2.)

Change the rules or tokens so every text meets WCAG AA (4.5:1).

## Context

- This ticket is closed; its work moved to [Apply the chosen palette and fix the colour contrast](57-apply-chosen-palette.md) when the palette decision landed.
- This is a pure CSS fix, so no new test is needed. Keep `ui_design.md` in step with any token change.

## Done when

- Dialogue uses the documented colour. Each measured text is at least 4.5:1.
- `python build.py` is green. Commit after user approval.

## Answer

Absorbed into [Apply the chosen palette and fix the colour contrast](57-apply-chosen-palette.md).
Both findings (dialogue mapped to `--color-accent-red`; `#888` small text at
3.46:1) and the 4.5:1 check moved into that ticket, so the palette values and
the contrast fixes land in one commit instead of two sessions over the same
rules. Closed as absorbed, not fixed.
