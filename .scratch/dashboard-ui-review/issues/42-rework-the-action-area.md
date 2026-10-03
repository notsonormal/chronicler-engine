# Rework the action area so a text check cannot strand it

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

With text check enabled the action area strands the player in five ways, all in the same DOM region and the same save/restore path:

- A confirm through the send preview ("Send" or "Send Original") swaps `#action-area` with `/action/confirm`'s response, which renders the command input disabled because generation has just started. Nothing re-renders the action area when generation ends — the only `action-area-refresh` listener in `assets/index.html` has no dispatcher, and the 5s status poll touches only the status span and the Send button — so the input stays disabled while the status reads "Ready". Typing does nothing; only a reload recovers.
- The ✓ on a log entry is a read-only check, but when it finds issues `/check-text` returns the full send-preview template (Send / Send Original / Cancel) into `#text-check-result` under the still-live command form, so a check can re-submit a historical entry as a new turn. A clean result is a bare "No issues found" with no reference to its entry and no dismiss control, and Cancel restores a stale copy of it because `saveActionArea()` snapshots the whole action-area HTML.
- The preview replaces the whole `#action-area`, so while it is open the phase text and the Send/Stop control are not in the DOM: submitting a flagged command mid-generation (Enter submits even though the button is disabled) hides the in-flight turn's status entirely.
- Opening and cancelling the preview both drop focus to `<body>`; the correction textarea is never focused.
- During generation the primary button is relabelled "Stop" and disabled, with no cancel route in `router.rs` — a label for an action it cannot perform.

Fix them as one change: the action area needs one state machine (idle → checking → preview → generating → idle) that reconciles the input, the status display, the primary button and the check result, rather than five independent patches.

## Context

- Findings C1 (P1), C2, C3, C4 (P2) of [Review the text-check-enabled flow](04-review-text-check-enabled.md) and 6.3 (P2) of [Review swipes, the options dock and the Thinking states](06-review-swipes-options-thinking.md).
- Evidence (local only): `tmp/ui-review/76-after-confirm-input-stuck-disabled.png` (greyed input at status "Ready"; DOM `inputDisabled: true, btnDisabled: false`), `79-log-check-with-issues-preview.png` (check → Send → a new turn was generated), `74-log-entry-check-clean-result.png`, `77-submit-during-generation-preview-hides-status.png` (DOM `statusEl: false` while `/status/generating` answered `thinking`), `81-turn-t1.png` and `88b-options-thinking.png` (the disabled "Stop").
- Mechanism: `action_confirm_handler` returns `render_action_area()` without the `HX-Retarget: #status-display` swap that `dispatch_with_status_headers` uses (`src/adapters/driving/http/builders/headers.rs`).
- Harper's suggestion quality is not the UI's to fix, but the surface must stop presenting an auto-applied set of suggestions as an authoritative "Did you mean?" — on the review day "recieve"/"phon"/"arround" became "relieve"/"pron"/"aground".
- Out of this ticket: the ✓ still offered while text check is disabled, and the missing ✓ on the command input (`checkCurrentInput()` has no caller) — both [Settings panel: roles, buttons and text-check controls](15-settings-panel-prototype.md). The preview's tofu ✍ joins [Decide the icon-button approach](13-decide-icon-buttons.md). Where status and health belong in general is [Decide how the dashboard shows each kind of failure](08-decide-failure-display.md)'s.
- A real cancellation route is a bigger decision than this ticket. If wiring the button to one looks like the right fix for the "Stop" label, stop and raise it rather than building it here.

## Done when

- After a confirm through the preview the status returns to Ready and the command input accepts and submits a new command with no reload.
- A check on a log entry cannot submit a turn; the result names its entry, can be dismissed, and Cancel does not resurrect an unrelated earlier result.
- Opening the preview mid-generation leaves the phase text on screen and updating.
- Focus lands in the correction textarea when the preview opens and returns to the command input on Cancel.
- The primary button never claims an action it cannot perform.
- A browser test covers submit → preview → confirm → Ready → type, the log-entry check with issues, the clean-result dismiss, and open → Cancel focus.
- Test placement follows `tests/STRATEGY.md`, and the answer names the tier. Until [Decide what a tier-1 test may observe](31-decide-tier-1-observations.md) resolves, write new spec Givens and Thens in `CONTEXT.md` terms rather than field names.
- `python build.py` is green, the user reviews the diff, and the work is committed through `/commit-and-push`.

## Answer

The action area is now a static shell driven by one client state machine, so a text check can no longer strand it. All five stranding paths are removed at the mechanism level rather than patched one by one:

- **C1 (P1, input stuck disabled):** `/action/confirm` returns the status fragment with `HX-Retarget: #status-display` (via `dispatch_with_status_headers`) instead of a freshly rendered disabled action area. The command form and status display are never replaced, so no input can be stranded; the status poll's reconciler unlocks it.
- **C2:** `/check-text` renders a new read-only `TextCheckResultTemplate` ("Checked entry #N", dismiss control, issues list, no forms) instead of the send preview. `saveActionArea`/`restoreActionArea` and the whole-area snapshot are deleted, so Cancel cannot resurrect an earlier result, and a log-entry check cannot submit a turn.
- **C3:** the send preview renders into its own `#action-preview` element, so `#status-display` stays in the DOM and keeps updating while the preview is open, including mid-generation.
- **C4:** focus follows the preview surface — `#corrected-textarea` on open, command input on close/cancel.
- **6.3:** the primary button no longer claims a "Stop" it cannot perform; while generating it is a disabled "Generating…" indicator. The ticket explicitly offered this non-cancel route, so **no decision was raised and no cancel route was built**.
- **Harper wording:** the preview is retitled "Text check suggestions" with a "Suggestions only…" note, and **Send Original** is primary (nothing auto-applies) with **Send with edits** secondary.

State machine (client, `assets/index.html`): `deriveActionState()` reads live DOM in priority order `checking` → `generating` → `preview` → `idle`; `applyActionState(state)` is the single writer for the input and primary button; `syncActionState()` re-derives after every htmx swap, status poll and submit and moves focus on the preview open/close edge.

Tests: 16.9/16.10 reworded, 16.11 rewritten, 16.12–16.15 added. 16.12 is the tier-3 acceptance proof (real text check enabled, misspelled submit → preview → confirm → Ready → form still present, input enabled → new command accepted with no reload and the log grows). 16.9–16.11 and 16.13–16.15 are tier-2 stub tests. The server contract (confirm → bare status span + `HX-Retarget`; `/check-text` → read-only panel) is pinned at tier 1 in `tests/http/requires_migration/{text_check,fragment}.rs`, renamed in place with the quarantine count unchanged at 85. The stub now renders its check result through the engine's own `TextCheckResultTemplate` and consumes `add_status_swap_headers`, so the canned shape cannot drift from the shipped one.

Validation: full gate `python build.py --no-fmt` green — architecture 1, guardrails 137, integration 1531 passed / 2 skipped, browser 35 passed; spec-coverage 177 declared / 177 covered / 0 gap / 0 orphan / 0 untagged; validate-docs OK. No commit made.

Follow-ups wired: [Align the frontend docs with the dashboard](23-align-frontend-docs.md) now also blocked by this ticket (it made `docs/diataxis/reference/frontend/dashboard.md` stale about the preview, the "Stop" button and `data-original-html`); the separate dead `action-area-refresh` listener in `assets/index.html` was added to [Delete the dead `htmx:refresh` calls and fix the docs](49-delete-dead-htmx-refresh.md).
