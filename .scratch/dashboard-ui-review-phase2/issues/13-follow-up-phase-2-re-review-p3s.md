# Follow up on small issues found in the phase-2 re-review

Type: task (AFK)
Status: open
Blocked by: —

## Question

Fix each P3 from the [2026-10-11 re-review](../re-review-2026-10-11.md), or drop it with a reason.

## Items

- **S5** The command form's inline-error "Details" toggle is drawn as a second Send button (150x40), and its message is 16px. The `#command-form button` rule in `assets/styles.css` matches every button in the form; scope it to `#submit-btn`. Shot `tmp/p2-07/08-unreachable.png`.
- **S6** The location header lands one Message late: movement sets `pending_location`, and the next Message takes it (`src/domain/model/state/game_state.rs:123`, `:190`). This is a narrative-pipeline rule, not dashboard code. Decide whether the arriving narration should carry the header, or rule it out of scope with an owner.
- **S7** `truncated_preview` (`src/adapters/driving/http/builders/presets.rs`) appends "…" after a full stop ("intelligence.…"). Drop the trailing punctuation before the ellipsis.
- **S8** The text-check preview draws the Original with a line-through while its note says nothing changes unless the player chooses "Send with edits". Decide whether the line-through stays.

## Done when

- Every item is fixed or dropped with a reason.
- `python build.py` is green. Commit after user approval.
