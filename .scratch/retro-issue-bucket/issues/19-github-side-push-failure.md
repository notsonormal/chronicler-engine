# Add a GitHub-side push-failure path

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

A `git push` failed with `remote: Internal Server Error`. The user said "Try again"; the agent retried seven times with the same GitHub-side error before stopping. `git push --dry-run` succeeded, proving auth, negotiation and the local pack were fine. The same repo pushed cleanly 53 minutes later with no local change. Later the user pre-warned "There might be some issues with pushing so only try and push once".

`commit-and-push/SKILL.md:140` covers only "remote has diverged (push rejected) → `git pull`", which sends the agent down the wrong path.

What should the failure path be?

- **Localize-then-stop.** `git push --dry-run` to localize; stop after ~2 attempts on a GitHub-side error; record the `Request ID` lines; never force-push; re-push from a later session.
- **Backoff instead of stop.** Retry with a delay, then stop.
- **Both, plus the "one push per session" rule when the user flags push trouble.**

## Context

- Source: `/reflect` 2026-10-07, session `01a11748`.
- Evidence: `assets/reflect-2026-10-07/synthesis.md` (Accepted row 12).
- `.agents/skills/commit-and-push/SKILL.md:140` — the only push-failure branch.
- Observed request id: `E212:14F2FE:397F6:A1AF3:6AC679DB`; the same refs pushed cleanly later (`5fa46615..d77eca37`).
- Related: [Stop `commit-and-push` from staging the whole shared tree](13-commit-staging-shared-tree.md).
