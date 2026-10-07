# Make the `tdd` skill fire on test-strengthening work

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

`tdd/SKILL.md:1` triggers only on "wants to build features or fix bugs test-first, mentions 'red-green-refactor', or wants integration tests". Sessions that add, strengthen, or de-tautologize tests never open it, so the mutation proof survives only when a lead hand-writes it into a ticket. Ticket 34 needed the instruction verbatim; a later hardening session re-improvised it. Both touched tests without opening the skill.

The 2026-10-04 reflect accepted this description tune and it never landed.

Should the description gain test-quality triggers?

- **Yes.** Add triggers for adding, strengthening, or de-tautologizing tests.
- **Yes, narrowly.** Add only "strengthen a test" and "de-tautologize".
- **No.** `code-review` and `test-police` already cover test quality.

## Context

- Source: `/reflect` 2026-10-07, divergent reviewer.
- Evidence: `assets/reflect-2026-10-07/synthesis.md` (parent amendment 2).
- `.agents/skills/tdd/SKILL.md:1` — the current description.
- Prior art: `tmp/reflect/final_01a10894-87dd.md` (2026-10-04 Accepted, unlanded) and `tmp/reflect/final_01a1089d-5ec4.md`.
- Related: [Make the mutation check unconditional for state-repair tests and fixtures](23-unconditional-mutation-check.md).
