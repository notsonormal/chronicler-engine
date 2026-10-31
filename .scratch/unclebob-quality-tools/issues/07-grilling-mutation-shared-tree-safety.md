# Grilling: how does a mutation run keep its result and its tree in step?

Type: grilling (HITL)
Status: open
Blocked by: 03

## Question

What rule makes a mutation result trustworthy in a checkout that other agents build and edit at the same time, and what does the tool do when the run dies?

## Context

Evaluation: [Research: which mutation engine works on this codebase, and what it shows](./03-research-mutation-engines.md), asset [03-mutation-engines.md](../assets/03-mutation-engines.md). The two failures of 2026-10-22 (branch `autoplay-engine`) are in ticket 03's Context: a stale green result (session `4e7d7150`) and code rewritten from memory (session `ccfcc177`).

Measured in 03:
- `cargo-mutants --in-place` writes no git commit or source hash into its output. A `SIGKILL` leaves the mutated file in the tree. Bare cargo changes the linker fingerprint, and one full rebuild (251 s) followed.
- The throwaway driver keeps `<file>.orig` and `<file>.pending`, restores with `--recover`, records git HEAD and a `git status` hash before and after each mutant, and holds the build slot.
- A linked worktree costs one target seed (344 s) and keeps mutants out of the shared tree. `scripts/remove_worktrees.py` spares a worktree younger than 24 h.

## Decisions wanted

1. Where a run happens: a linked worktree for each run, or in place in the main checkout?
2. Tree proof: must every result print git HEAD and a hash of `git status --porcelain`?
3. Crash recovery: is restore-on-recover required, and is `git stash` still ruled out?
4. Does this rule apply to hand-run mutation checks too (the 2026-10-22 failures were hand runs)? This relates to retro-issue-bucket [23 — unconditional mutation check](../../retro-issue-bucket/issues/23-unconditional-mutation-check.md).

## Out of scope

Whether mutation testing has value: see [Grilling: does mutation testing have value here, with which engine?](./06-grilling-mutation-testing-value.md).
