# Fix dialogue colour and small-text contrast

Type: task (AFK)
Status: open
Blocked by: 14

## Question

Two colour problems:

- Quoted dialogue renders in `--color-accent-red` (`rgb(255,68,68)`, 3.6:1 on the narration bubble). `ui_design.md` specifies orange `#ffb347`. Red speech also reads as an error. (Finding 3.1.)
- The swipe counter and timestamps use `#888` on `#1a3a3a`: 3.46:1 at 11–12px. (Finding 3.2.)

Change the rules or tokens so every text meets WCAG AA (4.5:1).

## Context

- Blocked by [Decide whether to keep the neon palette](14-decide-palette.md), because a palette change would redo this.
- This is a pure CSS fix, so no new test is needed. Keep `ui_design.md` in step with any token change.

## Done when

- Dialogue uses the documented colour. Each measured text is at least 4.5:1.
- `python build.py` is green. Commit after user approval.
