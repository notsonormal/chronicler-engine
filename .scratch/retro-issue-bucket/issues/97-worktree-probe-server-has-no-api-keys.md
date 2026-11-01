# A probe server in a worktree has no API keys

Type: grilling
Status: open
Blocked by: —

## Question

A probe server started from a linked worktree finds no `.env`, so a Connection with no stored key fails. The permission rule `*.env` stops an agent from reading or copying the main checkout's `.env`. How must a worktree probe server get the API keys of the main checkout?

## Context

Source: the coordinator session of 2026-11-01 for prompt-flexibility tickets 20 and 21. Implementer sessions `b87fc4b3` (ticket 20) and `b8898777` (ticket 21).

- Both implementers worked in worktrees under `/workspace/ce-wt/`. Each ran into `Denied by policy: 'path' ... '.env' (rule '*.env')`.
- Both then started the worktree binary with the main checkout as its working directory, so the server loaded `/workspace/chronicler-engine/.env` itself. Ticket 21 used a scratch folder under `tmp/` in the main checkout. No secret was printed, but the work goes around the intent of the deny rule.
- The coordinator brief caused part of this: it said "the main checkout's .env may hold them".
- `ENVIRONMENT.md` and `.agents/skills/chronicler-ui-investigator/SKILL.md` say how to start a probe server, but not where its keys come from in a worktree.

## Options to decide

- **A.** `build.py run` takes the main checkout's `.env` when the worktree has none (the server reads it, the agent does not).
- **B.** A git-ignored `.env` symlink in each new worktree, made by the worktree script.
- **C.** Document the cwd route in `ENVIRONMENT.md` as the approved way.
