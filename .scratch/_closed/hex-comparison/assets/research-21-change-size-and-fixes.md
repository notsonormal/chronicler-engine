# Research 21: Do large changes need more fixes?

Status: reference

Sources: the git history of this repo (HEAD `dab127b5`, 2026-11-02, 881 non-merge commits and 28 merges) and of the Hex clone `tmp/hex-compare/anomalyco-hex/` (HEAD `7609574ca92c`, 2026-10-31, 1108 non-merge commits and 38 merges). Scripts and raw results are in `tmp/hex-compare/r21/`. No code, doc or git state was changed.

## Summary

**Known.** The share of commits that a later fix repairs grows fast with size. In Chronicler it goes from 13% (under 100 lines) to 95% (3000 lines or more). In Hex it goes from 24% to 100%.

**Known.** The repaired lines per 1000 changed lines do not grow with size. In Chronicler the five buckets give 19.8, 39.3, 20.4, 14.8 and 18.1. In Hex they give 31.0, 42.6, 22.1, 15.0 and 21.4. A commit of 1000 lines or more has a rate of 0.67 times the rate of a smaller commit in Chronicler (95% limits 0.38 to 1.13). In Hex the ratio is 0.64 (0.41 to 1.00).

**Inferred.** The data do not show that a large commit needs more fixes per line. They also do not show that it needs fewer. The cause of the high share is the number of lines, not a higher rate for each line.

**Known.** One cost does grow with size: the number of later fix commits for each landed commit. In Chronicler it goes from 0.16 (under 100 lines) to 4.3 (3000 lines or more).

**Known.** One subset shows a rising rate. Chronicler `feat` commits give 4.9, 14.2 and 23.0 repaired lines per 1000 for the buckets 100–299, 300–999 and 1000–2999. It is a subset that I chose after I saw the first tables, so chance can explain it (inferred).

**Known.** The largest 10% of Chronicler commits (88 commits, 1801 code lines or more) hold 58% of all changed code lines. Only 19 of the 88 are features. 22 are batches of several tickets or findings. 43 are refactors, migrations or moves of code.

## 1. Method

### 1.1 Population and size

**Known.** The size of a commit is the added plus the deleted lines in code files (`git log -M -l0 --numstat`). A commit with no code line is not in the size tables, and it is counted below.

| | Chronicler | Hex |
|---|---|---|
| Code files | `.rs`, `.html`, `.js`, `.css` under `src/` or `tests/`, and under the old prefix `chronicler_engine/src` and `chronicler_engine/tests` (the code lived there until `d8644394`, 2026-08-11) | every `.rs` file except `vendor/` (`src/`, `crates/`, `tests/`, `build.rs`) |
| Non-merge commits | 881 | 1108 |
| With code lines | 517 | 803 |
| No code line | 364 | 305 |
| Code lines, all commits (added + deleted) | 503,499 (300,739 + 202,760) | 566,642 (414,972 + 151,670) |
| Docs and tickets (`.md`, `docs/`, `.scratch/`, `old-docs/`) | 386,581 | 46,944 |
| Other text (scripts, data, `.py`, `.ts`, `.sh`, `.swift`, `.toml`) | 90,940 | 94,600 |
| Excluded: `Cargo.lock` and other lock files | 10,332 | 32,212 |
| Excluded: generated index blocks (`AGENTS.md`, `docs/AGENTS.md`, `tests/AGENTS.md`) | 8,165 | 5,179 |
| Excluded: images | 574 text lines (binary files give no count) | 99 text lines (1359 `.png` files give no count) |
| Excluded: `vendor/` (added, then deleted) | none | 168,288 (84,144 + 84,144) |

**Known.** The size with docs, for the commits that have any text line: Chronicler median 409 and p90 2954 (859 commits). Hex median 131 and p90 1349 (1085 commits). The size in code files only: Chronicler median 471 and p90 2392 (517 commits). Hex median 175 and p90 1488 (803 commits). The 59 Hex pipeline commits have median 80 over all files, as in research 02. The figures 377.5 and 3049 in the ticket come from a count that I could not reproduce. My count over all files gives 405.5 and 2995, so the gap is small. I did not find the cause.

**Known.** The buckets are 1–99, 100–299, 300–999, 1000–2999 and 3000 or more code lines. The tables use the commits up to 14 days before the last commit of each repo (Chronicler 2026-10-19, Hex 2026-10-17). That leaves 439 of 517 Chronicler commits and 196 of 803 Hex commits. The Hex project is 3.5 months old and most of its commits are new. Section 3 shows the tables without the cut-off.

### 1.2 Fix commits

**Known.** Rule: the subject starts with `fix` or `Revert`, in any case, and the next character is not a letter (`^(fix|revert)(?![a-z])`). This rule also takes `Fix #N:` (the Hex pipeline form) and `Fix the name ...`.

| | Chronicler | Hex |
|---|---|---|
| Fix commits | 68 (67 `fix`, 1 `Revert`), 7.7% of commits | 240 (225 `fix`, 4 `Revert`, 11 `fix(ci)`), 21.7% of commits |
| With code lines | 56 | 210 |
| No code line (left out of attribution) | 12 | 30 |
| With code lines but no deleted or changed line (pure additions, no blame) | 5 | 22 |

**Known.** Hand check of 20 random matches and 20 random non-matches for each repo (seed 21, list in `tmp/hex-compare/r21/`). I read the subject, the body and the file list.

| | Matches that are not a repair of a fault | Non-matches that are a fix |
|---|---|---|
| Chronicler | 2 of 20 (10%): `6b678175` (dead-code removal), `dfb81eb8` (rename and restructure) | 2 of 20 (10%): `75919da6` (build fixes), `7adaf39a` (build fix). Each changes 1 code line or none. |
| Hex | 4 of 20 (20%): `10cbda49`, `6815b6ee`, `49391d47` (animation and design changes), `3607595d` (a `Fix #N:` title for a feature request) | 2 of 20 (10%): `bead7484` (Windows caption icon), `614d52a7` (compromised root certificates) |

**Inferred.** A sample of 20 gives wide limits: 2 errors in 20 is consistent with 1% to 30%. The rule misses fixes under a feature title (Hex pipeline commits for issues) and it takes some design changes. I did not fix the rule, so the two errors partly cancel. A search for the words `fix`, `bug`, `regress`, `repair`, `broken` in the subjects of non-matches finds 69 commits in Chronicler (47 with code) and 23 in Hex (17 with code). I did not sort them.

### 1.3 Attribution

**Primary method, known (SZZ).** For each fix commit with code lines, I took the lines that the fix deletes or changes in code files (`git diff -w --ignore-blank-lines -M -U0` of parent against fix, files with status M or R). I removed three kinds of line: blank lines, comment-only lines, and lines that `git diff --color-moved` marks as moved. Then I ran `git blame -w -M -C` on the parent for the remaining lines. Each blamed non-merge commit is a commit that the fix repairs. Skipped lines:

| | Chronicler | Hex |
|---|---|---|
| Moved | 1196 | 602 |
| Comment-only | 1133 | 287 |
| Blank | 534 | 56 |
| Blame failures | 0 | 0 |
| Lines blamed | 9066 | 4308 (5 lines in 2 pairs blamed to a commit outside the list, dropped) |

**Known.** Added and deleted files in a fix commit give no blame (the diff filter is `MR`). A repaired commit can be a fix commit itself.

**Check, known.** I looked at 6 random pairs for each repo (fix subject, repaired subject, file, time gap). In all 12 the file and the subjects fit. I did not read the 12 diffs. For example `fcd76b9f` repairs `971cc4a0` in `tests/browser/stub/action_controls.rs`, one day later.

**Cross-check, known.** Commit B counts as repaired when a later fix commit with at least one code line changes one of the same code files within 14 days of B. This method over-counts, because every fix commit marks all earlier commits that touched a file in the window.

## 2. Result

The columns: commits in the bucket; changed code lines; share of commits that a later fix repairs (SZZ); repaired lines per 1000 changed lines with 95% bootstrap limits; cross-check share; later fix commits for each commit (SZZ). The limits treat each commit as independent. One fix commit repairs many commits at once, so the true limits are wider.

### 2.1 Chronicler (439 commits with code up to 2026-10-19)

| Bucket (code lines) | Commits | Changed lines | Repaired by a later fix | Repaired lines per 1000 (95% limits) | Cross-check share | Fix commits per commit |
|---|---|---|---|---|---|---|
| under 100 | 87 | 3,086 | 11 (12.6%) | 19.8 (8 to 33) | 43.7% | 0.16 |
| 100–299 | 92 | 17,321 | 39 (42.4%) | 39.3 (17 to 68) | 51.1% | 0.53 |
| 300–999 | 127 | 71,711 | 87 (68.5%) | 20.4 (13 to 30) | 66.9% | 1.31 |
| 1000–2999 | 112 | 196,706 | 95 (84.8%) | 14.8 (11 to 19) | 76.8% | 2.58 |
| 3000 or more | 21 | 116,252 | 20 (95.2%) | 18.1 (6 to 43) | 71.4% | 4.33 |
| All | 439 | 405,076 | 252 (57.4%) | 17.8 | | |

### 2.2 Hex (196 commits with code up to 2026-10-17)

| Bucket (code lines) | Commits | Changed lines | Repaired by a later fix | Repaired lines per 1000 (95% limits) | Cross-check share | Fix commits per commit |
|---|---|---|---|---|---|---|
| under 100 | 72 | 3,287 | 17 (23.6%) | 31.0 (10 to 63) | 61.1% | 0.35 |
| 100–299 | 60 | 11,096 | 33 (55.0%) | 42.6 (26 to 61) | 78.3% | 1.25 |
| 300–999 | 38 | 18,544 | 23 (60.5%) | 22.1 (11 to 36) | 89.5% | 1.34 |
| 1000–2999 | 18 | 31,023 | 16 (88.9%) | 15.0 (7 to 24) | 94.4% | 3.67 |
| 3000 or more | 8 (too small to compare) | 61,376 | 8 (100%) | 21.4 (not computed) | 100% | 21.0 |
| All | 196 | 125,326 | 97 (49.5%) | 22.0 | | |

**Known.** The cross-check share is higher than the SZZ share in the small buckets of both repos (Chronicler under 100 lines: 43.7% against 12.6%), and lower in the large Chronicler buckets (3000+: 71.4% against 95.2%). Both methods rise with size. The cross-check count of linked fix commits per 1000 lines falls with size (Chronicler 21.1, 5.5, 2.6, 1.5, 0.5; Hex 70.6, 31.2, 14.9, 5.2, 3.2). A fix commit is a unit of work, and it does not grow with the size of the repaired commit.

**Known.** The 3000+ bucket of Hex has 8 commits. They are mostly snapshot commits: `b1671672` (ship HEX 2.0.9, 17,453 lines), `64a340a8` (ship HEX v2 desktop app) and `df2edf84` (Hex 2.2.0). A few of them draw most fixes: `b1671672` has 56 later fix commits. The import `fb617e50` (`scrawl-capture`, 39,162 lines, 2026-10-25) is in the no-cut-off set only.

### 2.3 Does the fix rate per line grow with size?

**Known.** No, not in the pooled rate. The table below gives repaired lines per 1000 changed lines in the five buckets, for the main table and for each check.

| Variant | Repo | under 100 | 100–299 | 300–999 | 1000–2999 | 3000+ |
|---|---|---|---|---|---|---|
| Main table (cut-off) | Chronicler | 19.8 | 39.3 | 20.4 | 14.8 | 18.1 |
| | Hex | 31.0 | 42.6 | 22.1 | 15.0 | 21.4* |
| No cut-off (all commits) | Chronicler | 17.2 | 34.6 | 20.3 | 17.0 | 16.2 |
| | Hex | 15.3 | 13.4 | 8.2 | 6.3 | 6.9 |
| Commits that are not refactors | Chronicler | 21.1 | 48.1 | 19.8 | 19.0 | 29.0* |
| | Hex | 38.7 | 57.4 | 29.7 | 16.2 | 23.0* |
| Refactor commits only | Chronicler | 11.4* | 18.4 | 21.1 | 8.8 | 8.5 |
| | Hex | 2.8* | 3.9* | 4.6* | 2.6* | 3.4* |
| Only fix commits of 300 code lines or less count | Chronicler | 7.5 | 4.0 | 2.7 | 1.5 | 1.0 |
| | Hex | 18.3 | 21.5 | 14.8 | 3.5 | 9.3* |
| Fixes in the same merged branch left out | Chronicler | 19.8 | 37.5 | 20.1 | 14.0 | 18.1 |
| | Hex | 31.0 | 42.6 | 22.1 | 15.0 | 21.4* |
| Fixes within one day of the repaired commit left out | Chronicler | 11.3 | 35.5 | 19.8 | 13.9 | 18.0 |
| | Hex | 29.8 | 33.5 | 9.2 | 8.4 | 19.2* |
| Repaired lines in test files left out | Chronicler | 7.5 | 5.8 | 6.1 | 5.2 | 4.9 |
| | Hex | not possible, the tests are in `src/` | | | | |
| `fix(ci)` fixes left out | Hex | 31.0 | 42.6 | 22.1 | 15.0 | 21.3* |
| Commits sized with docs and tickets | Chronicler | 24.6 | 24.2 | 25.7 | 17.4 | 15.6 |
| | Hex | 36.2 | 42.3 | 17.8 | 18.6 | 20.5 |

`*` The bucket has fewer than 10 commits, and it is too small to compare.

**Known.** Two groups, 1000 lines or more against less than 1000 lines, main table:

| | Under 1000 lines | 1000 lines or more | Ratio (95% limits) |
|---|---|---|---|
| Chronicler | 23.9 per 1000 (306 commits) | 16.0 per 1000 (133 commits) | 0.67 (0.38 to 1.13) |
| Hex | 29.9 per 1000 (170 commits) | 19.2 per 1000 (26 commits) | 0.64 (0.41 to 1.00) |
| Chronicler, no cut-off | 22.9 (352) | 16.7 (165) | 0.73 (0.45 to 1.17) |
| Hex, no cut-off | 10.1 (690) | 6.7 (113) | 0.66 (0.40 to 1.07) |

**Known.** One exception. Chronicler `feat` commits (91 in the cut-off set) give 4.9 (100–299, 12 commits), 14.2 (300–999, 25) and 23.0 (1000–2999, 42) repaired lines per 1000. The 95% limits of the first and the last bucket are 0 to 12 and 15 to 34, so they do not overlap. Hex `feat` commits fall with size (under 100: 47.7, 100–299: 40.6, 300–999: 42.3, 1000–2999: 19.3). **Inferred:** I looked at about nine subsets, so one of them can rise by chance.

**Known.** A rank correlation of size against repaired lines per line is positive for each commit (Chronicler 0.38, Hex 0.26). **Inferred:** this number is biased. Small commits are mostly at zero repaired lines, and the few that are repaired have a high ratio. A constant rate per line gives a positive rank correlation. I use the pooled rate.

**Known.** The pooled rate over all buckets is similar: Chronicler 17.8 and Hex 22.0 repaired lines per 1000 (cut-off). Hex has smaller commits and no lower rate.

**Known.** The commits are of similar age in every bucket (median age 113 to 136 days in Chronicler, 65 to 101 in Hex), so the time to get a fix does not explain the trend.

### 2.4 Limits of the method

- **The fix rule is wrong in 10% to 20% of the matches (known)** and misses fixes under feature titles (known). A fix commit can mix a fault repair with a refactor. 20 of 56 Chronicler fix commits with code have more than 300 lines. The row "only fix commits of 300 code lines or less count" in section 2.3 shows that this changes the level a lot (Chronicler 1.0 to 7.5 per 1000 in the buckets, against 14.8 to 39.3 in the main table; Hex 3.5 to 21.5 against 15.0 to 42.6). In that row the rate falls with size in both repos.
- **Repaired lines are not faults (known).** One fault can repair 1 line or 200. SZZ counts lines that a later commit changed. It does not tell the cause. A repair of a test (an assertion that a later commit rewrote) counts as a repair.
- **70% of the repaired Chronicler lines are in test files (known)**: 5069 of 7220. Nine of the 10 files with most repaired lines are test files. Hex keeps its tests inside `src/` files, so I cannot make this split for Hex. The two repos are not comparable on this point.
- **Hex is young (known).** Without the cut-off, 76% of the Hex commits with code (607 of 803) are less than 14 days old, and the median age of a bucket is 6 to 8 days. With the cut-off, 196 commits remain. They include the first import commits of July. The 3000+ bucket has 8 commits.
- **Squash merges hide review fixes (inferred).** Only 25 of 68 Chronicler fix commits and 21 of 240 Hex fix commits sit on a merged branch. I counted fixes in the same branch as the repaired commit: 234 of 1406 Chronicler pairs (20% of the repaired lines) and 22 of 780 Hex pairs (2%). The table row "same merged branch left out" shows no change in the trend. A fix within one day of the repaired commit is another proxy for a review fix or a same-session fix: 14% of Chronicler pairs (17% of the lines) and 26% of Hex pairs (34% of the lines). The median time from a commit to the fix that repairs it is 14.3 days in Chronicler and 6.7 days in Hex.
- **Commits are not independent (known).** One fix commit repairs many commits. The bootstrap limits are too narrow.
- **No cause (inferred).** Large commits are a different kind of work from small commits (section 4). A flat rate does not prove that size has no effect, and it does not prove the reverse.
- **Not read.** I did not read the fixes to check that SZZ finds the cause (the first commit that introduced the fault). SZZ is known to find the last commit that touched a line, which is often a later refactor. I looked at 6 pairs per repo, and no more.

### 2.5 Confounders

**Refactor commits (known).** Section 2.3 gives the table. Blamed commits by type, repaired lines per 1000 changed lines, cut-off set:

| Type of the repaired commit | Chronicler (commits) | Hex (commits) |
|---|---|---|
| `feat` | 32.1 (91) | 24.6 (54) |
| `fix` | 29.2 (40) | 32.0 (87) |
| `refactor` | 11.3 (158) | 3.7 (41) |
| `test` | 12.1 (43) | not enough commits |
| `docs` | 19.8 (20) | not enough commits |
| `chore` | 9.2 (17) | 26.2 (7) |

A large refactor is repaired at about a third of the rate of a feature in Chronicler and at a seventh in Hex. **Inferred:** a refactor keeps the behaviour, so a later fix finds less to repair.

**Files with the most repaired lines (known).** The two lists use all pairs, with no cut-off.

| Chronicler file | Repaired lines | Fix commits |
|---|---|---|
| `tests/browser/stub/dashboard.rs` | 1408 | 4 |
| `tests/browser/stub/story_log.rs` | 832 | 2 |
| `tests/http/screen.rs` | 360 | 2 |
| `tests/flow_llm_tests.rs` | 275 | 3 |
| `tests/http/swipe_new.rs` | 241 | 3 |
| `tests/http/prompt_presets.rs` | 187 | 6 |
| `tests/http/settings.rs` | 177 | 5 |
| `tests/test_utils/stub_server.rs` | 169 | 9 |
| `src/adapters/driven/llm/transport/utils/response_tests.rs` | 161 | 1 |
| `src/adapters/driving/http/prompt_presets/handlers/prompt_presets.rs` | 153 | 6 |

| Hex file | Repaired lines | Fix commits |
|---|---|---|
| `src/app_window.rs` | 783 | 38 |
| `src/dictation_processor.rs` | 275 | 12 |
| `src/meeting.rs` | 268 | 8 |
| `src/recognition.rs` | 194 | 14 |
| `src/linux_portal.rs` | 145 | 6 |
| `src/audio.rs` | 104 | 9 |
| `src/feedback.rs` | 99 | 4 |
| `src/meeting_watcher.rs` | 95 | 8 |
| `src/linux_app.rs` | 94 | 4 |
| `src/linux_input.rs` | 91 | 11 |

The 10 files hold 44% of the repaired lines in Chronicler and 50% in Hex. In Chronicler, a few commits attract many repairs. The commit `e539bb7d` (feat, 3524 lines) has 1074 repaired lines from 7 fix commits. The 8 most repaired commits hold 30% of the repaired lines (Hex 35%).

**Review fixes in the same branch (known).** See section 2.4.

**`fix(ci)` in Hex (known).** 11 commits (8 from the pipeline, 3 other). 10 of them change Rust code, 63 lines in all, so their blame reaches 27 lines. Only 4 of these lines are in the commits of the main table, because the commits that they repair are mostly new (after the cut-off). The row "`fix(ci)` left out" in section 2.3 shows the same rates. Chronicler has no `fix(ci)` commit.

## 3. Commits with no cut-off

For readers who want the numbers with every commit. The Hex commits are young, so the shares are low.

| Bucket | Chronicler commits | Share repaired | Per 1000 | Hex commits | Share repaired | Per 1000 |
|---|---|---|---|---|---|---|
| under 100 | 98 | 11.2% | 17.2 | 304 | 9.9% | 15.3 |
| 100–299 | 106 | 37.7% | 34.6 | 212 | 23.6% | 13.4 |
| 300–999 | 148 | 68.2% | 20.3 | 174 | 36.8% | 8.2 |
| 1000–2999 | 140 | 86.4% | 17.0 | 78 | 56.4% | 6.3 |
| 3000 or more | 25 | 96.0% | 16.2 | 35 | 85.7% | 6.9 |

## 4. What makes the large Chronicler commits large

**Known.** The largest 10% of the 881 non-merge commits are 88 commits with 1801 code lines or more. Together they changed 294,048 code lines, which is 58% of all changed code lines (Hex: the largest 111 commits hold 74%). 64% of their changed lines are in test files (line-weighted; the median commit has 67.5%). 34 of the 88 are from 2026-10 and 29 from 2026-05.

**Sorting rules (inferred, my judgement).** One kind for each commit, applied in this order. The subject, the body and the file list were the evidence.

1. Several tickets, findings or topics: the body or subject names two or more separate tickets, findings or subsystems, or the commit is a pass that closes a batch of review findings.
2. Migration: a numbered step of the Rust rebuild (`core C1` to `C6`, `P1` to `P4`, `R1` to `R5`), a schema or storage move, or a rewrite of the app shell.
3. Generated or moved files: 60% or more of the added lines, or of the deleted lines, also appear on the other side of the same commit (`tmp/hex-compare/r21/moveshare.py`, lines of 6 characters or more).
4. Test growth: the commit adds tests and nothing else.
5. Feature: the commit adds new behaviour under one title.
6. Refactor: the commit changes structure with the same behaviour and moves less than 60% of its lines.
7. Other: no label is possible from the subject and body.

| Kind | Commits | Code lines | Share of the lines |
|---|---|---|---|
| Several tickets, findings or topics in one commit | 22 | 59,523 | 20.2% |
| Refactor | 20 | 59,930 | 20.4% |
| Generated or moved files | 9 | 58,359 | 19.8% |
| Migration | 14 | 56,696 | 19.3% |
| Feature | 19 | 50,154 | 17.1% |
| Other (an unlabeled "Partial commit", a "docs: auto-index" commit that swept up test work, one settings fix with a design change) | 3 | 7,316 | 2.5% |
| Test growth | 1 | 2,070 | 0.7% |

**Known.** Three groups explain the size:

- **Planned structural change: 43 commits (refactor, moved, migration), 47% of the lines.** The 11 steps of the Rust rebuild are one block: `a54ec457` (P1) to `36a1e1c7` (C6, which deletes 15,601 lines), on 2026-10-17 to 2026-10-21. They changed 46,584 code lines in 5 days. Nine commits split or extract modules with the same lines on both sides, for example `fe09985a`, `b8877b04` and `61cb9048` (all in 2026-05).
- **Batches: 22 commits, 20% of the lines.** Two sub-kinds. Seven of the 22 are `fix` commits that close a review or a gate (for example `29fb773a`, `82531b0b`, `fcd76b9f`, `2885df2c`). The others are early commits with several topics in the subject (`0dfcefe5`, `9d71c622`, `4003b05c`, `a72fe79c`, May 2026), and commits that carry two tickets (`7314ccc6`, `72cfffdc`).
- **Features: 19 commits, 17%.** Most are a numbered stage of a plan (`action-answers` stages 2 to 6, `dialog` stages 2 and 3, `live` stages 3a and 3b).

**Known.** Test growth alone is almost absent: 1 of 88. Tests are large because they change with the code (64% of the lines), not because commits add tests alone.

**Inferred.** The unit of a large commit is the stage of a plan, not a ticket. A plan stage lands as one commit, with its code, its tests, its specs and its docs, and then one review pass fixes the batch. The fix commit of that pass is also large and repairs many earlier commits in one step.

## 5. Questions for ticket 17

1. The pooled fix rate per line does not grow with size (ratio 0.67, limits 0.38 to 1.13). The number of fix commits for each landed commit grows from 0.16 to 4.3, and the share of commits that need a fix grows from 13% to 95%. Which of the two costs does the owner want to cut: the repaired lines per 1000 (flat), or the fix work for each landed change (it grows with size)?
2. 43 of the 88 largest commits are refactors, moves and migration steps, and 22 are batches. Features are 19. Should a rule for small changes apply to all kinds, or only to features and fixes, and should a plan stage that lands as one commit be split so that each commit can pass the gate alone?

## What I did not do

- I did not change the fix rule after the hand check, and I did not sort the 69 and 23 non-matches that contain a fix word.
- I did not read the diffs of the SZZ pairs. I looked at the subjects and files of 6 pairs for each repo.
- I did not measure fix severity, the time to repair, or the cost of the gate for each commit.
- I did not use the 14-day cross-check window as a range of values. It is fixed at 14 days, as the ticket says.

## Files

`tmp/hex-compare/r21/`: `collect.py` (per-commit file changes), `classify.py` (file classes, fix rule), `szz.py` (blame attribution), `analyse.py` and `show.py` (tables), `large.py`, `moveshare.py`, `large_kinds.py` (the largest 10%), and the JSON results (`chron_res.json`, `hex_res.json`, `large_kinds.json`). Order: `collect.py`, `szz.py`, `analyse.py`, `show.py`.
