# Triage: retro issue bucket

Read-only triage of the tickets in `issues/`: pass 1 covers 01–29, pass 2
(at the end) covers 30–82. This file classifies each
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

---

# Pass 2: tickets 30–82

## Method (pass 2)

- Checkpoint: HEAD `511e28dd`, 2026-10-21. One checkout (main);
  `wf/rebuild-core` and `wf/rebuild-response` are merged.
- Same verdicts as pass 1, plus **RESOLVED** for tickets already closed, and
  **→ NN** to show that a DROP folds into another ticket.
- Four read-only reviewers re-checked every cited `file:line`. Their raw
  reports are in `tmp/triage-retro/batch-{A,B,C,D}.md` (git-ignored).

## Verdict table (pass 2)

| # | Ticket | Verdict | Evidence check | Practical fix |
| --- | --- | --- | --- | --- |
| 30 | `--no-browser` tier and gate verdict | RESOLVED | Implementation of 03 and 22; on the map | — |
| 31 | Skip browser tier on unchanged inputs | DROP (weak) | `--no-browser` and the journal tree identity exist; `AGENTS.md` no longer defines "green"; the ticket says the skip would have saved 0 runs | None; iterate with `--no-browser`, run the full gate once (03) |
| 32 | Review bundle takes a path list | PROCEED (scoped) | `prepare_review_bundle.py:258-266` has only `--uncommitted`, `--ref`, `--out` | `--paths-from FILE`; decide with 59 and 13 so one scope file feeds bundle, comment pass and commit |
| 33 | UI names in ticket copy must exist | DROP (weak) → 75 | Copy now at `generation_status.rs:31` names no control; one incident, caught by spec review | Example in 75's brief checklist |
| 34 | Sweep callers of a changed port | PROCEED (small) | Instance fixed (`connection_test_service.rs:76`); no "callers" rule in `code-review/SKILL.md:71-75` | One clause in the Spec brief: list the callers of each changed port contract (`rg`) |
| 35 | Lint spec scenario shape | PROCEED (small, narrowed) | One identifier left at `docs/specs/games.md:186`; a Given-after-Then rule would fail 4 deliberate multi-case scenarios | Identifier regex only in `validate_feature_spec.py`; fix `games.md:186`; drop the shape rule |
| 36 | Next free scenario number | DROP (weak) → 64 | No retired-number record; cost was a few `git log -S` calls | `--next` flag on 64's helper |
| 37 | Comment finder covers JavaScript | PROCEED (small) | `comment_finder.py:36` and `:93` skip `.js`; 5 hand-written `assets/dashboard-*.js` | Add `.js` (skip `*.min.js`); one edit with 29 |
| 38 | Audit bloated comments | PROCEED (scoped) | Cited `retry.rs` is gone; `CODING_STANDARDS.md:18` still has no length rule | Decide a length rule and where long reasons go, then audit a fresh top-N; after 37 |
| 39 | Rethink the Development Loop | RESOLVED | On the map | — |
| 40 | Lead holds file tools | DROP (stale) | Herdsman removed; `.pi/extensions/delegate/` replaced it; the parent keeps its tools | None |
| 41 | Tail of long delegated report | DROP (stale) | Cut now in repo (`.pi/extensions/delegate/lib.ts:206-210`, 20 000 chars) and the notice names `result.json` | Optional: raise the cap in the extension |
| 42 | Doc review checks ticket claims | PROCEED (small) | `document-review/SKILL.md:18` skips `.scratch/`; §1 (`:38`) names no ticket Answers | One sentence in §1: grep the effort's ticket Answers before deleting a claim |
| 43 | `reviewer`-has-no-shell premise | DROP (stale) | Sentences removed in `8a8ba6bb`; `.pi/agents/*.md` roles all grant `bash` | None |
| 44 | Comment pass renames or gates | PROCEED (small) | `chronicler-comment-fixer/SKILL.md:51` (rename) contradicts `:53` (delete and flag) | Reconcile: in-file rename or gate is applied, only cross-file changes are flagged; same edit as 52 |
| 45 | Done-when versus Answer | PROCEED (small) | Resolve step `docs/agents/issue-tracker.md:30` and `wayfinder/SKILL.md:128` say nothing about Done-when | One clause: an Answer that narrows a Done-when line edits it or waits for the user |
| 46 | Commit in its own step | PROCEED (small) → 13 | Root cause is harness parallelism (outside repo; pi doc unverified); `commit-and-push/SKILL.md:74,162` still `git add -A` | One line: commit in its own message after `git status --porcelain`; fold into 13 |
| 47 | Next free bucket number | PROCEED (small) | No collision today (01–82 unique on all branches); map Notes give no command; map Open-tickets list stops at 50 | Add the `git log --all --diff-filter=A` command to map Notes; fix the stale Open-tickets list |
| 48 | Trim build commands | RESOLVED | Implementation of 39; on the map | — |
| 49 | Comment pass and string literals | PROCEED (small) | Live: `comment_finder.py` lists `tests/browser/stub/failure_display.rs:771,907` (markers in raw strings) | Rename the marker to `__MEASURE_CLAMPED_TEXT__` in that one file |
| 50 | Dead-path guardrails | DROP (stale) — resolve | Fix on main (`51efaafc`, ancestor of HEAD); meta rule at guardrails `mod.rs:164`, `walkers.rs:94` | Mark resolved; residual: dead exemptions (`structure.rs:135`, `free_fn.rs:22-28`) are not checked |
| 51 | Each branch of a design rule tested | PROCEED (small) | Instance fixed (`job.rs:153`, test `tests/http/options.rs:537`); Spec brief (`code-review/SKILL.md:76`) has no per-branch check | Add (d) to the Spec brief: list each branch and its test |
| 52 | Comment pass lists deleted reasons | PROCEED (small) | Output format (`SKILL.md:118-139`) has no list of deleted reason comments | Add a "Deleted reasons" table; same edit as 44 |
| 53 | Focused checks miss a binary | DROP (weak) → 80 | Proposed `integration` rule misses its own guardrails incident (`AGENTS.md:358`); conflicts with 39 | 80's slice branch already says `--no-browser` before report |
| 54 | Spec Given a client can reach | PROCEED (small) | `tests/STRATEGY.md:116-143` has no reachability rule | One sentence in STRATEGY "Spec scenarios"; review-only |
| 55 | Tier-1 oracle reads production writes | DROP (stale) → 23 | Snapshot deleted; instance cannot recur; `tdd/SKILL.md:51` trigger still "looks suspicious" | Widen 23's trigger: a test that turns green in a step that changes its subject |
| 56 | Ticket edit lists from the tree | DROP (weak) → 75 | Both instances in the closed `rust-rebuild-build` map | Checklist line in 75 |
| 57 | After-plan docs after code fixes | PROCEED (small, two files) | Docs step precedes code steps in both after-plan skills, not only plus-review | Move the docs step after the last code step in both copies |
| 58 | Fixed point of a multi-branch map | PROCEED (small, two files) | Step 12 says `--ref <fixed-point>` with no derivation, in both copies | One sentence: derive via `git log --first-parent` or read map Notes; same ref to `healthcheck.py duplicates --ref` |
| 59 | Bundle: ref plus working tree | PROCEED (small) | `prepare_review_bundle.py:108` still `{ref}...HEAD`; no worktree mode | `--ref REF --worktree`; decide with 32 |
| 60 | Timing summary lists every step | PROCEED (small) | `build.py:1487-1489` runs `duplicates`, `worktrees`, `note` without `_record_step`; newest full log confirms | Record report-only steps with `[OK]`/`[WARN]` |
| 61 | Close-out of an execution map | PROCEED (small) | No close-out operation; three maps end three ways | Add "Reach the Destination" to `docs/agents/issue-tracker.md` Wayfinding operations |
| 62 | `AGENTS.md` timeout wording | PROCEED (small) | `AGENTS.md:347` wording; config denies `timeout` | "Set the bash tool's `timeout` parameter to 1200"; merge with 70 |
| 63 | Deny `rg -r` | PROCEED (small, owner approval) | No `rg -r` rule in the permission config; mid-command match after `cd …;` unverified | Deny rule with a teaching reason; owner applies it |
| 64 | Show a spec scenario by number | PROCEED (small) | No script; `validate_feature_spec.py:42` `SCENARIO_RE` already parses headings | `--show <n.m>` (and 36's `--next`) |
| 65 | Docs identifier check | PROCEED (scoped) | `validate_docs.py` checks paths only; the cited ghosts are gone from `docs/` | Report-only `healthcheck.py docs-identifiers`; needs a noise filter; decide with 82 |
| 66 | Docs-hygiene runs the validator | PROCEED (small) | `chronicler-docs-hygiene/SKILL.md:23` has no read-only sentence | One sentence: `validate_docs.py` is read-only, run it even under a no-build brief |
| 67 | Red gate names the failing test | PROCEED (small) | `build.py:394` keeps only `LEAK`/`FLAKY`; `FAIL` lines go to the log only | Keep `FAIL` lines; print them in the epilogue |
| 68 | Narrow loop runs fmt | PROCEED (small) | `clippy` step has no fmt check (`build.py:646-650`) | `cargo fmt --check` inside the `clippy` step; does not reopen 39 |
| 69 | Codemod for multi-site rewrites | PROCEED (scoped) | No structural-rewrite tool installed (`sg` is util-linux); one session | Brief rule (file by file, `clippy` after each); a tool is a dependency choice; low priority |
| 70 | Build wait helper | PROCEED (small) | `AGENTS.md:347` silent on background runs; no `--wait`; coverage ran 877 s (near the 1200 s cap) | One sentence: run builds in the foreground; merge with 62 |
| 71 | test-police coverage and target dir | PROCEED (small) | `test-police/SKILL.md:84` wrong `cd`; `:86`, `:113` `--target-dir target/test-police` | Remove the target-dir advice, fix the `cd`; choose the coverage source |
| 72 | code-review single-session mode | PROCEED (small) → 80 | `code-review/SKILL.md:11`, §4 always spawns two reviewers | The combined reviewer of 80 |
| 73 | Bundle README patch slice | PROCEED (small) | README template gives no slice command; contradicts itself on order (`:174` vs `:193`) | One awk line; fix the order sentence |
| 74 | `build_slot.py --status` verdict | PROCEED (small) | `build_slot.py:123-125` prints "free" then "stale" holder | Relabel the holder line, or print it only when the slot is held |
| 75 | Delegate brief quality | PROCEED (small) | Delegation rules in `AGENTS.md` have no brief checklist | 3-item checklist; absorbs 11, 33, 56, 66 |
| 76 | `tests/AGENTS.md` names wait helpers | PROCEED (small) | `tests/http/support/http_requests.rs:1` index line omits `wait_idle`, `wait_settled`, `wait_for_jobs` | Name them in the `//!` line; regenerate |
| 77 | Plan text vs free-fn guardrail | PROCEED (small) | `CODING_STANDARDS.md` does not name the rule (`tests/infrastructure/guardrails/free_fn.rs:8-15`) | One line in `CODING_STANDARDS.md`; plan-review item via 81 |
| 78 | Stale `guardrails.md` fails the gate | PROCEED (small, narrowed) | Generator already prints the fix (`generate_guardrails_doc.py:387`); `build.py` hides step output from stdout | Epilogue shows the last error line of a failed step; same change as 67 |
| 79 | `ScriptedModel` unused replies | PROCEED (scoped) | `Drop` (`scripted_model.rs:262-268`) checks stray calls only; 163 call sites | Opt-in `expect_all_used()`; strict mode needs a sweep and may race running Jobs |
| 80 | `/simple-after-plan` slice mode | PROCEED (small) | Skill has no slice branch; the owner chose option A on 2026-10-21 | Write the slice branch as specified; absorbs 72 and 53 |
| 81 | Plan review checks rows vs content | PROCEED (small) | `improve-ai-plan/SKILL.md` has no such check | One checklist item; merge with 77 |
| 82 | Lint recurring Diátaxis findings | PROCEED (scoped) | R7 instance fixed; `vale` not installed and not a gate step | `validate_docs.py` warning for backticked signatures outside fences; decide with 65 |

Counts (pass 2, 53 tickets): RESOLVED 3, PROCEED (small) 33, PROCEED (scoped)
7, DROP 10 (40, 41, 43, 50 stale; 31, 33, 36, 53, 55, 56 weak or folded).
Ticket 50 needs only its resolve step.

## Corrections to pass 1

- **03, 22** — now resolved (see the map).
- **11** — premise stale: `.pi/agents/scout.md:5` grants `bash`. The brief
  check survives only as one line of 75's checklist.
- **08, 11, 40, 41** — delegation is now repo code (`.pi/extensions/delegate/`,
  `f3af8992`). "Outside the repo" no longer holds for the delegate side;
  `read` itself is still the pi harness.
- **Map Open-tickets list** — stops at 50 and omits 37–47, 49 and 51–82.

## Residuals found in pass 2

- Dead exemption paths: `structure.rs:135` (`server/mod.rs`) and
  `free_fn.rs:22-28` (`application/generation/slot.rs`,
  `application/pipeline/spawn.rs`) name no file. Ticket 50's meta rule checks
  scope paths, not exemptions. `layers.rs:280` passes whatever the `mod.rs`
  skip does. Candidate for a new ticket.

## Clusters (pass 2)

| Cluster | Tickets | Single decision |
| --- | --- | --- |
| Build epilogue names the cause | 67, 78, 60 | What the epilogue prints for a red or report-only step |
| Build runs and waits | 62, 70, 74 | Foreground runs with the bash `timeout` parameter; one `--status` verdict |
| Narrow loop | 68 (31, 53 dropped) | `fmt --check` in the `clippy` step |
| Review bundle scope | 32, 59, 73, 58 (and pass-1 13, 12, 21) | Bundle modes, slice command, where the fixed point comes from |
| After-plan workflow | 80, 72, 57, 58, 61 | Slice branch with one combined reviewer; docs step order; close-out |
| Ticket resolve step | 45, 61, 42 (and pass-1 01, 20) | What `docs/agents/issue-tracker.md` Resolve and close-out require |
| Delegate brief checklist | 75, 66 (and 11, 33, 56) | A short checklist in the `AGENTS.md` delegation section |
| `code-review` Spec brief | 34, 51 (and pass-1 21, 27) | Callers sweep, per-branch tests, evidence class |
| comment-fixer contract | 37, 38, 44, 49, 52 (and pass-1 12, 29) | Coverage, in-place renames, deleted-reason list, string markers |
| Spec tooling and rules | 35, 54, 64 (36 folded) | `validate_feature_spec.py` helpers; which spec rules stay review-only |
| Doc lints | 65, 82, 66 | Which doc findings become `validate_docs.py`/`healthcheck.py` warnings |
| Plan versus repo rules | 77, 81 | `CODING_STANDARDS.md` names the guardrails; plan review checks them |
| test-police refresh | 71, 76 | One pass over `test-police` and the wait-helper index line |
| commit-and-push | 46 (and pass-1 13, 19, 22) | Explicit paths; commit in its own step |
| Tool misuse | 63, 69 | Deny rule versus brief line for `rg -r`; multi-site rewrite rule |
| Map housekeeping | 47, 50 (and pass-1 02, 17) | Numbering command in Notes; resolve 50; refresh Open tickets |
| Test proof | 79 (55 folded into 23) | Strict or opt-in unused-reply check |

## Recommended next step (pass 2)

1. **Housekeeping, no grilling:** resolve 50; close the 10 DROP tickets with
   their reason; refresh the map's Open-tickets list. *Done 2026-10-21.*
2. **80 (+72, 53)** — the owner already chose; only the trigger wording is open. *Done 2026-10-21.*
3. **Build epilogue (67, 78, 60)** and **build runs (62, 70, 74)** — small
   `build.py` and `AGENTS.md` edits with daily payoff.
4. **commit-and-push (13, 46, 19)** — still the top pass-1 item.
5. **comment-fixer contract** — one skill edit plus one script edit. *37 and 29 done 2026-10-21; 38, 44, 49, 52, 12 remain.*
6. Then the remaining small clusters, and the scoped tickets last: 32, 38,
   65, 69, 79, 82.
