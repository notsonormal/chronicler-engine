# Align the frontend docs with the dashboard

Type: task (HITL)
Status: open
Blocked by: 08, 12, 22, 42

## Question

Where do `docs/diataxis/reference/frontend/ui_design.md`, `dashboard.md`, `docs/diataxis/explanation/dashboard_design.md` and `docs/diataxis/reference/game_flow.md` still disagree with the dashboard? Make them agree.

## Context

- Theme 6 of the review found these drifts:
  - the reset button location
  - NPC portrait labels (fixed by 22)
  - the dialogue colour (fixed by 12)
  - the error banner as the error path (changed by the Theme 1 work from 08)
- The code review of [Rework the action area so a text check cannot strand it](42-rework-the-action-area.md) found more drift, all ownerless until now:
  - `dashboard.md:73,78,117` and `dashboard_design.md:55,64` still describe a "Stop" button; the primary button is now a disabled "Generating…" indicator.
  - `dashboard.md:80` still describes the preview replacing the action area and Cancel restoring it from `data-original-html`; the preview now renders in `#action-preview` and Cancel calls `closeActionPreview()`.
  - `ui_design.md:341,349,352` still say the preview replaces the action area and name the old buttons and the `:has(.text-check-preview)` selector; the labels are now "Send Original" / "Send with edits" / Cancel and the selector is `#action-preview:not(:empty)`.
  - `game_flow.md:142` says a manual entry check returns the same preview shape; `/check-text` now returns the read-only `TextCheckResultTemplate`.
- **One choice is the user's:** the docs put the reset button in the header, but the code has it on the Games tab, with a confirm prompt. Ask whether the docs follow the code, or the button moves. This is why the ticket is HITL.
- Added from [Decide how the dashboard shows each kind of failure](08-decide-failure-display.md): the state names **Healthy**, **Degraded** and **Unreachable** are deliberately not in `CONTEXT.md`, so `dashboard.md` is where they get defined for readers. Also remove the header's `Connected` and the `#error-notification` toast from these docs — the error path is now the failure banner ([51](51-add-failure-banner.md)), the inline slot and the clamped status display.
- Run `python build.py validate-docs` after the edits.

## Done when

- The docs match the dashboard.
- `python build.py` is green. Commit after user approval.
