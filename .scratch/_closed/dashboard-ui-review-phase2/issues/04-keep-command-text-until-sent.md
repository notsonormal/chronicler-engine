# Keep the command text until a send goes through

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

The command input's lifecycle is wrong on two paths, and its failure slot breaks the action area. Clear the input exactly when a command has been consumed.

## Items

From the [re-review](../../dashboard-ui-review/re-review-2026-10-09.md):

- **R9 (P2)** After "Send with edits" the corrected text is sent, but the input keeps the original misspelled text. `onCommandAfterRequest` (`assets/dashboard-action-area.js`) clears the form only when no preview is open, and the confirm path never clears it. Shot `tmp/t24/18-text-check-preview.png`. "Send Original" likely behaves the same [inferred].
- **R21 (P2)** A send that fails at the transport level (engine stopped) clears the input, so the typed command is lost. Keep it on any failure. Shot `33`.
- **R22 (P3)** The form's inline "The engine is unreachable." slot renders inside the fixed 64px action area: the input row moves up over the story log and the slot is cut off at the viewport bottom. It stays after the engine recovers until the next good send. Shots `33`, `34`.

## Done when

- Each item is fixed, with stub-tier browser coverage for R9 and R21.
- `python build.py` is green. Commit after user approval.

## Answer

All three items are fixed in the client (plus one CSS rule and one preview-template
attribute). The rule the ticket asks for is now explicit: the input is cleared only
when a 2xx answer consumed the command.

### R9 — a confirmed preview consumes the command text

- `assets/dashboard-action-area.js`: `onCommandAfterRequest` no longer clears the form
  on any outcome; it clears only when the request succeeded and no preview is open. A
  new `onPreviewSendAfterRequest(event)` clears the input on a successful confirm and
  then closes the preview, so both **Send with edits** and **Send Original** consume the
  text (the latter was only inferred in the review; it shares the handler). Cancel still
  calls `closeActionPreview()` and keeps the text.
- `src/adapters/driving/http/templates.rs`: both confirm forms in
  `TextCheckPreviewTemplate` now call `onPreviewSendAfterRequest(event)` instead of
  `closeActionPreview()`.

### R21 — a send that cannot reach the engine keeps the typed command

- `assets/dashboard-action-area.js`: `requestSucceeded(event)` wraps htmx's
  `event.detail.successful` (false for a transport failure and for a non-2xx answer), and
  `assets/index.html` passes the event into `onCommandAfterRequest(event)`. A failed send
  now leaves the typed command in place for a retry; a 500/503 keeps it too.

### R22 — the inline error no longer overflows the action area, and it clears on recovery

- `assets/styles.css`: `.action-area:has(#command-form [data-error-slot]:not([hidden]))`
  drops the fixed `height` for `height: auto` + `min-height: var(--action-area-height)`
  and the same vertical padding the open preview uses. The area grows and the story log
  shrinks, instead of the form's second row spilling above and below the 64px box (over
  the story log and past the viewport bottom).
- `assets/dashboard-errors.js`: a slot written by `htmx:sendError` is marked
  `data-unreachable`, and `clearUnreachableSlots()` clears every marked slot in the same
  `htmx:afterRequest` branch that already calls `clearUnreachable()`. The stale "The
  engine is unreachable." line now goes when the banner goes, on the first successful
  response after recovery, rather than waiting for the next good send.

### Tests (all tier 2 — stub browser, `tests/browser/stub/`)

`tests/browser/stub/dashboard.rs`:

- `test_confirmed_preview_consumes_the_command_text` (SCENARIO 16.36) — the preview keeps
  the submitted text; "Send with edits" and "Send Original" each leave the input empty;
  Cancel keeps the text.

`tests/browser/stub/failure_display.rs`:

- `test_dead_engine_keeps_the_typed_command` (SCENARIO 16.37).
- `test_action_area_makes_room_for_the_inline_error` (SCENARIO 16.38) — the area grows
  past its resting height, the command row sits below the story log, the slot's bottom is
  inside the viewport, and after the stub rebinds the slot and the banner are gone while
  the typed command survives.

Spec scenarios 16.36–16.38 added to `docs/specs/browser_dashboard.md`; `dashboard.md`
updated in two places (the Unreachable bullet and a new "Command-input lifecycle"
paragraph). `tests/browser/stub/support.rs` gained `read_command_input`;
`tests/test_utils/stub_server.rs` gained `StubLifecycleHandle` (`stop`/`restart`, and
`StubServer::stop` now delegates to it) plus the shared `serve_stub` helper, so the
recovery half of 16.38 runs on one page. The handle exists because `with_stub_page`'s
closure cannot capture the `&StubServer` across its returned future.

Sensitivity was checked by reverting each fix alone and re-running its test: the
template revert fails 16.36 ("Send with edits must consume the command text"), the JS
gate revert fails 16.37 ("a send that cannot reach the engine must keep the typed
command"), the CSS revert fails 16.38 ("the action area must grow ... got 64 against a
resting 64") and the `dashboard-errors.js` revert fails 16.38 ("the engine's recovery
must clear the inline error").

### Full gate

`python build.py` green in the worktree: `logs/build_20261010_000219.log` — fmt, clippy,
guardrails (162), spec-coverage, docs, architecture (1), integration (1595 passed, 0
failed, 2 skipped), browser (72 passed, 0 failed).

### Follow-ups (non-blocking)

- A failed `/action/confirm` (transport failure or 5xx) closes the preview, and the
  inline error `inlineErrorSlotFor` creates lives inside the preview's own `<form>`, so
  `closeActionPreview()` wipes it: only the banner reports that failure. Not one of this
  ticket's items; filed in ticket 06.
- A 200 `ConcurrentGeneration` answer ("Still thinking...") still clears the input,
  although that command was refused rather than consumed. The status display's `wait`
  state is the only signal for it, and ticket 02 owns that markup, so this ticket leaves
  the rule as "2xx with no preview open". Filed in ticket 06.
