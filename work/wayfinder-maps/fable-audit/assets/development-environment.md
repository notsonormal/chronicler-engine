# The development environment: rules, skills, guardrails and the flow of work

Audit of 2026-11-10 at commit `b0c8fe41`. This report covers group D of the catalogue (`audit-catalogue.md`): D1 "rules, skills and guardrails", a part of D2 "independence of the tests" (the mutation run of section 5) and D3 "the flow of work".

## How to read this report

- **This report builds on your own audit.** `work/wayfinder-maps/retro-issue-bucket/audit.md` sorts a fix to the environment into five classes and says which class works: a mechanical check works, wrong prose must be corrected, and a vigilance rule seldom holds. I use those classes. I propose no new prose rule.
- **Labels.** "Read it" means I read the cited lines. "Measured" means I counted from a file that the tools write (the permission log, the build journal, `git log`, the ticket files). "Scout" means a cheap model read it and I did not check each row.
- **I filed nothing.** The bucket has 25 open tickets. Section 6 says which rows of this report I would add to it, if you want any.

## 1. Summary

- **The mechanical part of the environment is sound.** Section 2. The guardrails prove that they can fail, the exception lists shrink, and the one secret path is protected.
- **Nine sentences or files are wrong today.** Section 3. Each is "wrong prose" in the terms of your audit: the text causes the mistake, and an edit removes the cause. Each fix is small.
- **Most rules are prose.** 204 rules, 157 of them (77 %) with no check behind them. Your audit already explains why this class decays. Section 4 has the numbers and the ten rules that a one-line check can hold.
- **The work waits for you, not for the agents.** 111 open tickets, 79 of them opened in November, and 55 of them are grilling tickets that need your answer. Section 4.
- **The unit tests pin the story rules well.** A mutation run on twelve files caught 168 of 190 small changes (88 %). Eight of the 22 that it missed show code that can be deleted. Section 5.
- **The numbers in `AGENTS.md` for the cost of a build are too low** by a factor of two to three against the build journal. Section 3, row 3.

## 2. What I checked and found sound

- **A guardrail cannot pass by being empty.** `registry_tests.rs` fails when a public check has no detection test (`every_public_check_rule_has_a_detection_test`), when a walker row has no detection case, and when a rule is not reachable from `mod.rs`. A scope that matches no file fails with the name of the guardrail (`a_dead_scope_names_the_guardrail_and_the_scope`). This is the failure that the study in your wiki describes (9 of 38 checks could fail in the place they were written for). It does not apply here.
- **The exception lists shrink.** The legacy unit-test list held 106 files on 2026-10-17 and holds 79 today. The exception tables that I read (`HTML_ANSWER_ALLOWED`, `DRAFT_CONTROL_EXCEPTIONS`) carry a reason for each row, and for the draft controls and the part writes a guardrail fails when an exception is no longer used. I searched the guardrail commits since October for a raised limit or a new exemption constant. I found these tables and no raised limit.
- **Few lint escapes.** The source has 24 `#[allow]` attributes outside tests. Most are `clippy::expect_used` on a fixed regular expression.
- **The `.env` rule holds for bash too.** The permission log shows `cat .env`, `sed … .env` and `grep -c … .env` blocked with `policy_denied`. The path rule applies to commands, not only to the read tool.
- **The build slot works as documented.** One compile step at a time across all checkouts, with a fail-open after the wait limit.
- **The mutation research exists.** `work/wayfinder-maps/unclebob-quality-tools/assets/03-mutation-engines.md` has the method and the rules. My run of this audit follows them (a copy of the tree, the build slot, no stash).

## 3. Text that is wrong today

| # | What is wrong | Proof | Fix |
|---|---|---|---|
| 1 | Three role files declare the same name. `judge-deepseek.md`, `judge-glm.md` and `judge-opus.md` all have `name: judge-glm`. Two judge roles cannot be called. The role list of this session offers `judge-glm` on the Opus model, so the last file wins | Read it (`.pi/agents/*.md`, line 2) | Give each file its own name |
| 2 | `AGENTS.md` says that the permission restrictions "exist to prevent the agent from touching git without supervision". No permission rule names git. The deny block for `git reset`, `rebase`, `checkout`, `restore`, `clean -f` and others was removed in `5142f1da` (2026-09-29) and `19712ddd` (2026-10-03) | Read it (`git log -p` of the config) | Decide: restore the denials for the destructive commands, or correct the sentence. Today the rule "do not commit without approval" is prose only |
| 3 | The warm costs in the `AGENTS.md` table are too low. Median of the green runs in the journal: full gate 350 s (table: 145 s), `--no-browser` 111 s (50 s), `integration` 59 s (21 s), `guardrails` 27 s (10 s), `clippy` 18 s (12 s), `browser` 234 s (92 s) | Measured (`logs/build_history.txt`, 999 runs, 2026-10-28 to 2026-11-09) | Generate the column from the journal, or delete it. An agent that plans with 145 s sets a time limit that the gate does not meet |
| 4 | `CODING_STANDARDS.md` requires `//! [DOC: …]` on line 1 of every source file. `src/lib.rs:1` has `// [DOC: …]`, and `src/main.rs` has none. The guardrail exempts both (`MODULE_DOC_EXEMPTIONS`) | Read it | Name the two exemptions in the rule |
| 5 | The generated `guardrails.md` says "Warns" for three rules (long comment runs, separator comments, single-letter variables). `assert_violations` in `mod.rs` panics on any violation, so each of the three fails the build | Read it | Change the doc comment of the three checks. The generator copies it |
| 6 | The `implement-spec` skill tells the lead to merge each implementer's work "with a **merger subagent**" and to commit on an integration branch. No `merger` role exists, and `AGENTS.md` forbids a commit without approval | Read it (`SKILL.md:23`, `:31`) | Correct the skill to what the delegate tool has, or mark it as not for this repository |
| 7 | The `thermo-nuclear-code-quality-review` skill says "if there is a clear path to improving the implementation that involves restructuring some of the codebase, go for it". `AGENTS.md` says that a review must not change code. The `reviewer` role has no write tool, so the tool grant stops it, not the rule | Read it (`SKILL.md:24`) | One sentence in the skill: the output is findings |
| 8 | `.pi/plan-mode.json` lists 17 default tools. Several are not registered in this harness (`grep`, `find`, `ls`, `intercom`, `subagent`, `rag_index`, `rag_query`, `rag_status`, `visual_explainer`, `graphify`, `describe_image`) | Read it. I did not test what plan mode does with an unknown tool | Check plan mode once, then correct the list |
| 9 | Two skills copy text that another file owns. `wait-what` restates the Simplified Technical English block that `.pi/APPEND_SYSTEM.md` appends to every turn. `chronicler-after-plan-workflow-plus-review` declares itself a hand-kept copy of `chronicler-after-plan-workflow` | Scout, two rows checked | Replace each copy with a pointer |

One scout row was wrong, and I removed it: "the guardrails enforce limits that no document states". The limits (2,000 non-blank lines for each file, nesting depth 3, five comment lines) are in the generated `docs/diataxis/reference/coding_standards/guardrails.md`. What holds: neither `AGENTS.md` nor `CODING_STANDARDS.md` sends an agent to that file, so an agent learns a limit from the failed gate. In the terms of your audit that is a mechanical check with a loud error, and it needs no rule.

## 4. Measurements

### The rules

A scout listed every rule in the hand-written agent documents. I checked the classes of about twenty rows.

| Source | Rules |
|---|---|
| `tests/STRATEGY.md` | 57 |
| `AGENTS.md` | 41 |
| `docs/AGENTS.md` | 37 |
| `CODING_STANDARDS.md` | 20 |
| `docs/agents/*` | 18 |
| `tests/AGENTS.md` | 11 |
| `.pi/APPEND_SYSTEM.md` | 10 |
| `ENVIRONMENT.md`, `.pi/agents/*` | 10 |
| **Total** | **204** |

| What holds the rule | Rules | Share |
|---|---|---|
| Prose only | 157 | 77 % |
| A check (guardrail, script, lint) | 35 | 17 % |
| A skill | 6 | 3 % |
| The code itself | 4 | 2 % |
| A permission | 2 | 1 % |

- In the other direction, about 60 checks have no written rule. The largest groups are the 13 screen guardrails, the four answer-words guardrails and the `arch-lint.toml` layers. This is the healthy direction: the check is the rule.
- The agent-facing root documents hold about 15,400 words. 6,264 of the 13,833 words in the rule sources are generated indexes.
- **Ten prose rules that a small check can hold** (list in `leads/d1-rules-inventory.md`, part 5): no `erDiagram` in a Mermaid fence, no task reference in a code comment, no `networkidle` under `tests/`, no `deprecated` or `_legacy` shim, no duplicate `SCENARIO` tag in one test binary, and five rules for doc wording that Vale can hold. The ranking is a guess: no file records which rule agents break most.

### The permission log

Measured from the review log of the permission extension, 2026-09-27 to 2026-11-10 (5,489 lines).

| Blocked call | Count |
|---|---|
| A `grep` at the repository root | 178 |
| The `timeout` command | 97 |
| `find /` | 42 |
| A `.env` path | 15 |
| Other | 9 |
| **Total** | **341** |

- That is about eight blocked calls for each day. Each costs one turn, and the deny reason corrects it. In your classes this is a model slip with a loud error. It needs no rule.
- Against this, 1,916 calls were approved without a prompt, and you answered 12 prompts in six weeks. The permission system does not slow the work.

### The build journal

Measured from `logs/build_history.txt`: 999 runs in 13 days, 13.7 hours of build time.

| Command | Runs | Failed | Median green |
|---|---|---|---|
| `test-pattern` | 367 | 41 % | 12 s |
| `clippy` | 159 | 38 % | 18 s |
| `validate-docs` | 101 | 3 % | under 1 s |
| `fmt` | 88 | 0 % | 2 s |
| Full gate | 64 | 23 % | 350 s |
| `vale` | 57 | 24 % | under 1 s |
| `--no-browser` | 32 | 18 % | 111 s |
| `integration` | 30 | 26 % | 59 s |
| `guardrails` | 24 | 29 % | 27 s |
| `browser` | 16 | 31 % | 234 s |

- A failed `test-pattern` run is mostly a test that fails first on purpose. A failed full gate is not: 15 of 64 full gates failed, at about six minutes each.
- The journal has no column for the step that failed. So it cannot answer the useful question: which step fails a full gate most often, and could a cheaper command have found it first? One more column would answer it.
- The journal keeps the newest 1,000 rows. That is 13 days at this rate.

### The tickets

Measured from the `Status:` and `Type:` lines under `.scratch/` on 2026-11-10.

- 111 open tickets in 18 efforts: 88 open, 16 needs-triage, 6 claimed, 1 needs-info.
- 79 of the 111 were opened in November. The median age is 7 days. 21 are older than 14 days, and the oldest is 80 days.
- By type: 55 grilling tickets (35 of them marked for you), 13 bugs, 11 prototypes, 11 tasks, 6 research.
- The six claimed tickets include one claimed on 2026-10-09 (`pi-explain/issues/02`) and one on 2026-10-22 (`conceptual-integrity/issues/05`). A claim has no end date, so a dead session keeps its ticket.

**Reading (inferred).** The agents make tickets faster than the tickets close, and the largest group waits for a decision of yours. More audit findings in ticket form would add to the same queue. This audit therefore keeps its questions in the reports, with a recommendation on each.

### The commits

Measured from `git log` since 2026-08-01: about 40 % of the commits touch product code (`src/` without tests, `assets/`, `data/`). About 40 % touch only the environment (`.agents/`, `.pi/`, `scripts/`, `.scratch/`, `build.py`, root documents). The last full week had 15 commits, against 86 in the week before.

### The skills

A scout listed the 41 skill folders. I checked the rows that this report uses.

- 26 of the 41 skills can only be started by name (`disable-model-invocation: true`). For the other 15, the description is the only routing signal.
- Groups that overlap: three workflows for the time after an implementation, three grilling skills, seven review skills, four verification skills. Only one group is unclear for a model, because all three members can start on their own and claim the same task: `document-review`, `diataxis-doc-review` and `chronicler-docs-hygiene`.
- The scout reported `TaskCreate` and `session_list` as tools that do not exist. Both exist in this harness. A scout's report of a stale reference needs a check against the live harness.

## 5. How strong the tests are: a mutation run

**Measured.** I ran `cargo-mutants` 27.1.0 on the twelve files that hold the story rules, the parsers and the token budget. A mutant is one small change to the code, such as `<` to `<=` or a function body replaced by a fixed value. A mutant is "caught" when a test fails. The run took the build slot for 125 minutes in four parts, on a copy of the tree.

**Limit of the run.** It used the unit tests of the library only (1,066 tests), not the integration tests under `tests/`. A "missed" mutant can still be caught there. For each row below I searched `tests/` for the function name and found no test of it. That search is by name, so it is not a proof.

| File | Caught | Missed | Does not compile | Caught, of those that compile |
|---|---|---|---|---|
| `generation.rs` (the plan rules) | 28 | 0 | 11 | 100 % |
| `tail.rs` | 13 | 0 | 4 | 100 % |
| `action.rs` | 9 | 0 | 1 | 100 % |
| `story_window.rs` | 7 | 0 | 0 | 100 % |
| Options `parser.rs` | 5 | 0 | 1 | 100 % |
| `sanitize.rs` | 2 | 0 | 0 | 100 % |
| `history_cut.rs` | 33 | 1 | 2 | 97 % |
| `trigger.rs` | 17 | 1 | 0 | 94 % |
| Quantifier `parser.rs` | 12 | 4 | 3 | 75 % |
| `token_budget.rs` | 21 | 7 | 2 | 75 % |
| `game_rules.rs` | 14 | 5 | 4 | 74 % |
| Quantifier `orchestration.rs` | 7 | 4 | 0 | 64 % |
| **Total** | **168** | **22** | **28** | **88 %** |

**Reading.** The plan rules and the story shape are well pinned. The 22 missed mutants fall into four groups. Three of the groups point at code, not at tests.

| Group | Missed | What it shows |
|---|---|---|
| Code that does nothing | 8 | The code-fence step of the Quantifier parser can return nothing, an empty string or `"xyzzy"`, and every test passes (4 mutants). The step after it takes the text from the first `{` to the last `}`, and that covers a fenced answer. The retry condition after a failed Quantifier call can be any comparison (3 mutants), because the loop ends at the same place. `set_currently_meeting` on a move can be inverted (1 mutant), because `currently_meeting` has no reader. Each can be deleted |
| Checks that no test makes fail | 4 | `assert_state_consistency` and its three parts in `game_rules.rs` can each return `Ok(())`. No test gives them a state that is wrong. They do run in the shipped build: `default = ["testing"]` switches `diagnostics` on |
| The answer length and the history reserve | 7 | `history_reserve_tokens` can return 0 or 1, and its division can be any operator (4 mutants). Three boundaries of `TokenBudget::response_tokens` can move. These numbers decide how long an answer may be and when the history is cut. `sanitize.rs` scores 100 % only because it has 2 mutants: no test calls it by name |
| Boundaries | 3 | `Lt` in `Encounters::check_condition` at the threshold, one `>` in `HistoryCut::of`, one `<` in the Quantifier retry on a Low reading |

- **The first group is the useful one.** A missed mutant that replaces a whole function is evidence that the function is not needed. This agrees with two points of the other reports: `currently_meeting` has no reader, and the Quantifier parser has more steps than its inputs need.
- **The run answers D2 in part.** Your wiki states the risk that tests which the same agent wrote pass by design. On these twelve files the unit tests catch 88 % of small changes, and on the plan rules they catch all of them. I have no reference figure to judge 88 % against, and no figure for the other files of this repository. The owner's earlier run found 10 survivors on `parser.rs` and `gate.rs` (`work/wayfinder-maps/unclebob-quality-tools/map.md:28`).
- **Cost.** About 35 seconds for each mutant on this machine with the build slot. The whole of `src/` would take days. A run on the files of one change is affordable. `work/wayfinder-maps/unclebob-quality-tools/` already has this plan.
- **To repeat it:** `work/wayfinder-maps/fable-audit/assets/local-only/evidence/mutants/run_shards.sh`. The lists are in `local-only/evidence/mutants/<part>/mutants.out/`.

## 6. Rows that could go to the retro bucket

Only if you want them there. Each is "wrong prose" or "mechanical".

| Row | Class | Size |
|---|---|---|
| The three judge files with one name (section 3, row 1) | Wrong prose | Two lines |
| The git sentence and the removed denials (row 2) | Your decision first | One paragraph or one config block |
| The warm-cost column (row 3) | Mechanical: generate it | A few lines in `build.py` |
| A "failed step" column in the build journal | Mechanical | A few lines in `build.py` |
| An end date or a session id on a claim | Mechanical | A check in the ticket script |
| The two skills that contradict `AGENTS.md` (rows 6 and 7) | Wrong prose | A few lines each |

## 7. What I did not check

- **Whether a rule changes what an agent does.** No record links a rule to the sessions that followed it. Your wiki has the method (load a rule, and measure with and without it). It needs many runs and model credit.
- **The skills in use.** I read the skill files. I did not run a skill to see whether it gives the result it promises.
- **The session records.** About 1,840 sessions are indexed. A count of rework (a delegate result that the lead had to redo) is possible from them. `work/wayfinder-maps/weekly-retro-2026-11-04/` already does this by hand for one week.
- **The Python build scripts as code.** `build.py` and `scripts/` hold 10,253 lines. I read parts of them for facts. I did not review them.
