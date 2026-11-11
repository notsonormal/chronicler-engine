# Dispositions: a proposal for each candidate

Written by [Propose a disposition for each candidate](../issues/01-research-propose-dispositions.md). Read at `HEAD` `20d6d23c` (2026-11-04) plus the working tree. Each grilling ticket fills the **Decision** column of its rows.

## Totals

| Grilling ticket | Rows | Needs the owner | Confirm in a batch | Fix here | Route | Drop |
|---|---|---|---|---|---|---|
| [03 effort close and review](../issues/03-grilling-effort-close-and-review.md) | 6 | 3 (R2, R14, R22) | 3 (R3, R15, B5) | 4 | 1 | 1 |
| [04 tooling, trial harness, target dirs](../issues/04-grilling-tooling-trial-harness-target-dirs.md) | 9 | 3 (R1, R6, R17) | 6 | 7 | 0 | 2 |
| [05 product fault routes](../issues/05-grilling-product-fault-routes.md) | 10 | 3 (R5, R9, R10) | 7 | 3 | 7 | 0 |
| All | 25 | 9 | 16 | 14 | 8 | 3 |

No row moved to another grilling. R21 stays in 04 although its `PINNED_WORDS` part depends on R5 (05).

## The evidence check

`git diff --name-only 20d6d23c` lists 7 files: `.gitignore`, `.pi/extensions/delegate/lib.ts`, `.pi/extensions/delegate/lib.test.ts`, `docs/agents/issue-tracker.md`, and 3 files of `work/wayfinder-maps/long-term-memory-and-state/` (`map.md`, `issues/05-memory-candidates.md`, `issues/07-card-change.md`). No row cites the first four. Only two rows cite a changed file:

- **R1** cites `long-term-memory-and-state/map.md:74`. Re-read: the line still names `tmp/ltm09/redmist_julian/trial-1` as the new reference trial. It holds, at the same line. The other R1 citation, `assets/baseline-faults.md:5-7`, is in a resolved ticket's asset and is not on the diff list.
- **R14** cites `map.md:25,58`. Re-read: `/tdd` is still at line 25. The "Standing rules" bullet moved from line 58 to line 57. It holds.

Every other cited file is unchanged since `20d6d23c`, so the report stands for it. I re-read some lines anyway, where a proposal needed the exact code: the byte-500 slice (R4), the lint list of `src/lib.rs` (R4), `_gate_tail` and `clean_tmp_dirs` (R1, R20), `MAX_LINES` (R19).

Untracked files (not in the diff): the new ticket and asset files of the long-term-memory map (`issues/10`, `11`, `assets/card-probe*`). No row cites them.

## Grilling 03: how an effort closes and what review owns

Evidence column: "unchanged" means unchanged since `20d6d23c`.

| Row | Evidence | Owner (map, ticket) | Fix size and files | Class | Proposal | Reason | Who decides | Decision |
|---|---|---|---|---|---|---|---|---|
| **R2** defects that span tickets reach `main`; the close-out is typed by hand | unchanged; `git log --merges` since 10-28 lists 2 merges, both on 10-30 | none. Searched bucket, all active maps. Near: bucket 86, 99, 100. Won't do 61 is a map close-out, a different operation | one session. New `.agents/skills/close-effort/SKILL.md`, a pointer in the "Decide, then build" Notes of active maps | structure | fix here | `/code-review` already takes a branch scope. The missing part is the step that calls it, and the merge form | needs the owner | |
| **R3** tests that cannot fail | session data, unchanged | Unclebob quality tools 06 (value) and 07 (shared-tree safety): open, unblocked, because their blocker 03 is resolved. Bucket 23: open | none here. A new evidence line in 06 and in 23 | check | route (evidence) | The map Notes give the mutation decision to Unclebob. Its research (ticket 03, `assets/03-mutation-engines.md:53`) found that all 10 surviving mutants are equivalent in the files it tried, and it names a throwaway driver. 9 weak tests that review found in 8 efforts is the other sample | confirm in a batch | |
| **R14** standing rules copied from map to map | changed (LTM map): holds, line moved 58 to 57 | none. Bucket 99 and 95 are near; neither moves the rules to one file | one session. New `docs/agents/build-ticket.md`, a pointer in `AGENTS.md`, one small edit in each of the LTM and dashboard-design maps | structure | fix here | 2 active maps carry copies. One names the deleted `/tdd`. The copies also sit in the done maps layout-trials and prompt-flexibility, which stay as they are | needs the owner | |
| **R15** grilling rounds too large | unchanged. `grilling/SKILL.md:10` still says "a maximum of 5 questions" | bucket 88 (open). Won't do 10: the new evidence is the volume, not the wording | tiny. `.agents/skills/grilling/SKILL.md:10` (5 to 3), and one line for the recommendation | output request | fix here | The owner already set 3 in this map and in hex-comparison. The skill still says 5. Pattern 6 (a verdict line) joins the same edit | confirm in a batch | |
| **R22** unused and overlapping skills | unchanged. 37 `SKILL.md` files in `git ls-files` | none. Searched bucket and maps: no ticket | tiny. Delete up to 3 skill folders under `.agents/skills/` | one-off | fix here | `chronicler-after-plan-workflow-plus-review` says it is "intentionally a copy". No live map, doc or skill step names the two `chronicler-after-plan-workflow*` skills. `grill-me` is a 7-line alias of `/grilling`, 0 uses | needs the owner | |
| **B5** the hex-comparison lesson is not in `wayfinder/SKILL.md` | unchanged (the skill is unchanged; the lesson is at `_closed/hex-comparison/assets/conclusion.md:106`) | none. Searched bucket and maps | one sentence in `.agents/skills/wayfinder/SKILL.md`, step "Map the frontier" | vigilance rule | drop | No evidence that such a rule held before (bucket audit, class "vigilance"). The long-term-memory map put its measures first (tickets 03, 04) without a rule. Alternative if the owner disagrees: the one sentence | confirm in a batch | |

R22 note: the report counts 17, 2 and 0 uses for the three grill skills and 12, 2 and 3 uses for the three after-plan skills (`findings/tooling-env.md` §9). It does not say which count belongs to which skill. The maps name `/simple-after-plan`, so I infer that it has the 12. `grill-me` is the 0.

### Questions for grilling 03 (round 1: 3 questions, and one batch confirm)

**Q1. R2: how does an effort close?** Each ticket gets its own review. No step reviews the whole branch. On prompt-flexibility, a review of the whole branch then needed an 8-phase fix plan. Layout-trials merged on 2026-11-03 with per-ticket reviews only (inferred). The owner typed the close-out 4 times, and the merges after 2026-10-30 were fast-forwards. A reviewer session costs $1.68 on average. A whole-branch review is one more reviewer run on a larger diff (inferred). The most expensive efforts of the week cost $20 to $104.
- A. A last ticket in each map runs the whole-branch review. No new skill.
- B. Keep per-ticket reviews only. Add one line to `AGENTS.md` for the merge form.
- C. One skill `close-effort`: `/code-review` over the merge-base with `main`, the full gate, `git merge --no-ff`, switch to `main`, delete the local branch. Push stays with the owner's word.

Recommended: C.

**Q2. R14: where do the standing rules of a build ticket live?** They are the test-first rule, the `/simple-after-plan` end, the doc-direction rule and the new-API rule. Each map copies them. The live long-term-memory map names `/tdd`, a skill that no longer exists. The owner also adds missing steps by hand.
- A. Keep them in each map (today).
- B. In `AGENTS.md`, always loaded.
- C. In `.agents/skills/wayfinder/SKILL.md`.
- D. In one file, `docs/agents/build-ticket.md`. Maps and tickets point to it.

Recommended: D.

**Q3. R22: which skills go?** 8 skills had 0 uses in the week. 22 of 37 skills are hidden from the model, so unused skills cost only attention.
- A. Keep all.
- B. Delete the two `chronicler-after-plan-workflow*` skills and `grill-me`. Keep the other unused skills.
- C. Delete all 8 unused skills too.

Recommended: B.

**Batch confirm (no discussion unless the owner objects):** R3 route as evidence to Unclebob 06 and bucket 23 (no change to the gate meanwhile); R15 change the cap in `grilling/SKILL.md` to 3 and add the recommendation line; B5 drop.

## Grilling 04: tooling, trial harness, target dirs

| Row | Evidence | Owner (map, ticket) | Fix size and files | Class | Proposal | Reason | Who decides | Decision |
|---|---|---|---|---|---|---|---|---|
| **R1** trial evidence in a folder that the gate deletes | changed (LTM map): holds. `build.py` unchanged. On this machine `/workspace/tmp` does not exist, so only `tmp/` is cleaned | this map, ticket 04. Open tickets that need each folder: table below | one session with a script. `build.py` `_gate_tail`, `scripts/trial_setups/*.json` (`out`), `scripts/trial.py`, citations in 8 maps | structure | fix here | The owner's candidate: each map keeps its evidence in its own `assets/local-only/`, and `trial.py` writes there. The protective copy covers the gap until then | needs the owner | |
| **R6** `target/` inside the checkout | unchanged. Reproduced in this session: `grep -c . /dev/null` was denied by `grep * . *` | this map, tickets 02 (facts) and 04. No other owner (searched bucket: Won't do 71 is about a coverage dir, not this) | [Ticket 02](target-dir-dependencies.md): 12 files change and one new module owns the location. `CARGO_TARGET_DIR` must always be set (`build.py` `_cargo_env_for`). Ticket 04 decides the size. Then the owner edits the config | structure | fix here | Ticket 02: the move makes `grep * . *` and `grep * ./ *` unnecessary. `grep * /workspace/chronicler-engine *`, its `/` twin and `find / *` stay, because `tmp/` (2.7 GB) stays in the checkout | needs the owner | |
| **R11** `trial.py` has the shape of a script | unchanged | none. The long-term-memory check trial uses it | one session, the small part. `scripts/trial.py`, `scripts/tests/test_trial.py`, `scripts/validate_data.py` | check + structure | fix here (small part) | (a) a Python test that compares the mirrored constants and the slot fields with Rust, (b) a Connection named by provider and model, (c) a corrected docstring. Correction to the report (outside check C12): the id in `scripts/trial_setups/redmist_julian.json:14` is a Connection id string, not a probe-DB row id. What holds: `trial.py:459` writes settings with SQL, and nothing compares `usage.prompt_tokens` (`trial.py:303-305`) with the engine's estimate. The results journal goes to R1. The quality score goes to the LTM fog "The reading pack" | confirm in a batch | |
| **R17** browser tier 208 s, stale cost tables | unchanged. JUnit read: `target/nextest/default/junit.xml`, run of 2026-11-03 22:08 | none. Won't do 31 covers only the skip part | one session for the values: `AGENTS.md:374-388`, `ENVIRONMENT.md:65`, `build.py` docstring. The skip is a second session: `build.py`, `scripts/build_history.py` | check | fix here (cost values). Skip: reopen Won't do 31 (Q2) | The values are 1.5 to 3 times too low. Won't do 31 says the skip would have saved no run. New evidence: 87 gates ran on 29 `HEAD`s, and the browser tier is 210 s of a 359 s full gate. The ten slowest checks are in the table below | needs the owner | |
| **R18** structure index fills 321 of 422 lines | unchanged. `AGENTS.md` has 422 lines | Won't do 09 (owner decision, 2026-10-25) | none | one-off | drop | The `rg` count does not defeat the reason. The bucket audit (D1) names the test that would: one task with and without the block. Nobody ran it | confirm in a batch | |
| **R19** the build journal keeps one week | unchanged. `scripts/build_history.py:24` `MAX_LINES = 1000`; the file is 91,904 bytes | none. Searched bucket | tiny. `scripts/build_history.py:24` | one-off | fix here | A cap of 20,000 lines is about 1.8 MB. Rotation is not needed | confirm in a batch | |
| **R20** the dead DB dump and the stale-data trap | unchanged. `build.py:1383-1392` re-read | none. Searched bucket | one session. `build.py` (`_gate_tail`, `_copy_deployment_assets`), `scripts/tests/test_build_cli.py`, `ENVIRONMENT.md:95`, `chronicler-ui-investigator/SKILL.md:112` | one-off | fix here | Remove the dump and the `/workspace/tmp` path. Copy `data/` only with `--release`. The implementer first checks that nothing reads `<target>/debug/data` but the server | confirm in a batch | |
| **R21** tests that copy a table row for row | unchanged | none | tiny now. `scripts/tests/test_build_cli.py:58,461-490` | check | fix here (2 of 3 parts) | Delete the two copy tests. `PINNED_WORDS` waits for R5. Add no rule for the 15 connection tests: that is a vigilance rule | confirm in a batch | |
| **R23** maps and plans read again and again | session data, unchanged | none | none | one-off | drop | The report gives characters, not cost. 2.5 M plus 2.8 M characters are about 1.3 M tokens (inferred, 4 characters a token) in a week of $503. Reading a map is the job of the lead. No fix of one session cuts it without a loss | confirm in a batch | |

### R1 measure: the open tickets that need each folder

The oldest file, the maps and the size are in [the manifest](tmp-evidence.md). All citations in the manifest are repo paths by its definition. "Inferred" means the ticket reads a resolved ticket that names the folder.

| Folder | Oldest file | Open tickets that need it |
|---|---|---|
| `tmp/agents-restructure/` | 2026-10-06 | none (bucket map and assets only) |
| `tmp/character-art/` | 2026-11-02 | character-art-from-code 07 (claimed) |
| `tmp/comment-fixer/` | 2026-10-06 | none (bucket assets only) |
| `tmp/db-backup-20261031/` | 2026-10-31 | none (prompt-flexibility has no open ticket) |
| `tmp/dd-fixpass/` | 2026-10-29 | bucket 87 |
| `tmp/dup_now/` | 2026-10-09 | Unclebob 04 (inferred) |
| `tmp/final-review/` | 2026-10-27 | bucket 86 |
| `tmp/impeccable/` | 2026-11-01 | none by name (dashboard-design assets only) |
| `tmp/lt01/` | 2026-11-01 | bucket 103 |
| `tmp/lt02/` | 2026-11-01 | none (layout-trials has no open ticket) |
| `tmp/lt03/` | 2026-11-01 | bucket 103; LTM baseline stories |
| `tmp/lt05/` | 2026-11-02 | none |
| `tmp/lt10/` | 2026-11-01 | bucket 103 |
| `tmp/lt11/` | 2026-11-02 | bucket 103 |
| `tmp/lt15/` | 2026-11-02 | LTM 05, 06, 12; bucket 100, 103; LTM check trial |
| `tmp/ltm03/` | 2026-11-03 | LTM check trial (baseline stories; not yet a ticket) |
| `tmp/ltm04/` | 2026-11-03 | none by name |
| `tmp/ltm09/` | 2026-11-03 | LTM 05; LTM check trial (the reference trial) |
| `tmp/ltm11/` | 2026-11-04 | none by name |
| `tmp/maid/` | 2026-11-02 | character-art-from-code 07 (claimed) |
| `tmp/marinara/` | 2026-10-20 | none open. A clone: clone it again when needed |
| `tmp/mutation-03/` | 2026-10-31 | Unclebob 06, 07 (inferred) |
| `tmp/pf08/` | 2026-10-31 | none |
| `tmp/pf18/` | 2026-10-31 | none |
| `tmp/pf19/` | 2026-10-31 | none |
| `tmp/pf20/` | 2026-11-01 | none |
| `tmp/pf2021/` | 2026-11-01 | bucket 96 |
| `tmp/pstack/` | 2026-10-20 | none (code-concepts has no open ticket) |
| `tmp/reflect3/` | 2026-10-07 | none (bucket assets only) |
| `tmp/review/` | 2026-10-06 | bucket 86, 100 |
| `tmp/stage6/` | 2026-10-26 | none |
| `tmp/statechart/` | 2026-10-29 | statechart-models 04 (inferred) |
| `tmp/tla-ts-run/` | 2026-10-31 | statechart-models 04 (inferred) |
| `tmp/unclebob/` | 2026-10-09 | Unclebob 04, 05 (inferred) |

Three facts the manifest does not hold:

- 17 of the folders and files that open maps name under `tmp/` are already absent. Some of these are `/tmp/` paths, and some were deleted. I did not sort them. The names: `tmp/lt14`, `tmp/mutation-wt`, `tmp/dup_gate_overlap`, `tmp/transaction`, `tmp/gate-run1`, `tmp/r03`, `tmp/r04`, and the metacognitive-audit scripts (`tmp/flatten_sessions`, `tmp/extract_signal`, `tmp/locators`, and 5 more). Open ticket 07 of metacognitive-audit is the only open ticket among their maps.
- The dashboard-design map and its open ticket 01 name `tmp/dashboard-design/probe-fixture.db`. That path does not exist yet: ticket 01 makes it. A fixture there goes at 30 days like the rest.
- `tmp/research/` (1.2 MB, oldest 2026-10-21) exists. The open long-term-memory map names it as `/tmp/research/...`, an absolute path, so it is not a repo citation.

### R17 measure: the 10 slowest browser checks

Source: `target/nextest/default/junit.xml`, run `2026-11-03T22:08:19Z`. 44 tests, 620.0 s of test time, 208.0 s of wall time. The tests run about 3 at a time (inferred: 620 / 208). So the slowest test is not the wall time. The total decides it. A cut of the browser tier needs the per-check log, so I propose the `scout` ticket in the notes below.

| s | Test (`chronicler_engine::browser`) |
|---|---|
| 44.9 | `stub::screen::run_screen_checks` |
| 38.7 | `stub::dialog::run_dialog_checks` |
| 32.7 | `stub::failure_display::run_failure_display_slot_checks` |
| 27.5 | `stub::game_header::run_game_header_checks` |
| 27.3 | `stub::action_controls::run_action_controls_checks` |
| 23.9 | `stub::layout::run_layout` |
| 23.2 | `stub::reports::run_report_checks` |
| 22.8 | `stub::story_log_edit::run_story_log_edit_mode_checks` |
| 22.3 | `stub::story_log_edit::run_story_log_edit_failure_checks` |
| 22.1 | `stub::dashboard::run_dashboard_action_area_checks` |

The 10 tests sum to 285.4 s, 46 % of the test time. Each test is one runner that holds many checks. A finer timing needs the per-check log, not the JUnit report.

### Questions for grilling 04 (round 1: 3 questions, and one batch confirm)

**Q1. R1: where does evidence that a map must keep live?** The gate deletes each file under `tmp/` that is older than 30 days. The first deletes can come on 2026-11-05. A protective copy of 33 cited folders (617 MB) is in `assets/local-only/tmp-evidence/`. Open tickets need these folders: the long-term-memory check trial (`tmp/ltm09`, `tmp/lt15`, `tmp/ltm03`), bucket 86, 87, 96, 100, 103, character-art 07, and Unclebob 04 to 07.
- A. Keep the protective copy as the one home. Change the citations to it. The gate does not change.
- B. The gate stops cleaning under `tmp/`, except one named scratch folder.
- C. Each map keeps its evidence in its own `assets/local-only/`. `trial.py` writes its packs there. The citations change. The protective copy goes after a search finds no `tmp/<folder>` citation.

Recommended: C.

**Q2. R6: how do we stop the permission rules from denying safe commands?** `target/` holds 28 GB in the checkout, so the config denies `grep` with a standalone `.`. The rule `grep * . *` denied 39 calls this week, and a parse finds 0 that walk the repo root. A glob `*` crosses `;`, so one refusal can drop a whole command with a build in it. [Ticket 02](target-dir-dependencies.md) found that a move to `~/.cache/chronicler-engine/target/` changes 12 files plus one new module. `/workspace` and `/home/node` are two mounts, so the move is a copy of 28 GB or a cold build.
- A. Move the target dirs out of the checkout, then remove `grep * . *` and `grep * ./ *`. The other grep rules and `find / *` stay.
- B. Narrow the rules only (owner edit, no code). The target dirs stay.
- C. Leave it.

Recommended: A.

**Q3. R17: do we reopen Won't do 31, and skip the browser tier when its inputs are unchanged?** The Won't do says the skip would have saved no run. The week shows 49 full gates (4.45 h) and 38 `--no-browser` gates (1.35 h), 5.8 h of a 13.4 h journal. 87 gates ran on 29 `HEAD`s, up to 10 on one `HEAD`. The browser step is 210 s of a 359 s full gate (medians). The check would record the tree of each green browser tier in the journal, and skip the tier when the diff since that tree touches no browser input. Browser inputs: `src/adapters/driving/http/**`, `assets/**`, `tests/browser/**`, `tests/test_utils/**`, `Cargo.lock`. The owner decided Won't do 31 before this data. The cost values of the tables are fixed in every answer.
- A. Keep Won't do 31. Fix the cost values only.
- B. Reopen it. Build the skip with the journal tree.
- C. Fix the values, and first measure how many of the 87 gates had no browser input in their diff (cheap, from the journal and `git`). Decide after.

Recommended: C.

**Batch confirm:** R11 fix the small part (the rest: journal to R1, score to LTM fog); R17 fix the cost values; R18 drop; R19 raise the cap to 20,000; R20 fix; R21 fix 2 of 3 parts; R23 drop.

## Grilling 05: product fault routes

Owners: the maps layout-trials and prompt-flexibility have no open ticket. The only active map with product-code tickets is long-term-memory (open: 05, 06, 07, 12; no build ticket yet). Dashboard-design has 11 open tickets and no build ticket yet. The bucket says product defects are out of its scope (its filed 103 is an exception).

| Row | Evidence | Owner (map, ticket) | Fix size and files | Class | Proposal | Reason | Who decides | Decision |
|---|---|---|---|---|---|---|---|---|
| **R4** byte-500 panic | unchanged; re-read, holds. `response.rs:140`, `orchestration.rs:110` | none. Searched all maps | one session. `src/lib.rs` (deny list), `response.rs:140`, `orchestration.rs:110`, `parser.rs:95,120`, `action.rs:27`, `src/test_support/fixtures.rs:648`, a test with a multi-byte body | check + one-off | fix here | `rg` finds 5 string slices in production code, not 4, and 1 in `test_support`. 2 can panic. The lint flags all | confirm in a batch | |
| **R5** 6 hand-kept route tables | unchanged | none. Dashboard-design will edit routes and the stub, with no ticket for this | larger than one session. `router.rs`, `answer_words.rs`, 5 test files | structure | route: new ticket in dashboard-design, before its first build ticket | The next dashboard build tickets pay 4 to 6 hand edits for each new route | needs the owner | |
| **R7** event Narration sends the Narration twice | unchanged | long-term-memory (open 05, 06, 12 change the same assembler). No ticket | one session, with R16. `narrator.rs:185-194`, `assembler.rs:292-300`, `generation.rs`, `narrator_tests.rs:1989` | structure | route: new task ticket A in long-term-memory | Same files as the memory block. A later fix would conflict | confirm in a batch | |
| **R8** "Internal Server Error" for a Game create, rename, delete | unchanged | none | two sessions, as two tickets. A: `utils/error.rs:57,112`, `http/error.rs:13`, `games.rs:123,159,174`, `error_tests.rs:37`, spec 45.22 rows. B: `utils/fragment.rs:11` (14 callers), `screen.rs:66,70,82`, `response.rs:20,29`, `stub_server.rs:1508` | one-off + rule | fix here | Each piece is one session. No map owns the failure display | confirm in a batch | |
| **R9** the stub is a partial second engine | unchanged | none. Dashboard-design pays the churn | one session (option 1 of the finding). `stub_request_layers.rs`, `stub_server.rs`, `utils/request_layers.rs`, `screen.rs`, `guardrails/screen.rs` | structure | route: new ticket in dashboard-design, after R5 | About 20 % extra churn on each dashboard change (12 commits, 612 lines for +24) | needs the owner | |
| **R10** model workarounds, nothing measures them | unchanged | long-term-memory fog "The reading pack". Owner decision "No second model" (Out of scope) limits it | two small measures: store the parser confidence in the LLM message; add `estimate_tokens` against `usage.prompt_tokens` to the trial pack. The seam and the second model are not proposed | structure + check | route (evidence for the fog patch); drop the rest for now | The seam and the second model need a second model, which the owner ruled out | needs the owner | |
| **R12** settings with two owners | unchanged | long-term-memory. The writers add callers of the same call path. No ticket | one session. `job.rs:476,479-483`, `wiring.rs:50`, `llm_recorder.rs`, `request.rs:76-94`, `generation_parameters.rs:11`, `narrator.rs:143`, `llm_backend.rs:92,97` | structure | route: new task ticket B in long-term-memory, before the first writer | The writers are new Agents on the same recorder. Fix the split before they add more | confirm in a batch | |
| **R13** two copies of the agent prompt | unchanged | long-term-memory. A writer is a new Agent with a prompt | one session. `options/prompt.rs:71-94`, `quantifier/prompt.rs:84-107` | structure | route: new task ticket C in long-term-memory, before the first writer | Each new writer would copy the block again. One renderer, with no change of output. The history format change needs a trial: leave it out | confirm in a batch | |
| **R16** the player-text filter misses the typed Input | unchanged | long-term-memory, with R7. No ticket | with R7 (ticket A). `assembler.rs:470`, `narrator_tests.rs:133-139`, `prompt_system.md:89`, the 8 `narrator-input-*` CASEs (`narrator_tests.rs:1560-1675`; the coordinator recounted them on 2026-11-04) | one-off | route: in ticket A | The ticket carries the choice: delete the filter and its CASEs, or filter the history entries. I propose delete (nothing renders player text as a template, inferred) | confirm in a batch | |
| **R24** leftovers | unchanged | none | one session, last. `stub_server.rs` (7 functions), `work/issues/pending/23-story-log-earlier-messages-investigation-plan.md:13`, `tests/browser/stub/*` (`submit_command`), `token_budget.rs:7,36`, `assembler.rs:453` | one-off | fix here | The assembler and `TokenBudget` items ride in ticket A. The rest waits for R5 and R9, which rewrite the stub | confirm in a batch | |

### Questions for grilling 05 (round 1: 2 questions, and one batch confirm)

**Q1. R5 and R9: where does the route table and the thin stub go?** Each HTTP route is listed by hand in 6 tables. A new `POST` route costs 4 to 6 hand edits. The stub server copies the screen envelope and the two request layers: 12 commits changed 612 lines of it for a net growth of 24. The next dashboard-design build tickets pay this. The dashboard-design branch does not exist yet (its ticket 01).
- A. Fix here, on `main`, as two task tickets before dashboard-design ticket 01 makes its branch.
- B. Two new tickets in dashboard-design, before its first build ticket.
- C. Wait. Do it when a third dashboard ticket shows the same churn.

Recommended: B.

**Q2. R10: what happens to the model workarounds?** The single-user-message merge is off for every shipped Connection. The Gemma 4 fix runs on every answer of every model. The parsers accept several shapes and never record their hit rate. No trial can test them without a second model, and "No second model" stands.
- A. Do nothing.
- B. Add two small measures to the trial pack: the parser confidence, and `estimate_tokens` against `usage.prompt_tokens`. Record them as evidence for the long-term-memory "reading pack". Build no seam and add no second model.
- C. Also move all workarounds behind one per-Connection seam now.

Recommended: B.

**Batch confirm:** R4 fix here (one session); R7 and R16 as ticket A in long-term-memory; R12 as ticket B; R13 as ticket C; R8 fix here as two tickets; R24 fix here, last.

## Patterns that the report did not promote

Source: [owner friction](findings/sessions-owner.md), [delegate reports](findings/sessions-delegates.md) and the two code audits. A pattern that the report already promoted to a row is not listed. The rule of the bucket audit applies: do not propose a vigilance rule without evidence that one held.

| Pattern | Proposal | Reason |
|---|---|---|
| Owner 4: the owner fact-checks a claim | leave; add evidence to bucket 88 | The fix is a vigilance rule ("check X"), and the audit says such a rule seldom holds. Bucket 88 already asks to check each fact before a question |
| Owner 5: the report uses a term the owner must ask about | leave; add evidence to bucket 88 | A vigilance rule. Bucket 88 asks the same of a question: open with the effect the owner sees |
| Owner 6: the owner must ask for the verdict | promote into R15 | The `Recommended:` line is in the same skill edit |
| Owner 7: the owner points the agent at ST and ME | promote | `rg` finds no `SillyTavern` or `Marinara` entry in `CONTEXT.md`. A glossary entry with URLs and pinned commits is information, not a rule. One session or less. Add it to the 03 batch |
| Owner 8: the owner cannot judge a visual artifact from text | leave | The Inbox is the fix and is partly in place |
| Owner 9: an example becomes a hard constraint | leave | A brief convention. No evidence that a rule holds |
| Owner 10: the owner adds a step the ticket template should carry | already in R14 | The `docs/agents/build-ticket.md` of R14 carries the steps |
| Owner 11: the owner corrects the model of who can run what | leave; add evidence to bucket 99 | 99 is the open ticket |
| Owner 12: a one-word answer costs a round trip | leave | The cap of 3 questions (R15) cuts the volume. A "take the safe default" rule is a vigilance rule |
| Owner: `understanding-ladder` trigger fails | leave | The skill is in `~/.agents/skills/`, outside the repo |
| Owner: `document-review` against `writing-for-agents` boundary | leave | One instance. Two turns lost |
| Delegates 2: a new test repeats an old test | leave; add evidence to bucket 98 | 98 is open |
| Delegates 3: a change that the plan does not describe | leave; add evidence to bucket 86 | 86 is open |
| Delegates 5: doc or spec text left false | leave; add evidence to bucket 95 | 95 (doc step breaks the doc rules) is the nearest open ticket. Won't do 82 measured a lint at 0 of 10 fix rounds |
| Delegates 6: generated indexes not regenerated before review | promote | Fully mechanical, 9 reports. `prepare_review_bundle.py` can run `scripts/precommit_regenerate.py` first. File as a bucket ticket, class mechanical. Near: bucket 94 (the gate regenerates docs) and 100 (the review bundle) |
| Delegates 7: code with no caller after a refactor | promote | 5 reports: it recurred. `clippy` is blind to a `pub fn` with no caller. A `healthcheck.py` subcommand that runs `rg -w` over the deleted symbols of a diff. File as a bucket ticket, class mechanical |
| Delegates 8: `read` errors on image outputs | leave | A model slip, one turn each |
| Delegates 9: a worktree implementer has no API key | leave | Bucket 97 landed (`1a0c9162`) |
| Delegates 10: shared-checkout collisions | leave | Won't do group D2 (shared-checkout collisions) |
| Delegates 11, 12: briefs, cost by role | leave | `AGENTS.md` already tells the lead to use a `scout` for an investigation |
| Dashboard #5: guardrails pin source text | leave | Two of its checks go with R5. A rule "prefer a type or a DOM sweep" is a vigilance rule |
| Dashboard #8: old clone pairs in `tests/http` | leave | Not from this week. A separate drain task |
| Prompt #8: two session rules for one purpose | promote into R12 | `llm_backend.rs:92,97`. The ticket B text carries it. The gain on OpenRouter is guessed |
| Prompt #9, #10, #11 | already in R12, R24, R21 | |
| Prompt, bitter lesson 4: add a second model to the harness | leave | Out of scope (owner, 2026-11-04) |
| Prompt, bitter lesson 5: some prompt text is in Rust | leave | A trial that needs it can ask. No fault behind it |

## Notes for the coordinator

- **New tickets proposed.** Three task tickets in long-term-memory (A: R7 and R16, B: R12, C: R13). Two task tickets in dashboard-design (R5, R9), if Q1 of 05 says so. Two task tickets here for R8, one for R4, one for R24, and one for each batch fix of 03 and 04. Two bucket tickets (delegates 6 and 7). The CONTEXT.md entry (owner 7).
- **Fog patches.** One in long-term-memory: the "reading pack" gets the R10 measures and the R11 quality score. One research ticket, AFK, for the `scout` role: what the 44 browser tests spend time on, from the per-check log.
- **A concurrency risk.** The long-term-memory map has uncommitted edits of another session. Add a ticket file for each new ticket. Make one small edit in its `map.md`.
