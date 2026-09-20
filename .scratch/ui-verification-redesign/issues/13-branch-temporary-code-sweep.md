# Branch-wide temporary-code and scratch-artifact sweep

Type: task
Status: resolved
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

Resolved 2026-09-20. All nine items resolved explicitly below. Full gate
green on the final tree (`1605 integration / 137 guardrails / 20 browser /
1 architecture`, 0 failed; validator `141 declared, 141 covered, 0 gap(s),
0 orphan(s), 0 untagged, 0 surface mismatch(es), quarantine 85/85`;
`Total: 33.60s`).

### Summary table

| # | Item | Disposition |
|---|---|---|
| 1 | `scripts/measure_tiers.sh` | **deleted** — spent instrument; table survives in ticket 07's Answer |
| 2 | `scripts/stress_posture.sh` | **promoted to keeper** + target-dir fix applied |
| 3 | `tmp/` + worktree question | worktrees **(a) not a supported workflow**; spent evidence dirs deleted, residual left to `clean_tmp_dirs` |
| 4 | `logs/` | **declined** — covered by the existing cleanup path; one nuance recorded |
| 5 | `tests/test_utils/stub_fixtures/*.html` | **promoted to keeper**; drift-tax check declined with the ticket-07 bound |
| 6 | `.pi/extensions/smoke-break/` | **promoted to keeper** — belongs to its own effort, not this one |
| 7 | `tests/test_utils/tier2_stub.rs` | **keeper** — ratified tier-2 design, not scaffolding |
| 8 | `docs/plans/` plan-archival convention | rule recorded and applied — see below |
| 9 | uncommitted-tree deliverables | **all committed** before this ticket ran |

### A. One-off measurement instruments

**1. `scripts/measure_tiers.sh` — deleted.** It was a spent instrument: written
for ticket 07's tier-2-versus-tier-3 cost comparison, with an embedded test list
that had already gone stale (it named `tier2::test_slash_menu_opens_on_slash_tier2`
and `tier2::test_error_toast_on_action_failure_tier2`, neither of which exists —
the `_tier2` suffix was dropped in ticket 07 and `slash_menu.rs` deleted in the
per-surface re-split). Its measured table is preserved in ticket 07's Answer
("Wall times (measured directly through the binary)": tier-2 module 12 tests in
4.70 s, full browser binary 26 tests in 20.23 s direct). A script nothing
invokes, whose only output is already recorded elsewhere, that would need
repair to run at all, does not earn its keep. The **target-dir fix was not
applied** — repairing an instrument only to delete it is wasted work.

**2. `scripts/stress_posture.sh` — promoted to keeper; target-dir fix
applied.** This is the regression instrument for the exact race the map exists
to close: it runs the tier-3 posture test N times and counts "lost
interaction" against the ticket-03 legacy signature (`never settled
'#world-posture-status'`). Post-ticket-12 it is the standing way to re-verify
the settle-gate design on demand, so it stays. The fix:

```sh
TARGET_DIR="${CARGO_TARGET_DIR:-target}"
export CARGO_TARGET_DIR="$TARGET_DIR"
BIN="$(ls -t "$TARGET_DIR"/debug/deps/browser-* 2>/dev/null | grep -v '\.d$' | head -1)"
```

The export is essential, not cosmetic. `tests/test_utils/server.rs`
(`start_server_with_env`) resolves the engine binary from `CARGO_TARGET_DIR`.
Without the export, a script invoked with the variable on the command line would
find the right browser binary but the harness would shell out to `cargo run` per
test and stall on the package-cache lock — the exact failure ticket 07's Answer
recorded. Both the lookup and the export now honor the variable.

Verified two ways: a stock run (`bash scripts/stress_posture.sh 1`) passed in
3.31 s against `target/debug/deps/`; and
`CARGO_TARGET_DIR=target/nonexistent bash scripts/stress_posture.sh 1` exits
**1** with `no browser test binary found under target/nonexistent/debug/deps —
build first`, proving the override redirects the lookup rather than silently
falling back. The header comment's stale `retries = 1` reference (removed in
ticket 12) was corrected at the same time.

### B. Scratch and untracked artifacts

**3. `tmp/` and the worktree question — (a) worktrees are not a supported
workflow.**

Before/after: **24 MB → 23 MB** at sweep time. (The pre-sweep 11 GB → 21 MB
worktree removal is recorded in the 2026-09-19 comment above; this sweep starts
from that 24 MB residual.) The ~1 MB removed is this effort's spent evidence:
`tmp/acceptance/` (376 K — ticket 12's run logs and stdouts),
`tmp/posture_stress/` (212 K — the two 50-run loops),
`tmp/ticket14/` (16 K — boot probes with 0-byte logs), and the one-off
`tmp/acceptance_runs.sh`. Ticket 12 named the first two as its ticket-13
handover; their findings are already recorded in its Answer.

One naming correction: ticket 12's line says "`tmp/acceptance/` and the two loom
logs are left for ticket 13's sweep". **No file matching `*loom*` exists**
anywhere in the repo (`find` across the tree, and `grep` across `tmp/` and
`logs/`, both return nothing), so the phrase names a file that is not there. The
two logs ticket 12 actually produced are the stress-loop outputs
`tmp/acceptance/stress.out` and `tmp/acceptance/stress_final.out`, both swept
here with the rest of `tmp/acceptance/`. Recorded so a later reader does not go
looking for a loom file.

`clean_tmp_dirs` then ran over the rest and **removed nothing** — the oldest
`tmp/` files are 29.1 days old, just under its 30-day threshold. That is the
cleaner behaving correctly, not a defect: the residual is investigation output
(`*.diff`, `*.txt`, `screenshots/`, `test_server_logs/`) whose contract is to
age out on its own. No forced deletion was performed; the many-other-efforts
files (`tmp/crap/`, `tmp/review/`, etc.) are not this branch's to remove.

**Worktree verdict: (a) not a supported workflow.** The evidence:

| Candidate convention | Evidence |
|---|---|
| Created by tooling? | No — nothing in `build.py`, `scripts/`, or `.agents/` creates a worktree |
| Documented home? | No — no convention says where one lives; wt-07/wt-08 were ad-hoc in `tmp/` |
| Sanctioned concurrency mechanism | `--target-dir target/agent2` (`AGENTS.md` "Concurrent Builds") |
| Cost | Each worktree pays a full second compile (5.4 GB `target/`) |

The sanctioned mechanism for concurrent agents is `--target-dir`, which is
already documented and needs no new machinery. The 10.9 GB was a one-off for
ticket 07/08 isolation, not a workflow. **No `git worktree remove` step is
added to `run_cleanup`** — option (b) was declined because it would add
machinery for a workflow the repo does not support, and the ticket itself notes
such a step must not enter the gate's automatic housekeeping (it would delete
another agent's in-flight build). The `clean_tmp_dirs` placement consequence is
recorded in the ticket body and needs no guard, since `tmp/` is not the
worktree home.

**4. `logs/` — declined, covered by the existing path.** `logs/` is gitignored,
so no repo-hygiene action. `clean_old_logs` (`build.py`, called each run) trims
`build_*.log` older than 3 days; `_append_history` self-bounds
`build_history.txt` at 1000 lines. One nuance worth recording rather than
fixing: `clean_old_logs` matches `startswith("build_")` only, so the 28
`chronicler_*.log` engine daily logs (114 MB of the 115 MB) are outside its
scope and grow unbounded. That is a **general** repo-hygiene observation, not
this branch's artifact, so it is recorded here and left alone — per the
Out-of-scope line.

### C. Promotions

**5. `tests/test_utils/stub_fixtures/*.html` — promoted to keeper.** All ten are
`include_str!`'d by `tests/test_utils/tier2_stub.rs`; this is the ratified
tier-2 design, not scratch. The drift-tax question is **declined with the
ticket-07 bound**: ticket 07 already established that the failure direction is
loud, not silent — the fixture must keep the structural hooks the JS addresses
(`.log-entry` / `data-id` / `.text` / `data-raw-text` / `.edit-btn` /
`.message-actions`), so a template rename fails the test with element-not-found
rather than passing quietly. Ticket 14 has since pinned the real template's edit
hooks in a unit test, so the gate now has a structural check on the real
side too. A fixture-versus-template structural comparator would be machinery the
project does not want for a tax already bounded and paid knowingly.

**6. `.pi/extensions/smoke-break/` — promoted to keeper, but not this
effort's.** Committed deliberately in `3af5f2d` with its own decision history
(`docs/plans/pi-extension-smoke-break-nudge.md` — the design record, kept in
`docs/plans/` as a live reference, linked from `index.ts` and from the
implementation plan). No `.scratch` ticket owns it — the only mention in any
issue is this ticket. So: it stays, and it belongs to a different effort (or
none). Nothing in this sweep touches it.

**7. `tests/test_utils/tier2_stub.rs` + the stub tier — keeper, untouched.**
Confirmed as the ratified tier-2 design, exactly as the ticket anticipated.

### D. Branch-finalization residue

**8. `docs/plans/` plan-archival convention — rule recorded and applied.**

The convention is **ticket-tied**: a plan whose ticket is resolved is archived
to `old-docs/archived-plans/`; a plan still guiding open work stays in
`docs/plans/`. This follows the `chronicler-after-plan-workflow` skill
(rule 2: "Archive the recently used plan"; rule 3: wayfinder-tied plans follow
the wayfinder workflow instead) and matches the existing practice (`ticket-16`,
`ticket-20` already archived). `docs/plans/` is not itself the archive — the two
directories have distinct contracts.

Applied consistently to the **two ticket-tied plans in `docs/plans/`, both with
resolved tickets**, neither with any inbound reference (verified by search):

| Plan | Ticket | Ticket status | Action |
|---|---|---|---|
| `ticket-10-harness-deletion-pass-ui-verification-redesign-map.md` | ui-verif 10 | resolved | archived |
| `ticket-17-speaker-axis-type-rework-user-regen.md` | steering 17 (closed effort) | resolved | archived |

`docs/plans/` 31 → 29 files; `old-docs/archived-plans/` 17 → 19. This resolved
the ticket's premise in passing: `docs/plans/ticket-09-tier-3-conversion-...md`
**does not exist and never was committed** (`git log --all` finds no such path),
so the mixed state item 8 describes had already resolved itself — the remaining
asymmetry was the two above.

The non-ticket plans in `docs/plans/` (`mapless-worlds-plan.md`
"Decision-complete", `reliability-and-cancellation-plan.md` "Planning /
Sub-plans pending", the smoke-break design record, etc.) are **not** touched:
they are not ticket-tied, and some are still live references. Their archival is
a separate concern from this branch's residue.

**9. Uncommitted-tree deliverables — all confirmed committed.** At sweep time
`git status` shows **zero untracked files**. All four named deliverables are
tracked: `docs/specs/browser_prompt_presets.md`,
`scripts/tests/test_generate_guardrails_doc.py`, and
`tests/browser/prompt_presets.rs` are committed; the fourth,
`docs/plans/ticket-09-*.md`, was never committed because it never existed. No
deliverable was at risk, and none was deleted by accident.

### Verification

Full gate green on the final tree:

```
nextest: 1 passed, 0 failed    (architecture)
nextest: 137 passed, 0 failed  (guardrails)
nextest: 1605 passed, 0 failed, 2 skipped  (integration)
nextest: 20 passed, 0 failed   (browser)
141 declared, 141 covered, 0 gap(s), 0 orphan(s), 0 untagged,
0 surface mismatch(es), quarantine 85/85
Total: 33.60s
```

Files changed by this ticket: `scripts/measure_tiers.sh` (deleted),
`scripts/stress_posture.sh` (target-dir fix + comment correction), two plan
moves. No `src/` or test change — the application and its tests are untouched,
as the Out-of-scope section requires. `bash -n` passes on the modified script;
the script was executed twice (stock run and the override-failure case) to
confirm both the happy path and the env-var redirection.

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
