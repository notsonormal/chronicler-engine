# Gate `--no-browser` tier and a checkable gate verdict

## Summary

Add `python build.py --no-browser`: the full gate minus the browser tier, as the iteration default. Make every test-tier verdict checkable:
- The journal records the tree and the per-tier test counts.
- The epilogue prints each tier's count change since that tier's last recorded run.

Make two targeted `AGENTS.md` edits, fix one line in `tests/AGENTS.md`, and add a measured row to `ENVIRONMENT.md`. Resolve retro tickets 03 and 22 through a new implementation ticket 30.

Decided in conversation:
- Bare `build.py` stays the full gate.
- `--coverage --no-browser` is rejected.
- No changed-path warning and no raw-cargo rule.
- `## Your Responsibility` is unchanged, and there is no other doc sweep.

## Key Changes

**`build.py` — the flag**

- Add the top-level flag `--no-browser` (`dest="no_browser"`). Help text: "Run the full gate without the browser tier (the iteration default; the final run is bare `python build.py`)".
- Add `"no_browser"` to `_GATE_ONLY_FLAGS`, so a step subcommand rejects it in both argument orders.
- In `parse_args`, when `args.coverage and args.no_browser`, call `parser.error("--no-browser cannot be combined with --coverage: coverage needs the browser tier")`. The exit code is 2.
- In `_plan_gate_steps`, the `TESTS` branch adds no browser `GateStep` when `args.no_browser` is set. All other steps stay the same.
- In `run_gate`, print `Skipping the browser tier (--no-browser set).`, in the same form as the `--no-fmt` line.
- In the epilogue, after the `nextest:` lines, print `skipped: browser tier (run "python build.py browser")` when the gate run used `--no-browser`.
- The help text of the `integration` `StepSpec` becomes: "Run every test binary except browser, architecture and guardrails, the lib unit tests included (~20s warm)."
- Add `python build.py --no-browser` to the invocation-modes list in the module docstring.

**`build.py` — the verdict record**

- **Tier detection.** `_tier_for_cmd(cmd)` returns a tier name when `cmd` matches one of these commands, and `None` for any other command:

  | Tier | Command |
  |---|---|
  | `architecture` | `REGISTRY["architecture"].cmd` |
  | `guardrails` | `REGISTRY["guardrails"].cmd` |
  | `integration` | `get_integration_test_cmd()` |
  | `browser` | `get_browser_test_cmd()` |

  Coverage, `--include-llm`, `test-pattern` and LLM runs therefore map to `None`. Step mode and gate mode pass the same command strings, so they share one baseline per tier. `_timed_run` sets `_NextestSummary.tier` next to `_NextestSummary.label`.
- **Count capture.** `_stash_nextest_summary` writes `_NextestSummary.sizes[label] = (tier, size)`. The size is the `tests run` count plus the `skipped` count, both read with `_summary_count`.
  - Nothing is recorded when the tier is `None` or the output has no Summary line.
  - `_NextestSummary.lines` keeps its current shape, so the existing stash and epilogue tests stay valid.
  - The reset code that clears `lines` also clears `sizes`.
- **Tree identity.** `_tree_identity(repo: Path) -> str` is called once in `main()`, after `os.chdir`, with the repo root as `repo`.
  - Clean tree: `git rev-parse --short=10 HEAD`.
  - Dirty tree: `<short HEAD>+<8-hex sha256>`. The hash input is `git diff HEAD --binary`, then each untracked, non-ignored path with its blob id, in sorted order. The paths come from `git ls-files --others --exclude-standard -z`, and one `git hash-object --stdin-paths` call gives the blob ids.
  - Any git failure: `unknown`.
- **Journal format.**
  - The header becomes `timestamp | duration_s | exit_code | args | tree | tests`.
  - `tests` is `architecture=1 guardrails=162 integration=1565 browser=69`, in that tier order. It is `-` when no tier recorded a size.
  - `_append_history` gains the `tree` and `tests` parameters.
  - An old 4-column header in line 1 is replaced in place.
  - Old 4-column records stay as they are. Readers treat their missing columns as absent.
- **Baseline and count change.**
  - Before the epilogue, `_last_tier_sizes(path, tiers)` reads the journal without a lock (best-effort).
  - For each tier, it takes the newest record that names the tier and returns that record's timestamp, tree and size. The current run is not yet in the journal, so a run cannot match itself.
  - Each `nextest:` line that has a tier gets one of these suffixes:
    - `tests: 1565 (-7 vs 2026-10-08T19:46:00, tree a1b2c3d4e5)` for a count change, with a `+` sign for an increase
    - `tests: 1565 (same as <timestamp>, tree <tree>)`
    - `tests: 1565 (no earlier record)`
  - All of this work is best-effort. An exception prints the plain `nextest:` line and never changes the exit code.

**Docs**

- `AGENTS.md` Edit 1. Replace the `#### Final Validation` heading, code block and paragraph with:

  ````markdown
  #### Final Validation (once, at the end)

  ```bash
  python build.py              # Full gate: fmt + clippy + guardrails + every test, browser tier included
  python build.py --no-browser # The same gate minus the browser tier
  ```

  Green means the full gate ran on the tree you report. While you iterate, run one step
  (`clippy`, `test-pattern <pattern>`) or `python build.py --no-browser`.
  ````

- `AGENTS.md` Edit 2. The comment on the `integration` line becomes `# Every test binary except browser, architecture and guardrails (~20s, unit tests included)`.
- `tests/AGENTS.md:21`. The text "`python build.py` runs the fast suite only." becomes "`python build.py` runs the non-LLM suite only."
- `ENVIRONMENT.md`, table "Typical gate cost". Add one row for `--no-browser`. It shows the measured total, the time for each tier and "skipped" in the browser column, with the measurement date and the target-dir state.

**Tracker (`.scratch/retro-issue-bucket/`)**

- New ticket `issues/30-gate-no-browser-and-verdict-record.md`.
  - Header lines: `Type: task`, `Status: claimed`, `Blocked by: —`.
  - Body: the scope above, the evidence (the journal shows 114 gate runs, 63% of build wall time; session `01a11db3`), and the measured results.
- `issues/03`. `## Answer`: the `--no-browser` tier plus the ladder wording, with the journal evidence.
  - It states that the "premise partly stale" basis in `triage.md:29` does not hold: the "once" sentence existed and did not bind.
  - Set `Status: resolved`.
- `issues/22`. `## Answer`: "A machine stamp", which is the journal tree and test columns plus the epilogue count change.
  - Known limitation: in a shared checkout, the baseline can be another agent's run on another tree. The count change shows that the trees differ, but not which tests changed.
  - The commit-report citation stays out of scope.
  - Set `Status: resolved`.
- `map.md`. Replace `_Nothing yet._` under Decisions-so-far with gist lines for 03, 22 and 30. Remove 03 and 22 from "Open tickets".

## Implementation

### Phase 0: Claim

- [ ] #### Task 0.1: Create ticket 30 with `Status: claimed` (1 SP)

### Phase 1: The `--no-browser` flag

- [ ] #### Task 1.1: Flag, rejections and plan change (3 SP)
  - Edit these parts of `build.py`:
    - `parse_args`, including the coverage check
    - `_GATE_ONLY_FLAGS`
    - the `TESTS` branch of `_plan_gate_steps`
    - the skip line in `run_gate`
    - the epilogue skip line in `main()`
  - In `scripts/tests/test_build_cli.py`, the `GatePlanTests.gate_args` helper gains the default `no_browser=False`.
- [ ] #### Task 1.2: Help text and docstring (1 SP)
  - Update the help text of the `integration` `StepSpec` and the invocation modes in the module docstring.

### Phase 2: The verdict record

- [ ] #### Task 2.1: Tier detection and count capture (3 SP)
  - Add `_tier_for_cmd`.
  - Add `_NextestSummary.tier` and `_NextestSummary.sizes`. `_timed_run` sets `tier`.
  - Add the size parse to `_stash_nextest_summary`.
- [ ] #### Task 2.2: Tree identity (3 SP)
  - Add `_tree_identity(repo)`. Call it once in `main()` after `os.chdir`.
- [ ] #### Task 2.3: Journal columns, baseline and epilogue suffix (5 SP)
  - [ ] ##### SubTask 2.3.1: Add the `tree` and `tests` columns and the old-header migration to `_append_history` (3 SP)
  - [ ] ##### SubTask 2.3.2: Add `_last_tier_sizes`, render the suffix on `nextest:` lines, and update the `MainRunTests` mock (3 SP)

### Phase 3: Docs

- [ ] #### Task 3.1: `AGENTS.md` Edits 1 and 2, as written above (1 SP)
- [ ] #### Task 3.2: `tests/AGENTS.md:21` wording (1 SP)

### Phase 4: Validation, docs from measurement, review, tracker

- [ ] #### Task 4.1: Measure and gate (3 SP)
  - First run `python build.py --no-browser --no-fmt`, then the final run `python build.py --no-fmt`.
  - Neither run formats, because the checkout holds other agents' uncommitted Rust edits and this change touches no Rust.
  - Both runs use the default warm target dir.
- [ ] #### Task 4.2: Add the `ENVIRONMENT.md` row from the Task 4.1 `--no-browser` run (1 SP)
- [ ] #### Task 4.3: Run `/code-review` on this change (3 SP)
  - Make the bundle with `python scripts/prepare_review_bundle.py --uncommitted`.
  - Tell the reviewers to judge only `build.py`, `scripts/tests/test_build_cli.py`, `AGENTS.md`, `tests/AGENTS.md`, `ENVIRONMENT.md` and the tracker files.
  - Fix each confirmed finding, then run `python build.py py-tests` again.
- [ ] #### Task 4.4: Tracker edits (3 SP)
  - Write the Answers for 03 and 22.
  - Set 03, 22 and 30 to `resolved`.
  - Edit `map.md`.

## Test Plan

Write the Python unit tests in `scripts/tests/test_build_cli.py`.

- **Parsing**
  - `--no-browser` parses, and `command is None`.
  - `["clippy", "--no-browser"]` and `["--no-browser", "clippy"]` both raise `SystemExit`.
  - `["--coverage", "--no-browser"]` raises `SystemExit`.
- **Plan**
  - With `no_browser=True`, the plan has no step labelled `Running browser tests...` and still has `Running integration tests...`.
  - The default plan is the same as today's plan, and the existing order tests stay green.
- **Tier detection**
  - Each of the four commands maps to its tier.
  - These map to `None`: `get_coverage_cmd(...)`, `get_integration_test_cmd(include_llm=True)` and `_NEXTEST_RUN`.
- **Size parse**
  - `"Summary [ 20.5s] 1570 tests run: 1570 passed, 2 skipped"` gives 1572.
  - A stash with `tier=None` records no size.
  - `_NextestSummary.lines` keeps its 2-tuple shape.
- **Journal**
  - A write to a temp file gives 6 columns.
  - An old 4-column header is replaced, not duplicated.
  - A trim keeps the header.
  - `_last_tier_sizes` returns the newest record for each tier. It skips 4-column lines and `-` values, and it returns nothing for a tier that is not in the journal.
- **Suffix rendering**
  - Test each form: a negative change, a positive change, the same count, and no record.
- **Tree identity**
  - Use a temp git repo made with `git init`, one commit, and `-c user.email`/`user.name`.
  - A clean tree gives the short HEAD with no `+`.
  - A changed tracked file gives the `+` form.
  - A new untracked file changes the digest.
  - A directory outside a repo gives `unknown`.

## Per Task/Sub Task Validation Steps

- **0.1:** The ticket 30 file exists with `Status: claimed`.
- **1.1, 1.2, 2.1–2.3:**
  - `python build.py py-tests` passes.
  - `python build.py --help` lists `--no-browser`.
  - `python build.py clippy --no-browser` and `python build.py --coverage --no-browser` both exit 2.
- **3.1, 3.2:** Read back the edited regions. `git diff AGENTS.md` shows only the two regions, and the generated blocks have no changes.
- **4.1, run 1 (`--no-browser`):**
  - The log has no `Running browser tests` step.
  - The tail has `skipped: browser tier ...`.
  - The `nextest:` lines show `tests: N (no earlier record)`.
  - The newest journal line has 6 columns, a `+`-form tree and three tiers.
- **4.1, run 2 (full gate):**
  - All tiers are green.
  - `integration`, `architecture` and `guardrails` show `same as <run-1 timestamp>`.
  - `browser` shows `no earlier record`.
  - Read the result with `tail -n 10 "$(ls -t logs/build_*.log | head -1)"`.
- **4.2:** The row values match the run-1 log's step timing summary.
- **4.3:** Ticket 30 records each confirmed review finding as fixed or answered.
- **4.4:** Decisions-so-far in `map.md` has the three lines, and Open tickets does not list 03 or 22.

## Assumptions

- **Shared checkout.** Other agents' uncommitted work stays untouched and unstaged. Nothing is committed without explicit approval.
- **Journal readers.** Only `build.py` reads `logs/build_history.txt` (checked). Older records do not have the new columns, so the first new run of each tier shows "no earlier record".
- **Count change scope.**
  - The count change compares the size of each tier's test set. It does not name the added or removed tests.
  - The baseline can come from another agent's run on another tree, and the suffix shows that tree.
- **Tree timing.** The tree identity is taken when the run starts. An edit made during the run is not covered.
- **Journal locality.** Each worktree has its own `logs/` and journal, so each checkout has its own baselines.
- **Untracked files.** `tmp`, `logs/` and `target/` are git-ignored, so the untracked-file hash is small. Today it covers 0 files.
- **Not changed.**
  - `README.md` keeps its stale "(~1 min)".
  - The after-plan skills keep their "Run the full build" lines, which already state the once-at-the-end rule.
  - Keeping these lines avoids overcorrection.
