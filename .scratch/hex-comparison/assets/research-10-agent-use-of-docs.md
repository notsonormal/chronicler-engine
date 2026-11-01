# Research 10: how agents use the current docs and plans

Status: reference. Asset of the ticket [Research how agents use the current docs and plans](../issues/10-research-agent-use-of-docs-and-plans.md) in the [map](../map.md). Made on 2026-11-01.

This asset records what the data shows. It does not judge the doc layout, and it proposes no feature maps.

## Summary

- **Agents read docs (known).** 92% of 407 code-work sessions read at least one doc, plan or instruction file. 56% read a Diátaxis or spec file. 175 of those 227 sessions did it before their first code edit.
- **Many reads follow a brief (known for delegates).** 58% of the (session, doc) pairs of implementer delegates (423 of 736) name the doc in the brief. The `CODING_STANDARDS.md` read is 62 of 65 named.
- **Agents follow a `[DOC: …]` anchor rarely (known, with limits).** 14% of the cases (64 of 466 session and doc pairs). 26% of the sessions that read an anchored source file read one anchor target.
- **Docs change with the code (known).** 126 of 178 commits that touch `docs/diataxis/` or `docs/specs/` also change `src/`. Only 12 change docs alone. 16 only regenerate an index.
- **Plans and decisions.** The map and the ticket are the planning docs that agents read (39% and 46% of code-work sessions). `.scratch/_closed/` is read in 10 code-work sessions. No session read a `docs/adr/` file.
- **Recommendation.** C-42 drop. C-43 keep for ticket 16, in a narrow form. C-44 drop. Part 6 has the reasons.

## 1. Method

### Sources

| Source | What it holds | Known |
|---|---|---|
| `~/.pi/agent/sessions/--workspace-chronicler-engine--/` | 1471 `.jsonl` files, 2026-08-11 to 2026-11-01. 1355 have tool calls. | `tmp/hex-compare/r10/load.py` |
| `~/.pi/agent/sessions/--home-node-.herdr-worktrees-chronicler-engine-*` | 36 sessions with tool calls (worktrees). Counted apart, in part 3. | `load.py` |
| other session folders | 8 sessions (`--tmp--`, `--workspace--`, a prototype folder, `--workspace-pi-emote--`). Not counted. | `load.py` |
| `~/.pi/agent/delegate/*/job.json` | 332 delegate records. `role` and `label` sort the delegate sessions. | `an.py` |
| `git log --since=2026-08-11 --no-merges` | 399 commits. | `churn.py` |

The session files kept growing during this work. The counts are from one load at about 20:15 UTC on 2026-11-01. The newest sessions (this one included) are in the data or not by chance (known).

### The scripts

The scripts are in `tmp/hex-compare/r10/` (git-ignored): `load.py`, `an.py`, `kinds.py`, `m2.py`, `final.py`, `anchors.py`, `plans.py`, `churn.py`, `churn2.py`. They reproduce each number below. The asset has no transcript text. It has counts, paths and git hashes.

### What counts as a doc read

- A `read` tool call on the path (known).
- A `bash` command whose first word is `cat`, `head`, `tail`, `sed` (not `sed -i`), `nl`, `bat`, `less`, `more`, `rg`, `grep`, `egrep`, `fgrep` or `awk`, and that names a literal path that ends in `.md`. One path in one command is one read.
- A `rg` or `grep` on a doc is a search and not a read. Part 3 gives the split.
- Not counted: a glob (`docs/specs/*.md`), a directory, a `git show`, a path inside a heredoc script, and a read through the `codemode` tool (194 calls). The count of doc reads is therefore a **lower bound**. In code-work sessions, 534 `bash` commands name `docs/diataxis` or `docs/specs` as a directory or a glob, with no literal file (known).
- A `bash` edit is found only with a pattern (`sed -i`, `perl -i`, `git apply`, `patch`, `tee`, a redirect into `src`, `tests` or `docs`). Other edits by `bash` are missed.
- Root `AGENTS.md` is in the system message of many sessions (part 3, "Always-loaded"). A read of it is a deliberate extra read.

### Path forms

The sessions come from four checkout roots: `/workspace/chronicler-engine`, `/home/moridin84/projects/chronicler-engine`, `/workspace/ce-wt/<name>` and `/home/node/.herdr/worktrees/chronicler-engine/<name>`. `an.py` (`norm`) maps each form to a repo-relative path. A path outside these roots is `external`.

### Doc groups

| Group | Paths |
|---|---|
| instructions | `AGENTS.md`, `docs/AGENTS.md`, `tests/AGENTS.md`, `CODING_STANDARDS.md`, `CONTEXT.md`, `ENVIRONMENT.md`, `README.md`, `PRODUCT.md`, `tests/STRATEGY.md` |
| Diátaxis Reference, Explanation | `docs/diataxis/reference/`, `docs/diataxis/explanation/` |
| specs | `docs/specs/` |
| plans | `docs/plans/` (also `plans/`, a symlink to it), `old-docs/archived-plans/` |
| scratch | `.scratch/<effort>/map.md`, `.scratch/<effort>/issues/`, `.scratch/<effort>/assets/`, `.scratch/_closed/` |
| skills | `.agents/skills/` (counted apart; the harness loads a skill) |

"Domain docs" in this asset means Reference, Explanation and specs.

### The rule that sorts a session

The first rule that matches wins. The rule is in `kinds.py`.

1. A delegate with role `reviewer`, `judge-glm` or `judge-qwen`: **review**.
2. A delegate with role `researcher` or `scout`: **research**.
3. A first message that starts a skill in {`code-review`, `thermo-nuclear-code-quality-review`, `test-police`, `document-review`, `no-ai-slop`}: **review**. The skill `commit-and-push`: **landing**. The skill `research`: **research**.
4. The session edits a file in `src/`, `tests/`, `data/`, `assets/`, `styles/`, `scripts/`, `tools/` or `build.py`: **code-work**.
5. A first message that starts a skill in {`wayfinder`, `grill-with-docs`, `grilling`, `improve-ai-plan`, `improve-codebase-architecture`, `retro`, `reflect`, `correct`, `consider-the-bitter-lesson`}: **planning**.
6. The session edits only docs, instructions or skills: **doc-work**. It edits only `.scratch/` or plans: **planning**. Any other edit: **other-edit**.
7. The first message starts with the marker `<!-- backpass:`: **tool-audit** (an analysis run of the `backpass` tool, not a work session).
8. The first message names a code review, or starts with "read-only" and then recon, audit or architecture: **review** or **research**.
9. Anything else: **unknown**. The rule does not guess.

The delegate role came from `job.json` (328 sessions matched) or, for the rest, from the session name `delegate · <role> · <label>`.

### Failure modes (known)

- Lines that are not JSON: 7 lines in 6 files. They were skipped and counted.
- A session of unclear kind: 276 main-folder sessions (20% of 1355) are `unknown`. They have no edit, no skill and no delegate role. Most are questions and short chats.
- 27 implementer delegates (23 `implementer`, 2 `strong-implementer`, 2 `generalist`) made no edit that the method sees. They are `unknown`. Some may have edited by `bash` or `codemode` (inferred).

## 2. Part 1: the kinds of session

Main folder, sessions with tool calls (known: `kinds.py`).

| Kind | Sessions | Share |
|---|---|---|
| code-work | 407 | 30% |
| review | 223 | 16% |
| planning | 181 | 13% |
| research | 98 | 7% |
| landing (`commit-and-push`) | 66 | 5% |
| tool-audit (`backpass`) | 62 | 5% |
| doc-work | 32 | 2% |
| other-edit | 10 | 1% |
| unknown | 276 | 20% |
| **Total** | **1355** | |

By month: 2026-08 164 sessions (47 code-work), 2026-09 241 (60), 2026-10 888 (281), 2026-11 62 (19). The 888 for October matches ticket 08.

By who ran the session:

- 336 sessions with tool calls are delegates (341 delegate sessions in all). Roles: 116 `reviewer`, 71 `implementer`, 54 `strong-implementer`, 49 `researcher`, 38 `scout`, 6 `judge-*`, 2 `generalist`.
- Of the 407 code-work sessions, 91 are delegates (implementer roles) and 316 are not.
- Doc-work is small: 32 sessions edit only docs, skills or instructions (2%). Most doc edits happen inside code-work sessions (part 3).

### Reconciliation with ticket 08 (known)

Ticket 08 reports for October: 351 sessions that read a file in `src/`, 122 Reference, 100 spec, 89 `CONTEXT.md`, 52 Explanation. File reads: `src/` 3218, Reference 331, specs 243.

- A test that matches the path text `/src/` gives 351 sessions and 3218 reads. That reproduces ticket 08 exactly (known). That count includes reads in other repos, such as the Hex clones in `tmp/hex-compare/` (inferred).
- A repo-relative `src/` path gives 316 sessions and 2831 reads. The ticket 08 figure for `src/` is therefore about 12% high.
- Reference, spec and `CONTEXT.md` give 110, 86 and 86 sessions here. The gap to ticket 08 is 3 to 14%. I did not find the cause. A likely cause is a looser path match (inferred).

## 3. Part 2: the docs that code-work sessions read

### 3.1 Which docs

Code-work sessions in the main folder (407). Delegates are the 91 implementer sessions. "Before edit" is the number of sessions that read the group at least once before their first code edit (known: `m2.py`).

| Group | All: sessions (%) | Not delegate (316) | Delegate (91) | Before first code edit (all) | Distinct files |
|---|---|---|---|---|---|
| `AGENTS.md` (root) | 109 (27%) | 31% | 13% | 58 | 1 |
| `docs/AGENTS.md` | 43 (11%) | 12% | 7% | 22 | 1 |
| `tests/AGENTS.md` | 77 (19%) | 18% | 23% | 55 | 1 |
| `CODING_STANDARDS.md` | 172 (42%) | 34% | 71% | 141 | 1 |
| `CONTEXT.md` | 80 (20%) | 19% | 21% | 49 | 1 |
| `ENVIRONMENT.md` | 52 (13%) | 13% | 12% | 29 | 1 |
| `tests/STRATEGY.md` | 153 (38%) | 30% | 65% | 134 | 1 |
| Diátaxis Reference | 165 (41%) | 37% | 52% | 103 | 22 |
| Diátaxis Explanation | 48 (12%) | 9% | 22% | 27 | 8 |
| specs | 164 (40%) | 34% | 60% | 124 | 37 |
| `docs/agents/` | 99 (24%) | 31% | 1% | 83 | 3 |
| `docs/plans/` | 104 (26%) | 17% | 54% | 89 | 54 |
| `.scratch` maps | 159 (39%) | 48% | 8% | 143 | 25 |
| `.scratch` tickets | 188 (46%) | 51% | 30% | 162 | 293 |
| `.scratch/_closed/` | 10 (2%) | 3% | 0% | 4 | 12 |
| skills | 182 (45%) | 49% | 30% | 128 | 54 |

Known facts from this table:

- 373 of 407 sessions (92%) read at least one doc, plan or instruction file. 368 do so with root `AGENTS.md` left out.
- 227 sessions (56%) read a domain doc. For non-delegates it is 159 of 316 (50%). For delegates it is 68 of 91 (75%).
- Of the 227, 175 read a domain doc before their first code edit, and 52 only after. 109 read specs and Diátaxis, 55 only specs, 63 only Diátaxis.
- Specs are read as often as Reference (40% and 41%). Explanation is read 3.4 times less often (12%).
- Every one of the 31 specs and 19 Diátaxis docs that exist today was read by at least one code-work session. The most read: `docs/diataxis/reference/frontend/dashboard.md` and `docs/specs/browser_dashboard.md` (54 sessions each), `docs/specs/actions.md` and `guardrails.md` (43 each).
- These sessions made 5024 tool reads of source code. They made 5825 doc reads, tool and `bash` together. The two numbers are not comparable one to one: a `bash` command with three doc paths counts as three reads.
- Of the `bash` mentions of a domain doc, 1371 use a read word (`cat`, `sed`, `head`) and 729 use a search word (`rg`, `grep`). A search of a doc is a weaker sign of reading.

By month (the share of code-work sessions that read a Reference doc, or a spec): Reference 30% in August (47 sessions), 38% September (60), 42% October (281), 58% November (19). Specs: 19%, 43%, 43%, 37%. The August figures are lower. The docs moved to the root on 2026-08-11 (known from the ticket). The cause of the lower figure is not known.

### 3.2 Reads that a brief caused

For the 91 implementer delegates, a doc read is **named in the brief** if the path or the file name is in the first message (known: `an.py` and the inline check).

| Group | (session, doc) pairs | Named in the brief |
|---|---|---|
| `CODING_STANDARDS.md` | 65 | 62 |
| `tests/STRATEGY.md` | 59 | 50 |
| `docs/plans/` | 51 | 50 |
| specs | 268 | 89 |
| Diátaxis Reference | 110 | 45 |
| Diátaxis Explanation | 28 | 12 |
| `CONTEXT.md` | 19 | 14 |
| `.scratch` tickets and maps | 52 | 40 |
| all groups above and the rest | 736 | 423 (58%) |

So a delegate reads standards, plans and tickets because its brief says so. It chooses Reference and specs more often by itself: 45 of 110 Reference reads and 89 of 268 spec reads were named. The 313 reads that the brief did not name are the best sign of an agent that finds a doc by itself (inferred). For non-delegate sessions the first message is the owner's prompt or a skill text. I did not measure how often these name a doc.

### 3.3 The `[DOC: …]` anchor (known, with limits)

Every `src/` file with a `//!` header (221 of 320 `.rs` files) has `[DOC: docs/diataxis/reference/….md]` on line 1. The anchors exist since 2026-08-11 (commit `d8644394`). `anchors.py` pairs each source file that a session read by tool with the doc that the file names. The test: did the session read that doc at any time?

| Cut | Sessions that read an anchored file | Read at least one target | (session, target) pairs | Target read |
|---|---|---|---|---|
| all reads | 184 | 48 (26%) | 466 | 64 (14%) |
| reads that start at line 1 | 163 | 43 (26%) | 356 | 55 (15%) |
| October, line 1 | 102 | 32 (31%) | 190 | 38 (20%) |
| August and September, line 1 | 59 | 10 (17%) | 164 | 16 (10%) |

Of the 64 pairs, 51 read the target after the first read of the source file, and 18 before it (some pairs did both). Limits:

- The anchors are those of today. A file's anchor may differ in older sessions (inferred).
- A source file that a session read by `bash` (`cat`, `sed`) is not counted.
- A session may get the same fact from the code. The data does not show whether it needed the doc.

For the edit side: 163 sessions edited an anchored file, and 24 of them read an anchor target before the first edit of that file.

### 3.4 Doc edits by code-work sessions

104 of 407 code-work sessions (26%) edit a spec or a Diátaxis doc: 71 a spec, 61 a Diátaxis doc. For non-delegates it is 92 of 316. For delegates it is 12 of 91. In 230 of the 260 (session, doc) pairs the session read the doc earlier (known: 88%). The `docs/AGENTS.md` rule "a behaviour change edits its spec and its test" is therefore followed by the sessions that act, mostly by the non-delegate session (inferred).

### 3.5 Always-loaded (known, with a gap)

- A few sessions of September and most of October record the system message with its sections. In October, 573 of 888 sessions (287 delegates and 286 others) have a `project_context` section. Its median size is 26,899 characters, about the size of `AGENTS.md` (29,524 bytes). 315 sessions of October have no such section, and the 164 sessions of August and 173 of September have no system message record at all.
- I did not check whether the 315 other sessions get `AGENTS.md` by a different path. This is a gap for C-01.
- 27% of code-work sessions still read root `AGENTS.md` by a tool or `bash`, in 260 reads. 217 of these are `bash` reads. The cause is not known.

### 3.6 Worktree sessions (apart)

The worktree folders have 36 sessions with tool calls: 10 code-work, 2 planning, 2 research, 1 review, 1 doc-work, 20 unknown. All 10 code-work sessions read a doc. 9 read a spec, 4 a Reference doc. The sample is too small for rates.

## 4. Part 3: plans and decisions

Main folder, sessions that read the group (any kind of session), from `plans.py`.

| Kind (sessions) | `docs/plans/` | `old-docs/` | map | ticket | `_closed/` | asset |
|---|---|---|---|---|---|---|
| code-work (407) | 104 | 10 | 159 | 188 | 10 | 57 |
| planning (181) | 25 | 4 | 148 | 127 | 9 | 69 |
| review (223) | 55 | 13 | 23 | 83 | 0 | 28 |
| research (98) | 10 | 2 | 52 | 63 | 9 | 43 |
| doc-work (32) | 6 | 0 | 4 | 3 | 0 | 2 |
| landing (66) | 14 | 1 | 5 | 12 | 0 | 1 |
| unknown (276) | 20 | 1 | 17 | 35 | 2 | 24 |

Distinct files read in all sessions: `docs/plans/` 95 (including files that later moved), `old-docs/` 15, maps 32, tickets 466, `_closed/` 46, assets 133, `docs/adr/` 0.

### Plans (C-43)

- 104 of 407 code-work sessions read a plan (26%). 89 of the 104 read it before their first code edit. Reviewers read plans too: 55 of 223.
- 49 of the 91 implementer delegates read a plan (54%). In 50 of their 51 (session, plan) pairs the brief names the plan (known). A plan is read because a brief points to it.
- 120 of the 139 tool reads of a plan start at a line offset or have a limit (known). Agents read plans in slices. Whether a Status line at the top is seen then is not known.
- No data shows whether a "limits" section, a landing order or a Status rule changes what an agent does with a plan. This research cannot measure that (known).

### Maps and tickets (the work list, C-42)

- The `.scratch` map is the first doc, plan or skill that code-work sessions read in 96 of 383 sessions that read one (25%). Next: skills 67, plans 57, tickets 39, root `AGENTS.md` 33, `CODING_STANDARDS.md` 33 (known).
- 159 code-work sessions read a map, 188 read a ticket, and 135 of the 188 read both.
- Commands that search `.scratch` for `Status` lines: 677 commands in 276 sessions (20% of 1355). 484 of the commands name one effort folder (238 sessions). 33 name `.scratch` as a whole (22 sessions). 160 could not be sorted. So at most 193 commands in about 110 sessions (8%) can be a search across efforts (inferred).
- 986 commands list or find under `.scratch`, in 464 sessions. Most look into one effort (inferred, not sorted).

### Decisions (C-44)

- No session read a `docs/adr/` file. The folder does not exist (known: `ADR-FORMAT.md` says to create it lazily). 79 calls in 38 sessions name the path `docs/adr`; they are `bash`, `write` and search calls, with no file read.
- `.scratch/_closed/` is read in 31 sessions: 10 code-work, 9 planning, 9 research. 46 distinct files.
- A ticket file and a map file are read in 516 and 414 sessions. Tickets hold the `## Answer` and maps hold `## Decisions so far`. So these are the places where an agent meets a decision today (inferred; the data does not show which section the agent read).
- 85 sessions (6% of 1355) use a tool that reads or searches past sessions (`session_read`, `session_search`, `session_list`, `recall`): 44 code-work, 21 planning. This is a sign that some agents search old sessions for an earlier decision (inferred; the tool calls do not show the reason).
- The data cannot show how often an agent reversed a decision for lack of a record. The Hex argument (R07-17, Dax Raad, "agents undo a choice") is not testable here.

## 5. Part 4: doc churn

Commits since 2026-08-11, without merges: 399. 164 change a `.rs` file in `src/`. The doc paths are `docs/diataxis/` and `docs/specs/`. A "generated" file is one of `docs/AGENTS.md`, `AGENTS.md`, `tests/AGENTS.md`, `guardrails.md`, `http_routes.md` (the pre-commit hook regenerates them). Known: `churn.py`.

| Kind of commit | Commits | Share |
|---|---|---|
| touches a doc and `src/` | 126 | 71% |
| touches a doc and `tests/` code, no `src/` | 20 | 11% |
| only regenerates a generated file | 16 | 9% |
| docs only (no code) | 12 | 7% |
| docs and scripts, data or assets (no `src/`, no `tests/`) | 4 | 2% |
| **Total** | **178** | |

By month (all kinds): 2026-08 12, 2026-09 41, 2026-10 112, 2026-11 13. In October the `docs/diataxis/` count alone is 105 of 259 commits. Ticket 08 had 93 of 234. I could not reproduce its range; a different branch or date range is the likely cause (inferred).

Other facts:

- 126 of the 164 `src/` commits (77%) also change a spec or a non-generated Diátaxis doc. 100 touch a spec. 87 touch a Diátaxis doc.
- Median: 3 non-generated doc files per commit. Lines changed in docs: 15,474 in the 126 commits with `src/`, 1,972 with tests only, 895 with scripts or data, 390 in the 12 docs-only commits.
- The most changed docs (commits): `reference/frontend/dashboard.md` 42, `specs/browser_dashboard.md` 40, `specs/settings.md` 31, `specs/prompt_presets.md` 30, `specs/actions.md` 27, `reference/game_flow.md` 27, `reference/storage.md` 27.
- Subjects of the 126 commits with `src/`: `feat` 56, `refactor` 25, `fix` 24, `test` 9, `docs` 6, `chore` 3, other 3.
- **Does a doc follow its code?** `churn2.py` pairs each Reference doc with the source files that name it in an anchor. 473 (commit, doc) cases touch an anchored file. In 133 of them (28%) the commit also edits the doc. `dashboard.md`: 33 of 108. `game_flow.md`: 20 of 95. `storage.md`: 21 of 65. A doc that is edited in most commits of its area would "copy the code" (the `docs/AGENTS.md` rule). No doc is near that, except the two generated ones (inferred).
- **The 16 docs-only or doc-and-tooling commits.** By their subjects (inferred): 5 rewrite or trim for style (for example `a4dc7228`, `a58af5b9`, `9c51803d`, `47f97eef`, `1b29f70b`), 4 correct a doc to match shipped behaviour (`42002bd8`, `2f4d787d`, `95ebb6ee`, `1d8f3422`), 5 record plans or tickets (`e31c30b5`, `8036bbc3`, `5523248f`, `19712ddd`, `3af5f2df`), 2 are build changes that also touch docs (`478b582c`, `f3647e37`).

## 6. Recommendations for C-42, C-43 and C-44

| Practice | Recommendation | Reason (from the data) |
|---|---|---|
| **C-42** One ranked work list and a live-constraints block | **Drop.** | Agents look inside one effort folder (484 of 677 status searches). A search across efforts is in 22 to about 110 sessions of 1355 (2% to 8%). The map is the document that code-work sessions read first most often (96 of 383 sessions). The owner picks the next ticket, and this data covers agents only. If the owner wants a ranked list for himself, that is an owner question and not an agent need (inferred). |
| **C-43** Plan conventions (Status line, landing order, limits section) | **Keep for ticket 16, in a narrow form.** | Plans are a real input: 104 code-work sessions and 55 review sessions read them, and 89 of the 104 read before the first edit. A delegate reads a plan because its brief names it (50 of 51). The cost is low: 4 plans exist today. The data does not show a gap that a limits section would close. Ticket 16 should ask only whether to add a limits section and a Status rule to the plan template. |
| **C-44** A decision record folder | **Drop.** | No session read `docs/adr/`. Agents meet decisions in tickets and maps (516 and 414 sessions read one). `_closed/` is read in 31 sessions. 85 sessions search old sessions, a modest cost. The reversal risk that Dax Raad names cannot be measured with this data (known). Drop the folder as a separate practice. If ticket 16 wants a decision rule, it is one line in the plan template. |

## 7. Questions that the data raises for the doc practices

The questions are for the coordinator, who turns the map entry **Docs organized as feature maps** into tickets. Each question names the data and the practices it touches.

1. **How should an agent find the doc for the code it touches? (C-09, C-11, C-01.)** Agents follow the `[DOC: …]` anchor in 14% of the cases (20% in October). Delegates read 58% of their docs because the brief names them. Is the aim a route that agents find alone (anchor, module map, capability page), or a route that the brief and the skills carry? The two aims need different work.
2. **Do agents need the Explanation docs? (C-01, C-03.)** 12% of code-work sessions read an Explanation doc, against 41% Reference and 40% specs. Eight Explanation files were read at all. Is the Explanation layer meant for agents, for the owner, or for both? The data cannot show which.
3. **Do the docs mislead an agent when it reads them? (C-03, C-11.)** The data shows that docs are read (56% of code-work sessions read a domain doc) and that they change with the code (28% co-change per area). It does not show any case where a read doc was wrong. A follow-on AFK ticket could sample the 230 (session, doc) pairs where a session read a doc and then edited it, and count the edits that fix a stale statement. That would price C-03 and C-11 with evidence.
4. **Is a spec the right unit for a capability page? (C-09, C-10.)** Specs are the most read doc type (37 files, 164 sessions). `browser_dashboard.md` and `actions.md` head the list. Autoplay is split over two specs. The data does not say that agents need one page per capability. It does say that agents already read specs by file name, so a change of the unit changes what a brief and a skill name (inferred).

C-02, C-10 (stable ids) and C-32 (numbers in test names) have no data here. No session measure shows whether exact values or ids help an agent. These stay owner questions, with the `docs/AGENTS.md` rule "Values stay in the code" as the written conflict.

## 8. What I could not find or did not read

| Gap | Effect |
|---|---|
| A read is not use. A session may read a doc and ignore it. | The counts are an upper bound on use and a lower bound on reads (globs, `codemode`, heredocs, `git show` are missed). |
| Which edits `bash` made. | Some implementer sessions are `unknown` (27). Code-work may be a little higher. |
| Why 315 October sessions have no `project_context` section. | C-01 cost cannot be set exactly. |
| Whether non-delegate prompts name docs. | The brief share (58%) is for delegates only. |
| Whether the anchors of old sessions equal those of today. | The anchor rates may be off. |
| The reason a session called a past-session tool. | The 85 sessions are a sign, not a proof. |
| Who ran the 276 `unknown` sessions and why. | They may hold doc use that no rule found. |
| The cause of the gap to ticket 08 (3 to 14%). | The counts here use stricter paths. |
| The `backpass` audit runs (62 sessions) and their output. | They audit past sessions against `AGENTS.md` (first message marker). I did not look for their reports. |
