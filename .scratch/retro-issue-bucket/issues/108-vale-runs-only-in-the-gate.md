# A doc sentence that fails Vale reaches a commit and blocks every gate after it

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Vale runs only in the full gate and in `build.py vale`. A commit can hold a sentence that fails Vale, and then the full gate of every agent fails on it. Must Vale run before a commit, or earlier in the loop?

## Context

Source: `/retro` 2026-11-08, the coordinator session of statechart-models tickets 08, 10, 11, 12, 13 and 16 (`01a1bd22`), and its subagent sessions (t08 `3d5d0267`, t10 `9340bd47`, t12 `549a889a`).

- **known** Commit `25db6ee5` (an owner edit of `AGENTS.md`) held a sentence of more than 25 words. Every full gate after it failed at the Vale step, which comes before clippy and the tests.
- **known** t08 ran `sleep 240; python build.py vale` and waited 4 minutes for another agent to fix the line. t10 hit the same failure 3 times.
- **known** t12's first full gate failed on Vale because a new scenario heading had more than 25 words. `build.py vale` costs under 1 s, and the full gate costs minutes.
- **known** The pre-commit hook regenerates the indexes (`scripts/precommit_regenerate.py`), but it does not run Vale.
- **known** Again on 2026-11-09: commit `4e3522bb` (an owner edit of `AGENTS.md`, the delegation rule) held a sentence of 39 words. The full gate of statechart-models ticket 14 failed at the Vale step. I split the sentence into a list and ran the gate again (6 more minutes).

## Options to decide

- **A.** The pre-commit hook runs Vale on the staged `.md` files and refuses the commit on an error.
- **B.** The Development Loop table says: after a docs or spec edit, run `build.py vale` before the full gate.
- **C.** Both.
