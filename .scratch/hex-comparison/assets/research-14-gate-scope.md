# Research 14: what a docs-only gate run costs, and which steps it can skip

Status: reference. Asset of the ticket [Measure what docs-only gate runs cost](../issues/14-research-docs-only-gate-cost.md) in the [map](../map.md). Made on 2026-11-01. Covers practice C-21 of [candidates.md](./candidates.md).

Each claim is **known** (a file, a log or a command shows it) or **inferred** (a reading of that evidence). A claim is known unless it says inferred.

## Summary

- **Few gate runs cover a docs-only change (known).** The journal holds 83 full-gate runs. Method (b) maps 80 of them to a commit. Only **1 run** covers a docs-only commit and **1 run** covers a tracker-only commit (`.scratch/` only). Together they took 739 s of 20,068 s (3.7 %).
- **Docs and tracker commits already skip the gate by hand (inferred).** The window holds 30 such commits. 2 of them have a gate run. 9 docs-only commits show 20 `validate-docs` runs and 4 `vale` runs.
- **A change-scoped gate would save about 720 s in 5 days (inferred).** That is 3.6 % of gate time. The saving is small because the demand is small.
- **Every Rust or test file reads only `src/`, `tests/`, `assets/` and `data/` (known, by search).** No Rust code or test reads `docs/`, root docs, `scripts/` or `.scratch/`. So the three Rust test steps cannot fail on a docs-only change.
- **The gate is slower than `AGENTS.md` says (known).** 16 passing full gates since 2026-10-31 have a median of 330 s. `AGENTS.md` says about 145 s.
- **Recommendation.** Write the skip list as a rule. Do not build change detection in `build.py` yet. Part 5 gives the reasons and the one cheap guard to add.

## 1. Method and what it misses

### Source of the runs

- `logs/build_history.txt` has 999 rows, from 2026-10-27T20:19 to 2026-11-01T20:00 (known). The file keeps at most 1000 lines (`scripts/build_history.py:24`, `MAX_LINES = 1000`). So it covers about 5 days.
- A full-gate run is a row with `args` equal to `(full gate)` (45 rows), `--no-browser` (37 rows), or `--no-browser --target-dir target/agent-review` (1 row). That makes **83 runs**. Coverage runs (3 rows) and `guardrails --no-fail-fast` are not counted, because they run a different plan.
- Exit codes of the 83 runs: 68 pass, 12 exit 1, 3 exit 100 (nextest failure). The failed runs are counted apart (part 2).

### Method (b): the file list of the child commit

- For each run I took the short HEAD from the `tree` column. A run on a dirty tree (`HEAD+digest`) belongs to the first non-merge child commit of that HEAD, committed no earlier than 10 minutes before the run. A run on a clean tree (`HEAD` only) belongs to the HEAD commit itself.
- I took the file list of that commit (`git diff-tree`) and sorted the files in four kinds:

| Kind | Paths |
|---|---|
| tracker | `.scratch/**` |
| doc | `docs/**`, `old-docs/**`, any `*.md` outside `src/`, `assets/` and `data/`, `.agents/**` (not `*.py`, `*.sh`), `.pi/**`, `.vale*` |
| script | `build.py`, `scripts/**`, `.agents/**/*.py`, `.agents/**/*.sh` |
| code | every other path: `src/`, `tests/`, `assets/`, `data/`, `Cargo.*`, `clippy.toml`, `arch-lint.toml`, `.config/` |

- A commit is *tracker-only*, *docs-only* (doc and tracker), *docs+scripts*, or *code* (any code file).
- Result: 80 runs map to a commit. 3 runs have no non-merge child, because the next commit is a merge of `origin/main`. They are listed as "merge".

### What method (b) misses

| Gap | Effect |
|---|---|
| Each checkout keeps its own `logs/` (known, ticket). The `remove-worktrees` step removes a linked worktree that is clean, pushed and older than 24 h (`scripts/remove_worktrees.py`, docstring). A removed worktree takes its `logs/` with it (inferred). | The count of gate runs is a **lower bound**. Only the main checkout is seen. `git worktree list` shows one worktree now. |
| A run on a dirty tree can cover two commits made later. I attach it to the first. | A late commit can look as if it had no gate. |
| A commit with no gate run in the main checkout may have had one in a worktree. | "30 docs and tracker commits, 2 gated" can be too low. It cannot be too high. |
| A run attaches to a commit by HEAD and time. A manual retry on the same HEAD attaches to the same commit. | Per-commit counts include retries. |
| Duration is the total of the journal row. It excludes the wait for the build slot (known: `build.py:1105` docstring; the total of one log, 306.3 s, matches its row, 307.2 s). | Times do not include queue time. |

### Method (a): the session logs (cross-check only)

- I scanned the 196 pi session files since 2026-10-27 in `~/.pi/agent/sessions/--workspace-chronicler-engine--/`. I took each `bash` call that runs `python build.py` with no subcommand, and the `edit` and `write` paths of that session since the previous gate call.
- Result: 66 gate calls. 12 follow code edits, 8 follow doc and script edits, 2 follow doc edits only, 1 follows tracker edits only. **43 follow no `edit` or `write` call.** In those, a subagent or a shell command made the change, so method (a) cannot sort them.
- Method (a) misses all edits that a subagent makes. So I use it only as a cross-check. It agrees with method (b): 3 of 66 calls are docs or tracker only (method (b): 2 of 83). No session text is quoted here.

## 2. What the gate runs cost

### Totals (known)

| Quantity | Value |
|---|---|
| Rows in the journal | 999 (48,960 s in all) |
| Full-gate runs | 83 (20,068 s, 41 % of all journal time) |
| Passed / failed | 68 / 15 (failed: 1,706 s) |
| `(full gate)`, passed | 39 runs, median 361.8 s |
| `(full gate)`, passed, since 2026-10-31 | 16 runs, median 330 s, range 283 to 449 s |
| `--no-browser`, passed | 28 runs, median 103.6 s |

The failed runs range from 12.9 s to 168.7 s. 7 of them stopped before the Rust steps, in under 45 s. All 15 failed runs belong to code commits.

### By kind of change (method (b), known)

| Kind of commit | Runs | Seconds | Share of gate time |
|---|---|---|---|
| code | 73 (15 failed) | 17,170 | 85.6 % |
| docs+scripts | 5 | 1,532 | 7.6 % |
| merge | 3 | 627 | 3.1 % |
| docs-only | 1 | 301 | 1.5 % |
| tracker-only | 1 | 438 | 2.2 % |

The 5 docs+scripts runs all follow commits that edit `build.py`. All 5 are `--no-browser` runs.

### Commits in the window (known)

The window starts at the first journal row. It holds 71 non-merge commits. The table counts those with a gate run attached.

| Kind | Commits | With a gate run |
|---|---|---|
| code | 36 | 22 |
| docs+scripts | 5 | 2 |
| docs-only | 16 | 1 |
| tracker-only | 14 | 1 |

- **Inferred:** the owner and the agents already skip the full gate for 28 of 30 docs and tracker commits. The narrow commands exist (`AGENTS.md`, command table: `validate-docs` and `vale` take under 1 s). Method (b) attaches 20 `validate-docs` runs and 4 `vale` runs to the 16 docs-only commits.
- **Inferred:** if all 30 had run the full gate, the cost would be about 30 × 330 s = 9,900 s. The manual habit already avoids this cost.
- The two runs that did cover a docs or tracker commit are the commit `42002bd8` (docs, `prompt-flexibility` ticket 24; one run, 301 s) and the commit `6cf23131` (tracker, `hex-comparison`; one run on the clean tree after the commit, 438 s).

### Cost by step (known)

The numbers are medians of the step times in the 54 gate-plan logs under `logs/build_*.log` (`--- Step Timing Summary ---`). The times are warm: no Rust change in a docs-only run means no compile.

| Step | n | Median (s) |
|---|---|---|
| install-hooks, remove-worktrees | 54 | 0.06, 0.09 |
| fmt | 51 | 1.5 |
| validate-data | 54 | 0.28 |
| clippy | 54 | 1.2 |
| test-structure | 54 | 0.08 |
| spec-coverage | 54 | 0.26 |
| seam-cases | 54 | 0.15 |
| docstrings | 54 | 0.05 |
| py-tests | 54 | 10.0 |
| http-routes-check, guardrails-doc-check | 38 | 0.08 each |
| validate-docs | 51 | 0.53 |
| vale | 15 | 0.45 |
| duplicates (report-only) | 51 | 2.0 |
| copy data and assets | 31 | 0.27 |
| guardrail tests | 51 | 35.0 |
| integration tests | 46 | 64.7 |
| browser tests | 31 | 234.4 |

- **Inferred:** the three Rust test steps take about 334 s. Every other step takes about 17 s together. So a gate with the Rust test steps skipped costs about 5 % of a full gate.
- **Known:** the median of the browser step in the gate logs is 234 s. `AGENTS.md` (command table) gives about 92 s for `python build.py browser`, and `ENVIRONMENT.md:65` gives 92 s (measured 2026-10-10). I do not know why the numbers differ. Machine contention is a guess.

## 3. What can break each step

Search for readers of `docs/` in Rust (known):

- `include_str!` and `include_bytes!` in `src/` and `tests/` name only `data/prompt_presets/**` (`src/domain/model/template_tests.rs:107,127,132`), `assets/index.html`, `tests/test_utils/*`, and `src/.../router.rs` (`tests/http/screen.rs:71,86`).
- `CARGO_MANIFEST_DIR` is used for `data/` (`src/bootstrap/load_tests.rs:265`, `src/domain/model/settings_tests.rs:846,863`) and for `tests/infrastructure/guardrails` (`registry_tests.rs:14`).
- The guardrail tests read `src/`, `tests/` and `assets/` (`tests/infrastructure/guardrails/mod.rs:118-147`, `:505-520`, `:536`, `:632`).
- The string `docs/` appears in Rust only in messages and in prefix checks. `structure.rs:74-87` checks that a DOC anchor starts with `docs/diataxis/reference/`. It does not open the file. The file check is `validate-docs` (`scripts/validate_docs.py`, rule `BROKEN_DOC_ANCHOR`).
- No Rust code starts `python`, `git`, `bash` or `sh` (search of `Command::new`).
- `py-tests` read `build.py`, `scripts/`, `scripts/lld-linker.sh` (`scripts/tests/test_build_cli.py:669`), the real `router.rs` (`scripts/tests/test_extract_http_routes.py:242`), and `.agents/skills/chronicler-comment-fixer/scripts` (`test_comment_finder.py:16`). A search for the repo root and for `docs` in `scripts/tests/` finds no test that reads a real doc. This is **inferred from a text search**, not from a run.

| Step | Files that can break it | Source |
|---|---|---|
| fmt | `*.rs`. It also rewrites the two generated docs | `build.py:1264-1275` (`_plan_gate_steps`) |
| validate-data | `data/**/*.json`, `data/schemas/*`, `scripts/validate_data.py` | `validate_data.py:64-88` |
| clippy | `*.rs`, `Cargo.toml`, `Cargo.lock`, `clippy.toml`, the lint block in `src/lib.rs` | inferred |
| test-structure | `src/**/*.rs`, `tests/**/*.rs`, its own script | `check_test_structure.py:12,58,83,92` |
| spec-coverage | `docs/specs/*.md`, `tests/http/**`, `tests/browser/**`, its own script | `validate_feature_spec.py:34-39` |
| seam-cases | `src/**/*_tests.rs`, `tests/storage/**`, its own script | `validate_seam_cases.py:55-56,223-226` |
| docstrings | `scripts/**/*.py` | `check_python_docstrings.py:72` |
| py-tests | `build.py`, `scripts/**`, `router.rs`, the fixture files | see above |
| http-routes-check | `router.rs`, `docs/diataxis/reference/frontend/http_routes.md`, `scripts/extract_http_routes.py` | `extract_http_routes.py:34,292,321` |
| guardrails-doc-check | `src/lib.rs`, `arch-lint.toml`, `tests/infrastructure/guardrails/*.rs`, `docs/diataxis/reference/coding_standards/guardrails.md`, its script | `generate_guardrails_doc.py:29-33` |
| validate-docs | `docs/**/*.md`, `.agents/skills/*/SKILL.md`, the root docs, and the DOC anchors in `src/**`, `tests/**` and `*.toml` | `validate_docs.py:1099,1106,1242-1243`, module docstring |
| vale | `docs/**` (standard docs), `AGENTS.md`-class root docs, `tests/AGENTS.md`, `tests/STRATEGY.md`, `.vale.ini` | `vale_lint.py:47-80` |
| duplicates (report-only) | changed files against `main` | `healthcheck.py:186` |
| copy data and assets | `data/**`, `assets/**` | `build.py:1301-1317` |
| guardrail tests | `src/**`, `tests/**`, `assets/**` | `guardrails/mod.rs` (above) |
| integration tests | `src/**`, `tests/**`, `data/**`, `assets/index.html`, `Cargo.*` | search above |
| browser tests | the same as integration, plus the stub fixtures and `assets/**` | `stub_server.rs:80-82,994` |

- **Known:** no step reads `.scratch/`. A search of `build.py`, `validate_docs.py`, `vale_lint.py`, the generators and `precommit_regenerate.py` finds no `scratch`.
- **Correction of a ticket fact (known).** The ticket says `build.py` has no logic that reads the changed files. That holds for the gate plan. But the report-only `duplicates` step is already scoped to files changed against `main` (`scripts/healthcheck.py:186-218`, `list_changed_files`). The gate plan already runs it (`build.py:1279`). So a changed-file reader exists and can be reused.

## 4. Skip list for each kind of change

The proposal sorts a change by its paths with the table in part 1. Rules:

- A mixed change runs the union of the lists of its kinds.
- A path that fits no kind counts as **code**.
- An empty change (a clean tree) runs the full gate.
- The two cheap steps `install-hooks` and `remove-worktrees` always run.

| Kind | Run | Skip | Run time (inferred) | Risk of the skip |
|---|---|---|---|---|
| **tracker-only** (`.scratch/**`) | nothing. `validate-docs` is an option | every step | 0 to 1 s, against 438 s | **Very low.** No step reads `.scratch/` (known). A later change that makes a step read it would need to change this list. |
| **docs-only** | `validate-docs`, `vale`, `spec-coverage`, `http-routes-check`, `guardrails-doc-check` | clippy, test-structure, seam-cases, docstrings, py-tests, validate-data, duplicates, copy, guardrail, integration and browser tests | about 1.4 s, against about 330 s | **Low.** (1) A Rust test could start to read `docs/`. None does today. A guardrail would catch it (part 5). (2) A `.md` file in `assets/`, `data/` or `src/` counts as code, so it is not skipped. (3) A hand edit of a generated doc fails the two freshness checks, which run. (4) A broken DOC anchor fails `validate-docs`, which runs. |
| **scripts-only, not `build.py`** | `py-tests`, `docstrings`, the step that the script implements, `validate-docs` | all Rust steps, `vale`, the other checks | about 12 s plus one check | **Low.** The Rust tests do not read `scripts/` (known). A script that rewrites a tracked file runs its own step. |
| **`build.py` or a build helper** (`build_slot.py`, `build_report.py`, `build_history.py`, `target_seed.py`) | as scripts-only, plus the `--no-browser` gate | the browser tier only | about 100 s, against about 330 s | **Medium.** `build.py` builds the nextest command line, so only a cargo run proves a change to it. The browser filter (`build.py:212`, `_BROWSER_CMD`) would be unproven. The owner already does this: all 5 docs+scripts runs are `--no-browser`. |
| **code** | the full gate | nothing | about 330 s | None. |

Savings (inferred, method (b)):

- tracker-only and docs-only: 2 runs, 739 s, replaced by about 20 s. Saving about 720 s in 5 days (3.6 % of gate time, 1.4 % of all journal time).
- build helper changes: no saving over the present habit.
- scripts-only without `build.py`: no such commit exists in the window, so the saving is 0 in the data.

## 5. Recommendation

**Do not build change detection in `build.py` now. Write the skip list as a rule, and add one guard.** Reasons:

1. **The demand is small.** 2 of 83 gate runs (3.7 % of gate time) cover docs-only or tracker-only commits. The people and agents already skip the gate for 28 of 30 such commits (lower-bound caveat, part 1).
2. **A wrong skip costs more than the saving.** The scoping rule must be right for every future path. The safe fallback ("unknown path means code") keeps the risk low but adds code, tests and a new failure mode to `build.py`.
3. **A rule costs one paragraph.** `AGENTS.md` can state: a change in `.scratch/` or `docs/` runs `python build.py validate-docs` and `python build.py vale`. It adds `python build.py spec-coverage` when `docs/specs/` changed. The table in part 4 is the text.
4. **One guard makes the rule safe.** Add one guardrail test: Rust code and tests read files only under `src/`, `tests/`, `assets/` and `data/`. The guard turns the "inferred" claim of part 3 into a failing test, if someone adds a reader of `docs/`. I have not checked how much work it is. A search for `include_str!`, `read_to_string` and `CARGO_MANIFEST_DIR` suggests a small walker rule (guess).
5. **If the owner wants the code anyway**, the work is about one session (guess): a path classifier, a filter in `_plan_gate_steps` (`build.py:1264`), and tests in `scripts/tests/test_build_cli_gate.py`. It can reuse `list_changed_files` (`healthcheck.py:186`). The base must be the working tree against `HEAD` plus untracked files, because the gate runs before the commit. An empty result must run the full gate (3 of the 83 runs were on a clean tree).

### A larger saving that C-21 does not name (inferred)

- 6 of the 83 runs are a `(full gate)` run on a tree that an earlier `--no-browser` run had already passed. The tree identity (`HEAD+digest`) was the same. The 6 repeats took 2,152 s. About 1,400 s of that is the browser tier (6 × 234 s), which was new. About 750 s repeated work. A rule such as "run `python build.py browser` after a passing `--no-browser` run on the same tree" would save that. The same tree identity is already in the journal (`scripts/build_history.py:39`, `tree_identity`).
- The browser tier is 65 % of a full gate (234 s of about 360 s). That is the largest single cost, and it is outside this ticket.

### Corrections to carry into the map

- `AGENTS.md` (command table) says the full gate takes about 145 s. The journal shows a median of 330 s (16 passing runs since 2026-10-31) and 362 s (all 39). The browser step is 234 s in the gate logs, against 92 s in the docs.
- C-21 states that the gate "runs the same for a docs-only change". That is true by design. But in practice the owner and agents already avoid the full gate for such a change.

## 6. Questions for ticket 16

1. Does the owner want the skip list as a **written rule** in `AGENTS.md` (with one guardrail), or as **change detection in `build.py`**? My recommendation is the rule. The data shows the practice already holds, and the code saves about 720 s in 5 days.
2. Should the gate skip a step that already passed on the identical tree (the repeat of `--no-browser` followed by the full gate; about 750 s in 5 days)? It is a different scope idea from C-21, and it uses the tree identity that the journal already holds.

## 7. Sources

- `logs/build_history.txt` (999 rows, 2026-10-27 to 2026-11-01); `scripts/build_history.py:24,39`.
- `logs/build_*.log`: 519 files, 54 with a gate-plan timing summary.
- `build.py:212,230,1105,1191-1203,1226,1264-1317`; `scripts/healthcheck.py:186-218`; `scripts/validate_docs.py:1099,1106,1242`; `scripts/validate_feature_spec.py:34-39`; `scripts/validate_seam_cases.py:55-56`; `scripts/vale_lint.py:47-80`; `scripts/remove_worktrees.py`.
- `tests/infrastructure/guardrails/mod.rs:118-147,505-520`; `tests/infrastructure/guardrails/structure.rs:74-87`; `src/bootstrap/load_tests.rs:265`; `src/domain/model/settings_tests.rs:846,863`; `src/domain/model/template_tests.rs:107-132`; `tests/test_utils/stub_server.rs:80-82`.
- `git log --all` and `git diff-tree` for the child commits (`42002bd8`, `6cf23131`, `96824e52`, `87122efa`).
- Session ids: 196 files under `~/.pi/agent/sessions/--workspace-chronicler-engine--/` with a start time on or after 2026-10-27T20. Counts only.
- The scripts that made the numbers are in `tmp/hex-compare/gate_scope.py`, `gate_scope2.py` and `session_scan.py` (git-ignored; they read the files above).
