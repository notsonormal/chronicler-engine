# Check a delegated brief against the role's tool grants before dispatch

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

A wayfinder research brief asked for `git log --follow` and `git show` archaeology, but the `scout` role exposes only `read`/`ls`/`find`/`grep`. The scout spent its session on workarounds and then reported "this session exposes only `read`/`ls`/`find`/`grep`/`ask_owner` — no shell … Q4 is not recoverable with the tools I have". Two other sessions hit the same wall ("I have no bash/git tool in this role", "this role has no shell tool").

How should a brief be checked before dispatch?

- **Verify each evidence source against the role's grants.** The dispatcher lists the sources the answer needs and confirms the role can reach them; unanswerable questions are re-scoped or the role gains shell.
- **State the role's tools in the brief.** Every delegated brief opens with the tool set the role actually has, so the brief is written to fit.
- **Both.**

## Context

- Source: `/reflect` 2026-10-07, sessions `01a11247`, `01a10d6b`, `01a10dc7`.
- Evidence: `assets/reflect-2026-10-07/synthesis.md` (Accepted row 3).
- Agent definitions live in `.pi/agents/` and the Herdsman definition roster; `scout` is read-only, `researcher` has web tools, `implementer`/`generalist` have `bash`.
- A re-scope is sometimes the right answer: the question may be answerable by reading files, not by `git log`.
