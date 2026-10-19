# Decide how to respond to the review of the whole rebuild

Type: grilling (HITL)
Status: open
Blocked by: 17

## Question

The review of the whole rebuild (2026-10-19, range `bc7d6cd4..8036bbc3`) found no plan item missing. But it found defects a player can hit, and the thermo-nuclear reviewer did not approve the code shape of `job.rs` and `gate.rs`. Answer the findings as one set, not one by one. What is the response, and does it come before the merge to `main`?

- **Option A: a bug-fix pass, then merge.** Fix the defects a player can hit (findings 1, 2, 10, 11, and the small medium items), and commit the uncommitted fixes of the review. Merge `wf/rebuild-core` to `main`, which reaches the Destination. Move the shape findings (3 to 6) to an effort after the build.
- **Option B: a review of the resulting architecture first, then a refactor slice, then merge.** Treat findings 3 to 6 as one design question: the data of a generation kind in its type, the event rules in `game_rules`, one room lookup, `Job::run` as one loop, and request planning moved out of `gate.rs`. Then fix the defects in the new shape, where some of them go away. Finding 1 is a runtime branch that a data-carrying kind removes. This needs its own design pass (`/improve-codebase-architecture` or a grilling on the shape), and it may need new tickets on this map.
- **Option C: merge now, and do all of it after the build.** The rebuild is green and matches the plan. Every finding goes to a new effort.

The answer also decides these questions, which hang on the same choice:

1. The fixes of the review are committed on `wf/rebuild-core` as `2885df2c` (owner, 2026-10-19). Do they stay there, and is that branch the one to merge?
2. `wf/rebuild-review` (`fa1a25e1`) lacks the last three commits of `wf/rebuild-core`. Which branch is the one to merge?
3. Whether to delete the `InMemory` storage backend (thermo H8).
4. Whether `testing` stays a default Cargo feature (findings 8 and 9: `tx_token()` reachable from `src/`, `JobTasks` in production, the Contradiction check only under `diagnostics`).
5. Whether the shape findings sit inside this map's Destination ("target design version 2 runs") or past it (plan §10).

## Context

- The architecture reviews of the rebuilt code on three models (tickets 14, 15 and 16) and their synthesis, [Synthesize the three architecture reviews of the rebuilt code](17-synthesize-the-architecture-reviews.md), give this ticket its view of the code shape. Option B starts from the synthesis.

- The findings, de-duplicated and verified: [FINDINGS.md](../assets/whole-rebuild-review/FINDINGS.md). There are 29 items: 6 high, 14 medium and 9 low, plus what the review already fixed.
- The run record, with its steps, gates and decisions: [tracker.md](../assets/whole-rebuild-review/tracker.md). Its `tmp/` paths are git-ignored scratch.
- One file for each source: [findings/](../assets/whole-rebuild-review/findings/). The full reviewer texts are in `full-review-*.md`; thermo-nuclear has the refactor proposal ("What would make this approvable").
- High findings that the lead verified in code:
  - A bare `/impersonate` answers `500` "missing: the impersonate direction" (`gate.rs::require_job_data`). Design §4.5 asks for the direction only on a Retry of an Impersonate Input.
  - `impl From<EngineError> for Failure` (`job.rs`) puts the raw error text into the short failure line. When a read inside `JobWriter` fails, the player sees "The new text could not be saved."
- Gates: the full gate is green on the tree of `2885df2c` (`logs/build_20261019_223126.log`). Test-police ran it 6 times with no flake, and coverage is 90.6 %.
- Plan: [the build plan](../../../docs/plans/rust-rebuild-build-plan.md) §9.7 (the merge) and §10 (past the build). Design: `.scratch/turn-state-model/assets/target-design.md`.

## Answer
