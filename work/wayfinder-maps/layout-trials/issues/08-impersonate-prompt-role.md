# Try the role of the Impersonate prompt

Type: task (HITL)
Status: closed
Blocked by: 05, 12

## Question

Do the Impersonate Inputs get better when the impersonation instruction comes last in the call, as a `system` message (as SillyTavern does) or as a `user` message (as Marinara Engine does)?

## Context

- Moved from Prompt flexibility on 2026-11-01. There it was ticket 11, "Decide whether to run ST and ME live". The live run of ST or ME is dropped (owner, charting of this map). This trial makes the same check on this engine, which was the third option of the old ticket.
- Target part: the Inputs that Impersonate writes for the Persona.
- Known: ST sends the impersonation prompt as a trailing `system` message (`public/scripts/openai.js:1382` at the pinned commit `06bde939fb1e9c4c8d8641d810f0a916b5bce127`). ME sends it as a trailing `user` message (`packages/server/src/routes/generate.routes.ts:7495` at the pinned commit `457e827a3d3291be978121f8c50c98c5dd9d644d`). See [How ST and ME let an author change a call](../../prompt-flexibility/issues/02-st-me-prompt-configuration.md) and its [asset](../../prompt-flexibility/assets/st-me-prompt-configuration.md).
- Known: our Impersonate call sends one system message and one user message. The `role` and `instructions` of the impersonate preset are in the system message (Layer 0). Its `writing_style` and `output_format` are in the post-history splice, near the end of the user message. `<PlayerInput>` renders only when the owner gives a direction (`docs/diataxis/reference/narrative/prompt_system.md`).
- Known (plan review, 2026-11-01): the model port sends two messages only (see [Try the history as chat messages](07-history-as-chat-messages.md)). So the `system` side needs the same message-list change in the worktree, and the same `scripts/trial.py` change on `layout-trials`. The `user` side can append the text to the end of the user message with no port change. Two `user` messages in a row usually reach the model as one turn (inferred).
- Known: `scripts/trial.py` labels a call as Impersonate when its system prompt holds "player character" (the `<role>` text of the impersonate preset). Keep the `<role>` in the system message, or the pack labels the call as a Narration, with no error.
- So the trial has up to two sides against the baseline: the instruction as a trailing `system` message, and as a trailing `user` message. Which text moves (the preset `instructions`, or a new closing sentence) is a trial choice. The answer states it and its reason.
- Models treat the `system` role differently, so [Decide how to read an output trial](05-read-an-output-trial.md) decides whether this candidate also runs on a second model.
- How: a linked worktree from `layout-trials` with a detached HEAD, as in [Try the changing blocks after the history](03-changing-blocks-after-history.md). Run the trials that ticket 05 sets, then read the comparison with the owner, with its method.

Done when the owner keeps or drops each side, the reading notes extend the criteria asset, and the worktree is removed. A kept side gets a new task ticket (map Notes).

## Out of scope (owner, 2026-11-02)

No measured fault is behind this candidate. The long Inputs and the scripting of other Characters that the judge pilot found are mainly drift and role breaches (guessed); [Sort the inconsistencies by their cause](13-sort-inconsistencies-by-cause.md) and [Stop the role breaches with stop sequences](14-stop-role-breaches.md) look at them. Reopen when a measured fault points at the place of the Impersonate instruction.
