# Branch-wide temporary-code and scratch-artifact sweep

Type: task
Status:
Blocked by: 09, 10, 11, 12

## Question

The redesign branch accrued scaffolding that was **correct for the work at the
time and is not meant to live on**. Some of it is load-bearing and must stay;
some is a one-off instrument that has served its purpose; some is a worktree or
`tmp/` artifact that was never tracked at all. Sweep it once, after the
acceptance gate, with each item resolved explicitly — deleted, promoted to a
keeper, or declined with evidence.

This is deliberately **after ticket 12**, because the acceptance gate needs
`scripts/stress_posture.sh` and the current `tmp/` state to run; deleting an
instrument before it has produced its final evidence inverts the order.

Standing preference from the map applies: *be suspicious of accumulated
diagnostics and harness structure; judge whether each piece earns its keep.*

### In scope

**A. One-off measurement instruments (`scripts/`)**

1. `scripts/measure_tiers.sh` — written for ticket 07 to compare tier-2 stub
   cost against tier-3 engine-backed cost. Its embedded test list is **already
   stale**: it names `tier2::test_slash_menu_opens_on_slash_tier2` and
   `tier2::test_error_toast_on_action_failure_tier2`, neither of which exists
   (the tier-2 ports dropped the `_tier2` suffix; `slash_menu.rs` is gone
   entirely). It also silently assumes the default `target/` dir. Decide:
   repair its list and parameterize the target dir, or delete it as a spent
   instrument with its measured table preserved in the ticket-07 Answer.
2. `scripts/stress_posture.sh` — still required by ticket 12, so it is
   evaluated here only for its **post-12** fate: keep as a
   regression instrument for the settle-gate design, or delete. If kept, it
   needs the same `CARGO_TARGET_DIR` / `--target-dir` fix as item 1 — under the
   concurrent-agent workflow it currently grades `target/debug` while the agent
   just built `target/<name>`, i.e. it can report 50/50 against a build the
   agent did not make.

**B. Scratch and untracked artifacts**

3. `tmp/` — was 400 entries / 11 GB. **The two git worktrees
   (`tmp/wt-07`, `tmp/wt-08`) were removed 2026-09-19 (see Comments), taking
   `tmp/` to 21 MB** — each worktree's 5.4 GB was entirely its own `target/`
   build directory, so a worktree is not a cheap "just a checkout". The
   residual ~20 MB is investigation output (`.diff` / `.txt` dumps,
   `screenshots/`, `test_server_logs/`), which is exactly what
   `clean_tmp_dirs` is for.

   **Open question: are worktrees a supported workflow here at all?** This is
   the item's real content, and it is genuinely undecided rather than a
   defect to fix. Facts established by survey:
   - Nothing in `build.py`, `scripts/`, or `.agents/` creates a worktree — the
     sanctioned concurrency mechanism is `--target-dir target/agent2`
     (AGENTS.md:434). The two removed worktrees were a manual one-off for
     ticket 07/08 isolation.
   - No convention says where a worktree should live. They were put in `tmp/`
     by ad-hoc choice, and `tmp/` is gitignored scratch whose contract is "gets
     cleaned" — so that placement is the thing to question, not the cleaner.
   - **Consequence worth knowing (not a bug to fix):** `clean_tmp_dirs`
     (build.py:840) walks `tmp/` recursively and unlinks files older than 30
     days, leaving directories in place. If a worktree lives under `tmp/` for
     more than 30 days, its checked-out files (and its `.git` file) age past
     the threshold and get deleted, leaving a registered-but-gutted worktree.
     This did not fire for wt-07/wt-08 (days old, not 30+). If worktrees are
     kept, the fix is placement (a home outside `tmp/`), not a guard in the
     cleaner — a guard would only be needed if `tmp/` is deliberately the
     worktree home.

   Decide: (a) worktrees are not a supported workflow — record that and rely on
   `--target-dir`, in which case the 10.9 GB was a one-off and needs no
   machinery; (b) worktrees are supported — give them a documented home
   outside `tmp/`, and add a `git worktree remove` step to `run_cleanup`
   (explicit `--cleanup`, **not** the gate's automatic post-test housekeeping,
   which would delete another agent's in-flight build).
4. `logs/build_history.txt` and `logs/build_*.log` — untracked and gitignored
   (`logs/`), so no repo-hygiene action; confirm they are covered by the
   existing cleanup path rather than growing unbounded.

**C. Promotions (temporary → permanent, if justified)**

5. `tests/test_utils/stub_fixtures/*.html` — 10 canned fragments captured from
   a live engine (ticket 06). **Load-bearing**: `tier2_stub.rs` `include_str!`s
   all ten. Not a deletion candidate. The real question is the drift tax the
   ticket-07 Answer flagged: is there a cheap check that a fixture still
   structurally matches the Askama template it mirrors? If not, record why (the
   ticket-07 Answer already bounds it) rather than building machinery.
6. `.pi/extensions/smoke-break/` and its two plan docs — a project-local pi
   extension, committed deliberately, with its own decision history. Confirm it
   is intended to stay (it is not UI-verification work at all, so it may belong
   in a different effort or none).
7. `tests/test_utils/tier2_stub.rs` + the stub tier itself — **not** temporary;
   this is the ratified tier-2 design. Listed only so the sweep does not
   mistake a keeper for scaffolding.

**D. Branch-finalization residue**

8. The plan file `docs/plans/ticket-09-tier-3-conversion-...md` — `docs/plans/`
   holds 31 plan files, 30 tracked, and there is an existing archival path
   (`old-docs/archived-plans/`, 17 files). Decide the convention: does a plan
   whose ticket is resolved get archived, stay in `docs/plans/`, or is the
   directory itself the archive? Apply one rule consistently rather than
   leaving a mixed state.
9. Uncommitted-tree question: at survey time four files are untracked
   (`docs/plans/ticket-09-*.md`, `docs/specs/browser_prompt_presets.md`,
   `scripts/tests/test_generate_guardrails_doc.py`,
   `tests/browser/prompt_presets.rs`) — all of them ticket-09 deliverables,
   none of them scratch. Confirm each is committed by the time this ticket
   runs, so the sweep does not delete a real deliverable by accident.

### Out of scope

- Deleting anything the acceptance gate (ticket 12) still needs.
- `src/` application code — the branch made no application change; this sweep
  does not touch it.
- Re-litigating tier placement (`tests/STRATEGY.md` rule, ticket 11) or the
  settle-gate design (tickets 04/06/09).
- General repo hygiene outside this branch's artifacts.

## Answer

*(record per item: deleted / promoted to keeper / declined-with-evidence; for
items 1–2 state whether the target-dir fix was applied; for item 3 paste the
before/after `tmp/` size and, for the worktree question, which branch was taken
— (a) not a supported workflow, or (b) supported with a documented home.)*

## Comments

**2026-09-19 — item 3 partially executed (worktrees removed).**

Both worktrees were verified clean and fully merged before removal:

| Check | wt-07 | wt-08 |
|---|---|---|
| Uncommitted changes | 0 | 0 |
| Unique commits vs HEAD | 0 | 0 |
| Stashes | 0 | 0 |
| Branch merged into HEAD | yes | yes |

Removed with `git worktree remove` (not `rm -rf`) so git's worktree
registration is cleared rather than left stale; `git worktree prune` confirms a
clean list. `tmp/` went **11 GB → 21 MB**.

Space breakdown that explains the size: each worktree's `target/` was 5.4 GB;
its source tree was only ~30 MB. A worktree therefore pays a **full second
compile** of the engine, which is the cost to remember when spawning one.

Still open on item 3: the residual ~20 MB is investigation output (`*.diff`,
`*.txt`, `screenshots/`, `test_server_logs/`) — the `clean_tmp_dirs` contract.
The merged branches `ui-verif/ticket-07` / `ui-verif/ticket-08` were left in
place (merged and harmless; branch deletion deferred as a user-controlled git
operation).

**2026-09-19 — correction to an earlier note in this ticket.** The first pass at
this item recorded a "live hazard in `clean_tmp_dirs`" that needed a guard. That
framing was wrong and is withdrawn. `clean_tmp_dirs` trims aged scratch files
from `tmp/`, which is its job; it is not confused about worktrees. The
worktrees were placed inside `tmp/` by ad-hoc choice, so the placement is the
thing to question, not the cleaner. The accurate statement is in item 3
above, including the one fact worth knowing: a worktree left under `tmp/` past
the 30-day threshold would age out and end up registered-but-gutted. That is a
consequence of placement, and the fix (if worktrees are kept) is a home outside
`tmp/` — not a `clean_tmp_dirs` guard.
