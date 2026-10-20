# Check the docs again after the code-changing steps of the after-plan workflow

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

In `/chronicler-after-plan-workflow-plus-review`, step 4 (docs) runs before steps 6, 9 and 10, which change code. In this run the step-6 fix pass removed the `CancellationToken` of a job entry. The lead's brief told it not to touch `docs/`. Four places then described a cancel handle that did not exist (`game_flow.md`, `rust_idioms.md`, `CONTEXT.md`, ticket 12). Two of the step-12 reviewers found it, not the workflow. How does the workflow keep docs true after its own code changes?

- **Move the docs step after the last code-changing step** (after step 10, before the gate).
- **Keep step 4, and add a short re-check** after step 10: search the docs for every name that the fix passes removed or renamed.
- **Make each fix brief carry its doc follow-up**: a brief that removes or renames a part lists the docs that name it.

## Context

- Source: `/retro` 2026-10-19 on the whole-rebuild review: lead session `01a15614` and its subagents.
- Lead session `01a15614`: the brief of `fix-code` says "do not touch docs/ (another pass owns docs)"; review-code finding Standards 2 and thermo M9 report the stale cancel-handle text.
- `.agents/skills/chronicler-after-plan-workflow-plus-review/SKILL.md` steps 4, 6, 9, 10, 12.
