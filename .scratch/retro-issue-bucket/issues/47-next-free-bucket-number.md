# Read the next free bucket number across every checkout

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

The bucket's own rule is "A new ticket takes the next free number" (map Notes → Numbering). In this run the next free number lived in another checkout: the worktree's `.scratch/retro-issue-bucket/issues/` ends at 36, while the main checkout holds 37, 38 and 39 as untracked files. The retro was corrected twice inside five minutes — "issues 37 and 38 already exist … new issue numbers start at 39", then "a third issue already exists … start NEW issue numbers at 40" — and without that the pass would have created a second ticket numbered 37 and a second numbered 39, which the map's numbering rule forbids outright.

How should the next free number be found?

- **Look across every checkout before numbering.** The map Notes name the command (`git log --all --diff-filter=A --name-only -- .scratch/retro-issue-bucket/issues/`, plus untracked files in sibling worktrees) or a small script prints the highest number in use.
- **File every bucket ticket in one checkout.** The bucket lives in the main checkout; a worktree writes there or returns the drafted file to the coordinator. This run shows the cost of the split both ways: 37–39 are invisible from the worktree, and the tickets this pass files (40–47) are invisible from the main checkout until the branches meet.
- **Claim the number first.** The filer writes the next number into the map Notes before creating the file, so a second filer sees it (the wayfinder claim move). Serializes filers, which parallel runs do not.

## Context

- Source: `/retro` 2026-10-11 on the phase2-after-plan pass; both corrections came from the Manager to the Lead and to this retro (`message_supervisor` / `steer_agent`).
- Main checkout (`/workspace/chronicler-engine`): `git status --porcelain -- .scratch/retro-issue-bucket/` → `?? …37-comment-finder-javascript-coverage.md`, `?? …38-audit-bloated-comments.md`, `?? …39-rethink-the-development-loop-section.md`; its map's Open-tickets list still ends at 36, so the list is not the source of truth either.
- The bucket map is branch-local (`.scratch` is tracked); the numbers this pass files are only on `phase2-after-plan`.
- Related: [Show the next free scenario number for a spec](36-next-free-scenario-number.md) — the same class of gap for spec scenarios, where a retired number is invisible for the same reason.
