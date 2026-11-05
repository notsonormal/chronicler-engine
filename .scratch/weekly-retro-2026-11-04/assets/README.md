# Weekly retro 2026-10-28 .. 2026-11-04

The week shipped much good work. Per-ticket review caught local defects well. It did not catch defects that span tickets, and those defects reach `main`. The highest-cost problems are in three places. Trial evidence is in a folder that the gate deletes. No step reviews an effort as a whole before it merges. Review is the only control for some defect classes that a machine can find. The bitter-lesson lens points the same way for the environment and for the product: put effort into measurement and checks that scale with compute, and put less into hand-written rules and hand-built workarounds.

## The week in numbers

| Measure | Value |
|---|---|
| Commits since `3525193c` | 79 (21 `feat`, 11 `fix`, 12 `docs`, 5 `refactor`) |
| Code churn (`src`, `tests`, `scripts`, `build.py`, `assets`) | +19,289 / −8,261 lines in 281 files |
| `.scratch` churn | +25,708 lines in 366 files (maps, tickets, assets, prototypes) |
| pi sessions | 255: 67 lead, 188 delegate |
| Model cost | $503 (lead $264, reviewer $113, strong-implementer $83, researcher $23, implementer $19, scout $0.79) |
| Tool calls | 14,351 (bash 11,014, read 1,923, edit 482) |
| Build runs (journal) | 999 runs, 13.4 h; full gate 49 runs, 4.45 h; `test-pattern` 378 runs, 44 % red; `clippy` 161 runs, 40 % red |
| Most expensive efforts | dashboard action-answers $104, prompt-flexibility $47, character art $22, long-term memory $22, layout-trials $20 |
| Owner follow-up turns in lead sessions | 399; 78 are 25 characters or fewer (`yes` ×17) |

The `test-pattern` and `clippy` red rates are mostly TDD red steps and first compiles. They are the loop working, not waste (`findings/tooling-env.md` §1).

## Candidates, by severity

Category is the retro category. Fix class: **structure** (make it unrepresentable), **check** (a deterministic test or lint), **rule** (a judgement call for review), **one-off**. "Bucket" is `.scratch/retro-issue-bucket/map.md`. "Verified" is the outside check (`findings/outside-check.md`).

### High

| # | Candidate | Category | Fix class | Evidence | Tracker |
|---|---|---|---|---|---|
| R1 | **Trial evidence is in a folder that the gate deletes.** Every full gate deletes files older than 30 days under `tmp/`, and also under `/workspace/tmp`, which is outside the repo. The reference trial of the long-term-memory outside check (`tmp/ltm09/redmist_julian/trial-1`) and the judged stories of its baseline (`tmp/lt15/`, `tmp/ltm03/`) are only there. They go at about 2026-12-03. | Information access | structure | `build.py:1383-1386` (`_gate_tail`, `clean_tmp_dirs`); `.scratch/long-term-memory-and-state/map.md:74`; `assets/baseline-faults.md:5-7` | new |
| R2 | **Defects that span tickets reach `main`.** Each ticket gets a Standards, Spec, doc and comment review of its own diff. Only a review of the whole branch finds the cross-ticket defects. On prompt-flexibility, that review needed an 8-phase fix plan. Layout-trials merged on 2026-11-03 with per-ticket reviews only. This audit found defects in its code that per-ticket review missed (R16, and two session rules for one purpose). The close-out has no step for this review. The owner typed the same close-out 4 times. On 2026-10-30 the owner corrected a fast-forward ("do you not know what merge is?"). The next 3 merges (2026-10-31, 11-01, 11-03) were fast-forwards again. The owner did not object to those 3, so I don't know if a merge commit still matters. | Skill health | structure (a close-out script or skill) | `old-docs/archived-plans/prompt-flexibility-review-fixes.md` (Origin); `findings/code-prompt-llm.md` §1, §5, §8; `findings/sessions-owner.md` §3; `git log --merges` | new |
| R3 | **Tests that cannot fail come back in every effort.** 9 instances in 8 efforts: an assert that the setup makes certain, an `is_ok()`-only check, a guard that no test drives. Review found each one, and review is the only control. A mutation run on the changed lines finds all of them. | Automated checks | check | `findings/sessions-delegates.md` §1 | adds evidence to 23, 87; `unclebob-quality-tools` 03 |
| R4 | **A provider error can panic the Job.** `&error_body[..500]` panics when byte 500 is inside a multi-byte character. Since `bf3b87a5`, every provider's non-2xx answer goes through this line. The same fault is at `quantifier/utils/orchestration.rs:110`. `clippy::string_slice` is off. | Automated checks (product) | check + one-off | `src/adapters/driven/llm/transport/utils/response.rs:140` | product |
| R5 | **Each HTTP route is listed by hand in 6 tables.** The router, `ANSWER_WORDS`, `PINNED_WORDS`, `idle_rows`, `GAME_BOUND_ROUTES` and the stub router. Checks that parse source text keep them in step. One of them is a tier-1 test that reads `router.rs` with `include_str!`. A new route costs 4 to 6 hand edits. | Coding standards | structure (one route table in `src/`) | `findings/code-dashboard-http.md` §1 | product |
| R6 | **`target/` (28 GB) is inside the checkout, so the permission globs deny safe commands.** `grep * . *` denied 42 calls, and a parse finds 0 that walk the repo root (`grep -c . file`, `cd src; grep -rn … .`). A glob `*` matches across `;`, so one refusal can drop a whole command with a build in it. Move the target dirs out of the checkout through `_target_paths`, then the grep rules can go. Only the owner can edit the config. | Tool economy | structure | `.pi/extensions/pi-permission-system/config.json`; `findings/tooling-env.md` §2; `findings/sessions-delegates.md` §4 | new |

### Medium

| # | Candidate | Category | Fix class | Evidence | Tracker |
|---|---|---|---|---|---|
| R7 | **An event Narration sends the Narration before it twice**: once as the last history entry, once as "Previous narration" in the user text, beside "Do NOT repeat". The assembler guesses "the Input is already in the history" by a string compare, although the Job knows it. | Product | structure | `narrator.rs:185-194`; `assembler.rs:292-300`; `findings/code-prompt-llm.md` §1 | product |
| R8 | **The tuner sees "Internal Server Error" in place of the engine error** when a Game create, rename or delete fails (`AnswerPolicy::GenericText`). Two older failure paths stay beside `ErrorAnswer`: `render_or_error_fragment` answers `200` with an error inside (14 callers), and `/screen` uses `bad_request` and `internal_error`. | Product | one-off | `utils/error.rs:57,112`; `http/error.rs:13`; `games.rs:123,159,174`; `findings/code-dashboard-http.md` §3, §4 | product |
| R9 | **The stub server is a partial second engine.** It copies the screen envelope, the two request layers and the order of refusals. 12 commits changed 612 of its lines for a net growth of 24, about 20 % extra churn on each dashboard change. Two checks exist only to keep the copy in step. | Coding standards | structure (make the layers generic, so the stub calls engine code) | `findings/code-dashboard-http.md` §2 | product |
| R10 | **Model workarounds are spread out, and nothing measures them.** The single-user-message merge is off for every shipped Connection. The Gemma 4 fix is in 3 places and runs on every answer of every model. The parsers accept several shapes, and their confidence goes only to the logs. The token constants (`CHARS_PER_TOKEN` 4, margins) are hand-tuned, but `trial.py` already reads `usage.prompt_tokens` and compares nothing. | Bitter lesson (product) | structure + check | `findings/code-prompt-llm.md` Bitter lesson 1-3 | product |
| R11 | **`trial.py` is the lasting evaluation harness, but it has the shape of a script.** Its setup names a Connection by an id that only one probe DB has (the id is not in `data/settings.json`, so a fresh DB does not have it). It writes the settings row with SQL, and its test replays the Rust migrations by regex. It mirrors 3 Rust constants with no check. The slot rule is in Rust, `trial.py` and `validate_data.py`. It has no score for prompt quality. | Bitter lesson (environment) | structure | `scripts/trial.py:459`; `scripts/trial_setups/redmist_julian.json:14`; `findings/tooling-env.md` §4; `findings/code-prompt-llm.md` §7 | new |
| R12 | **Some settings have two owners.** The Job and the recorder each resolve the Connection of one call (this split already caused the bug fixed in `856e36a5`). A preset's `custom` object can set `reasoning_effort`, routing and the session, which the Connection also sets. `max_tokens` travels twice in `CallPlan`. | Product | structure | `job.rs:476`; `wiring.rs:50`; `request.rs:76-94`; `findings/code-prompt-llm.md` §2, §3, §9 | product |
| R13 | **Two copies of the agent prompt.** `options/prompt.rs` and `quantifier/prompt.rs` share 67 lines, and ticket 19 made the same edit in both. The agents still send the history in an older format than the Narrator. | Product | structure | `findings/code-prompt-llm.md` §4 | product |
| R14 | **Standing rules are copied from map to map, and the copies go stale.** The live long-term-memory map still says build tickets use `/tdd`, which was deleted on 2026-11-01. The review-fixes plan removed `/tdd` from two other maps only. The owner also adds missing template steps by hand ("there's no document review step in ticket twenty-four"). Put the standing rules of a build ticket in one file that maps point to. | Navigation | structure | `.scratch/long-term-memory-and-state/map.md:25,58`; `findings/sessions-owner.md` §10 | adds evidence to 99 |
| R15 | **Grilling rounds are too large to answer well.** 45 numbered-answer turns in 16 sessions. The owner: "you just asked me a ton of questions, so I'm not really confident in my answeres". The hex-comparison map then set "at most 3 questions in each round", but only in that map. | Skill health | rule (in `grilling/SKILL.md`) | `findings/sessions-owner.md` §1 | adds evidence to 88; new evidence against Won't do 10 |
| R16 | **The player-text filter no longer reaches the typed Input**, and the default Narrator fixture hides this. Since layout-trials ticket 15, the typed Input is the last history entry, and the filter runs only in the dropped layer. About 60 call sites use a fixture shape that production never sends. | Product | one-off | `assembler.rs:470`; `narrator_tests.rs:133-139`; `findings/code-prompt-llm.md` §5 | product |
| R17 | **The browser tier grew from 92 s to about 210 s in three weeks, and the cost tables do not show it.** Every "Warm cost" value in `AGENTS.md` is 1.5 to 3 times too low. A skip of the browser tier matches Won't do 31. The first step is to find the slow checks from the JUnit timings, and to generate the cost column from the journal. | Automated checks | check | `findings/tooling-env.md` §1, §6 | Won't do 31 (skip part only) |

### Low

| # | Candidate | Category | Evidence | Tracker |
|---|---|---|---|---|
| R18 | 321 of the 422 lines of `AGENTS.md` are the generated structure index. Agents ran `rg`/`grep` 5,451 times this week, and `rg -m2 '^//! ' src` gives the same facts. | AGENTS.md / bitter lesson | `AGENTS.md:7-328` | Won't do 09 (owner decision) |
| R19 | The build journal keeps 1000 lines (about one week) and the logs keep 3 days, so a weekly retro loses its own data. This retro misses the first hours of 2026-10-28. | Information access | `scripts/build_history.py:24` | new |
| R20 | `build.py` tail: the DB dump reads a path that nothing writes, and the copy of `data/` into the target dir makes a stale-data trap that two docs describe instead of remove. | Automated checks | `build.py:1389-1392`; `findings/tooling-env.md` §5 | new |
| R21 | Tests that copy a production table row for row (`PINNED_WORDS`, `EXPECTED_COMMANDS`), and one test group for each optional Connection field (15 tests over one mapping). | Coding standards | `findings/code-dashboard-http.md` §6; `findings/tooling-env.md` §8; `findings/code-prompt-llm.md` §11 | new |
| R22 | 8 skills had no use this week. Two families overlap: three grill skills and three after-plan skills, one of which says it is "intentionally a copy". | Skill health | `findings/tooling-env.md` §9 | new |
| R23 | Maps were read 426 times (2.5 M characters) and plans 296 times (2.8 M characters). One staged plan was read whole 41 times in 16 sessions. | Tool economy | `local-only/data/analyse_out.txt`; this file | new |
| R24 | Leftovers: stub functions in the shape of deleted handlers, a stale plan line, test-only `TokenBudget::fallback`, an unused `MessageType::System` branch, `submit_command` private to one file while ten files copy it. | One-off | `findings/code-dashboard-http.md` §7, §8; `findings/code-prompt-llm.md` §10 | product |

## Bitter lesson

Sutton's lesson: methods that scale with computation beat methods that build in human knowledge, and the built-in knowledge first helps, then stops helping, then gets in the way.

### Environment

1. **Steering is mostly prose, and the controls are mostly people.** 37 skills, a 422-line `AGENTS.md`, maps of 10 to 20 KB with standing rules, and 4 review delegates for each ticket. Review is 47 % of delegate spend, and it is the only control for 4 defect classes that a machine can find (R3, dead `pub fn`, stale generated indexes, copies of tests). Compute that scales (mutation runs, derived tables, lints) is the better place for that money.
2. **The trend is already right where it was tried.** The STE rules became Vale checks, and `document-review` dropped its prose copy (`38043843`). The build output became structured (`672ac677`). Each of these removed prose and added a check. R5, R17 and R21 are the same move.
3. **Hand-written permission globs guard a hazard that a path change removes** (R6).
4. **The structure index is navigation for a model that cannot search** (R18). The owner kept it on purpose (Won't do 09). The new evidence is the `rg` count.
5. **The hex-comparison lesson is not in the skill that needs it.** Its conclusion says: "Look for the mechanism of the whole system, and test it with one cheap measure, before the map splits the other project into single practices." 20 tickets ran before the one cheap measure that settled the main question. `wayfinder/SKILL.md` does not carry this lesson, and the owner's pattern 2 (`findings/sessions-owner.md`) shows the same cost.

### Product

1. **Long-term memory: the effort's own data says the faults are not memory faults.** Kind 1 of the three memory faults (a fact that left the history) has 0 confirmed instances in both judged stories. The asset says that "a memory of the dropped entries fixes none of the confirmed faults. Several of them come before the first cut" (`assets/baseline-faults.md`). The remaining faults (the story clock, who knows what) happen with the facts in the context, so they are faults of model reasoning. The research also says "while the history fits, the full history beats every memory system", and the model has a 1 M-token window. Hand-built writers, a `secrets` field and presence records work around a model weakness. The bitter lesson says a better model can make them dead weight. A cheap measure first: judge the same setup with one stronger narrator model, or with reasoning on. "No second model" is an owner decision of 2026-11-02 (`layout-trials/map.md:17`), so this is a question for the owner, not a finding. **Owner, 2026-11-04: dropped. "No second model" stands** (map, Out of scope).
2. **The workarounds have no exit test** (R10). Each workaround needs one seam and a count of how often it changes an answer. Then a trial can say when to delete it.
3. **The trial harness is the general method, and it needs the investment** (R11, R1): stable inputs, kept results, a quality score (a judge inside `compare`), and more than one model.
4. **Good direction:** layout changes were decided by measured trials, with the numbers in the commit (`1989d3fe`: 63.9 % to 86.0 % cached). The map says "An inconsistency inherent to the model is accepted, not engineered around." Point 1 asks whether the long-term-memory effort keeps to that rule.

## What went well

- One chat-completions backend replaced three providers (`bf3b87a5`), and one Connection slot table replaced the copies (`d6daa238`).
- The part-route removal was staged well: the tests moved to `/screen` first, and then 14 routes went with no caller left (`ddaea736`, `ff4703ca`, `d4e67887`).
- One client reader (`reportAnswer`) writes every request report, and the stub shares the real failure words.
- Agents use the narrow build loop: 367 of 378 `test-pattern` runs use `--test-target`, with a median of 13 s.
- Briefs to delegates are rarely wrong: 4 of 188 reports say so.
- `scout` is almost free ($0.79 for 13 runs) and gave usable findings. It is under-used for its price.

## Method and limits

- **Sessions.** `scripts/sessions.py` and `scripts/analyse.py` read all 255 session files from 2026-10-28 on. Two `scout` agents mined the owner turns and the delegate reports (`findings/sessions-*.md`). Error classes and the effort labels are heuristics.
- **Code.** Three `reviewer` agents audited the net change since `3525193c` by area (`findings/code-*.md`). They read code and git only. They ran no build.
- **Outside check.** A `judge-qwen` agent from a different model family checked the 13 load-bearing claims against the repo and tried to falsify each one (`findings/outside-check.md`). Result: 12 hold, 1 partly holds, 0 fail. The partial one (C12) was the wording "probe-DB row id". The id is a Connection id string, not a row id. The point holds: the id exists only in one probe DB (`rg` finds it in no file under `data/`). The wording of R11 is corrected. One verdict rests on absence: no record of a whole-branch review of layout-trials exists in `tmp/review/` or in its map. So "layout-trials had no effort-level review" is inferred, not known.
- **Limits.** The build journal holds 1000 lines, so the first hours of 2026-10-28 are missing (R19). The working-tree changes of another session were out of scope. Costs come from the session usage records. `jscpd` duplicates: `local-only/data/duplicates.txt`.

## Files

- `findings/code-prompt-llm.md`, `findings/code-dashboard-http.md`, `findings/tooling-env.md`: code and tooling audits.
- `findings/sessions-owner.md`, `findings/sessions-delegates.md`: process.
- `findings/outside-check.md`: verification.
- `local-only/data/`: the extracted session data. It stays on this machine (git-ignored). `scripts/sessions.py` and then `scripts/analyse.py` make it again from the session logs in `~/.pi/agent/sessions/`. `scripts/`: the extractors. `briefs/`: the agent briefs.
- This folder was written at `tmp/retro-2026-11-04/` and moved here on 2026-11-04. The briefs and the data are records, so they still name the old `tmp/retro-2026-11-04/data/` path. That path is now `local-only/data/`.
