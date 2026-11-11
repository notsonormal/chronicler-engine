# Research 19: do the docs mislead the agents that read them

Status: reference. Asset of the ticket [Research whether the docs mislead the agents that read them](../issues/19-research-do-docs-mislead.md) in the [map](../map.md). Made on 2026-11-01.

This asset records what the data shows. It does not judge the doc layout and proposes no structure. It quotes no session transcript. It gives counts, paths, commits, session ids and tool-call indexes.

## Summary

- **Stale statements are rare in doc edits (known for the sample, inferred for the population).** Of the 230 (session, doc) pairs, about 8% (about 18 pairs) fix a statement that was wrong before the session started. The 95% limits are 3.5% to 20.7%. Part 3 gives the estimate.
- **Most edits record the change itself (known).** About 79% of the pairs record the behaviour that the session (66%), or the same planned unit of work (13%), changed. About 8% are renames or moves. About 6% are wording only.
- **No agent acted on a wrong statement (known, 8 of 8).** In each of the 8 stale cases, the agent found the error before it edited code or ran a build on the wrong statement. 7 of the 8 were found because a ticket, a review or an audit sent the agent to check the doc. 1 was found in passing.
- **Every stale case is in a Diátaxis doc (known).** 0 of 8 are in `docs/specs/`. The 8 cases are 6 Reference docs and 2 how-to docs. 33 of the 70 reviewed pairs are specs.
- **The two scripts catch 2 of 8 (known).** Today's `scripts/validate_docs.py` flags the line citations in 2 of the 8. `scripts/validate_feature_spec.py` catches 0 of 8. The other 6 are wrong names, wrong places and wrong behaviour.
- **Meaning.** C-03 holds in practice for the edits that agents make. C-11 would not have caught any of the 8 cases in specs, and a new check of a map would need a fact check to catch the 6 content cases. Part 6 has the reasons.

## 1. Method

### The pair list (known)

- The pairs come from the rule in part 3.4 of [research 10](./research-10-agent-use-of-docs.md): a code-work session, in the main session folder, that edits a spec or a Diátaxis doc, with a read of the same doc earlier in the session.
- `tmp/hex-compare/r10/sessions3.pkl` still exists. `tmp/hex-compare/r19/pairs.py` rebuilds the list from it. It gives **260 pairs in 104 sessions, and 230 pairs where the doc was read before the first edit.** These are the numbers of research 10. No pair was skipped: all 70 reviewed session files and doc paths resolve. Some docs have since moved or been deleted. That does not matter, because the work reads the edit, not the current file.
- The pickle holds the sessions as of about 20:15 UTC on 2026-11-01. A later session is not in the population.

### The sample (known)

A plain random sample of 40 pairs is too small to find a rare kind. So the sample has two parts.

| Part | Draw | Seed | Pairs | Script |
|---|---|---|---|---|
| 1 | Random sample of the 230 | `random.seed(19)` | 45 | `r19/sample.json` |
| 2 | Random sample of the flagged pairs that part 1 did not draw | `random.seed(1919)` | 25 | `r19/sample2.json` |

A pair is **flagged** when no source or test edit comes before the doc edit in the session, or when one word or fewer of the doc change also appears in those code edits. `r19/census.py` computes the flag for all 230 pairs. It is a screen only. It is not a classification.

- 86 pairs are flagged and 144 are not.
- Part 1 holds 14 flagged and 31 not flagged pairs. Part 2 adds 25 flagged pairs. So 39 of the 86 flagged pairs and 31 of the 144 other pairs were read: 70 pairs in 51 sessions.
- The screen has a known weakness (inferred): a stale statement can sit as a second edit inside a pair whose main edit records the session's change. The method records the main kind, so such a case is not counted.

### The kinds

For each pair I read the diff of the first edit and of the other edits, the tool calls from the doc read to the edit, and the brief. I dated a stale statement with `git log -S` on the doc text and `git log` on the source file that the statement names. I made the kind decision alone, without a second reader (limit).

| Kind | Meaning |
|---|---|
| (a) | The edit records the behaviour that this session changed. Sub-kind **a-planned**: the behaviour was changed earlier in the same unit of work, and the brief or the hand-off names the doc update. |
| (b) | The edit fixes a statement that was wrong before the session started, and no change of the session or of its unit made it wrong. Sub-kind **b-drift**: the statement was right once, and the code moved on. Sub-kind **b-born**: an agent wrote the statement wrong less than 4 days before. |
| (c) | A rename, a move of a block between docs, or an index regeneration. |
| (d) | Wording only, or a trim that follows a doc rule. No fact changes. |
| (e) | `unknown`. |

The ticket's kind (a) covers "the session's own new behaviour". I split off **a-planned**, because 8 of the 70 pairs are doc steps of a rebuild whose code earlier sessions had built (the step "docs of the core step", and the fix passes that came from the review of that step). Those docs were out of date against the working tree when the session began. The brief named the update. I do not count them as stale statements.

## 2. The kinds in the sample (known)

| Kind | Part 1 (45, random) | Part 2 (25, flagged) | Flagged pairs read (39) | Other pairs read (31) | Estimated share of the 230 |
|---|---|---|---|---|---|
| (a) own change | 30 | 12 | 18 | 24 | 66% |
| (a) a-planned | 8 | 0 | 3 | 5 | 13% |
| (b) stale | 2 | 6 | 8 | 0 | 7.7% |
| (c) rename or move | 3 | 4 | 6 | 1 | 7.8% |
| (d) wording | 2 | 3 | 4 | 1 | 5.9% |
| (e) unknown | 0 | 0 | 0 | 0 | 0% |

The share is the stratum sizes (86 and 144) applied to the stratum shares, divided by 230 (inferred). The a-planned pairs of part 1 are `#2`, `#16`, `#21`, `#29`, `#34`, `#35`, `#39` and `#41` (the index is the position in `sample.json`).

**What (e) = 0 means.** It does not mean that every pair was clear. Four pairs hold a rewrite, a split or a move where the kind is a judgement: part 1 `#26` and `#43`, part 2 `#1` and `#12`. I gave them (c) or (a) with the reason in the table of part 7.

## 3. The rate of stale statements

### The estimate (inferred, from a stratified sample)

| Stratum | Pairs in the 230 | Read | Stale (b) | Share of the stratum, 95% exact limits |
|---|---|---|---|---|
| Flagged | 86 | 39 | 8 | 20.5% (9.3% to 36.5%) |
| Other | 144 | 31 | 0 | 0% (0% to 11.2%) |

- **Point estimate.** `(86 × 8/39) / 230 = 7.7%`, which is about 18 pairs of 230.
- **Conservative 95% limits: 3.5% to 20.7%.** The lower limit takes the lower bound of the flagged stratum and 0 for the other. The upper limit takes the upper bound of each stratum.
- **Normal approximation with 0 stale pairs in the other stratum: 4.1% to 11.2%.**
- **Part 1 alone**, as a plain random sample: 2 of 45 (4.4%), Wilson 95% limits 1.2% to 14.8%.

The conservative limits are wide. The reason is the 144 pairs in the other stratum, where I read 31. The screen predicts few stale statements there, but it does not prove none (inferred).

### What the number counts and does not count

- It counts an edit that fixes a wrong statement, in a doc that the same session read earlier. A wrong statement that an agent read and did not fix is not in the data. The real share of stale statements that agents read is at least this share (known, from the method).
- The 7 sessions of the 8 stale cases are 7 of the 51 reviewed sessions (inferred: one session holds 2 stale pairs).
- An agent that reads a doc and finds an error more likely edits it. A pair where the agent read and edited is therefore not a random draw of all reads (inferred).

## 4. The 8 stale cases

`P1` is `sample.json`. `P2` is `sample2.json`. The index is the position in the file. Ages are from `git log` (known) or from the first commit that holds the text (inferred, because the statement may have been wrong from its first day).

| Case | Session | Doc | What was wrong (no quote) | Sub-kind | Age when fixed | How the agent found it |
|---|---|---|---|---|---|---|
| P1 #8 | `01a0ff7e` (2026-10-03) | `docs/diataxis/how-to/debugging.md` | Three source line numbers, and the name and place of the engine's database file | b-drift | The files changed on 2026-09-09 and 2026-09-27 (`e3ca2c00`, `83103583`). The doc was last edited on 2026-10-01. So 6 to 24 days (inferred). | A `/retro` audit whose brief asks for out-of-date docs. |
| P1 #18 | `01a0ff6f` (2026-10-03) | same doc | Source line numbers, and a note about a test helper that the doc says is a debugging aid | b-drift | Same files, so 6 to 24 days (inferred). | An after-plan session that starts from a report. |
| P2 #3 | `f55fe5bb` (2026-11-01) | `reference/narrative/agent_system.md` | What the Quantifier `type` field does | b-born | The text was first written by `3d3a61ed` at 00:10 UTC on 2026-11-01. The session ran at 18:59 UTC. So 19 hours. | A review step of the plan names the doc, and a plan decision (D2) says to fix the doc and not the code. |
| P2 #5 | `01a0ff60` (2026-10-03) | `reference/coding_standards/integration_test_standards.md` | The place of a test helper, and a statement that sync failures use the nextest `--retries` option | b-drift | `--retries` was in the test setup until `b8d9e498` (2026-09-20), so 13 days or more (inferred). | An after-plan code review lists two doc items. The agent checks each one against the code (calls 118 to 124) and then edits. |
| P2 #6 | `01a0f3bf` (2026-09-30) | `reference/coding_standards/unit_test_standards.md` | 12 statements: helper names, module paths, a file count, method names | b-drift | The doc was last edited on 2026-09-14 (`9c51803d`). The code changes are from August to September (inferred). | Ticket 33 ("make the test standards docs match the code") is an audit. |
| P2 #13 | `01a0ff6f` (2026-10-03) | `reference/coding_standards/integration_test_standards.md` | The source file that holds two test helpers | b-born | `9c446198` (2026-09-30, the commit of ticket 33) wrote the line. So 3 days. | The same after-plan session as P1 #18. |
| P2 #14 | `01a1974b` (2026-11-01) | `reference/narrative/prompt_system.md` | Which preset a call uses, how the `{{…}}` filter reads, and what "default preset" allows | b-drift | The text is from the first commit of the doc (`d8644394`, 2026-08-11). So at most 82 days (inferred). Commit `42002bd8` corrects it. | Ticket 24 ("correct the docs") from a research asset. The agent checks each fact in the code first (calls 9 to 16). |
| P2 #17 | `01a09772` (2026-09-12) | `reference/frontend/dashboard.md` | The slash palette lists two commands. The code lists three since `c14da461` (2026-09-11 22:58 +01:00). | b-drift | About 22 hours. | In passing, in a phase that sweeps prose references. 1 of the 8. |

Facts that hold across the 8 (known unless marked):

- **Where.** 6 Reference docs and 2 how-to docs. 0 specs. In the 70 reviewed pairs: 33 spec pairs, 29 Reference pairs and 8 other Diátaxis pairs.
- **Who.** 7 of 8 are non-delegate sessions. 1 is an implementer delegate (P2 #3).
- **How found.** 7 of 8 were found because the task was an audit, a review item or a doc ticket. 1 of 8 was found in passing.
- **Type.** 2 are born wrong, within 4 days of the edit. 6 are drift, from 22 hours to at most 82 days.
- **A related fact outside the sample (known).** Ticket 33's answer says that its first rewrite of the test standards introduced five false claims, and that a code review found and fixed them ([the ticket](../../dashboard-ui-review/issues/33-fix-test-standards-drift.md)). So agent-written doc text is itself a source of wrong statements.

## 5. Did the wrong statement change what the agent did (known)

**Answer: not in the 8 cases.**

For each case I read the tool calls from the first read of the doc to the edit, and the tool results that failed.

| Case | Doc read at call | Edit at call | Failed tool results in or before the window | Action on the wrong statement |
|---|---|---|---|---|
| P1 #8 | 119 | 129 | none | none |
| P1 #18 | 1 | 19 | none | none |
| P2 #3 | 2 | 46 | call 44 (a build that fails because a parallel phase has left the tree broken) | none |
| P2 #5 | 121 to 130 | 127 | none in the window (calls 25 and 42 are before the first doc read at 70) | none |
| P2 #6 | 5 | 35 | call 43 (a guardrails run that timed out) | none |
| P2 #13 | 0 | 18 | none | none |
| P2 #14 | 10 | 17 | calls 2 and 3 (shell commands of the first look at the ticket) | none |
| P2 #17 | 17 | 100 | call 20 (a shell command) and call 89 (an edit of a validator script with its two sides swapped) | none |

The failed calls do not touch the wrong statement (inferred, from their position and from the assistant text beside them). In the cases that came from an audit or a review, the agent checked each claim in the code before it edited the doc.

**A weak signal from outside the sample (inferred).** The test-standards docs named a builder that did not exist as the pattern to use. Ticket 33 found it. In the window between the removal of the builder (`83103583`, 2026-09-27) and the fix (`9c446198`, 2026-09-30), the sessions that mention `SqliteTestAppBuilder` in a tool call are the audit and the fix sessions of ticket 33 only (the search of all main-folder sessions in `sessions3.pkl`). No implementer session wrote code against it in that window. This is absence of evidence. The sessions of that window were few, and a wrong name in a guardrail message would show as a failed build, which this search does not look for.

## 6. Could the two scripts have caught the 8 cases (known)

| Script | Rule | Caught |
|---|---|---|
| `scripts/validate_docs.py` | `LINE_CITATION` flags a backticked `file.rs:42` | **2 of 8**: P1 #8 (3 citations in the old text) and P1 #18 (2). The regex was run on the old text of each edit. |
| `scripts/validate_docs.py` | `BROKEN_SOURCE_PATH` flags a backticked `src/`, `tests/`, `scripts/` or `assets/` path that does not exist | 0 of 8. The paths in the old texts of the 8 edits exist today, and the broken paths of ticket 33's list are not in these edits. |
| `scripts/validate_docs.py` | `UNKNOWN_ROUTE`, broken links, front matter | 0 of 8. |
| `scripts/validate_feature_spec.py` | Each scenario has a test, each tag has a scenario, and the surface rule | 0 of 8. 0 of the 8 are specs, and the script does not read prose. |

- The `LINE_CITATION` rule was added in `2032470a` on 2026-10-03T11:14Z. That is the commit that holds the doc fix of P1 #8. The two cases were fixed, and the rule was added, on the same day (known from `git log -S`). `BROKEN_SOURCE_PATH` followed in `ab708744` on 2026-10-04.
- Both cases also held a statement that no script can check: a file name for the database (P1 #8) and a note on a helper (P1 #18).
- So **2 of 8 are in the class that a script catches today, and 6 of 8 need a fact check against the code.** The 6 are wrong names and places (P2 #5, #6, #13), wrong behaviour (P2 #3, #14) and a list that has not grown with the code (P2 #17).

## 7. What this means for C-03 and C-11

The ticket does not ask for a layout or a structure, and this part does not give one.

### C-03: docs change with the behaviour, in both directions

- **The rule is followed by the sessions that act (known).** About 79% of the read-then-edited pairs record the change of the session or its unit. The 33 spec pairs that I read show no stale statement. All 8 stale cases are outside specs.
- **The rule does not reach the stale cases (inferred).** A rule that says "the change edits the doc that the change makes false" cannot act on a doc that no change names. P1 #8, P1 #18, P2 #5, #6 and #14 went stale by drift over 6 to at most 82 days, in how-to and test-standards docs that copy facts: line numbers, helper names, file counts and option names. Ticket 33 pruned such copies, and the `docs/AGENTS.md` rule "Values stay in the code" covers the same class.
- **Cost seen so far (known): no agent action on a wrong statement, 0 of 8.** The cost found is the audit work: ticket 33, ticket 24, and the retro and after-plan sessions. This research cannot say how many wrong statements agents read and did not meet (part 3).
- **A second source (known).** Two of the 8 were born wrong less than 4 days earlier, and ticket 33's first rewrite added five. The practice "docs change with the behaviour" does not make new text right.
- **Recommendation for ticket 20 (inferred).** Keep C-03 as covered. If a new rule is wanted, the data supports a narrow one for fact copies in prose docs, and not a rule about the timing of the doc edit.

### C-11: a mechanical check on the doc map

- **A check of a spec map would not have caught any of the 8.** `spec-coverage` already exists, and 0 of 8 are in specs.
- **A check on ids and names has a ceiling.** The class that scripts catch today is 2 of 8 (line citations, now closed). The class that remains is 6 of 8: the statement is a fact about code. A map check on ids, links or test tags would not see it. A check for it would have to compare a statement with the code. This repository has no such check, and I did not find one in Hex (research 04: 11 of 611 named checks no longer resolve there).
- **The rate is low but not zero.** About 8% of the doc edits (3.5% to 20.7%) fix a statement that was wrong before. Whether a feature map, if adopted, changes that rate is not measured here.
- **Recommendation for ticket 20 (inferred).** Keep C-11 as covered and carry the constraint "any adopted map gets a mechanical check" with the limit above. A check on a new map would protect the link between ids and tests, and not the truth of the statements.

## 8. Limits and what I did not do

| Limit | Effect |
|---|---|
| One reader made the kind decisions, without a second reader. | The kind of the 4 judgement pairs (part 2) may differ for another reader. A change of those four does not change the (b) count. |
| The kind of 31 other pairs rests on the screen and on the diffs, not on a read of every tool call. | A stale statement inside such a pair can be missed. The upper limit of the rate allows for this (part 3). |
| Only a doc that the session read and then edited is in the data. | The share of wrong statements that agents read and did not fix is not known. |
| The age of a statement is from `git log -S` on text, or from the date of the code change. | "At most 82 days" for P2 #14 is an upper bound. The statement may have been wrong from the first day. |
| The 8 a-planned pairs depend on a brief or a hand-off that names the doc update. | If the owner counts the a-planned pairs as stale, the (b) share rises from about 8% to about 21% (the a-planned share is about 13%, inferred). I do not count them, because the doc lagged a working tree that the brief already named, and no session read it as a finished doc. |
| The data does not show whether a wrong statement made a session slower. | A detour of a few tool calls does not show in the counts. |
| The 230 pairs exclude delegates' reads by `codemode`, reads by `git show`, and `bash` edits that the method misses (research 10, part 1). | The population is a lower bound. |
| No session text is quoted in this asset. | The reader can use the call indexes with `r19/ctx.py` to read a window. |

## 9. The pairs

`P1 #n` is the position in `r19/sample.json`. `P2 #n` is the position in `r19/sample2.json`. The kind is the main kind. "Calls" are the doc edit call indexes. The reason is the evidence for the kind.

| Pair | Session | Doc | Kind | Calls | Reason |
|---|---|---|---|---|---|
| P1 #0 | `01a12cf6` | `reference/game_flow.md` | a | 142 | The session wrote the room-name behaviour in code (calls 86 to 117) and then the doc. |
| P1 #1 | `01a02ed3` | `specs/retrigger.md` | a | 376 | The session removed a Message variant (calls 357 to 374), then the spec words. |
| P1 #2 | `01a15033` | `specs/swipe_new.md` | a-planned | 342 | Hand-off of the C4 switch names the specs as part of the work. |
| P1 #3 | `01a12208` | `specs/worlds.md` | a | 104 | Status code of the world create failure, changed by the same plan. |
| P1 #4 | `01a07704` | `specs/swipe_new.md` | a | 355, 359 | The session flattened the stored swipe inputs (`45033a6e`). |
| P1 #5 | `01a11da4` | `reference/frontend/dashboard.md` | a | 55 | The session deleted scenario 31.7 (calls 50 to 53). |
| P1 #6 | `01a0d514` | `reference/coding_standards/integration_test_standards.md` | a | 459, 854 | The session removed `--settings-path` (many code edits). Second kind: a citation of line numbers removed. |
| P1 #7 | `01a103d2` | `specs/browser_story_log.md` | a | 215 | New focus checks added with the code. |
| P1 #8 | `01a0ff7e` | `how-to/debugging.md` | b-drift | 129 | Part 4. |
| P1 #9 | `01a12208` | `reference/frontend/ui_design.md` | a | 132 | Tooltip text of the same plan. |
| P1 #10 | `01a0f8af` | `specs/worlds.md` | a | 152 | Storage change in the same ticket. |
| P1 #11 | `01a12385` | `reference/frontend/dashboard.md` | a | 63 | A new `wait` state in the code of the session. |
| P1 #12 | `01a09642` | `specs/games.md` | a | 208 | The session wrote the specs and the tests. |
| P1 #13 | `01a12666` | `specs/failure_display.md` | a | 52 | The session's change to a reply without content. |
| P1 #14 | `01a0f3fd` | `specs/prompt_presets.md` | a | 70, 98, 105, 120 | Add forms of the ticket, edited more than once as the design settled. |
| P1 #15 | `01a076b1` | `reference/coding_standards/integration_test_standards.md` | a | 30 | The session restructured `build.py`; the doc was right before it (`git show f3647e37^:build.py`). Same commit as the doc change. |
| P1 #16 | `01a1604c` | `reference/game_flow.md` | a-planned | 18 | After-plan step "update the docs that the changes make false", after the autoplay branch work. |
| P1 #17 | `01a1086c` | `specs/browser_dashboard.md` | a | 75 | New scenarios of the ticket. |
| P1 #18 | `01a0ff6f` | `how-to/debugging.md` | b-drift | 19 | Part 4. |
| P1 #19 | `01a0f40f` | `specs/browser_story_log.md` | a | 53, 89 | Edit-mode ticket. |
| P1 #20 | `63ee340e` | `specs/failure_display.md` | a | 197 | A scenario that the session wrote, corrected after a test run (call 196). The wrong text is the session's own. |
| P1 #21 | `01a15033` | `specs/dashboard.md` | a-planned | 144 | Same as P1 #2. |
| P1 #22 | `3127c4ac` | `specs/settings.md` | a | 41, 58 | Context window field built by the delegate. |
| P1 #23 | `01a0f946` | `specs/games.md` | a | 142 | Follow-up ticket with code edits. |
| P1 #24 | `01a0017d` | `reference/game_flow.md` | c | 255 | A heading rename after the session replaced the `INV-` ids with generated docs. |
| P1 #25 | `01a12280` | `explanation/dashboard_design.md` | a | 126 | The JavaScript file split in the same plan. |
| P1 #26 | `01a1267c` | `reference/storage.md` | c | 172 | A link description renamed after the session deleted `ui_design.md`. The later full rewrite (call 389) left no change in the landing diff (`a58af5b9`). |
| P1 #27 | `01a19471` | `explanation/prompt_system_design.md` | d | 25 | A sentence split. |
| P1 #28 | `01a05437` | `reference/narrative/ai_steering.md` | a | 70 | The session removed a steering surface. |
| P1 #29 | `ec2e69f8` | `reference/storage.md` | a-planned | 56 | The delegate builds the docs of the core step. |
| P1 #30 | `01a07704` | `reference/storage.md` | a | 179, 299 | Same as P1 #4. |
| P1 #31 | `01a12222` | `reference/game_flow.md` | a | 175 | New failure type in the code of the session. |
| P1 #32 | `01a11878` | `specs/dashboard.md` | a | 138, 303 | New icon scenario with the code. |
| P1 #33 | `01a103d2` | `specs/browser_worlds.md` | a | 167, 235 | New focus scenario with the code. |
| P1 #34 | `01a14c0c` | `reference/startup.md` | a-planned | 130 | Slice of the rebuild plan. |
| P1 #35 | `ec2e69f8` | `reference/game_flow.md` | a-planned | 53 | Same as P1 #29. |
| P1 #36 | `01a076b1` | `how-to/debugging.md` | a | 28 | Same as P1 #15. |
| P1 #37 | `01a1086c` | `reference/frontend/dashboard.md` | a | 76 | New section for the ticket. |
| P1 #38 | `01a0017d` | `explanation/architecture.md` | c | 253 | A link and its description changed after the guardrails doc was regenerated. |
| P1 #39 | `01a14c0c` | `specs/actions.md` | a-planned | 396 | Slice of the rebuild plan. |
| P1 #40 | `01a122c2` | `reference/coding_standards/integration_test_standards.md` | a | 265 | The session added a second provider key. |
| P1 #41 | `01a14c0c` | `reference/storage.md` | a-planned | 129 | Slice of the rebuild plan. |
| P1 #42 | `01a10889` | `reference/frontend/ui_design.md` | d | 108 | A trim to follow the doc rule, named by a review finding in the brief. |
| P1 #43 | `01a103d2` | `specs/browser_prompt_presets.md` | a | 237, 239 | The session's own scenario (the file was written whole at call 239). |
| P1 #44 | `01a1125b` | `explanation/two-state-channels.md` | a | 114 | The registry change of the session. |
| P2 #0 | `01a12660` | `reference/narrative/narration_system.md` | a | 73, 111 | Doc first, then 12 code edits for the reply without content. |
| P2 #1 | `01a09772` | `specs/options.md` | c | 53 | Scenario blocks moved to per-surface specs. |
| P2 #2 | `01a14a01` | `specs/settings.md` | a | 145 | New scenario of the rebuild plan. |
| P2 #3 | `f55fe5bb` | `reference/narrative/agent_system.md` | b-born | 46 | Part 4. |
| P2 #4 | `f1641e19` | `reference/storage.md` | d | 75 | A review rule: a reason moves to another layer, and one word is replaced. |
| P2 #5 | `01a0ff60` | `reference/coding_standards/integration_test_standards.md` | b-drift | 127 | Part 4. |
| P2 #6 | `01a0f3bf` | `reference/coding_standards/unit_test_standards.md` | b-drift | 35, 42, 48, 66, 73 | Part 4. |
| P2 #7 | `01a1076d` | `specs/browser_options.md` | a | 111 | Scenario rewrite with the code. |
| P2 #8 | `f1641e19` | `specs/browser_dashboard.md` | c | 87 | One word renamed in three places (a review item). |
| P2 #9 | `01a12242` | `specs/worlds.md` | a | 197 | A scenario of the same plan, given its heading. |
| P2 #10 | `01a12208` | `specs/prompt_presets.md` | a | 103 | Status codes of the same plan. |
| P2 #11 | `01a0735e` | `specs/prompt_presets.md` | d | 114 | A scenario's wording follows a stronger test (a review comment). |
| P2 #12 | `01a09772` | `specs/prompt_presets.md` | c | 57 | A scenario block moved out. |
| P2 #13 | `01a0ff6f` | `reference/coding_standards/integration_test_standards.md` | b-born | 18 | Part 4. |
| P2 #14 | `01a1974b` | `reference/narrative/prompt_system.md` | b-drift | 17, 18 | Part 4. |
| P2 #15 | `01a1743c` | `reference/frontend/dashboard.md` | a | 247 | The cleanup that the session committed made the sentence false (call 241), and the session fixed it. |
| P2 #16 | `01a1267c` | `reference/narrative/options.md` | c | 170, 359, 360 | A link description renamed after the session deleted a doc. |
| P2 #17 | `01a09772` | `reference/frontend/dashboard.md` | b-drift | 100 | Part 4. |
| P2 #18 | `01a14c0c` | `specs/story_log.md` | a | 291, 361, 401 | Rebuild code of the session. |
| P2 #19 | `01a14b68` | `specs/options.md` | a | 148, 149, 298 | Rebuild code of the session. |
| P2 #20 | `01a0ee72` | `specs/browser_dashboard.md` | a | 55 | The session fixed the repeating failure box. |
| P2 #21 | `01a12ae9` | `specs/options.md` | d | 9, 28 | A spec cleanup plan: terms only. |
| P2 #22 | `01a12265` | `explanation/dashboard_design.md` | a | 81 | The session moved the inline script to `assets/dashboard.js` (calls 76 and 77). |
| P2 #23 | `01a12cf6` | `specs/actions.md` | a | 84, 164 | New scenario 1.14; second kind: wording. |
| P2 #24 | `01a10354` | `specs/browser_story_log.md` | a | 31 | New scenario 30.10 with the test. |

## 10. Sources

- Session files: `~/.pi/agent/sessions/--workspace-chronicler-engine--/`. The pair list and the screen are in `tmp/hex-compare/r19/` (git-ignored): `pairs.py`, `census.py`, `sample.json`, `sample2.json`, `dump.py`, `brief.py`, `ctx.py`, `land.py`, `overlap.py`. `README.txt` there names each script.
- Commits cited: `3d3a61ed`, `4c0811b4` (agent_system.md), `9c446198`, `2032470a`, `ab708744`, `83103583`, `e3ca2c00`, `b8d9e498`, `c14da461`, `42002bd8`, `d8644394`, `f3647e37`, `a58af5b9`, `45033a6e`, `2f4d787d`.
- Scripts read: `scripts/validate_docs.py` (`LINE_CITATION`, `SOURCE_PATH_TOKEN`, `classify_file`), `scripts/validate_feature_spec.py` (docstring).
- Other: [research 10](./research-10-agent-use-of-docs.md), [candidates](./candidates.md) (C-03, C-11), the closed ticket `.scratch/_closed/dashboard-ui-review/issues/33-fix-test-standards-drift.md`.
