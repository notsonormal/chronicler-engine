# Research 04: the ways this repo makes writers align

Ticket: `issues/04-research-ways-to-align.md`. Evidence: the working tree and local git history at head `c103092f` (2026-10-22). Read-only. No build was run.
Labels: **known** = I read it in a file or in git. **inferred** = one reading step from known facts. **guessed** = weak signal.
Writers: **agent session** = a session with no memory of other sessions. **person** = the owner or another human who can remember and talk.
Domain words follow `CONTEXT.md`. A path with no prefix is relative to the repo root.

## Summary

- Known: I found 20 ways. The ticket's minimum list gives 9 (`CONTEXT.md`, `CODING_STANDARDS.md` with `AGENTS.md`, skills, `arch-lint.toml`, guardrail tests, seam contracts, generated indexes, review, precedent). I added 11 (list below).
- Known: only 3 ways always reach an agent session: `AGENTS.md` (with its generated Structure block), the pi system-prompt addendum `.pi/APPEND_SYSTEM.md`, and the compiler. The other ways need a trigger, a gate run, a brief, or chance.
- Known: no check in the repo compares a symbol or a word with `CONTEXT.md`. The checks that touch `CONTEXT.md` test its source paths only (`scripts/validate_docs.py:208`). The two banned-name rows ban `static`/`Atomic*` and `Narrator*`/`*Agent*` in named files (`tests/infrastructure/guardrails/walker_rows.rs:13-30`).
- Known: structure and behaviour have many ways that fail a build (arch-lint, 7 walker rows, `CASE` and `SCENARIO` tags, DOC anchors, clippy). Names have one written source (`CONTEXT.md`) and no way that fails a build.
- Known: three ways name `CONTEXT.md` as the source of language: `AGENTS.md:390` (no verb of reading), `.pi/APPEND_SYSTEM.md:14`, and 8 files in 7 of 45 skills. `CODING_STANDARDS.md:7` names `docs/` as the source of symbol names.
- Known: ways fail in four recorded forms: stale text (`04c9180d`), a writer bypasses the way (`--no-verify`, `428f676b`), the way sees less than the rule needs (`walker_rows.rs:65-72`, `arch-lint.toml:20`), and a rule marked review-only (10 "Check: review-only" lines in `tests/STRATEGY.md`).
- Known, extra input: 6 control names (`MessageService`, `NarrativeState`, `InputBuffer`, `ProcessActionResult`, `GenerationGate`, `GameState`) have no `CONTEXT.md` line at any time I checked, and fell to 0 in the same commit as Snapshot (`36a1e1c7`). That commit follows a plan that lists the deletions (`docs/plans/rust-rebuild-build-plan.md:299`).
- Inferred: the falls of Snapshot, Action Pipeline and Engine commit follow the deletion of the code that held the word, which a plan ordered. A plan alone explains them. The entry is also a step of that plan (`:381`).
- Main limit: git shows no read. I cannot say which writer met which way. One identity wrote 782 of 787 commits (ticket 03).

## Ways found

Minimum (9): `CONTEXT.md`; `CODING_STANDARDS.md` and `AGENTS.md` (two rows); skills; `arch-lint.toml`; guardrail and architecture tests; seam contracts and `CASE` tags; generated indexes; review; precedent in the code.

Added by me (11): A1 plans and tickets under `docs/plans/` and `.scratch/`; A2 the pre-commit hook; A3 the `build.py` gate and its checker scripts; A4 feature specs and `validate_feature_spec.py`; A5 `tests/STRATEGY.md` and its Check lines; A6 Diátaxis docs, DOC anchors and `validate_docs.py`; A7 the compiler and clippy; A8 the permission config; A9 the delegate roles and the brief; A10 the pi system-prompt addendum `.pi/APPEND_SYSTEM.md`; A11 retro and incident rules.

Not a way (known): `.pi/extensions/mrn-context/index.ts` injects the model name, build log and usage into a turn. It aligns no design.
Outside the repo (known): the owner's live instructions in a session. They have no file. I did not count them as a way.

## Table

"Reach" says which writers meet the way. "Meet" says how a writer meets it: **loaded** (always in context), **pointer** (a line the writer follows), **trigger** (a skill or command), **gate** (a build step), **brief** (a delegation message), **chance** (the writer opens the file).

| # | Way | What it aligns | Reach | How a writer meets it | How it fails (recorded) | Evidence | Label |
|---|---|---|---|---|---|---|---|
| 1 | `CONTEXT.md` (153 lines, 22 commits) | Names, avoided words, the meaning of a concept | Agent sessions that follow a pointer; people | Pointer: `AGENTS.md:390`; `docs/agents/domain.md:26`; 7 skills; addendum A10 | Avoided words stayed in code in 6 terms; entries followed code in some; the file contradicts itself in places; no check reads its terms | `AGENTS.md:390`; `docs/agents/domain.md:12,26,28`; ticket 03; section 1 below | known |
| 2a | `AGENTS.md` (408 lines) | Process, build commands, delegation and permission rules, module names with one-line summaries | Every agent session in the repo; people by choice | Loaded. Lines 7-311 are the generated Structure block | Size: 305 of 408 lines are the index. Stale lines: C6 left 3 lines of `tests/AGENTS.md` that named deleted files, "and only a review found them" | `AGENTS.md:7-311`; commit `04c9180d`; session system prompt (this session carries it) | known |
| 2b | `CODING_STANDARDS.md` (35 lines) | Symbol naming rule, comment rules, review rules, test rules | Agent sessions that follow the pointer; reviewers | Pointer: `AGENTS.md:349` ("Read it before writing or reviewing code"); skills `code-consistency-check`, `code-review` read it | The naming rule maps symbols to `docs/`, "where such symbols are present", not to `CONTEXT.md`. No check tests the rule | `CODING_STANDARDS.md:7`; `.agents/skills/code-consistency-check/SKILL.md` (Setup); section 2 below | known |
| 3 | Skills in `.agents/skills/` (45 dirs; 21 locked to outside sources) | Process (review axes, plan steps), a design vocabulary (`codebase-design`: "Use these terms exactly"), writing rules | Agent sessions that invoke a skill; 22 skills are owner-only (`disable-model-invocation: true`) | Trigger: description match or owner command | Coverage: `CONTEXT.md` is named in 8 files of 7 skills; `code-review` does not name it. Outside skills carry outside wording (`mattpocock/skills`) | `skills-lock.json`; `.agents/skills/codebase-design/SKILL.md`; `.agents/skills/code-review/SKILL.md` step 3 | known |
| 4 | `arch-lint.toml` (193 lines) | Structure: which layer may import which (15 deny rows in the generated table) | Any writer whose change reaches the gate | Gate: `tests/infrastructure/architecture.rs`; run by `build.py` | Scope: it skips `**/*_tests.rs` and `**/tests/**`, because arch-lint 0.4.3 "cannot distinguish a test fake from a production leak". It sees imports, not names | `arch-lint.toml:11-17,20`; `docs/diataxis/reference/coding_standards/guardrails.md` (arch-lint table) | known |
| 5 | Guardrail and architecture tests (`tests/infrastructure/`; 191 guardrail tests on 2026-10-21) | Structure (file shape, DOC anchors, mod purity), 7 design-rule rows, the unit-test allowlist | Any writer whose change reaches the gate | Gate: `build.py guardrails`. A failure names the rule | The row sees files, not functions (rule 13). The legacy list names files that still hold unit tests. The rows ban structure words only | `walker_rows.rs:9-30,65-72`; `unit_test_allowlist.rs:1`; `logs/build_history.txt` (row `guardrails=191`) | known |
| 6 | Seam contracts and `CASE` tags (103 tags in 5 files) | Behaviour a box promises at a port | Any writer who edits a declaring test file; any writer who adds a unit test | Gate: step `seam-cases`; the unit-test allowlist guardrail | A file with no tag is not a declaring file, so the tag check does not scan it. "A unit test in a declaring file calls only that box's entry point" is review-only | `scripts/validate_seam_cases.py:1-47`; `tests/STRATEGY.md:45`; `git grep` of `// CASE` | known |
| 7 | Generated indexes (4 files: `AGENTS.md` Structure, `tests/AGENTS.md`, `docs/AGENTS.md`, `guardrails.md`) | Names of modules and files, with the `//!` line-2 summary of each; the doc catalogue; the guardrail table | Agent sessions through `AGENTS.md` (loaded); people | Loaded (`AGENTS.md`) or pointer (the others) | The hook aborted on a routine difference; agents used `--no-verify`, "which skips regeneration and lets a stale index land" | `scripts/precommit_regenerate.py:1-9`; commit `428f676b`; `AGENTS.md:347` | known |
| 8 | Review (skill `code-review`; the whole-rebuild review; owner review) | Anything the reviewer is told to check: standards, spec, docs hygiene, test quality | The diff under review | Trigger: owner command, or a step of the after-plan skill (steps 10-11) | The reviewer reads what the brief names. `code-review` step 3 says "such as `CODING_STANDARDS.md`" and names no glossary. A review found the stale lines of `04c9180d` after the gate passed | `.agents/skills/code-review/SKILL.md` steps 3-4; `.scratch/_closed/rust-rebuild-build/assets/whole-rebuild-review/findings/docs.md:10` (DOC-4); `slop-and-hacks.md` (S-11) | known |
| 9 | Precedent in the code | Names, shape, idiom: a writer copies what it reads | Agent sessions and people that open the neighbouring code | Chance | The precedent keeps its own word. `NpcCard` has 122 `src/` lines at the Character entry, 184 at `1d8f3422`, 124 at head. `ScriptedModel::turn` is new code (`90128752`) beside a deprecated word | Section 3 below; `code-concepts/assets/research-10-synthesis.md` §1.1 rows X01-X07 (concepts that hold in all 7 code areas) | known (counts), inferred (copying) |
| A1 | Plans and tickets (`docs/plans/` 21 files; `.scratch/` 26 maps, 457 issue files) | The design shape, the order, the deletion list, and new names before the code | The session that reads the plan; delegated sessions through the brief | Trigger: the owner starts a plan or ticket; commit messages cite `Plan:` and `Ticket:`; after-plan step 1 | Plans are exempt from doc rules (`docs/AGENTS.md:241`). Nobody revises them to follow the glossary (ticket 03). A plan can leave stale doc lines for a later step (`36a1e1c7`) | `docs/plans/rust-rebuild-build-plan.md:299,338,381,389`; commit `90128752`; `.agents/skills/chronicler-after-plan-workflow/SKILL.md` step 1 | known |
| A2 | Pre-commit hook (`scripts/git-hooks/pre-commit`) | The four generated files only | Any writer who commits with the hook installed | Gate-like: runs at `git commit` | `--no-verify` skips it. A foreign edit in the file aborts the commit | `scripts/git-hooks/pre-commit`; `428f676b` | known |
| A3 | `build.py` gate and its checkers (GATE_ORDER has 18 entries) | Format, data, clippy, test structure, spec coverage, seam cases, docstrings, routes, docs, duplicates, guardrails, tests | Writers who run it. `AGENTS.md` says every build action goes through it | Gate: by instruction (`AGENTS.md:355`; after-plan step 12) | No CI file: `.github/` does not exist. A narrow run (`test-pattern`, `--no-browser`) skips steps. Nothing fails a commit that skips the gate (A2 runs the indexes only) | `build.py:770-789`; `ls .github` (absent); `AGENTS.md:355-370` | known |
| A4 | Feature specs (`docs/specs/`, 28 files, 356 scenarios) and `validate_feature_spec.py` | Behaviour in client terms; the link scenario - test by `SCENARIO` tag (366 tag lines) | Writers who change a route or a spec | Gate: step `spec-coverage` | 72 tests sit in a quarantine with no tag (count may only go down; it was 86 at `3c0cba93`). Spec text restates internal names in client terms (`4eeadba2`) | `scripts/validate_feature_spec.py:1-20,81`; commit `3c0cba93`; commit `4eeadba2` | known |
| A5 | `tests/STRATEGY.md` (249 lines) and `tests/AGENTS.md` | Where a test lives; what a scenario states; each rule names its Check | Writers who write or review tests (pointer `CODING_STANDARDS.md:30`) | Pointer, then gate for the rules that have a step | 10 "Check: review-only" lines: no step fails on them | `tests/STRATEGY.md:3-5,45,61,114,132,136,142,153,185,206,220` | known |
| A6 | Diátaxis docs, DOC anchors and `validate_docs.py` | Names, invariants (Reference), rationale (Explanation); the link from a `src/` file to its Reference doc | Writers who open the doc or the file (line 1 of every `src/` file) | Pointer (`docs/AGENTS.md`), gate (step `validate-docs`), chance (the anchor) | Docs and code disagree in 23 listed items; the validator checks paths, anchors and citations, not claims | `docs/AGENTS.md:23-36`; `scripts/validate_docs.py:195-210`; `code-concepts/assets/research-10-docs-vs-code.md` §2 | known |
| A7 | Compiler and clippy lints | Structure through visibility and types; no `unwrap`, `panic`, `dbg` and so on | Every writer, at compile time | Gate and local build | It checks what the types express. A word is not a type | `src/lib.rs:4`; `src/application/games/active.rs:39` (`pub(super) fn set`); commit `ac938c25` ("rule 16 is compiler-enforced") | known |
| A8 | Permission config (`.pi/extensions/pi-permission-system/config.json`) | Which tool calls a session may make (deny `*.env`, `timeout`, repo-root `grep`, edits of the config) | Agent sessions of pi; not people | At each tool call | Default is `"*": "allow"`. Git use is limited by text, not by the config (`AGENTS.md:396`). Each deny carries a reason that names a past hang | `.pi/extensions/pi-permission-system/config.json`; `AGENTS.md:392-396` | known |
| A9 | Delegate roles and the brief (`.pi/agents/*.md`, `AGENTS.md:398-408`) | What a subagent reads first and what it must return | Delegated agent sessions | Brief: the `files` list becomes "Files to read first" | Role files hold a header only (7 files, 76 to 190 bytes). `AGENTS.md` records a case where an `implementer` run "came back wrong on points its brief had already decided" | `.pi/agents/implementer.md`; `.pi/extensions/delegate/lib.ts:134`; `AGENTS.md:398-408` | known |
| A10 | Pi system-prompt addendum (`.pi/APPEND_SYSTEM.md`) | Prose style (ASD-STE100), and "Ground language strictly in ... `CONTEXT.md`" | Agent sessions of pi (all of them) | Loaded | It names `CONTEXT.md` with no word on what to do at a gap. First seen in `8a972120` (2026-08-09) | `.pi/APPEND_SYSTEM.md:14`; `git log -S'Ground language strictly'` | known |
| A11 | Retro and incident rules (skill `retro`; commits that add a check after a failure) | The other ways: it adds a gate step, a row or a pointer | Writers of the next change | Gate or pointer, after the fact | Each case follows a failure: `428f676b` (bypass), `04c9180d` (stale root docs), `977ba3f7` (rows "land with the code they guard") | commits `428f676b`, `04c9180d`, `48553d10`; `walker_rows.rs:9` | known (commits), guessed (skill role: I did not read the skills) |

## Per-way notes

### 1. `CONTEXT.md`

- Known: `AGENTS.md:388-390` sits under "Agent Skills > Domain docs": "Single-context: one `CONTEXT.md` glossary at the repo root (per `docs/agents/domain.md`)". It has no verb of reading. `AGENTS.md:349` for `CODING_STANDARDS.md` does: "Read it before writing or reviewing code."
- Known: `docs/agents/domain.md:12` says to proceed silently when `CONTEXT.md` is missing. Line 26 says "use the term as defined"; line 28 says a missing term is "a signal ... reconsider ... or a real gap".
- Known: the skills that name it: `tdd`, `wait-what`, `improve-codebase-architecture`, `document-review`, `chronicler-docs-hygiene`, `domain-modeling` (2 files), `codebase-design/DESIGN-IT-TWICE.md`.
- Known: `scripts/validate_docs.py:208` lists `CONTEXT.md` in `ROOT_DOCS`. The validator checks the source paths it names (rule `BROKEN_SOURCE_PATH`) and no other rule.
- Known: the file defines Game as "A concrete playthrough session" and its `_Avoid_` line names Session and playthrough (ticket 03, section 10). Known: whole-rebuild review finding CR-2: `CONTEXT.md:40` still said "a cancel handle" after the code removed it (`review-code.md:21`).
- Known: the 22 entry commits and the counts of avoided words are in ticket 03. I did not redo them.

### 2. `AGENTS.md` and `CODING_STANDARDS.md`

- Known: this session has `AGENTS.md` in its system prompt, so the file is loaded for agent sessions. For a person it is a file.
- Known: `CODING_STANDARDS.md:7`: "Symbols ... must use verbose, domain-aligned names that map 1-to-1 with concepts in the `docs/` (where such symbols are present)." The rule names `docs/`, not `CONTEXT.md`.
- Known: `AGENTS.md:341` says to read tests before source. `AGENTS.md:334` says to hold a reasoned position at a gap. These two lines are the nearest text to "reason from the forces".
- Known: I found no rule in `tests/infrastructure/` or `scripts/` that compares a symbol with a word of `CONTEXT.md` or `docs/`.

### 5. Guardrail rows

- Known: the 7 rows (2 banned-name, 3 call-site, 1 type-check, 1 call-count) carry rule numbers 1, 2, 3, 6, 7, 13, 18 (`walker_rows.rs`). Known: the `ac938c25` message names "Rule 11 (arch-lint ...)" and "rule 16 is compiler-enforced"; the plan names "the rule-13 row (§9.5)" (`docs/plans/rust-rebuild-build-plan.md:338`). Inferred: one numbered list in plan §9.5 spans arch-lint, the walker rows and the compiler. I did not read §9.5.
- Known: each row's reason is on the comment lines above it (`walker_rows.rs:9-12`).

### 9. Precedent

- Known: `NpcCard` was added on 2026-04-12 (`2bbad636`), before the glossary (2026-06-29). `src/` lines with `NpcCard`: 122 at `bc774107` (Character entry), 184 at `1d8f3422`, 127 at `4eeadba2`, 124 at head. The Character entry says "NPC (use Character)".
- Known: `git grep 'fn turn\b'` at `90128752^` finds none. `ScriptedModel::turn` is in `src/test_support/scripted_model.rs:81` at head. `CONTEXT.md` has had "Turn: Use Message + Swipe" since `bc774107`.
- Known: the code-concepts synthesis lists concepts that hold in all 7 code areas (rows X01-X07: ports and adapters, dependency rule, composition root, single writer, concrete type over trait, sync core, errors as values). Inferred: a shared shape across areas is precedent that held; I did not test that writers copied it.
- Known: two skills treat precedent as a check. `code-consistency-check`: "Match existing patterns, not 'clever' code". After-plan step 9: check that code is "consistent with any existing patterns or, if the new pattern is an improvement, that older code is updated to match".
- Guessed: an agent session opens the files its task names and the files that grep returns. Git cannot show which files it opened.

### A9. Delegate roles and the brief

- Known: `.pi/agents/*.md` hold `name`, `model`, `thinking` and (for 5 of 7) `tools`. They have no body.
- Known: the brief of this task names the ticket, the map, `AGENTS.md` and `CODING_STANDARDS.md` as files to read first, and lists rules for the asset. It does not name `CONTEXT.md`. Inferred: a delegated session meets a way only if the brief or the loaded files name it.
- Known: `AGENTS.md:398-408` tells the delegating agent what a brief must hold for each role ("names the design, the files and the checks").

## Meeting points between the ways

1. `CONTEXT.md` and the plan: the build plan lists "`CONTEXT.md`: **Snapshot** goes. The terms of target design §1 enter" as one of its own C7 steps (`docs/plans/rust-rebuild-build-plan.md:381`). Which of the two carries a term to a writer who reads only one?
2. `CODING_STANDARDS.md:7` and `CONTEXT.md`: the naming rule points at `docs/`; the glossary lives in `CONTEXT.md`. When the two disagree, which one does a writer follow? (Ticket 03: `docs/diataxis/reference/storage.md` has the heading "Transactions"; the entry says avoid Transaction.)
3. Precedent and the entry: `NpcCard` and `Storage::transaction` stand in code that the entry names as avoided. Does a writer who reads both copy the code or follow the entry?
4. Arch-lint, the guardrail rows and the compiler: they share one numbered list of rules (1 to 18). Is that list a single place where three ways meet, and who updates it?
5. `tests/STRATEGY.md` Check lines: each rule names a step or "review-only". Which rules have two ways that check them, and which have one?
6. Review and the gate: the review found stale root-doc lines after a green gate (`04c9180d`). Which way finds a stale name that a green gate leaves?
7. Hook, gate and `--no-verify`: a writer can skip the hook and the gate. Does the way that is skipped leave a trace for the next writer (stale index; no log row)?
8. Brief and loaded files: a delegated session gets `AGENTS.md` and the brief. If the brief omits `CONTEXT.md`, is the pointer at `AGENTS.md:390` enough?
9. `.pi/APPEND_SYSTEM.md:14` and `AGENTS.md:390`: two always-visible pointers to `CONTEXT.md` with different wording ("Ground language strictly" against "Single-context"). Which wording does a writer act on?
10. Specs and the glossary: `4eeadba2` restated internal names "in client terms" in specs. Do specs and `CONTEXT.md` give two vocabularies for one thing?
11. Plan and precedent: a plan that deletes the module also deletes the precedent that held the old word (`36a1e1c7`). Where no plan deletes the module (`NpcCard`), does the precedent stay?

## Which other ways would produce the same falls without the entry?

This answers the last open question of ticket 03. I did not redo its counts.

**Known: what the three named falls look like**

| Term | Fall in code | The commit | What drove it (in the commit or the plan) |
|---|---|---|---|
| Snapshot | 760 -> 37 lines | `36a1e1c7` (core C6, 2026-10-19) | The plan's Deletions row names "Snapshot (records, storage, mappers, schema)" (`docs/plans/rust-rebuild-build-plan.md:299`). C6 says "the Snapshot tables leave the schema" (`:338`). The C6 message lists the deletions and does not cite the entry |
| Action Pipeline | 45 -> 0 lines | `36a1e1c7` | The same row names `ActionPipeline` "with `PipelineRun` and its five entry files" (`:299`) |
| Engine commit | 3 code lines -> 0 | `36a1e1c7` | The plan does not contain the words (0 hits). All 3 code lines (`phase_engine_commit` and one label) were in `src/application/pipeline/action_pipeline/core.rs` at `9cbf3112`. `36a1e1c7` deletes that file (`583 ---`). Inferred: the word left with its file |

**Known: the plan also puts the entry change inside the plan.** `docs/plans/rust-rebuild-build-plan.md:381` lists `CONTEXT.md` ("Snapshot goes") in the step C7 table. `4eeadba2` (C7) made that change and its message says "Snapshot and Action Pipeline go". The plan has a done-check: "A search of `src/` and `tests/` for `GameStateSnapshot`, `save_snapshot`, `load_latest_snapshot` and `snapshot_id` finds them only in the schema migration history" (`:389`). Ticket 07 records that the search ran: 2 lines, both in `plumbing.rs`, "Clean" (`.scratch/_closed/rust-rebuild-build/issues/07-core-step.md:676`).

**Known: a control inside this repo.** Six names that `CONTEXT.md` never contained fell (all but 4 `<GameState>` tag lines) with the same commit. Counts of lines in `src/` and `tests/`, at `51efaafc` (before C6), `36a1e1c7` (C6), head:

| Name | `51efaafc` | `36a1e1c7` | head | `CONTEXT.md` lines (before C6, head) | Named in plan line 299 |
|---|---|---|---|---|---|
| `MessageService` | 28 | 1 | 0 | 0, 0 | yes |
| `NarrativeState` | 12 | 0 | 0 | 0, 0 | yes |
| `ProcessActionResult` | 49 | 0 | 0 | 0, 0 | yes |
| `InputBuffer` | 7 | 0 | 0 | 0, 0 | no (named in the C6 message) |
| `GenerationGate` | 26 | 0 | 0 | 0, 0 | no (named in the C6 message as "the old GenerationGate") |
| `GameState\b` | 167 | 8 | 4 | 0, 0 | yes (the 4 at head are the prompt tag `<GameState>`, another sense) |

Inferred: a plan that orders a deletion produces the fall of a name with no entry.

**Known: the two other falls (ticket 03, items 6 and 7).**
- Narrator Action: `narrator_action` was a method of the old pipeline. It left in `d03c337f` ("Extract narration_generation module and fix test-police findings 1-4", 2026-08-31). The message does not cite the word or the entry. The commit names a review (test-police findings) and a code extraction. Inferred: a review-driven refactor removed the word.
- Replay blob: removed in `45033a6e` ("flatten swipe generation inputs"; "Ticket: narrator-modes-and-options #16"). The message lists a `CONTEXT.md` change. A ticket drove it.

**Known: where a review or a cleanup edited words one by one.**
- `321f78e2` (2026-10-11): "Say 'message', not the deprecated 'turn'" (docs 18 -> 15, code 62 -> 65 per ticket 03).
- `2885df2c` (2026-10-19): the fixes of the whole-rebuild review. It includes "'turn' and plan words out of comments and error text" (comments 23 -> 18). The review's docs scout reported "Deprecated term 'turn'" as DOC-4 (`findings/docs.md:10`).
- Inferred: a review is the only way in the repo that I found which edits an avoided word as a word. Its effect in these two commits is small.

**Known: what no way did.** No guardrail row, lint or test names an avoided word. I searched `tests/infrastructure/` for `snapshot`, `action_pipeline`, `turn` and `NpcCard`; the only hits are the `tree_snapshot` and `src_snapshot` helpers (a source-tree snapshot, another sense).

**Inferred, in order:**
1. A plan, a ticket or a review-driven refactor explains each of the 5 falls without the entry. In 4 of the 5 the commit or the plan names the code that carried the word (Snapshot, Action Pipeline, Narrator Action, Replay blob). In the fifth (Engine commit) the word left with its file.
2. The 6 terms whose avoided word stayed (ticket 03) are words whose code no plan deleted: `NpcCard`, `Storage::transaction`, `ScriptedModel::turn`, `steering_instruction`, `retry`.
3. The entry and the plan are one act in the C7 case: the plan lists the entry edit.

**Not shown:** whether the writers of C6 read `CONTEXT.md` first. The plan could have named the same deletions with no entry; I found no case of a deletion without a plan, and no case of an entry without a plan or a ticket in the same window. A repo with a glossary and no plan, or a plan and no glossary, would be the control. This repo has none.

## Questions raised

1. Is a plan a vocabulary? The plan names the old words, the new words and the files to delete in one table (`:299`), and it schedules the glossary edit. Does the glossary add anything the plan has not already put in front of the writer?
2. Only 3 ways load into every agent session (`AGENTS.md`, the addendum, the compiler). Only the first two carry words, and both point to `CONTEXT.md` without a rule for a gap. What does a session with no other way meet?
3. Names have one written source and no failing check; structure has several failing checks. Does that difference show in the counts of ticket 03 (words stayed, structure held)?
4. A way that is skipped leaves a trace in some cases (the stale index of `428f676b`) and no trace in others (a session that never opened `CONTEXT.md`). Which ways can a later writer see were skipped?
5. A review finds an avoided word as a word (DOC-4). Which input made the docs scout look for it: the glossary, the docs-hygiene skill, or the brief? I did not read the scout's brief.
6. The delegate role files carry no body, and the brief carries the design. What does a delegated session know about the project's ways that the brief did not give it?
7. `AGENTS.md:334` ("Hold a reasoned position") and `:341` are the nearest text to "reason at a gap". No way in the table tells a writer what to do at a gap in the vocabulary except `docs/agents/domain.md:28` ("reconsider ... or note it for `/domain-modeling`"), which a person or a skill reads. Who notes the gap when no skill runs?
8. Two ways record the rule's own strength: `tests/STRATEGY.md` ("Check: review-only") and `walker_rows.rs` (one reason per row). Does any way record how strong the vocabulary rule is?
9. Writer assumptions: every way here assumes a writer that can read the repo. Ways 1, 2a, 3, 8, A1, A9 and A10 also assume the writer follows a pointer or a brief. Ways 4 to 7, A3, A4 and A7 assume only that the writer runs the gate. Does the split (reads versus runs) hold for people in a team of 7 who also talk?
10. Which of these ways would a team with several other teams (the map's transfer context) have, and which exist only because this repo has agent sessions with no memory (A8, A9, A10)?

## What I did not do

- I did not read the bodies of the skills `retro`, `wayfinder` or the docs-hygiene scout brief. A11 is partly guessed for this reason.
- I did not run the gate, any test or the hook. The count of 191 guardrail tests and the gate order come from `logs/build_history.txt` and `build.py`.
- I did not check whether a session opened a file. Git and the repo do not record it.
- I did not open `charting-notes.md`, `research-01-*` or `research-02-*`.
- I did not change any file except this asset. The scratch folder `tmp/r04/` was not needed.
