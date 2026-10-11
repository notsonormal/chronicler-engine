# Decide whether the comment pass renames or gates instead of delete-and-flag

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

The KEEP test says "If the reason is real but the code could carry it, delete and raise a reshape flag" (`.agents/skills/chronicler-comment-fixer/SKILL.md`, "# The KEEP test"), and "# Reshape flags" adds "Reshape flags are proposals. An approved flag leaves this skill and runs as normal gated production work." So the pass deletes the comment and walks away. In the phase-2 pass that deleted the only text naming a real constraint in two places; the next pass — the Standards review — re-found both by hand, which cost a review finding and a fix round for each. The corrected forms were a rename and a visibility gate, both one-line edits inside files the pass was already editing.

Where does a constraint that the code could carry belong?

- **The pass applies it.** When the fix is a rename or a visibility/cfg gate in the file being judged, the pass makes it and reports a rewrite, not a flag. Flags stay for changes that cross files or need a design call.
- **The flag stays, its disposal is mandatory.** The pass reports every flag, and the caller of the pass must dispose of each one (apply or record why not) before the pass counts as done; the brief carries the list.
- **Leave it.** The code review is the net that catches a lost constraint, and it did. Accept the extra review finding and fix pass per loss.

## Context

- Source: `/retro` 2026-10-11 on the phase2-after-plan pass; comment-fixer session `01a12c59-8ad9-718a-a951-63c7789417e4`, delivered to Lead `11b62cb1` (idx 35).
- The pass classified 606 comment lines; the finder fell 606 → 507 (52 removed / 34 added as rewrites, 81 fully deleted lines). It offered 6 flags, including `src/adapters/driving/http/utils/error.rs:refusal_message` — "so the deleted 'without its layer prefix' doc is carried by the code" — and `src/application/pipeline/action_pipeline/retry.rs:retry_last_response` (`click_time` → `state_at_click`).
- The flags went unapplied. The Standards axis then found the losses itself ("`refusal_message` (the comment pass deleted the only text naming its layer-prefix strip)"; "restoring the `test-only` doc on `message.rs:158`"), and the fix pass applied them: `refusal_message_without_layer_prefix_for_validation_and_config` (63 characters — the truthful name, and the length shows "let the name carry it" is not always free), `state_at_click`, and an assertion replacing the dead `let _ = new_narrations;` in `tests/http/actions.rs`.
- The `Message::set_event_header` "test-only" doc (`message.rs:158`) was deleted with **no** flag at all; it is now `#[cfg(test)] pub(crate)`, which the compiler enforces.
- Map Notes: this ticket decides a skill rule; the fix is a one-file skill edit if option 1 or 2 is chosen.
- Related: [Scope the comment pass to the lines the diff adds](12-comment-pass-scope-to-the-diff.md), [Audit the bloated comments in the dense files](38-audit-bloated-comments.md).
