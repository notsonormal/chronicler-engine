# Weekly audit 2026-11-04: tooling-env

## Summary

The build loop works, and agents use the narrow form of it: 367 of 378 `test-pattern` runs use `--test-target` (median 13 s). Most of the build time goes to the full gate, and the browser tier drives that cost. The browser tier grew from 92 s to about 210 s in three weeks, but every cost number in `AGENTS.md` and `ENVIRONMENT.md` still shows the old state. The 44 % and 40 % fail rates of `test-pattern` and `clippy` are mostly TDD red steps and first compiles, not waste. Other avoidable costs: permission rules that deny safe commands, an always-loaded structure index, and a trial harness tied to the storage schema.

## Findings

### 1. High — The full gate takes 43 % of the build time, and the browser tier is the main part of it

- Evidence (known): `logs/build_history.txt` has 49 full gates (4.45 h) and 38 `--no-browser` gates (1.35 h), so 5.8 h of 13.4 h. The green medians are 359 s for the full gate and 105 s for `--no-browser`. In the 15 gate logs, the browser step median is 210 s and the integration step median is 57 s. The newest JUnit report has 44 browser tests with 620 s of test time (`run_screen_checks` 44.9 s). `ENVIRONMENT.md:65` records the same browser step at `92` s on 2026-10-10.
- Evidence (known): 87 gates ran on 29 HEADs, up to 10 on one HEAD.
- Evidence (known): the `test-pattern` failures are 138 red tests (exit 100), 15 compile errors and 12 "no tests matched" (exit 4). In the 3-day log window, the 15 `clippy` failures are 9 compile errors and 6 lints. This is the loop working as intended.
- Why review missed it: each ticket added a few browser checks, so no single diff showed the 2.3× growth.
- Fix class: **check**.
- Suggested fix: record in the journal the tree of each green browser tier. Let the gate skip the browser tier when the diff since that tree touches no browser input (`src/adapters/driving/http/**`, `assets/**`, `tests/browser/**`, `tests/test_utils/**`, `Cargo.lock`). Inferred saving: most of the 2.9 h/week of in-gate browser runs. A cheaper first step: find the slow checks in the 45 s runners, using the per-check timings that the JUnit report already holds.

### 2. High — The permission rules deny safe commands, and the actual hazard is the location of `target/`

- Evidence (known): `config.json` rule `grep * . *` denied 42 calls since 2026-10-28. A heuristic parse of those 42 commands (cwd taken from the `cd` segment, recursive flag, path argument) finds **0** that walk the repo root. Examples: `cd src; grep -rn ... .` and `grep -c . $f` (`.` is a regex there). This audit hit the rule the same way. Rule `grep * /workspace/chronicler-engine *` denied 27 calls, and 26 of them do walk the root.
- Evidence (known): `target/` is 28 GB, not "~10 GB" as the rule's reason says. It holds 7 target dirs (`crap01`, `rf-coverage`, `test-police`, `lt11`, `lt15`, `ui-investigator`, `debug`). The rest of the checkout is about 2.7 GB.
- Evidence (known): `timeout *` denied 23 calls, mostly `timeout 1200 python build.py`. Its reason states a preference, not a harm, and I do not know a harm.
- Evidence (known): `find / *` denied 15 calls, which looked for probe DBs, skills and session dirs.
- Why review missed it: the config is outside the diffs, and each denial costs only one turn.
- Fix class: **structure**.
- Suggested fix: put cargo target dirs outside the checkout. `build.py` `_target_paths` already owns the path, so it can map `target/<name>` to `~/.cache/chronicler-target/<checkout>/<name>`. Keep `target/nextest/` in place, because nextest writes its JUnit path there (`.config/nextest.toml`). Then remove the four `grep` rules. Remove `timeout` too, or give it a reason that names a harm. Change `find / *` to allow `-maxdepth`. The `--target-dir target/...` examples in the docs and two skills change with it.

### 3. Medium — Three quarters of `AGENTS.md` is a cache of a one-command lookup

- Evidence (known): `AGENTS.md:7-328` (321 of 422 lines, about 25 KB of 29 KB) is the AUTO-STRUCTURE block. Every session and every delegate loads it on every turn. `rg -m2 '^//! ' src` gives the same facts. The block needs the line-2 summary rule (`CODING_STANDARDS.md`), the pre-commit regeneration, and STE fixes of generated text (`38043843`).
- Why review missed it: the block is generated, so no diff shows its size as a choice. It grows by one line for each new file (+5 net this week).
- Fix class: **one-off fix**.
- Suggested fix: replace the block with one pointer line, for example: "File summaries: `rg -m2 '^//! ' src`". (the `writing-for-agents` *cache* test).

### 4. Medium — `scripts/trial.py` is a lasting harness, but it lacks stable inputs and kept results

- Evidence (known): it is lasting. 6 of its 9 commits come from feature tickets (`3d3a61ed`, `ce90c2cd`, `1359b375`, `1989d3fe`, `20d6d23c`), and it has `run`, `export` and `compare`.
- Evidence (known): the setup is not portable. `scripts/trial_setups/redmist_julian.json:14` names `"conn-1793449507061-3a1cb15f"`, which is a row id of one probe DB.
- Evidence (known): the script writes engine storage directly. `trial.py:459` has `update settings set connections = ?`. To test this, `test_trial.py:70` (`engine_tables`) replays the Rust migrations by regex over the string literals in `plumbing.rs`. `trial.py:44-53` mirrors three Rust constants (`Budget::MAX`, `Budget::DEFAULT`, `DEFAULT_MAX_HISTORY_TOKENS`), and no check enforces the mirror.
- Evidence (known): results are not kept. Packs go to `tmp/` (`"out": "tmp/pf18/redmist_julian"`), and every full gate deletes files older than 30 days there (`build.py:1386`, `clean_tmp_dirs`).
- Why review missed it: each ticket added only the coupling it needed.
- Fix class: **structure**.
- Suggested fix: name Connections in the setup by provider and model, and let the script find or create the row through `/settings`. Set the history size through the Connection form. Write `summary.json` of each trial to a tracked or long-lived results journal, keyed by setup, commit and model.

### 5. Medium — The gate's packaging tail causes a data trap and contains dead code

- Evidence (known): every gate copies `data/` into `<target>/debug/data` (`build.py:1301`, `_copy_deployment_assets`). The server then prefers that copy, so a world edit does not show until the next gate. `ENVIRONMENT.md:95` and `chronicler-ui-investigator/SKILL.md:112` describe the trap instead of removing it.
- Evidence (known, dead code): `_gate_tail` dumps `target_dir / "data" / "chronicler.db"` (`build.py:1389`). The server writes `chronicler_{port}.db` (`src/bootstrap/run.rs:50`), and `tmp/db_dumps/` does not exist. `project_root_tmp` (`build.py:1384`) points at `/workspace/tmp`, which is outside the repo.
- Why review missed it: these are leftovers of older storage and layout decisions. The build.py refactor tickets changed the step machinery, not the tail.
- Fix class: **one-off fix**.
- Suggested fix: remove the dump and the outer-tmp path. Copy the assets only with `--release`, so the dev server always reads the repo `data/`. Then delete the two trap paragraphs. About the size of `build.py` (1783 lines): the step registry and plan are one coherent job. The extra jobs are packaging, housekeeping and `run` (`build.py:1409-1553`). Remove the dead parts and move `run` to its own module.

### 6. Medium — The cost tables claim to be journal medians, but every value is 1.5 to 3 times too low

- Evidence (known): `AGENTS.md:388` says "Warm cost is the median of recent runs in `logs/build_history.txt`". The green medians this week, with the table value in parentheses, are: clippy 18 s (~12), `--test-target` 13 s (~6), integration 59 s (~21), guardrails 26 s (~10), browser 233 s (~92), `--no-browser` 105 s (~50), full 359 s (~145). `ENVIRONMENT.md:65` says 134 s (2026-10-10). The `build.py` docstring says "about 2 minutes".
- Why review missed it: the numbers are prose, and `38043843` rewrote these sentences but kept the values.
- Fix class: **check**.
- Suggested fix: generate the "Warm cost" column from the journal, the same way as the other generated blocks, or remove the column. Caveat: journal durations include build-slot waits.

### 7. Low — The measurement history lasts about as long as one retro window

- Evidence (known): `build_history.py:24` sets `MAX_LINES = 1000`, so the brief's "999 runs" is the cap, not the true count. Build logs are kept 3 days (`build.py:1712`).
- Fix class: **one-off fix**. Rotate old lines to `logs/build_history.<n>.txt` instead of deleting them.

### 8. Low — The build-tool tests pin the data table, not the behaviour

- Evidence (known): `test_build_cli.py:461-490` `EXPECTED_COMMANDS` is a verbatim copy of `REGISTRY`, and `test_clippy_matches_full_gate` (`:58`) pins one entry a second time. Every intended edit fails them, and they catch nothing that running the step misses. Otherwise the proportion is fine: about 2.2k test lines for about 2.4k code lines, with real fixtures under `scripts/tests/fixtures/build_output/`.
- Fix class: **rule** (the existing "tautological tests" rule). Delete the two tests.

### 9. Low — Skills overlap, and 8 had no use since 2026-10-28

- Evidence (known, session counts of invocations and `SKILL.md` reads): `correct`, `grill-me`, `handoff`, `improve-codebase-architecture`, `in-your-own-words`, `show-me`, `teach` and `wait-what` have 0 uses. Two families overlap: three grill skills (17, 2, 0 uses) and three after-plan skills (12, 2, 3 uses). 22 of 37 skills are hidden from the model, so they cost only cognitive load.
- Fix class: **one-off fix**. Merge each family into one skill, and hide or remove the unused skills.

## Bitter lesson

1. `trial.py` is the general method of this area: a run-and-compare loop over prompt choices. It measures cost and robustness only: cycles, tokens, cache %, failures and word counts (`trial.py:1395-1405`). Prompt quality is judged by a human reading `pack.md`. A scored comparison (a rubric or judge model over packs, kept across runs) would replace the hand-tuning of tickets 19-24 (`3d3a61ed`). It deserves more investment than `build.py`.
2. The structure index and the line-2 summary rule are hand-built navigation for a model that cannot search. Agents run `rg` now (finding 3).
3. The permission config is a hand-tuned list of string globs that cannot see the cwd or tell a regex from a path (finding 2). Moving `target/` out of the checkout removes the hazard, so the rules are no longer necessary.
4. Good direction: the STE length and semicolon rules became Vale checks (`styles/Chronicler/*.yml`), and `document-review` dropped its prose copy of them (`38043843`). Do the same with the cost table (finding 6). The journal already measures what the prose copies by hand.

## What went well

- The structured-output refactor (`672ac677`, `96824e52`) gives one compile-error and failing-test summary, from fixtures in `scripts/tests/fixtures/build_output/`. A failed clippy run ends with one `compile errors (first 1 of 1)` line with file and line (`logs/build_20261103_215121.log`).
- Agents use the narrow loop: 367 of 378 `test-pattern` runs use `--test-target`, and their median is 13 s.
- The rate-limit window rule has one home. `delegate/lib.ts:5` imports `windowEnded` from `rate-limit-wait/state.ts` (`1fceb065`), so the two extensions do not implement it twice.
