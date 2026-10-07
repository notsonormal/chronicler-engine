# Triage: retro issue bucket

Read-only triage of the 29 open tickets in `issues/`. This file classifies each
ticket by (a) whether the problem is still real in the tree and (b) whether a
practical fix exists. It does not resolve tickets; the map's grilling rule still
owns the decision.

## Method

- Checkpoint: HEAD `6077371b`, working tree at 2026-10-07 (two uncommitted edits
  from other sessions; the bucket itself is untracked).
- Every cited `file:line` was re-checked against the current tree. The bucket's
  own warning holds: the tree moves, and several findings are already stale.
- Verdicts:
  - **PROCEED (small)** — real problem, one-file prose/script fix, low risk.
  - **PROCEED (scoped)** — real problem, but needs a design choice or more than
    one file.
  - **DROP (stale)** — the cited instance is already fixed; the ticket's
    premise no longer holds.
  - **DROP (weak)** — real friction, but the repo-side fix is impractical, out
    of repo scope, or worth less than its cost.

## Verdict table

| # | Ticket | Verdict | Evidence check | Practical fix |
| --- | --- | --- | --- | --- |
| 01 | Ticket resolution owner after commit | PROCEED (small) | Recurred; resolve step still sits in `docs/agents/issue-tracker.md` with no session-boundary owner | Put the resolve step in `/commit-and-push` or in the map Notes |
| 02 | Quarantine pin conflicts | PROCEED (small) | Pin is `76`; lowering it is housekeeping, not a gate need | Map Notes: leave the pin alone per ticket; coordinator lowers once |
| 03 | Run the full gate once | PROCEED (small, narrowed) | Premise partly stale: `AGENTS.md:388` already says "run once before considering done" | Only the map's end-of-ticket criterion needs easing for docs-only edits |
| 04 | Stub fixture drift | PROCEED (scoped) | 5 fixtures still `include_str!`-copied in `tests/test_utils/stub_server.rs` | Choose: render from template, or add a template-vs-fixture drift check |
| 05 | Test response-body reads | PROCEED (small) | 3 quarantine readers at `1024`; quarantine files already import `support::http_requests` | Replace literals with `response_body`; guardrail optional |
| 06 | HTML escaping assertions | PROCEED (small) | Both `&#60;` and `&lt;` asserted; no written rule | Add `assert_escaped` helper, or assert the raw marker is absent |
| 07 | Settings test panel slicing | DROP (stale) | `card_html_slice` survives in `tests/test_utils/html.rs`, but the `connection-row-*` class collision was renamed away | No live failure mode; the prefix matcher is only a latent concern |
| 08 | `read` with `limit=0` | DROP (weak) | The tool is the pi harness, not this repo | A repo note cannot change tool semantics; fix belongs upstream |
| 09 | AGENTS.md restructure | PROCEED (scoped) | `AGENTS.md` is 430 lines; 300 are generator-owned; all 3 variants exist | Biggest decision: v2 base + `STRUCTURE.md` move as a follow-up |
| 10 | Grilling question lever and fact | PROCEED (small) | `grilling/SKILL.md:12` says "all the context" but defines neither fact nor lever | Add a restated-fact + named-lever requirement to the round format |
| 11 | Delegated brief vs tool grants | PROCEED (small) | Scout has no shell (global definition); brief asked for `git log` | Add a grant check to `wayfinder`'s delegation step |
| 12 | Comment pass scope to the diff | PROCEED (small) | `comment_finder.py` selects files, then lists every comment; prior fix unlanded | Add a diff-line mode using `git diff -U0` |
| 13 | commit-and-push stages whole tree | PROCEED (small) | `SKILL.md:75` and `:163` still say `git add -A`; incident real | Stage explicit paths; name files left out |
| 14 | Derived numbers in prose | DROP (stale) | `dashboard.md` no longer carries a hand-typed route count | The drift check is fuzzy; reopen only on a fresh instance |
| 15 | Stale references in skills | DROP (stale) | The `/check-text` reference is gone from `.agents/skills/` and `docs/` | Generalisation is a plain grep; fold into 28 if wanted |
| 16 | Validate skill edit from clean context | PROCEED (scoped) | `writing-for-agents/SKILL.md` names no clean-context run | Apply to behavioural skills only; fix the broken `26-*` cross-ref |
| 17 | Map Notes coordinator conventions | PROCEED (small) | Parallel tickets re-authored conventions and contradicted | Record pre-fan-out conventions in the map Notes; merge 02/03/20 |
| 18 | Standards review clone report | PROCEED (scoped) | `report/jscpd-report.json` exists, report-only, 2446 clones | Name it in `code-review`; needs a top-pairs threshold or summary |
| 19 | GitHub-side push failure | PROCEED (small) | `SKILL.md:140` covers only remote-diverged; 7 retries observed | Add dry-run localize, stop after ~2, record Request ID |
| 20 | Read implementing session | PROCEED (small) | No skill names `session_search`/`session_read` for resolution | Add it to `wayfinder`'s resolve step; merge with 01 |
| 21 | Never skip `tooling.patch` | PROCEED (small) | The skill has no skip instruction; the incident was a hand-written brief | Add one rule to `code-review` §4; low priority |
| 22 | Record gate provenance | PROCEED (small) | `build.py` already journals `logs/build_history.txt` (time, exit, args) | Add a tree hash; require the commit report to cite the last green |
| 23 | Unconditional mutation check | PROCEED (small) | The two cited tests are already fixed; the `tdd` gate is still "looks suspicious" | Make break-and-watch unconditional for state-repair tests and swallow-fixtures |
| 24 | Research answer adjacent gaps | PROCEED (small) | Recon found a real untested path; it landed nowhere | Require `## Answer` to record adjacent gaps |
| 25 | `tdd` skill triggers | PROCEED (small) | Description still omits test-strengthening; accepted twice, unlanded | One-line description edit |
| 26 | Probe server port allocation | PROCEED (scoped) | Skill still pins `3001` + one shared target dir; 3000 is the dashboard | Cheapest: per-session target dir; or reuse the file-locked allocator |
| 27 | Review finding evidence class | PROCEED (small) | `code-review` names no evidence class | Label findings quoted-rule / quoted-spec / inference |
| 28 | Docs-hygiene usage evidence | PROCEED (scoped) | Skill is read-only against `docs/diataxis/`; no usage phase | Extending it changes its scope; a separate skill is cleaner |
| 29 | Comment finder Python coverage | PROCEED (small) | `--all` globs Rust/HTML/CSS only; "full sweep" skips Python | Extend the glob, or add `--all-languages` |

Counts: PROCEED (small) 19, PROCEED (scoped) 6, DROP 4.

## Clusters (do these as one decision each)

Several tickets are facets of one decision. Grilling them separately repeats
context. The clusters:

| Cluster | Tickets | Single decision |
| --- | --- | --- |
| Map Notes conventions | 02, 03, 17, 20 (and 01) | What the map Notes must record before fan-out: shared counters, gate cadence, commit policy, resolve owner, transcript tools |
| commit-and-push | 01, 13, 19, 22 | Staging scope, resolve hook, push-failure branch, gate provenance |
| code-review | 18, 21, 27 | Review inputs and finding labels |
| Test conventions | 05, 06 | How a test reads a body and asserts escaping (07 dropped) |
| tdd | 23, 25 | When the mutation proof is mandatory, and the skill triggers |
| comment-finder | 12, 29 | Diff-line scope and Python coverage |
| docs-hygiene | 15, 28 | Scope: skills, and usage evidence |

## Stale-evidence detail

These four tickets cite an instance that no longer exists. Re-verify before
reopening:

- **07** — `tests/test_utils/html.rs::card_html_slice` still prefix-matches, but
  the colliding `connection-row-*` classes were renamed, so the described
  mis-slice cannot recur. The residual concern is latent, not observed.
- **14** — `docs/diataxis/reference/frontend/dashboard.md:165` now links the
  generated table instead of stating a count. No hand-typed route count remains
  under `docs/`.
- **15** — no `/check-text` reference remains outside `.scratch/` and
  `old-docs/`.
- **23 (instance only)** — `tests/http/games_switch.rs` no longer asserts on the
  recomputed status, and no `let _ = try_claim(...)` remains. The skill-level
  lesson still stands, so the ticket survives on the rule, not the incident.

Two more premise problems:

- **03** assumes `AGENTS.md` never says "once". It does, at line 388. Only the
  map's end-of-ticket line still needs the change.
- **16** links [`26-measure-generated-share.md`](issues/26-measure-generated-share.md),
  which does not exist. Issue 26 is `probe-server-port-allocation`. The
  "measure generated share" idea has no ticket; it belongs to 09.

## Recommended next step

Take the clusters in order, most-bang first:

1. **commit-and-push** (13, 19, 22) — highest value; the `git add -A` default
   already shipped another session's work once.
2. **Map Notes conventions** (01, 02, 03, 17, 20) — one prose block removes the
   parallel-ticket conflicts and the resolve fall-through.
3. **comment-finder** (12, 29) and **tdd** (23, 25) — two small script/prose
   changes each.
4. **Test conventions** (05, 06) — small and mechanical.
5. Then the scoped tickets: 04, 09, 16, 18, 26, 28.

Drop or leave stale: 07, 08, 14, 15.
