# Give agents one call that waits for a running build

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Agents that start a long build in the background poll it with `sleep N; tail`. In this run test-police slept 1085 s over eight calls and overshot the end of its build by about 4 minutes. Should `build.py` offer a wait?

- **`python build.py --wait`**: block until the newest build ends, then print its epilogue.
- **Tell agents to run builds in the foreground** with the 1200 s bash timeout, and never background them.

## Context

- Source: `/retro` 2026-10-19 on the whole-rebuild review: lead session `01a15614` and its subagents.
- Test-police `d448fab2` entries 147, 159, 258, 264, 274, 276, 278.
