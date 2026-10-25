# Audit: which open tickets fix a root cause

Read-only audit of the 66 open tickets, 2026-10-25, HEAD `253259b1`. It asks a
different question than `triage.md`. Triage asked "is the problem still real".
This audit asks "does the proposed fix change what the next agent does".

## The lens

A fix to an agent's environment is one of five kinds. Each kind has a different
chance to work.

| Class | What the fix is | Does it work? | Evidence in this bucket |
|---|---|---|---|
| **Mechanical** | Code: a script, a check, a better error message, or a change that makes the mistake unrepresentable | Yes. It works for every model and every brief | 37, 29, 50 landed and did not come back |
| **Wrong prose** | The skill or `AGENTS.md` text is false, stale, or contradicts itself, so it causes the mistake | Yes. The edit removes the cause | 71: the skill tells the agent to make a cold 7.4 GiB target dir |
| **Output request** | The brief or skill asks for a named item in the report, which the lead can check | Mostly. The item is visible, so a miss is visible | 52: the C6 brief asked for a list of deleted reasons; nothing had to be restored |
| **Vigilance rule** | "Remember to check X" in a skill, brief or `AGENTS.md` | Seldom. It costs context in every run and does not hold | 82: the brief named the doc rules from R4 on; the findings came back in 8 of 8 slices. 84: ticket 03 was resolved; one run still made 39 `build.py` calls |
| **Model slip** | One wrong tool call or one misread, which a loud error already corrects | No fix needed. A rule costs more than the slip | 08, 46, 62, 63: each slip cost one or a few turns |

A sixth case is not a ticket kind but a root cause: **process seams**. Many
tickets are symptoms of the workflow design: many sessions in one checkout,
hand-offs between lead and delegates, a fact written in two places (Done-when
and Answer, ticket and doc). A rule at each seam adds more process. The fix is
fewer seams, or isolation.

## Why prose rules decay (the bitter lesson)

Sutton's point: methods that use more compute and search beat hand-encoded
knowledge, and the hand-encoded knowledge becomes a cost. Here the hand-encoded
knowledge is the rule text in skills and `AGENTS.md`. This bucket shows the
decay directly:

- **The environment moves faster than the bucket drains.** 11, 40, 41, 43 went
  stale when the delegate harness changed. 13, 19 and 46 target
  `commit-and-push`, which `44eac750` deleted on 2026-10-23. 07, 14, 15 lost
  their instance. 10 of 85 tickets decayed before anyone acted.
- **Prose did not change behaviour where it was tried.** 82 and 84 above.
- **Many proposals were "accepted, unlanded" twice** (12, 13, 25, 27, 28). A
  fix that two retros accept and nobody applies is evidence of low pain.
- **The slips will go away without a rule.** `rg -r`, `read limit=0`,
  `timeout 1200 …`: a better model, or the error message, fixes them.

What survives a model upgrade: checks that compute a fact (tests, guardrails,
`validate_*.py`), tools that remove a manual step, and spending compute on
verification (a fresh-context run, a mutation run) instead of on rules.

The lens is not a ban on prose. Prose that is *wrong* must be fixed. Prose that
gives a repo fact the model cannot guess (a route, a helper name) is
information, not a rule. It is best put where the model meets it: in the
failing check's message or in a generated index.

## Verdicts

Epistemic status: **known** = re-checked in the tree today; **inferred** =
from the ticket text and `triage.md`, not re-checked.

### Close (30)

Stale, a slip, a one-off, or a vigilance rule where the existing net worked.

| # | Class | Reason to close |
|---|---|---|
| 01 | Seam | `commit-and-push` is gone (known). One occurrence, recovered from session logs. Fold the concern into decision D2 |
| 02 | Seam | The ratchet passes when the count is below the pin; the conflict came only from two parallel worktrees, once. The pin is now 72 (known) |
| 06 | Slip | The test failed loudly twice, then the agent read the output. Self-correcting |
| 07 | Stale | Triage pass 1 dropped it; the status was never changed |
| 08 | Slip | The `read` tool is the pi harness. Two wasted reads |
| 11 | Stale | `scout` has `bash` (triage pass 2) |
| 12 | Done | The user ruled on 2026-10-11; the `code-review` sentence landed on 2026-10-17. The open half is 49 |
| 13 | Stale | `commit-and-push` deleted in `44eac750` (known). The root cause, a shared checkout, goes to D2 |
| 14 | Stale | No hand-typed count remains (triage pass 1) |
| 15 | Fold → 65 | The same failure came back as `/status/generating` in a skill. That is evidence for 65's mechanical route check, not for a wider audit scope |
| 19 | Stale + external | Skill deleted. The cause was a GitHub outage |
| 20 | Slip | The user named the tool once and the agent used it. A model that knows its tools does not need the rule |
| 21 | Slip | A hand-written brief, once. The skill has no skip instruction |
| 24 | Vigilance | One incident. "Record adjacent gaps" is a habit rule |
| 28 | Not a skill | The user asked for usage archaeology and the agent did it. No skill is needed for a task the user names |
| 34 | Vigilance, net worked | The spec reviewer listed the callers without being told |
| 42 | Seam | Root cause: a fact lives in a ticket Answer and in a doc. A "grep the tickets" rule will not hold. See D3 |
| 45 | Seam | Same root cause as 42: Done-when and Answer state two rules. See D3 |
| 46 | Slip + stale | Parallel sibling tool calls; the skill is gone. One wasted commit |
| 47 | Seam | Caused by tickets filed from two checkouts. See D2 |
| 51 | Vigilance, net worked | The doc review caught it, with a one-line fix |
| 58 | One-off | The user named the fixed point once |
| 61 | Ceremony | Two maps closed two ways. No agent failed because of it |
| 66 | Slip | One scout misread "do not run `build.py`" |
| 69 | Model capability | Regex heredocs failed. A better model or a Python script fixes it; `ast-grep` is a new dependency for one session |
| 70 | Fold → 62 | Run builds in the foreground; one clause in the 62 reword |
| 75 | Vigilance | "Check your brief" for the lead. The delegate's report caught each case |
| 77 | Loud failure | The guardrail message teaches the fix at the cost of one gate run |
| 84 | Vigilance failed | The rule from 03 did not hold. Option B (refuse a repeat on an unchanged tree) is mechanical but saves little: most `clippy` runs were on a changed tree |
| 63 | Slip | Optional: the owner may add the deny rule, because it is mechanical and costs nothing. Do not add an `AGENTS.md` line |

### Fix: mechanical (21)

Code changes. Each one works whatever the model or the brief.

| # | Fix | Value |
|---|---|---|
| 49 | Rename the marker so it is not a comment (`__MEASURE_CLAMPED_TEXT__`). Still `// MEASURE_CLAMPED_TEXT` at `tests/browser/stub/failure_display.rs:771` (known) | High: it broke the browser tier once; tiny edit |
| 67, 78, 60 | The `build.py` epilogue prints the failing test, the last error line of a failed step, and a row for each report-only step | High: every red gate |
| 68 | `cargo fmt --check` inside the `clippy` step | Medium: saves a full gate run |
| 65 (+15) | Route check first: each `/status/…`, `/fragment/…` in `docs/` and `.agents/skills/` must be in the generated `http_routes.md`. Identifiers later, if ever | Medium: recurred twice; low noise |
| 35 | Identifier regex in `validate_feature_spec.py` (drop the shape rule, per triage) | Medium: one lint replaces a 29-line review finding |
| 85 | `comment_finder.py` flags change words, report-only | Medium |
| 23 + 83 | One decision: a mutation tool (`scripts/mutate.py`, or `cargo-mutants` on the changed functions) instead of a "when it looks suspicious" rule. This spends compute instead of vigilance | Medium-high: two dead tests survived a green gate; two hand mutations went wrong |
| 79 | `ScriptedModel` fails on unused replies (opt-in or strict) | Medium |
| 59 | `prepare_review_bundle.py --ref REF --worktree` | Medium: each plus-review run |
| 18 | The bundle carries the diff-scoped clone summary (not the 2446-clone repo report) | Low-medium |
| 04 | Render the 4 remaining stub fixtures from their templates, or a drift test (4 fixtures exist, known) | Low-medium |
| 05 | A guardrail or helper for `to_bytes(…, <literal>)`; literals remain in `tests/http/` (known). The panic is loud, so this is optional | Low |
| 26 | The probe port is still 3001 (known). Only matters with two probes at once | Low |
| 64 | `--show n.m` and `--next` on the spec validator | Low |
| 73 | One slice command in the bundle README | Low |
| 74 | `build_slot.py --status` prints one verdict | Low |
| 38 | Read the `long comment runs` guardrail threshold in `tests/infrastructure/guardrails/style.rs`, and check whether it catches the blocks in 38. If it does, close 38 | Low; a short read |
| 76 | Name the wait helpers in the `http_requests.rs` module summary (the index is generated) | Low, tiny |

### Fix: wrong prose or output request (9)

The text causes the mistake, or the fix asks for a visible report item.

| # | Fix | Why prose is the root cause here |
|---|---|---|
| 71 | Remove the `--target-dir target/test-police` advice (still at `test-police/SKILL.md:17,22`, known) and fix the `cd` | The skill orders a cold build: 2 × 7.4 GiB, 877 s |
| 62 (+70) | "Set the bash tool's `timeout` parameter to 1200; run builds in the foreground" | The current sentence reads as the denied `timeout` command; 3 agents in one run |
| 44 | Remove the contradiction between "rename" and "delete and flag" in `chronicler-comment-fixer` | The skill contradicts itself |
| 52 | The comment fixer's report lists each deleted reason comment | Output request; proven in C6 |
| 27 | Each review finding carries its evidence class | Output request; lets the lead triage instead of "apply all" |
| 10 | Each grilling question restates its fact and names its lever | Output request; the user is the check |
| 57 | Move the docs step after the last code step in both after-plan skills | Step order is structure, not vigilance |
| 54 | One sentence in `tests/STRATEGY.md`: a Given is a state a client can reach | A domain rule about specs, read when writing specs, not always loaded |
| 25 | One-line `tdd` description edit | A skill's description is its only trigger. Low value; verify with a fresh run or close |

### Decide with the user (5 decisions, 7 open tickets)

Root-cause questions. A rule at each symptom does not answer them. The
decisions also name closed tickets as evidence; those stay closed.

| Decision | Tickets | The question |
|---|---|---|
| D1 Always-loaded context (**won't do**, user 2026-10-25; ticket 09 set to `wontfix`) | 09 | Should the 300-line generated Structure block load in every run, when an agent can `ls` and `rg`? The v1/v2 variants are stale: the Development Loop was rewritten after them (39, 48). Only the v3 move changes context cost. This is measurable: run one task with and without the block |
| D2 Shared checkout (**C: accept**, user 2026-10-25; 01, 02, 13, 17, 32, 46, 47 set to `wontfix`) | 01, 13, 32, 46, 47, (02) | Many efforts edit one tree. Isolate each effort in a worktree, or accept the seams. Isolation removes 32's path list and 47's numbering problem; a path list (32) treats the symptom |
| D3 One home for a fact (**A: authority order**, no edit needed, user 2026-10-25; 42, 45, 81 set to `wontfix`) | 42, 45, 81 | A fact written in two places (Done-when and Answer; ticket and doc; plan test row and Content item) drifts. Make one place authoritative, or write it once |
| D4 Doc rules that keep failing (**won't do**, user 2026-10-25; A measured: 0 of 10 fix rounds saved; B and C rejected; 82 set to `wontfix`; 38 moved to mechanical) | 82 | 8 of 8 slices broke the same Diátaxis rules with the rule in the brief. Lint the mechanical subset, or relax the rule. A rule the model breaks every time may cost more than the review finds. 38: the `long comment runs` guardrail exists (known); re-check whether it already covers the case |
| D5 Verify skill edits with compute (**C: no**, user 2026-10-25; 16 set to `wontfix`; 17 moved to D2) | 16, 17 | Validate a skill edit with a fresh-context run (16). This is the bitter-lesson move: compute instead of the author's judgement. 17 is a fan-out convention; keep only if parallel fan-out is still in use |

## Upstream: the retro skill

The bucket grows faster than it drains (85 filed, about 20 dispositioned in
three weeks). The source is `/retro`. `.agents/skills/retro/SKILL.md` already
says "Default to building the check over writing the rule", but it has no
filter before filing. Every retro finds something, and a sentence is always
the cheapest option to write.

Proposed filter for step 4, before a candidate becomes a ticket:

1. Classify it: mechanical, wrong prose, output request, vigilance, slip.
2. Do not file a slip or a vigilance rule. Record it in the retro summary
   only.
3. File only if the problem recurred, or one occurrence cost more than one
   fix round.

## Outcome (2026-10-25)

The user closed every ticket in the Close table and every decision ticket. The closed ticket files are deleted. Their one-line gist and reason are in the map: Decisions so far, or Won't do. 30 tickets stay open: 21 mechanical and 9 prose.
