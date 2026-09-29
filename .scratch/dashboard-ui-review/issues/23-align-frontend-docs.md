# Align the frontend docs with the dashboard

Type: task (HITL)
Status: open
Blocked by: 08, 12, 22

## Question

Where do `docs/diataxis/reference/frontend/ui_design.md` and `dashboard.md` still disagree with the dashboard? Make them agree.

## Context

- Theme 6 of the review found these drifts:
  - the reset button location
  - NPC portrait labels (fixed by 22)
  - the dialogue colour (fixed by 12)
  - the error banner as the error path (changed by the Theme 1 work from 08)
- **One choice is the user's:** the docs put the reset button in the header, but the code has it on the Games tab, with a confirm prompt. Ask whether the docs follow the code, or the button moves. This is why the ticket is HITL.
- Run `python build.py validate-docs` after the edits.

## Done when

- The two docs match the dashboard.
- `python build.py` is green. Commit after user approval.
