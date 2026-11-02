# Task 13: Draft the work report

Ticket: [Draft the work report](../issues/13-task-draft-the-work-report.md). Covers C-20 of [candidates.md](./candidates.md).

The **work report** is the durable record that one agent session leaves when it finishes a change. It says which checks ran, which did not, and what the review verdict was. Labels: **known** (read or run in this session, with a source) and **inferred**.

The checks ran on 2026-11-01 in this checkout. No code, skill, `AGENTS.md` or `build.py` file was changed. The session logs gave counts and ids only.

## 1. Answer in short

- **Place (recommended):** the body of the commit. Each commit that changes tracked files carries a block of 4 to 5 lines.
- **Why:** a commit body is the only place that is in git, is tied to the exact change, and exists in every checkout. `logs/` is git-ignored. A ticket exists for few commits. A chat is lost.
- **The new part:** `logs/build_history.txt` already has a `tree` column. That id lets a report say on which tree each check ran. It also shows a gap that no one sees today (section 4).
- **Size of the change:** one session or less for the template and the rule (inferred). A helper script that lists the history rows of a commit is a second, optional task of one session (inferred).

## 2. Where the parts of a report exist today

| Part | Where it exists now | Durable? | Source |
|---|---|---|---|
| Which checks ran, and on which tree | One row per `build.py` run in `logs/build_history.txt`: timestamp, duration, exit code, arguments, `tree`, test counts | **No.** `logs/` is git-ignored. The file keeps 1000 rows. The oldest row today is 2026-10-27, so the window is about 5 days. | `.gitignore:212`; `scripts/build_history.py:23-24`, `:164-165`; `head`/`wc` of the file (known) |
| What each check printed | `logs/build_<date>_<time>.log`, 519 files, 26 MB | **No.** Same reason. A plan can name a log path, but a clone has no such file. | `ls logs`, `du -sh logs`; `old-docs/archived-plans/prompt-flexibility-review-fixes.md:4` names `logs/build_20261101_192133.log` (known) |
| Which checks did not run | Nowhere as a rule. A missing row proves nothing: a row can rotate out, or the run can happen in another checkout, which has its own `logs/` (inferred). | — | — |
| The review verdict | The chat. The `code-review` skill "presents the two reports" in the session. Review findings files go to `tmp/review/` (git-ignored). A plan may copy the verdict: the review-fixes plan has a "Deviations found after the build" section. | Only when a plan or ticket copies it | `.agents/skills/code-review/SKILL.md:78`; `old-docs/archived-plans/prompt-flexibility-review-fixes.md:170-177`; `ls tmp/review` (known) |
| Test results and a manual check | The `## Answer` of a ticket, for example ticket 97 (tests, a manual check in a linked worktree, review fixes) | **Yes**, but only if a ticket exists | `.scratch/retro-issue-bucket/issues/97-worktree-probe-server-has-no-api-keys.md`, Answer (known) |
| The commits of a plan, and its gate | The `Status:` line of a plan names the commits and the gate log | **Yes** for the commit list, **no** for the log it names | `prompt-flexibility-review-fixes.md:4` (known) |
| The final message of one delegate | `~/.pi/agent/delegate/<id>/result.json` with `text`, `stopReason`, `sessionFile`, and `job.json` with label and role. 359 of 368 delegate folders have a `result.json`. | **No.** Outside the repo, on one host. | `ls ~/.pi/agent/delegate`; keys read from the 359 files (known) |
| What the change is and why | The commit body. 190 of the last 300 commits have one, median 9.5 lines. 70 of them name a gate, a test run or a tool. 2 say that a check did not run. 60 mention a review. | **Yes** | `git log -300 --no-merges`, regexes on the bodies (known; the counts depend on the pattern) |

Gap (known): nothing durable says which checks did not run, and nothing durable carries the review verdict, unless a plan or a ticket happens to copy it.

## 3. The Hex form, for comparison

Hex's PR body is the report of one session. All 64 agent PR bodies use one template: `Fixes #N`, the form read back, `**Change**`, `**Gates**`, `Run:` with the session id (known, `research-02-issue-pipeline.md` rows 2 and 6). 13 to 18 of the 64 bodies state that a part was not compiled or not run (known, K-06 in `candidates.md`).

Mapping (inferred):

| Hex part | Chronicler part |
|---|---|
| `**Change**` | The commit subject and body. |
| `**Gates**` | The `Checks` and `Not run` lines. |
| `Run:` id | The `Record` line: a ticket path, a plan path, or a session id. |
| No tree id | Hex checks run on the PR head, so the tree is implicit (inferred). Chronicler commits after a local gate on a working tree that can change, so the tree must be named (section 4). |

The strict reviewer's verdict is a label on the PR in Hex. Chronicler has no PR, so the verdict goes into the `Review` line.

## 4. A finding: the tree id ties a history row to a commit

`build_history.py` records `tree` as the short HEAD, plus a digest of the working-tree diff and of the untracked files (`scripts/build_history.py:39-77`). The digest of a commit can be rebuilt after the commit: hash `git diff <parent> <commit> --binary` for the tracked changes, then the paths and blob ids of the added files.

**Known.** A one-off Python script (not committed) rebuilt the digests of three commits. All three match rows of the history:

| Commit | Parent | Rebuilt tree id | Rows with this id |
|---|---|---|---|
| `d6daa238` | `1fceb06559` | `1fceb06559+c05d0aea` | guardrails 18:57:22, clippy 18:57:40 |
| `4c0811b4` | `ddd73e9655` | `ddd73e9655+ba8ca441` | 16 rows from 19:05:12 to 19:10:36, all green |
| `b4684aea` + `1a0c9162` together | `34e8423ff8` | `34e8423ff8+32b86825` | vale, validate-docs, docstrings, guardrails |

Two results follow.

1. **A row can be matched to a commit.** A report can say "integration, green, on the tree of this commit". That is a stronger claim than "the gate passed". It is the missing link between `build_history.txt` and a commit.
2. **The history shows gaps that no one sees now.** Section 5 has the three cases. In two of them, a step that is relevant to the change did not run on the committed tree.

The id changes if a commit is amended or rebased, because the parent changes (inferred). So the report must be written at commit time.

## 5. The template, filled in for 3 recent changes

### The template

```
Work report
Checks: <step> <result> [<count>], ...  (tree <tree-id>)
Not run: <step>: <reason>; <step>: ran on <other tree-id>
Review: <skill or reviewer>, range <ref>..<ref>, <verdict>, <where the findings went>  | none
Limit: <one sentence: what the checks above cannot show>
Record: <ticket path | plan path | session id>
```

Rules:

1. `<tree-id>` is the value of `build_history.tree_identity` on the tree that the commit holds. The `Checks` line lists only steps that have a green row with that exact id.
2. A step that is green only on another tree goes in `Not run`, with that tree id.
3. Each entry in `Not run` has a reason. "Docs only" is a reason. "Ran out of time" is a reason. A blank is not.
4. `Review: none` is a valid entry. It says that no review ran. A missing line is not.
5. The block is for the owner, who approves each commit (`AGENTS.md:409`). A reader who wants the proof reads the line and the history row, not a promise.

### Case A: `4c0811b4`, refactor, phases C, D and E of the review-fixes plan (19:10:39)

Source of the facts: `git show 4c0811b4`, `logs/build_history.txt` rows on `ddd73e9655+ba8ca441`, the plan (known).

The commit body is empty. The plan `Status:` line names the commit and the full gate log. The rows hold the rest.

```
Work report
Checks: clippy ok; guardrails ok (309); integration ok (1631); validate-docs ok  (tree ddd73e9655+ba8ca441)
Not run: vale: last green on d6daa238d8+e5b235dd; py-tests: no row on this tree; browser: no row on this tree
Review: none before this commit
Limit: the plan's own Verify lines, not the gate, prove each phase.
Record: old-docs/archived-plans/prompt-flexibility-review-fixes.md
```

Check: the full gate ran later, on the clean head `1e7e489c6b` (row 19:26:40, 307 s, guardrails 309, integration 1625, browser 44). It covers `browser` for the final state of the branch, but not for the tree of this commit (known). A review ran after the build and found four places where the plan text and the result differ (`prompt-flexibility-review-fixes.md:170-177`). Nothing in the commit points to it.

### Case B: `b4684aea`, "code review findings of the review-fixes plan" (19:53:33)

Source: `git show b4684aea`, rows on `34e8423ff8+32b86825` and `+73a81de8` (known).

The commit body is empty. The rebuilt tree id of this commit alone is `34e8423ff8+cda42074`. **No row has that id** (known).

The next commit, `1a0c9162` (19:54:35), was made from the same working tree. The rows on `+32b86825` are the tree of both commits together (known, section 4). So one gate covers two commits. The gate rows are:

| Step | Tree on the row | Result |
|---|---|---|
| vale, validate-docs, docstrings, guardrails (309) | `34e8423ff8+32b86825` | green |
| clippy, integration (1626) | `34e8423ff8+73a81de8` | green, on an earlier tree |

`integration` ended at 19:53:03 after 69.9 s, so it started at about 19:51:53 (inferred: the timestamp is the end of the run). `vale` started on a new tree at 19:52:27 (known from the rows). So the tree changed while `integration` ran (inferred). No `integration` row carries the tree that was committed (known).

```
Work report
Checks: vale ok; validate-docs ok; docstrings ok; guardrails ok (309)  (tree 34e8423ff8+32b86825, batch of 2 commits: this one and the next)
Not run: clippy and integration (1626): ran on 34e8423ff8+73a81de8, an earlier tree; browser: no row; py-tests: see next commit
Review: code-review, range 893e3d94..34e8423f, four deviations written into the plan (section "Deviations found after the build"); no review of this commit
Limit: the four deviations are plan text, not code behaviour.
Record: old-docs/archived-plans/prompt-flexibility-review-fixes.md
```

### Case C: `1a0c9162`, `build.py run` links the main `.env` into a worktree (19:54:35)

Source: `git show 1a0c9162`, ticket 97 Answer, rows on `34e8423ff8+200c5eb3` and `+32b86825` (known).

This commit has a body (9 lines) that says what and why. It says nothing of checks. The ticket 97 Answer holds the tests (`LinkMainEnvTests`), one manual check in a real linked worktree, and the review fixes. That is a report in all but name, and it lives in the ticket.

The change touches `build.py` and `scripts/tests/test_build_cli.py`. The step that tests that file is `py-tests`. Its only green row today is on `34e8423ff8+200c5eb3` (19:49:09, 523 s, known). That tree is not the committed tree (`+32b86825`). No `py-tests` row has the final id (known).

```
Work report
Checks: vale ok; validate-docs ok; docstrings ok; guardrails ok (309)  (tree 34e8423ff8+32b86825, batch of 2 commits: the previous one and this one)
Not run: py-tests: ran on 34e8423ff8+200c5eb3 (green), not on this tree; clippy and integration: ran on an earlier tree; browser: no UI change
Review: the review fixes are in the ticket Answer; verdict not recorded
Limit: one manual check in one linked worktree; no test covers a broken link on a second host.
Record: .scratch/retro-issue-bucket/issues/97-worktree-probe-server-has-no-api-keys.md
```

The `Limit` line of case C repeats a fact of the Answer ("manual check in a real linked worktree", known). The claim about a second host is inferred and is shown only as the style of a limit line.

### What the three cases show

- **Known.** 0 of the 3 commits tell the reader which checks ran on the committed tree. 2 have no body (A and B). The third has no check line.
- **Known.** In cases B and C, the rows on the committed tree miss steps that are relevant to the change: `integration` and `clippy` in B, and `py-tests` in C.
- **Inferred.** The cause is an edit made while a step ran, or between two steps. Each step is correct. The set is not whole. A person reads "gate green" in the chat and cannot see it.
- **Inferred.** The cost of the line is small. It takes the rows of one tree id from the history. A helper script could print it (section 7).

## 6. Where the report goes: the candidate places

| Place | Durable in git | Tied to the exact change | Exists for every change | Cost | Verdict |
|---|---|---|---|---|---|
| **Commit body** | Yes | Yes, it is part of the commit | Yes. Each change that lands is a commit. The owner approves each commit (`AGENTS.md:409`), so the report is read at the moment of approval. | 4 to 5 lines. The median body is 9.5 lines, so a block of this size fits. | **Recommended.** |
| Ticket `## Answer` | Yes | By a path, not by a hash | No. A ticket exists for few changes. The last 300 subjects name a ticket or an issue in 56 cases (K-40 in `candidates.md`). | Already written in many tickets | Keep as is. A ticket Answer holds its own results. It does not need a second copy. |
| Plan `Status:` line | Yes | By a commit list | Only for plan work (4 plans in `docs/plans/`, 7 in `old-docs/archived-plans/`) | Already written | Keep as is. It points to commits. |
| A file in `logs/` | **No.** `logs/` is git-ignored and keeps 1000 rows. | No | Per checkout only | Free | **Reject.** It does not survive a clone. A plan that names a `logs/` path already has a dead link in any other checkout. |
| A file in `.scratch/` per change | Yes | By a name, and it cannot name the hash of its own commit | Yes, if the rule says so | A new file for each change. Parallel tickets add many files (inferred). | **Reject.** The file would need its own tracker rules and would grow without end. |
| The delegate `result.json` | **No.** Outside the repo, on one host. | By a session id | Only for delegates | Free | **Source, not place.** A delegate's final message holds the block. The session that commits copies it. |
| The chat only | No | No | Yes | Free | **Reject.** This is the state today, and it is the gap. |

## 7. Sessions with no commit and no ticket

Three kinds of session exist (inferred from the delegate records and `docs/plans/parallel-ticket-runs-without-commits.md`).

| Kind of session | Where the report goes | Reason |
|---|---|---|
| **A read-only session**: research, a review, an answer to a question. No tracked file changed. | The final message. For a delegate, `result.json` keeps it. No file in the repo. | No change landed. The block would describe nothing. If the work answers a ticket, the answer is in the ticket already. |
| **Edits left uncommitted, waiting for the owner**: the normal case in the plan "parallel ticket runs without commits" (`docs/plans/parallel-ticket-runs-without-commits.md`, "Decided in conversation") | The final message holds the block, with the tree id of the dirty tree. The session that commits copies it to the commit body. | The owner approves the commit. The tree id tells the committer whether the tree changed since the report. If the ids differ, the report is stale. |
| **An abandoned change**: the session reverts or stops | The final message says "no change landed" and why. The repo gets nothing. | A failed attempt that leaves no file has no change to describe. A reason that matters, for example a failed check that blocks the next try, goes in the ticket, if a ticket exists. |

A review verdict that no plan or ticket asked for stays in the chat. The report line says `Review: none`, or names the review. That is better than today. It is not full. State this limit in the rule.

## 8. Recommendation

1. **Place and form.** Put the work report in the commit body, in the block of section 5. A commit that changes only docs may use `Checks` and `Not run` only.
2. **No second copy.** A ticket Answer or a plan `Status:` line keeps its own results. It does not copy the block.
3. **Write time.** The session that commits writes the block, from the history rows of the tree id of the commit. A delegate puts its own block in its final message. The session that commits copies it. The history keeps only about 5 days of rows, so a block written later has no source.
4. **The size of the change.**
   - The template and one rule in the commit workflow (`AGENTS.md` Development Loop, the after-plan skills): **one session or less** (inferred).
   - An optional helper, `scripts/work_report.py <commit>`: it rebuilds the tree id of a commit and prints the history rows that match, as a `Checks` line. **One session** (inferred). It gives a risk: it rebuilds the id in a second place. Importing `tree_identity` from `build_history.py` and sharing one routine would avoid the copy (inferred).
   - No change to `build.py` is needed. The `tree` column already exists.
5. **What the report does not do.** It states the facts. It does not block a commit. Whether a missing check may block is a question of [ticket 17](../issues/17-grilling-decide-whether-anything-lands-without-the-owner.md).

## 9. Questions for ticket 16

1. Does the owner accept the commit body as the one place, with a block of about 5 lines in each commit that changes tracked files? The alternative is a ticket-only record, which leaves most commits without one.
2. Is a report that only states the facts enough, or must a commit stop when a relevant step has no green row on its own tree (case B and C)? This question sets the base of ticket 17.

## 10. Not checked

- The 3 cases come from one afternoon (2026-11-01) and one tree history. They are cases, not a rate. How often the committed tree misses a relevant step across the 1000 rows was not measured.
- The tree rebuild ran as a one-off on 3 commits. A commit with deleted binary files, renames or mode changes was not tried (inferred risk).
- The history rows older than 2026-10-27 do not exist.
- The content of the session transcripts was not read. Only counts and ids were used.
- The delegate folders without a `result.json` (9 of 368) were not examined. Some may still be running, or may have been stopped (inferred).
- The claim that two review kinds (`code-review` and the after-plan reviews) do not write a file outside `tmp/` rests on `SKILL.md:78` and on the plan. The two after-plan skills were not read in full.
