# Parallel ticket runs without commits

**Date:** 2026-10-22
**Status:** Draft — open questions (see "Open questions"). Do not implement until each question has an answer.
**Source:** the retro of the phase-2 parallel run (tickets 02–06 of `.scratch/dashboard-ui-review-phase2/`), and the brainstorm that followed it.

## Summary

Make a parallel run of tickets a repeatable process. Implementers work in their own worktrees. The coordinator merges each worktree into the main checkout, where the changes stay uncommitted. Commits happen only after the owner reviews the merged tree and approves the commit.

The candidate changes are a merge script, a rewritten orchestration skill, a duplicate scenario check, a worktree cleanup path and an empty-diff exit for the review bundle. Several of them depend on open questions. Each change names the questions it depends on.

## Why

In the phase-2 run, the coordinator improvised the process. These problems came from that:

- **Patch merge failed.** The coordinator moved work with `git diff` patches. `git apply --3way` refused files that the main checkout had already changed ("does not match index"). Plain `git apply` wrote `.rej` files, and 9 hunks in docs and specs were merged by hand.
- **Scenario numbers collided.** Two agents appended to `browser_dashboard.md` and both took 16.36. Two agents appended to `browser_story_log.md` and both took 30.20. A third agent took 16.39, which the coordinator had just given to another ticket. The guardrail did not catch any of these: `parse_spec_scenarios` in `scripts/validate_feature_spec.py` returns a `set`, so a repeated heading disappears without an error.
- **Follow-ups got lost.** The map Note says that follow-ups go under the follow-up ticket "while it is open". In a parallel run, that ticket lives in another worktree. Ticket 05 wrote "belongs in ticket 06" only in its own answer. The coordinator's brief deferred an item to ticket 02, which had a fixed scope. The Spec review found both gaps.
- **Each implementer ran the full gate.** Five full gates (about 5 minutes each with the browser tier) queued on the build slot, and then the merged tree needed its own full gate anyway.
- **The existing skill does not fit.** `implement-spec` commits on an integration branch, uses a merger agent and opens a draft PR. This repo commits only after owner approval (`AGENTS.md`, "Permissions System").

## Decided in conversation

- The integration-branch and draft-PR flow of `implement-spec` is rejected.
- The merged changes sit uncommitted in the main checkout until the owner reviews them. A commit needs the owner's approval.

Everything else in this plan is a proposal.

## Related work

- **`/simple-after-plan`** owns the review pass after implementation. Its full mode runs the comment pass, the doc review, `/code-review`, the gate and the retro. Its slice mode covers a *sequential* multi-slice build: one implementer per slice, `--no-browser` before the report, one reviewer, then one commit per slice. A parallel run is the other case: several tickets at once, merged into one tree. See Q1 and Q4.
- **Worktree scripts.** `scripts/remove_worktrees.py` (run by the full gate) removes only clean, pushed worktrees. `scripts/target_seed.py` seeds a new worktree's cold `target/` dir. `scripts/build_slot.py` keeps one heavy build at a time. No script merges a worktree.
- **Retro bucket tickets** (`.scratch/retro-issue-bucket/issues/`):
  - 17 *Map Notes coordinator conventions* asks which conventions belong once, before fan-out. It also records an earlier commit policy (Q2).
  - 02 *Quarantine pin parallel conflicts*: `REQUIRES_MIGRATION_TEST_COUNT` is a shared counter, like the scenario numbers (Q6).
  - 03 *Run the full gate once* (resolved) set the rule "run the full gate once, as the last step" (Q4).
  - 64 *Show a spec scenario by number* (with 36's `--next`) edits `validate_feature_spec.py`, like item 3 (Q6).
  - 32 *Review bundle path scope* and 59 *Review bundle ref plus uncommitted* edit `prepare_review_bundle.py`, like item 6 (Q8).

## Open questions

Each question lists the options, a recommendation, and the items that change with the answer.

### Q1. Where does the process live?

| Option | Effect |
|---|---|
| A. Rewrite `implement-spec` for this repo | One skill owns a parallel run. Its input becomes "a wayfinder map with AFK tickets". |
| B. A parallel mode in `/simple-after-plan`, next to slice mode | One skill covers sequential and parallel builds. But that skill is a post-implementation pass, and a parallel run is mostly orchestration. |
| C. A reference file in wayfinder (`wayfinder/PARALLEL.md`) | Lives where the maps live. Wayfinder grows a second job. |
| D. A new skill, and delete `implement-spec` | A clean name. One more skill for the owner to remember. |

Recommendation: A. The useful parts of `implement-spec` (worktree per ticket, coordinator owns the map, one fix agent after review) stay, and it hands review to `/simple-after-plan`. Affects: item 2, item 4.

### Q2. Do implementers commit on their worktree branches?

Retro ticket 17 records an earlier policy: "You can commit onto worktrees, they will be deleted afterwards. But the changes should sit uncommitted in [the main checkout] after everything is done."

| Option | Effect |
|---|---|
| A. No commits anywhere | Item 1 merges each file itself with `git merge-file`. Cleanup needs a manifest check (item 5), because `remove_worktrees.py` refuses dirty trees. `AGENTS.md` does not change. |
| B. WIP commits on `wf/*` branches, merged with `git merge --squash --no-commit` | Git does the 3-way merge and shows real conflict markers, so item 1 becomes a thin wrapper or goes away. `remove_worktrees.py` refuses unpushed commits, so cleanup still needs a separate path. `AGENTS.md` needs one line that allows these commits. |

Recommendation: B. Git's own merge is more reliable than a custom script, and it matches the earlier policy. Affects: items 1, 2 and 5.

### Q3. How many commits does a parallel run make?

A merged tree mixes the tickets of the run. Splitting it back into one commit per ticket is awkward after conflicts are resolved.

| Option | Effect |
|---|---|
| A. One commit per run | Simple. Map Notes that say "one ticket = one commit" need a parallel-run exception. |
| B. One commit per ticket, in merge order | The coordinator commits after each merge. The owner approves each commit during the run, so the run stops for the owner several times. |
| C. One commit per ticket, split at the end with `git add -p` | No stops, but a hunk that holds two tickets' changes cannot be split cleanly. |

Recommendation: A. Affects: item 2 (last step), the map Notes template.

### Q4. When do the gates run?

| Option | Effect |
|---|---|
| A. Full gate in each worktree, then on the merged tree | What phase 2 did. Each ticket proves itself, at about 5 minutes per gate on a shared build slot. |
| B. `--no-browser` in each worktree, full gate once on the merged tree | Same as slice mode and retro ticket 03. A browser failure shows only after the merge, where it is harder to assign to a ticket. |
| C. Focused checks in each worktree (`clippy`, `test-pattern`, the ticket's browser tests), full gate once on the merged tree | Cheapest. The implementer must choose the right checks. |

Recommendation: B, and the ticket's own browser tests by `test-pattern`. Affects: item 2 (implementer rules).

### Q5. How does a ticket that overlaps the others run?

Ticket 06 (a grab-bag of small fixes) touched the same files as 03 and 04. The coordinator gave it a worktree with the merged 03+04 changes staged as its baseline.

| Option | Effect |
|---|---|
| A. A worktree with the merged changes as its baseline | Runs in parallel with the rest. The baseline is fragile: an implementer that unstages it loses track of its own diff. |
| B. Run it last, in the main checkout, after all other merges | No baseline step and no merge for that ticket. It cannot run in parallel. |
| C. Split the overlapping ticket before the run | The best fit, but someone must split it by hand. |

Recommendation: B. Who decides that a ticket overlaps is also open: the coordinator by reading the tickets, or a `Touches:` line in the ticket. Affects: item 2 (step 1).

### Q6. How are shared counters kept apart?

Scenario numbers and `REQUIRES_MIGRATION_TEST_COUNT` are counters that every implementer may change.

| Option | Effect |
|---|---|
| A. Implementers take the next number; the coordinator renumbers after the merge; a duplicate check catches what the coordinator misses | Item 3. No change to how specs are written. |
| B. The brief reserves a number range per ticket | No collisions, but a manual step per run. |
| C. Per-file numbering from 1 | Does not help: both phase-2 collisions were inside one file. A migration of about 260 tags. |
| D. Slug identifiers from the scenario title | Collisions become rare. A migration of 260 tags, 46 docs and the validator. |

For the quarantine pin, retro ticket 02 lists its own options (an `--update-pin` flag, coordinator-owned lowering, a derived pin).

Recommendation: A for scenarios, and coordinator-owned lowering for the pin until ticket 02 is decided. Affects: items 2 and 3.

### Q7. Who fixes the review findings after the merge?

| Option | Effect |
|---|---|
| A. One fresh implementer in the main checkout | What phase 2 did. It has none of the ticket context. |
| B. Resume each ticket's implementer session for its own findings | Slice mode does this. But each session's worktree no longer matches the merged tree, so the fixes must happen in the main checkout, and several sessions then edit one tree. |
| C. The coordinator fixes them | Uses the coordinator's context, which is already the fullest. |

Recommendation: A. Affects: item 2 (step 7).

### Q8. What if the main checkout holds other work?

Other efforts may have uncommitted changes in the main checkout, or commit during the run (retro ticket 32 saw 14 unrelated changed files). Then the merge base is wrong, and the review bundle holds unrelated work.

| Option | Effect |
|---|---|
| A. The run requires a clean main checkout at start, and `HEAD` must not move during the run | Simple. Blocks a run while other work is in flight. |
| B. Merge into a dedicated run worktree, and copy the result to the main checkout at the end | Isolated, and close to an integration branch without commits. One more copy step. |
| C. Scope by path: the bundle and the commit take only the run's files (retro ticket 32) | Works in a shared tree. Needs ticket 32 first. |

Recommendation: A now, and C when ticket 32 is done. Affects: items 1, 2 and 6.

### Q9. How are merged worktrees removed?

`remove_worktrees.py` refuses dirty worktrees and worktrees with unpushed commits. After a run, every worktree is one of those.

| Option | Effect |
|---|---|
| A. `merge_worktree.py --discard`: remove with `--force` only when the worktree has not changed since its merge (manifest of hashes) | Safe and automatic. Needs the manifest. |
| B. The coordinator runs `git worktree remove --force` after the owner approves | No new code. Relies on the coordinator to check each tree. |
| C. Teach `remove_worktrees.py` that a merged tree is safe | One cleanup tool. That script's rule "clean and pushed" gets an exception. |

Recommendation: A with Q2 option A; with Q2 option B, a branch that `git merge --squash` merged can be checked with `git diff <merged tree> <branch>` instead of a manifest. Affects: item 5.

## Candidate changes

### 1. Merge one worktree into the main checkout — depends on Q2, Q8

With Q2 option A, a new `scripts/merge_worktree.py <worktree> [--dry-run]`:

- It refuses to start when the worktree's `HEAD` is not the main checkout's `HEAD`. That commit is the merge base.
- For each file the worktree changed (untracked files included):
  - **Modified:** `git merge-file <main copy> <HEAD copy> <worktree copy>`.
  - **New:** copied in; merged against an empty base when the main checkout already has the file.
  - **Deleted:** deleted only when the main copy still equals the `HEAD` copy; otherwise a conflict.
- One line per file (`merged`, `added`, `deleted`, `CONFLICT`); exit 1 on any conflict, with markers left in the file.
- It writes the manifest for item 5 and runs `validate_feature_spec.py` at the end.
- Tests in `scripts/tests/test_merge_worktree.py` against temporary repos: clean merge, conflict, a new file in two worktrees, a deletion, and the `HEAD` mismatch refusal.

With Q2 option B, the step is `git merge --squash --no-commit wf/<branch>` followed by `validate_feature_spec.py`. A script is needed only if that pair needs a wrapper.

### 2. The orchestration skill — depends on Q1, Q3, Q4, Q5, Q7, Q8

Proposed steps, with Q1 option A (rewrite `implement-spec`):

1. Read the map and its tickets. Pick the frontier tickets for the run, and mark any ticket that overlaps the others (Q5).
2. Claim each ticket (`Status: claimed`). Create one worktree per ticket from `HEAD`: `git worktree add -b wf/<effort>-<NN> /workspace/wt-<effort>-<NN> HEAD`.
3. Run at most 3 implementers at once (`ENVIRONMENT.md`, "Limits"). Each brief is a short pointer file: the ticket, the worktree, and the implementer rules.
4. Implementer rules:
   - Work only in its own worktree. Commit rules per Q2.
   - Write the answer only in its own ticket file. Do not edit `map.md` or any `Status:` line.
   - Leave shared counters to the coordinator (Q6).
   - Never defer an item to another ticket. List follow-ups in the final report.
   - Gate per Q4.
5. When an implementer finishes, merge its worktree (item 1), resolve conflicts, and renumber colliding scenarios in the spec, the test tag and the ticket answer.
6. File each reported follow-up in the map's follow-up ticket, with its source ticket.
7. When all tickets are merged, run `/simple-after-plan` (full mode) over the merged tree. Fixes per Q7.
8. Set `Status: resolved` and write the map's Decisions-so-far lines.
9. Report to the owner. Commit after approval (Q3). Remove the worktrees (item 5).

Remove from the skill: the integration branch, the merger subagent, the draft PR, the "merge the integration tip before reporting" step, `/tdd` as a hard rule (the ticket's own test rules apply), and the references to `/to-spec`, `/to-tickets` and `/setup-matt-pocock-skills`. Rewrite the `description` per `.agents/skills/writing-for-agents/SKILL.md`, and keep `disable-model-invocation: true`.

### 3. Duplicate scenario check — depends on Q6

- `parse_spec_scenarios` records each number with its line. A number that appears twice in one spec file fails: `<spec>: Scenario 16.36 declared twice (lines 303, 359)`. Exit 1.
- Update the module docstring's exit codes, add a test with a repeating fixture, and add one sentence to `tests/STRATEGY.md` ("Spec scenarios").

### 4. Pointer from wayfinder — depends on Q1

`.agents/skills/wayfinder/SKILL.md`: the line "The user may run unblocked tickets in parallel" points to the skill that Q1 picks.

### 5. Remove merged worktrees — depends on Q2, Q9

Per Q9. `remove_worktrees.py` does not change unless Q9 picks option C.

### 6. `prepare_review_bundle.py` fails on an empty diff — depends on nothing

- When the diff is empty, exit 2 with `error: the diff against <fixed point> is empty`. Today it writes an empty bundle and exits 0.
- Add a test for the empty case.
- Already done since the retro: the separate `assets` area and the `--uncommitted` line in the code-review skill.

## Order of work

1. Answer Q1–Q9.
2. Item 6. It depends on no question.
3. Item 3, after Q6.
4. Items 1 and 5, after Q2, Q8 and Q9.
5. Items 2 and 4, after the rest. The skill names the merge step and the cleanup step.

## Acceptance

- `python build.py` is green, Python tests included.
- Two worktrees that change the same doc file in different places merge with no hand edit, and a real conflict shows markers.
- With Q6 option A, a spec with a repeated scenario number fails `validate_feature_spec.py`.
- `prepare_review_bundle.py --ref HEAD` on a clean tree exits 2.
- The skill names no integration branch, PR or merger agent, and no commit outside the rules that Q2 and Q3 set.
- A dry walk-through of the skill on a two-ticket map has no step that needs a commit before the owner approves.

## Out of scope

- The other retro candidate (status states in `ui_design.md` against the CSS). The review stage caught it, so no check is planned.
- Retro tickets 02, 32, 59 and 64 themselves. This plan only names where they touch it.
