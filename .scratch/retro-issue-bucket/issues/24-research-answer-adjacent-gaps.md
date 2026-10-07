# Record adjacent gaps in a research ticket's answer

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

A research ticket was answered against an enumerated bug list and declared "the gap closed" — "none of the stuck-state bugs A19 names lacks a test today — the gap closed after A19 was written". The same recon had found a class-level hole: "No test anywhere drives a real Enter key that submits the command form — every command submission is a JS `requestSubmit()` or `button.click()`". That finding landed in no `## Answer` and no `Not yet specified` line, under a standing policy that "Everything should be automatically tested … if there is a gap in automatic tests then this has to be fixed."

Should a research answer carry adjacent gaps?

- **Yes.** The `## Answer` records gaps the recon found but did not cover, as a new ticket or a `Not yet specified` line.
- **File them immediately.** The resolver files the adjacent gap as a ticket rather than recording it in the answer.
- **Neither.** Keep the answer scoped to the question asked.

## Context

- Source: `/reflect` 2026-10-07, sessions `01a1124a`, `01a10d50`, `01a10d6b`.
- Evidence: `assets/reflect-2026-10-07/synthesis.md` (Accepted row 17).
- `.agents/skills/wayfinder/SKILL.md:77` — the Research ticket type.
- Sibling effort: `.scratch/dashboard-ui-review/issues/` holds the A19 work and its `_resolved/` answers.
