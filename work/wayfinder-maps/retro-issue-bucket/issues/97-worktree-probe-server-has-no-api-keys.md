# A probe server in a worktree has no API keys

Type: grilling
Status: resolved
Blocked by: —

## Question

A probe server started from a linked worktree finds no `.env`, so a Connection with no stored key fails. The permission rule `*.env` stops an agent from reading or copying the main checkout's `.env`. How must a worktree probe server get the API keys of the main checkout?

## Context

Source: the coordinator session of 2026-11-01 for prompt-flexibility tickets 20 and 21. Implementer sessions `b87fc4b3` (ticket 20) and `b8898777` (ticket 21).

- Both implementers worked in worktrees under `/workspace/ce-wt/`. Each ran into `Denied by policy: 'path' ... '.env' (rule '*.env')`.
- Both then started the worktree binary with the main checkout as its working directory, so the server loaded `/workspace/chronicler-engine/.env` itself. Ticket 21 used a scratch folder under `tmp/` in the main checkout. No secret was printed, but the work goes around the intent of the deny rule.
- The coordinator brief caused part of this: it said "the main checkout's .env may hold them".
- `ENVIRONMENT.md` and `.agents/skills/chronicler-ui-investigator/SKILL.md` say how to start a probe server, but not where its keys come from in a worktree.


New evidence, 2026-11-01 (ticket 22, implementer session `bc5215d2`):

- The implementer hit the same deny rule and used the same route: it started the worktree binary from a folder under the main checkout, so the server loaded the main `.env`.
- The coordinator brief pointed it there: "Where API keys come from: see the Answers of tickets 18 to 21". Those Answers describe the cwd route. So the route now spreads through ticket Answers.

## Options to decide

- **A.** `build.py run` takes the main checkout's `.env` when the worktree has none (the server reads it, the agent does not).
- **B.** A git-ignored `.env` symlink in each new worktree, made by the worktree script.
- **C.** Document the cwd route in `ENVIRONMENT.md` as the approved way.

## Answer

Decision (user, 2026-11-01): **A**. `build.py run` links the main checkout's `.env` into a linked worktree that has none. The server reads the keys; the agent never reads the file.

- `_link_main_env` in `build.py` runs in `run_server` before the build. It finds the main checkout with `target_seed.worktree_roots` (first entry). It keeps an existing `.env`, does nothing in the main checkout, and prints a line when the main checkout has no `.env`. `.gitignore` (`.env*`) covers the link.
- Tests: `LinkMainEnvTests` in `scripts/tests/test_build_cli.py`. A manual check in a real linked worktree made the link and `git status` stayed clean.
- `ENVIRONMENT.md` says where the server gets its keys in a worktree, and why it must start from the worktree: a server started from the main checkout reads the main `data/` (`resolve_engine_data_path` falls back to the cwd) and writes the main `logs/`.
- B was rejected: a `post-checkout` hook does not fix the worktrees that exist now, runs on every checkout, and fails silently. C was rejected: the cwd route tests the wrong `data/`.
- The Answers of prompt-flexibility tickets 20, 21 and 22 described the cwd route. Each now has a note that points to the link. Tickets 18 and 19 do not name the route.
- Review fixes: a broken link is made again, a failed link prints a line and the run goes on, and the docstrings and `ENVIRONMENT.md` name the parent-folder search and the keys a Connection does not store.
