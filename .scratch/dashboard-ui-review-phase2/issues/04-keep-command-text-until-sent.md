# Keep the command text until a send goes through

Type: task (AFK)
Status: open
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
