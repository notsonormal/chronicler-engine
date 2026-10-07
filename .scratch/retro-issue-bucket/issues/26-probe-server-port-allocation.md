# Allocate the probe server a free port under concurrency

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

A UI-investigation fix replaced one magic port constant with another: the probe server is pinned to port 3001 and one shared `tmp/probe-server` target dir. Two probe sessions collide, and the dashboard default is 3000. The user guessed "between 3000-3015 … maybe 3020?" and the agent answered "The integration-test band is 3010–3050 … 3001 | outside the band, free — chosen". Later the user had to ask for the prototype on port 3000, which the dashboard already held.

The repo already owns file-locked port allocation for exactly this problem.

How should the probe get a port?

- **Reuse the harness's reservation.** Take a port from the integration-test band via the existing file-locked allocator, not a fixed constant.
- **Per-session ports.** Derive a port from the session id, with a collision retry.
- **Keep a constant**, but move the probe target dir per session.

## Context

- Source: `/reflect` 2026-10-07, divergent reviewer (sessions `01a10dc7`, `01a10e1a`).
- Evidence: `assets/reflect-2026-10-07/synthesis.md` (Backlog "Probe-server port allocation"; the reviewer's D8 row was rejected as already-covered prose, the mechanism remains).
- `.agents/skills/chronicler-ui-investigator/SKILL.md` — the probe-server step ("a concurrent probe needs its own port and its own `--target-dir`").
- The integration harness's file-locked port reservation is the existing mechanism to reuse.
