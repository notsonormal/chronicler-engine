# Give `/simple-after-plan` a slice mode for multi-slice builds

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Plan §2 of the response build asks for `/simple-after-plan` on every commit. That is a comment-fixer subagent, a document-review subagent, `/code-review` with two more subagents, the full gate, and a retro on every slice. The owner stopped the lead after the first slice and asked for a lighter gate. What should the slice mode of the skill say?

**Owner's choice (2026-10-21): option A.** Add a "slice" branch to `.agents/skills/simple-after-plan/SKILL.md`. The open points of the grilling are its wording and its trigger:

- The implementer runs `python build.py --no-browser` once before it reports, and cleans its own comments.
- One `reviewer` checks spec, standards and comments/docs in one pass, from one review bundle (`prepare_review_bundle.py --uncommitted`).
- The fixes go back to the implementer's resumed session.
- The lead runs the full gate once, then commits.
- The retro runs once per effort, not once per slice.
- Open: which plans take the slice branch (a plan with several slices under one standing approval?), and which plan text (build plan §2 template) points at it.

Rejected: B (only add evidence to ticket 72) and C (change only the build-plan template).

## Context

- Source: lead session `01a15b99` (2026-10-20 to 2026-10-21), build of R1 to R9 of `docs/plans/rust-rebuild-response-build-plan.md`.
- The owner's message after R1 started: "You don't nesscarily need to go through the full /simple-after-plan workflow after every single task, or constantly do full builds if nothing has changed".
- Result of the slice gate: 8 combined reviews (2–5 min, about $0.9–1.8 each); every review found real items; every full gate after the fix round was green on its first run (9 commits and the merge).
- Related: [Give `/code-review` a single-session mode](72-code-review-single-session-mode.md). The combined reviewer of this run is that mode.
