# Research: what a CRAP report shows on this codebase

Date: 2026-10-31. Ticket: [01 — what a CRAP report shows on this codebase](../issues/01-research-crap-report.md).
Scratch evidence: `tmp/unclebob/` (source copy, four variant dumps, the two export scripts, the run output; git-ignored).

## Answer

The CRAP report runs in **4 seconds** on fresh coverage, and **one rule decides the whole ranking**: whether `?` counts as a branch.

- **`?` is 25.7 % of all cyclomatic complexity.** Over 917 scored functions the CC total is 2,836 with `?` and 2,106 without. 119 functions (13.0 %) have CC entirely from `?` — every `from_row` column list is one.
- **Counting `?` fills the top of the list with column lists.** Under the upstream rule, 7 of the top 20 are `from_row` functions with CC 12–19 and 100 % line coverage. They rank there because `rusqlite::Row::get` returns `Result`, so each column costs one decision. Dropping `?` sends all of them to rank 400+.
- **The list degenerates into a complexity list, because coverage is high.** 85.5 % of functions (784 of 917) have 100 % line coverage, and for them CRAP = CC exactly. Under region coverage 66.8 % sit at 100 %. So most rows say "this function has many branches", not "this function is untested".
- **Fresh coverage removes the two distortions of the 2026-10-25 run.** Wiring code now reads 50–100 % (`build_app_graph` 100 %, `recorder_with_storage` 77 %); the earlier 0 % was the two-day-old coverage file. The remaining 0 % rows are explained by call site, not by staleness — see `init_logging` below.
- **Branch coverage is not available.** `cargo llvm-cov report --lcov` emitted **0 BRDA records** on this toolchain, so crapper's branch path is dead here. Per-region counts from the JSON export are the available substitute, and they are cheap: they change 1 row of the top 20.
- **Of the top 20 under the upstream rule, 6 point at a testable risk** (my reading, inferred). 9 are artefacts of counting or of unreachable code, 4 are real but already well covered, and 1 (`migrate_up_to`) is a counting artefact that still points at a real 723-line function. Under the `?`-free rule 5 of 20 point at a coverage gap and 6 more are complexity-only rows.

## How the runs were made

| Step | Command | Cost |
|---|---|---|
| Fresh coverage | `python build.py --coverage --target-dir target/crap01 --no-fmt` | **487.6 s** gate-reported; 522 s wall |
| LCOV export (no test rerun) | `python tmp/unclebob/run_lcov.py target/crap01 tmp/unclebob/cov/crap01_full.lcov` | 12 s work, 74 s slot wait |
| JSON export, no ignore regex | `python tmp/unclebob/run_json.py target/crap01 tmp/unclebob/cov/crap01_full.json` | 21 s work, 116 s slot wait |
| CRAP, one variant | `python3.13 -m crapper --root tmp/unclebob/ce --use-existing-coverage` | **3.8–4.0 s** |

The coverage run was made twice. The first attempt started at 12:53:06 and the tool timeout killed it at 1200 s while it was at step 16 of 18; 328 s of that went to waiting for the build slot, which another agent held. The second attempt started at 13:13:13 and finished at 13:21:55. Its steps: integration with coverage 140.1 s, browser with coverage 259.8 s, coverage report 25.4 s. Totals: 1,833 + 44 tests passed, 3 skipped; **95.2 % line coverage** (10,942 of 11,499) on `build.py`'s own ignore set.

The target dir is dedicated (`--target-dir target/crap01`) and holds **6.1 GiB**, of which 3.4 GiB is `llvm-cov-target`. `ENVIRONMENT.md` requires this: the coverage run overrides `RUSTC_WRAPPER` and rebuilds every crate under instrumentation, so a shared `target/` would be rebuilt twice.

**Tree the coverage came from.** `git status --porcelain src/` was empty at 12:52 (before), 13:22 (after the coverage run) and 13:34 (after both exports). HEAD `bf3b87a5`. Ticket 03's mutation driver ran during the exports in `tmp/mutation-wt`, a separate worktree, so it cannot have touched `src/` here. The copy the CRAP report scored is `tmp/unclebob/ce/src/` — 221 files, taken from the same clean tree, with `*_tests.rs` (86 files), `src/test_support/` (10 files) and `src/adapters/driven/storage/test_support.rs` removed.

Tools: crapper commit `6fd92c4` (2026-10-21) in `tmp/unclebob/crapper`, rustc 1.88.0, cargo-llvm-cov 0.9.1.

## The four variants

| Variant | CC rule | Coverage | Report file |
|---|---|---|---|
| **v1** | upstream: `?` counts | LCOV line hits | `tmp/unclebob/out/v1.tsv` |
| **v2** | `?` dropped | LCOV line hits | `tmp/unclebob/out/v2.tsv` |
| **v3** | upstream: `?` counts | JSON regions | `tmp/unclebob/out/v3.tsv` |
| **v4** | `?` dropped | JSON regions | `tmp/unclebob/out/v4.tsv` |

v2 and v4 use a patched copy of the tool (`tmp/unclebob/crapper_noq/`), one line: `"try_expression"` removed from `_DECISIONS` in `languages/rust.py`. The region variant replaces crapper's `line → (covered, total)` map with one entry per LLVM region-entry segment (`tmp/unclebob/region_crap.py`), so crapper's own join by file and line range computes a region percentage.

`?` counted (`v1`) and dropped (`v2`) differ in every `from_row`:

| Function | CC with `?` | CC without |
|---|---|---|
| `DbGame::from_row` (18 columns) | 19 | 1 |
| `DbCharacter::from_row` (16) | 17 | 1 |
| `DbWorld::from_row` (14) | 15 | 1 |
| `DbPersona::from_row` (13) | 14 | 1 |
| `migrate_up_to` | 200 | 73 |

`migrate_up_to`'s CC 200 breaks down as 127 `?` + 66 `if` + 5 `for` + 1 `&&` over lines 25–747.

## Stability of the ranking

Overlap of the top N between two variants, out of 917 functions:

| Change | top 10 | top 20 | top 50 |
|---|---|---|---|
| CC rule only (v1 → v2) | 3 / 10 | **6 / 20** | 24 / 50 |
| Coverage shape only (v1 → v3) | 9 / 10 | **19 / 20** | 49 / 50 |
| CC rule only, on region coverage (v3 → v4) | 3 / 10 | 6 / 20 | 25 / 50 |

So the CC rule decides the list and the coverage shape does not. Once `?` is dropped, line and region coverage give the same 18 of the top 20.

Counts at or above a CRAP score:

| Threshold | v1 | v2 | v3 | v4 |
|---|---|---|---|---|
| ≥ 100 | 1 | 0 | 1 | 1 |
| ≥ 30 | 2 | 2 | 3 | 2 |
| ≥ 20 | 5 | 2 | 9 | 2 |
| ≥ 15 | 10 | 2 | 15 | 2 |
| ≥ 10 | 43 | 18 | 46 | 20 |
| ≥ 5 | 186 | 117 | 187 | 118 |

A threshold of 30 leaves 2–3 rows in every variant. A threshold of 10 leaves 18–46 rows.

## Top 20, v1 (upstream rule), annotated

Columns 5–7 give the rank in v2 / v3 / v4.

| # | Function | File:line | CC | Cov % | CRAP | Rank v2/v3/v4 | Why it ranks there | Real risk? | A developer would |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `migrate_up_to` | `adapters/driven/storage/utils/plumbing.rs:25` | 200 | 97.2 | 200.9 | 1 / 1 / 1 | 127 `?` + 66 `if` over 723 lines; CC 73 without `?` | size is real, the score is not | split the migration list per version range; CRAP adds nothing a line count does not |
| 2 | `init_logging` | `bootstrap/logging.rs:11` | 6 | 0.0 | 42.0 | 2 / 2 / 2 | 0 % in all four variants: its only caller is `src/main.rs:15`, which no test runs | no — unreachable from a test | exclude the file; `build.py` already does |
| 3 | `prepare_data` | `bootstrap/run.rs:44` | 13 | 63.9 | 21.0 | 34 / 7 / 34 | 7 `?` + 3 `if` + 2 `match`; DB pool, three seeders, world fallback — nothing boots it | yes | a test that boots against a temp dir; expensive |
| 4 | `process_world_dir` | `bootstrap/load.rs:93` | 20 | 87.0 | 20.9 | 3 / 6 / 4 | 7 `?` + 6 match arms + 5 `if` + 1 `for`; the seeding walk | yes, mostly covered | test the unreadable-manifest path |
| 5 | `start_server` | `bootstrap/run.rs:132` | 4 | 0.0 | 20.0 | 372 / 9 / 376 | CC is 1 + 3 `?`; CC 1 without. Rank comes from 0 % of server bring-up | no | nothing; the browser tier covers the server it starts |
| 6 | `DbGame::from_row` | `adapters/driven/storage/models/game.rs:30` | 19 | 100.0 | 19.0 | 452 / 3 / 411 | 18 `?`, one per column; CC 1 without | no — counting artefact | nothing |
| 7 | `StoryReads::story_row` | `adapters/driven/storage/stories.rs:248` | 10 | 55.6 | 18.8 | 402 / 15 / 406 | 9 `?`; the 55.6 % is one untested `.map_err` closure | partly | test the bad-timestamp path |
| 8 | `DbCharacter::from_row` | `adapters/driven/storage/models/character.rs:28` | 17 | 100.0 | 17.0 | 450 / 4 / 412 | 16 `?`, one per column | no | nothing |
| 9 | `run` | `bootstrap/run.rs:28` | 6 | 35.7 | 15.6 | 189 / 10 / 187 | 4 `?` + 1 `if`; the `--list-worlds` branch and the boot chain | yes | the same boot test as #3 |
| 10 | `DbWorld::from_row` | `adapters/driven/storage/models/world.rs:25` | 15 | 100.0 | 15.0 | 465 / 5 / 413 | 14 `?`, one per column | no | nothing |
| 11 | `process_preset_file` | `bootstrap/run.rs:177` | 14 | 94.6 | 14.0 | 14 / 18 / 15 | 4 `if` + 4 `?` + 3 `&&` + 2 `match`; the seed merge rules | real, well covered | nothing |
| 12 | `DbPersona::from_row` | `adapters/driven/storage/models/persona.rs:24` | 14 | 100.0 | 14.0 | 458 / 8 / 414 | 13 `?`, one per column | no | nothing |
| 13 | `handle_response` | `adapters/driven/llm/transport/utils/response.rs:114` | 7 | 47.8 | 14.0 | 26 / 14 / 21 | 2 `if` + 2 `match` + 2 `?`; non-success status and unreadable body untested | yes | a test with a mock HTTP error response |
| 14 | `Job::narrate` | `application/pipeline/job.rs:232` | 13 | 97.7 | 13.0 | 199 / 19 / 197 | 10 `?` + 2 `match`; reads six records then calls the narrator | real, well covered | nothing |
| 15 | `update_world_handler` | `adapters/driving/http/worlds/handlers/worlds.rs:163` | 13 | 97.8 | 13.0 | 4 / 20 / 5 | 9 match arms + 3 `if`; the world form merge | real, well covered | nothing |
| 16 | `JobPlan::for_request` | `domain/model/generation.rs:234` | 13 | 100.0 | 13.0 | 32 / 21 / 32 | 7 match arms + 5 `?`; 100 % covered, so CRAP = CC | no — complexity only | nothing |
| 17 | `action_check_handler` | `adapters/driving/http/action/handlers/actions.rs:50` | 11 | 79.4 | 12.1 | 5 / 17 / 3 | 8 match arms + 2 `if`; the settings and text-check error returns are untested | yes | test the settings-failure and text-check-failure returns |
| 18 | `DbLlmMessage::from_row` | `adapters/driven/storage/models/llm_message.rs:19` | 12 | 100.0 | 12.0 | 455 / 11 / 415 | 11 `?`, one per column | no | nothing |
| 19 | `DbPromptPreset::from_row` | `adapters/driven/storage/models/prompt_preset.rs:23` | 12 | 100.0 | 12.0 | 460 / 12 / 416 | 11 `?`, one per column | no | nothing |
| 20 | `DbSettings::from_row` | `adapters/driven/storage/models/settings.rs:25` | 12 | 100.0 | 12.0 | 462 / 13 / 417 | 11 `?`, one per column | no | nothing |

Tally (my reading, inferred): 7 counting artefacts (`from_row`), 2 unreachable-by-test functions, 1 artefact-dominated 723-line function, 4 real but well covered, **6 real coverage gaps**.

## Top 20, v2 (`?` dropped), annotated

The new entries are the ones v1 buries. Column 5 gives the rank in v1.

| # | Function | File:line | CC | Cov % | CRAP | Rank v1 | Why it ranks there | Real risk? | A developer would |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `migrate_up_to` | `adapters/driven/storage/utils/plumbing.rs:25` | 73 | 97.2 | 73.1 | 1 | still #1, at a third of the score | size is real | split it |
| 2 | `init_logging` | `bootstrap/logging.rs:11` | 6 | 0.0 | 42.0 | 2 | 0 % of unreachable code | no | exclude the file |
| 3 | `process_world_dir` | `bootstrap/load.rs:93` | 13 | 87.0 | 13.4 | 4 | real branching, 87 % | yes | as above |
| 4 | `update_world_handler` | `adapters/driving/http/worlds/handlers/worlds.rs:163` | 13 | 97.8 | 13.0 | 15 | 9 match arms | real, covered | nothing |
| 5 | `action_check_handler` | `adapters/driving/http/action/handlers/actions.rs:50` | 11 | 79.4 | 12.1 | 17 | error returns untested | yes | as above |
| 6 | `ApplicationError::source` | `application/errors.rs:62` | 3 | 0.0 | 12.0 | 21 | 2 match arms, 0 %: no caller in `src/` — it satisfies `std::error::Error` | no | ignore, or exclude trait plumbing |
| 7 | `LlmCallRecorder::provider_label` | `application/llm_recorder.rs:43` | 3 | 0.0 | 12.0 | 22 | 0 % although four production sites call it (`orchestration.rs:47`, `orchestration.rs:80`, `narrator.rs:52`, `wiring.rs:120`), all as a `tracing` field; `tracing` skips the argument when the level is off (inferred) | no | nothing; expect 0 % for log-only code |
| 8 | `GenerationFailureKind::line` | `domain/model/state/generation_status.rs:24` | 12 | 100.0 | 12.0 | 25 | 11 match arms, fully covered → CRAP = CC | no | nothing |
| 9 | `try_parse_json_full` | `application/agents/quantifier/utils/parser.rs:62` | 10 | 75.8 | 11.4 | 26 | three parse attempts with nested `if let`; the fence and brace fallbacks are partly untested | yes | test the two fallback shapes |
| 10 | `answer_for_error` | `adapters/driving/http/utils/error.rs:80` | 11 | 100.0 | 11.0 | 29 | 10 match arms, fully covered | no | nothing |
| 11 | `update_world_posture_handler` | `adapters/driving/http/worlds/handlers/worlds.rs:248` | 11 | 100.0 | 11.0 | 30 | 3 parse results in one tuple match | no | nothing |
| 12 | `Record::name` | `domain/model/state/generation_status.rs:109` | 11 | 100.0 | 11.0 | 32 | 10 match arms returning literals | no | nothing |
| 13 | `HarperTextChecker::check` | `adapters/driven/text_check/harper_text_checker.rs:52` | 10 | 87.5 | 10.2 | 33 | mode dispatch, one mode untested | yes, small | test the untested mode |
| 14 | `process_preset_file` | `bootstrap/run.rs:177` | 10 | 94.6 | 10.0 | 11 | seed merge rules | real, covered | nothing |
| 15 | `OptionsPromptBuilder::build_user_prompt` | `application/agents/options/prompt.rs:40` | 10 | 95.7 | 10.0 | 36 | template sections | real, covered | nothing |
| 16 | `QuantifierPromptBuilder::build_user_prompt` | `application/agents/quantifier/prompt.rs:54` | 10 | 96.3 | 10.0 | 37 | template sections | real, covered | nothing |
| 17 | `lint_kind_to_issue_kind` | `adapters/driven/utils/harper.rs:6` | 10 | 100.0 | 10.0 | 40 | 9 match arms, a pure mapping | no | nothing |
| 18 | `screen_handler` | `adapters/driving/http/screen/handlers/screen.rs:60` | 10 | 100.0 | 10.0 | 41 | render loop and two header parses | no | nothing |
| 19 | `Job::end` | `application/pipeline/job.rs:378` | 7 | 61.9 | 9.7 | 44 | the failure paths of a Job; the refused-save arms are untested | yes | test the refused save |
| 20 | `AgentRunner::from_configs` | `application/agents/runner.rs:23` | 9 | 90.9 | 9.1 | 27 | agent-type dispatch | real, covered | nothing |

Tally (my reading, inferred): **5 real coverage gaps**, 6 complexity-only rows at 100 % coverage, 3 zero-coverage rows of which 1 (`init_logging`) is a false alarm and 2 are log-only or trait-only code.

## Top 20, v3 (region coverage), annotated where it differs

Region coverage lowers most percentages, so it raises the scores of the untested error paths that line coverage hides. 19 of the 20 rows are the v1 rows.

| # | Function | File:line | CC | Cov % | CRAP |
|---|---|---|---|---|---|
| 1 | `migrate_up_to` | `adapters/driven/storage/utils/plumbing.rs:25` | 200 | 77.4 | 663.7 |
| 2 | `init_logging` | `bootstrap/logging.rs:11` | 6 | 0.0 | 42.0 |
| 3 | `DbGame::from_row` | `adapters/driven/storage/models/game.rs:30` | 19 | 67.9 | 31.0 |
| 4 | `DbCharacter::from_row` | `adapters/driven/storage/models/character.rs:28` | 17 | 68.0 | 26.5 |
| 5 | `DbWorld::from_row` | `adapters/driven/storage/models/world.rs:25` | 15 | 68.2 | 22.2 |
| 6 | `process_world_dir` | `bootstrap/load.rs:93` | 20 | 84.3 | 21.6 |
| 7 | `prepare_data` | `bootstrap/run.rs:44` | 13 | 64.6 | 20.5 |
| 8 | `DbPersona::from_row` | `adapters/driven/storage/models/persona.rs:24` | 14 | 68.3 | 20.2 |
| 9 | `start_server` | `bootstrap/run.rs:132` | 4 | 0.0 | 20.0 |
| 10 | `run` | `bootstrap/run.rs:28` | 6 | 31.8 | 17.4 |
| 11 | `DbLlmMessage::from_row` | `adapters/driven/storage/models/llm_message.rs:19` | 12 | 68.6 | 16.5 |
| 12 | `DbPromptPreset::from_row` | `adapters/driven/storage/models/prompt_preset.rs:23` | 12 | 68.6 | 16.5 |
| 13 | `DbSettings::from_row` | `adapters/driven/storage/models/settings.rs:25` | 12 | 68.6 | 16.5 |
| 14 | `handle_response` | `adapters/driven/llm/transport/utils/response.rs:114` | 7 | 42.2 | 16.5 |
| 15 | `StoryReads::story_row` | `adapters/driven/storage/stories.rs:248` | 10 | 61.8 | 15.6 |
| 16 | `DbSwipe::from_row` | `adapters/driven/storage/models/swipe.rs:18` | 11 | 68.8 | 14.7 |
| 17 | `action_check_handler` | `adapters/driving/http/action/handlers/actions.rs:50` | 11 | 70.0 | 14.3 |
| 18 | `process_preset_file` | `bootstrap/run.rs:177` | 14 | 89.3 | 14.2 |
| 19 | `Job::narrate` | `application/pipeline/job.rs:232` | 13 | 86.7 | 13.4 |
| 20 | `update_world_handler` | `adapters/driving/http/worlds/handlers/worlds.rs:163` | 13 | 97.6 | 13.0 |

Two readings of the same fact, and they matter for the value decision:

- **Region coverage exposes the `from_row` error path.** Every `from_row` falls from 100 % to ~68 %: the `?` line ran on the `Ok` path, and the `Err` region did not. So with `?` counted, region coverage makes the artefacts *louder* (CRAP 31 instead of 19 for the same code).
- **Region coverage alone would have found `DbSwipe::from_row`** at rank 16, which line coverage ranks 28th. That is a difference of 12 places, and it is the whole size of the coverage-shape effect.

## Traps and costs a build ticket must carry

1. **The tool cannot tell "no data" from "0 %".** `analyze.py::_coverage` turns a file the report does not mention into `0.0`. `build.py`'s own coverage export drops `src/main.rs`, `bootstrap/logging.rs` and `llm/transport/utils/client.rs` by regex, so a CRAP run on that file scores `init_logging` at 0 % and puts it at rank 2. Either export coverage without the regex (21 s, as done here) or exclude those paths from the CRAP run.
2. **It does not skip `*_tests.rs`.** `discover.py` has no Rust test suffix. The production scope needs a pre-filtered copy, as the map says. `src/adapters/driven/storage/test_support.rs` is a third test-only location (`#[cfg(feature = "testing")]`, `storage/mod.rs:25-29`) that neither the map nor the tool names.
3. **`--use-existing-coverage` only reads `target/coverage/**/lcov.info`** under `--root`. The LCOV must be copied there, and the file paths inside it may stay absolute: the tool's suffix join matched all 917 functions with no misses.
4. **The venv's interpreter symlinks vanish.** `tmp/unclebob/.venv/bin/python` and `python3` were removed twice in this session (the shared tree is busy). The `.pth` editable installs are not read from `PYTHONPATH`, so run the tool as `PYTHONPATH=tmp/unclebob/crapper/src:<venv site-packages> python3.13 -m crapper`.
5. **Coverage needs its own target dir** (6.1 GiB, 8m42s). A CRAP run against existing profile data is 12 s of export plus 4 s of scoring. So "reuse the last coverage run" is 500× cheaper than "run coverage on demand", and the numbers above show the reuse is good enough: the report is 4 s from an existing `coverage.json`.

## What a value decision needs

The facts, side by side:

| Question | v1 (`?` counts) | v2 (`?` dropped) |
|---|---|---|
| Rows at CRAP ≥ 30 | 2 | 2 |
| Rows at CRAP ≥ 10 | 43 | 18 |
| Top-20 rows that point at a coverage gap | 6 | 5 |
| Top-20 rows that are counting artefacts | 7 | 0 |
| Top-20 rows at 100 % coverage (CRAP = CC) | 8 | 6 |
| Top-20 rows that are unreachable, log-only or trait-only code | 2 | 3 |

Open questions this asset cannot settle:

1. Is a report whose top 20 is half artefacts still worth 4 s, if the artefacts are a fixed, known set that an ignore list removes (`from_row`, `story_row`, `migrate_up_to`, `init_logging`, `start_server`, `source`, `provider_label`)?
2. Which CC rule? Counting `?` needs a seven-entry ignore list and produces a 200 score for a migration list. Dropping `?` needs no ignore list and produces a list of 18 rows at CRAP ≥ 10, of which 6 are complexity-only.
3. Does a CRAP threshold ever belong in the gate, given that 85 % of functions sit at 100 % coverage and CRAP = CC for them? A threshold of 30 passes today with 2 rows; a threshold of 10 fails today with 18–46.
4. Is line coverage enough, or is the region export worth its extra 12 s? It moves 1 row of the top 20 but raises the score of every `from_row` by 40–60 % (`DbGame::from_row` 19.0 → 31.0).
