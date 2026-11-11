# Align the frontend docs with the dashboard

Type: task (HITL)
Status: resolved
Blocked by: 08, 66, 22, 42

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

## Answer

The four docs now describe the shipped action area, and `dashboard.md` defines the
three health state names.

- **`dashboard.md` — Action Area.** Rewritten around the static shell: the four
  client-derived states (**Checking** → **Generating** → **Preview** → **Idle**,
  derived in that order) with what each does to the submit button and the command
  input; the status display as its own poll with its own labels and three change
  events; the pre-flight path (engine commands skip the check, a disabled check or
  auto-check dispatches, flagged issues answer into `#action-preview`); the three
  preview controls **Send with edits** / **Send Original** / **Cancel**, with
  Cancel calling `closeActionPreview()` and returning focus to the command input;
  and the `.action-area` expansion while the preview is open. The old text claimed
  three states, a "Send" button inside the preview, a `data-original-html` restore
  and an action-area innerHTML swap — all gone.
- **`dashboard.md` — new "Failure and health states" subsection.** Defines
  **Healthy** (a role whose newest LLM attempt carries no error), **Degraded** (a
  role whose newest attempt carries an error: banner, per-role effect, Details
  disclosure, Settings row label and degraded sub-tab marker) and **Unreachable**
  (the client got no usable answer for a polled region — a transport failure or a
  non-2xx poll — and clears it on the next success), plus the role-health label
  **No calls yet**. These names stay out of `CONTEXT.md`, so this doc is their
  definition.
- **`ui_design.md`.** The Action Area entry names the shell's children and the
  expansion; the Send Button entry names the "Send" / "Generating…" labels; the
  Text Check Preview entry no longer says it replaces the action area — it renders
  inside `#action-preview`, above the command form — names the shipped controls
  (`Send with edits`, `Send Original`, `Cancel`) and states the shipped selector
  `.action-area:has(#action-preview:not(:empty))` instead of
  `:has(.text-check-preview)`.
- **`dashboard_design.md`.** The client-JavaScript list said the button transitions
  through "Ready ↔ Generating ↔ Error"; the button now transitions
  Send ⇄ Generating… and locks with the input while a pre-flight check runs. (The
  "Stop" wording in this file and the `/check-text` line in `game_flow.md` were
  already fixed by 54/66/70.)
- **`game_flow.md`.** The Text-Check Branch pre-flight bullet names the preview
  region and the shipped choices: **Send with edits**, **Send Original**, **Cancel**.
- **Reset button location (the HITL item).** Already resolved in the tree: the docs
  follow the code. `dashboard.md`'s Active Game row carries the reset button on the
  Games tab with its confirm dialog, matching
  `games/templates/games.rs`'s `hx-post="/reset" hx-confirm="Reset the current game? …"`.
  No doc claimed a header reset control.

`python build.py validate-docs` is green. The full gate was not run in this session
(the change is docs-only, plus unrelated dead-code removal from the post-review pass).
