# Measure whether large changes need more fixes

Type: research (AFK)
Status: resolved
Blocked by: —

## Question

In the git history of Chronicler, do large commits need more later fixes than small commits, per changed line? Does the same measure on Hex give a different fix rate?

## Context

- **Why** (owner, 2026-11-01). The aim of this map is to find what keeps Hex's code good while several developers build it fast with agents. Most single practices ended as covered or dropped. The one clear measured difference is the size of a change: a Hex pipeline commit has a median of 80 changed lines (p90 253), and a Chronicler commit has a median of 377.5 (p90 3049) (`assets/research-02-issue-pipeline.md` row 11, check K-41 in `assets/candidates.md`). This ticket tests whether size matters, with data. Its answer is the input of [Decide whether Chronicler works in small changes](./17-grilling-decide-whether-anything-lands-without-the-owner.md).
- **The data (known, 2026-11-01).**
  - Chronicler: 909 commits since 2026-03-14, 28 of them merges. Conventional subjects: 186 `refactor`, 146 `feat`, 105 `docs`, 73 `chore`, 63 `fix`, 53 `test`.
  - Hex clone `tmp/hex-compare/anomalyco-hex/` at `7609574c`: 1146 commits since 2026-07-14. 184 `fix`, 141 `feat`, 86 `docs`, 55 `refactor`, 46 `chore`, 31 `ci`.
- **Measure.**
  1. **Size.** For each non-merge commit, count the added and deleted lines in code files: `src/` and `tests/` for Chronicler, the Rust crates for Hex. Report the generated files that you excluded (the index blocks of `AGENTS.md`, `tests/AGENTS.md`, `docs/AGENTS.md`, `Cargo.lock`, images). Also report the total size with docs, so a reader sees both.
  2. **Fix commits.** A fix commit is a commit whose subject starts `fix` or `Revert`. State the rule, and check 20 random matches and 20 random non-matches by hand for errors. Report the error rate.
  3. **Attribution (primary method).** For each fix commit, run `git blame` on its parent for the lines that the fix deletes or changes in code files. Each blamed commit is a commit that the fix repairs (the SZZ method). Skip the blame of a line that only moved, and use `git blame -w -M -C`.
  4. **Attribution (cross-check).** A fix commit within 14 days that touches at least one of the same code files. This method over-counts. Report both methods.
  5. **Result.** Sort the commits into size buckets: under 100, 100–300, 300–1000, 1000–3000, over 3000 changed code lines. For each bucket give the number of commits, the share that a later fix repairs, and the repaired lines per 1000 changed lines. Give the same table for Hex.
  6. **What makes Chronicler commits large.** For the largest 10% of Chronicler commits, sort each into one kind: a feature, a refactor, several tickets in one commit, a migration, test growth, generated or moved files, other. Use the subject, the body and the file list.
- **Confounders to report, not to remove.**
  - A large refactor touches old code, so blame can find it as the cause of an old fault. Report the refactor commits apart.
  - Some files change often. Report the 10 files with the most repaired lines.
  - A fix that lands in the same branch before the merge is a review fix, not a fault that escaped. Where a merge commit shows the branch, count those fixes apart.
  - Hex commits are younger (since 2026-07-14), so a recent commit had less time to get a fix. Use the same cut-off for both: leave out the commits of the last 14 days.
  - Hex has many `fix(ci)` commits for other operating systems. Report them apart, because Chronicler has no such target.
- **Failure modes.**
  - A commit that blame cannot read (binary files, renames): skip it and count it.
  - A fix commit with no code lines (a doc fix): leave it out of the attribution and count it.
  - A bucket with fewer than 10 commits: show it, and mark it as too small to compare.
- **Not in scope.** Do not propose a size rule or a process. Record what the data shows. Change no code, no docs and no git state.
- **Rules.** Use the `/research` skill. Label each claim known or inferred. Scripts go in `tmp/hex-compare/r21/`. Give commit ids, not quotes from commit bodies.
- **Output.** Write `assets/research-21-change-size-and-fixes.md`. The `## Answer` gives:
  - the gist, with the two tables (Chronicler and Hex)
  - whether the fix rate per line grows with size, with the limits of the method
  - what makes the large Chronicler commits large
  - at most 2 questions for ticket 17
- **When done.** Add `## Answer` and set `Status: resolved`.

## Answer

Status of the evidence: known unless marked inferred. Asset: [research-21-change-size-and-fixes.md](../assets/research-21-change-size-and-fixes.md). Scripts: `tmp/hex-compare/r21/`.

**Gist.** The fix rate per changed line does not grow with the size of a commit, in either repo. The share of commits that a later fix repairs does grow (Chronicler 13% to 95%, Hex 24% to 100%), because a large commit has more lines. Repaired lines per 1000 changed lines stay flat or fall. A commit of 1000 lines or more has 0.67 times the rate of a smaller one in Chronicler (95% limits 0.38 to 1.13) and 0.64 in Hex (0.41 to 1.00). The cost that does grow is the number of later fix commits for each landed commit: 0.16 (under 100 lines) to 4.3 (3000 lines or more) in Chronicler.

**Chronicler** (439 commits with code lines, up to 2026-10-19). SZZ attribution: `git blame -w -M -C` on the parent of each fix commit.

| Bucket (code lines) | Commits | Changed lines | Repaired by a later fix | Repaired lines per 1000 (95% limits) | Cross-check share (14 days, same file) | Fix commits per commit |
|---|---|---|---|---|---|---|
| under 100 | 87 | 3,086 | 12.6% | 19.8 (8 to 33) | 43.7% | 0.16 |
| 100–299 | 92 | 17,321 | 42.4% | 39.3 (17 to 68) | 51.1% | 0.53 |
| 300–999 | 127 | 71,711 | 68.5% | 20.4 (13 to 30) | 66.9% | 1.31 |
| 1000–2999 | 112 | 196,706 | 84.8% | 14.8 (11 to 19) | 76.8% | 2.58 |
| 3000 or more | 21 | 116,252 | 95.2% | 18.1 (6 to 43) | 71.4% | 4.33 |

**Hex** (196 commits with code lines, up to 2026-10-17; the 3000+ bucket is too small to compare).

| Bucket (code lines) | Commits | Changed lines | Repaired by a later fix | Repaired lines per 1000 (95% limits) | Cross-check share | Fix commits per commit |
|---|---|---|---|---|---|---|
| under 100 | 72 | 3,287 | 23.6% | 31.0 (10 to 63) | 61.1% | 0.35 |
| 100–299 | 60 | 11,096 | 55.0% | 42.6 (26 to 61) | 78.3% | 1.25 |
| 300–999 | 38 | 18,544 | 60.5% | 22.1 (11 to 36) | 89.5% | 1.34 |
| 1000–2999 | 18 | 31,023 | 88.9% | 15.0 (7 to 24) | 94.4% | 3.67 |
| 3000 or more | 8 (too small) | 61,376 | 100% | 21.4 (not computed) | 100% | 21.0 |

**Does the rate grow with size?** No (known). It also does not clearly fall: the limits of the two groups touch 1.0. The pooled rate stays flat or falls in every variant of the asset table (no cut-off, without refactors, without same-branch fixes, without fixes within one day, without test files, and others). One subset is an exception: Chronicler `feat` commits rise from 4.9 to 14.2 to 23.0 repaired lines per 1000 (100–299, 300–999, 1000–2999 lines). It is a subset that I picked after the first tables, so chance can explain it (inferred). Hex's smaller commits do not show a lower rate: 22.0 repaired lines per 1000 in Hex against 17.8 in Chronicler.

**Limits of the method.**
- The fix rule (subject starts `fix` or `Revert`) is wrong for 2 of 20 matches in Chronicler and 4 of 20 in Hex. It misses 2 of 20 fixes in the non-matches of each repo. 20 samples give wide limits.
- SZZ counts lines that a later commit changed, not faults. 70% of the repaired Chronicler lines are in test files, and Hex keeps its tests inside `src/`, so the repos are not equal on this point.
- 20 of 56 Chronicler fix commits with code are larger than 300 lines and mix repair with refactor. When only fixes of 300 lines or less count, the rate is 1.0 to 7.5 per 1000 and it falls with size.
- Hex is 3.5 months old. 76% of its commits with code are less than 14 days old, and the cut-off leaves 196 commits.
- Squash and linear history hide review fixes. 17% of the Chronicler pairs sit in one merged branch, and 14% of them are within one day. For Hex it is 3% and 26%.
- Bootstrap limits treat commits as independent, but one fix commit repairs many. The true limits are wider.

**What makes the large Chronicler commits large** (inferred, my sort of the 88 commits of 1801 code lines or more; they hold 58% of the changed code lines, and 64% of those lines are in test files).

| Kind | Commits | Share of the lines |
|---|---|---|
| Several tickets, findings or topics | 22 | 20.2% |
| Refactor | 20 | 20.4% |
| Generated or moved files | 9 | 19.8% |
| Migration (11 are steps of the Rust rebuild, 2026-10-17 to 2026-10-21) | 14 | 19.3% |
| Feature | 19 | 17.1% |
| Other | 3 | 2.5% |
| Test growth alone | 1 | 0.7% |

The unit of a large commit is the stage of a plan: code, tests, specs and docs land together, and one later review pass fixes the batch.

**Questions for ticket 17.**
1. The fix rate per line is flat, but the fix work for each landed commit grows with size (0.16 to 4.3 fix commits) and so does the share of commits that need a fix (13% to 95%). Which cost does the owner want to cut?
2. 43 of the 88 largest commits are refactors, moves and migration steps, and 22 are batches. Should a small-change rule apply to all kinds, or only to features and fixes? Should a plan stage that lands as one commit be split?
