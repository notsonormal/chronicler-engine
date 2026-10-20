# Make the comment pass list every deleted reason comment

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

In the core step, the comment pass deleted reason comments that the code does not carry, and the lead restored them by hand: four in C4 and four in C5. The doc of `user_message` in `src/application/pipeline/job.rs` (the four sources of the user message a Narration follows) was deleted twice. In C6 the brief asked the pass to report "each deleted WHY-looking comment with its reason", and the lead could check all of them from the report. Nothing had to be restored.

Where should that report line live?

- **In the skill.** `.agents/skills/chronicler-comment-fixer/SKILL.md` adds to its report a table of each deleted comment that states a reason, with the reason it fails the KEEP test.
- **In the caller's brief only.** The after-plan skill's brief asks for the table.
- **Leave it.** The lead reads the diff.

Related: [44](44-comment-pass-rename-or-gate.md) is about reasons the code *could* carry. This ticket is about reasons the code does not carry.

## Context

- Source: `/retro` 2026-10-19 on the core step of the Rust rebuild (ticket 07, the C4 and C5 comment passes; the C6 comment pass `c6-comments`).
- Evidence: the C5 comment pass deleted the failed-event naming reason, the event-generation reason, the Options toggle reason and `user_message`'s doc in `job.rs`; the lead restored them before the C5 commit `8a8ba6bb`.
