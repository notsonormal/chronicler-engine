# Show Character names in the visual sidebar

Type: task (AFK)
Status: open
Blocked by: —

## Question

NPC portraits in the visual sidebar have no visible name and no tooltip, only `alt`. `ui_design.md` specifies portrait labels. Add a visible name to each Character portrait.

## Context

- Finding 3.5. Screenshot 01.
- The large empty area below the portraits is a layout question for [Decide the panel layout convention and supported viewports](19-decide-layout-convention.md).

## Done when

- Each portrait shows its Character's name.
- A test is placed by `tests/STRATEGY.md` (likely tier 1: the sidebar fragment is `curl`-observable).
- `python build.py` is green. Commit after user approval.
