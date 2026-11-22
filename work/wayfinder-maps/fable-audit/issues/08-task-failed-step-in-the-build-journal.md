# The build journal records which step failed

Type: task
Status: open
Blocked by: —

## Question

Add one column to the build journal: the name of the first step that failed (owner, 2026-11-22, D27 of [the decisions](../decisions.md)). It is row 1 of the optional rows in [the environment report](../assets/development-environment.md), and part of row N16 of [the list](../assets/findings-not-carried.md).

- **The fault (measured).** `logs/build_history.txt` has one row for each run of `build.py`: time, duration, exit code, arguments, tree, tests. In the 999 runs from 2026-10-28 to 2026-11-09, 15 of 64 full gates failed. A good full gate took about six minutes. The journal cannot say whether a failed gate stopped at clippy (18 s when run alone) or at the browser tests (234 s). So no number says whether an agent should run a cheaper command first.
- **The change.** `build_history.append` writes one more column at the end of the row, after `tests`. Its value is the label of the first `StepResult` that failed, and `-` for a run with no failed step.
  - A step with `report_only` is not a failed step (`StepResult.failed`, `build.py:817`).
  - A run that ends before any step has a result, such as a run that waits for the build slot and is stopped, has an exit code and no step. Write `-`.
- **Old rows stay as they are.** `last_tier_sizes` (`scripts/build_history.py:92`) reads the columns by position and accepts a row with six columns. It must go on reading old rows and new rows. Update `HEADER` and the module docstring.
- **The failing test first.** In `scripts/tests/test_build_history.py`: an append with a failed step writes its label in the new column, an append with none writes `-`, and `last_tier_sizes` reads a file that holds an old row and a new row.

## Rules

- Line numbers are from commit `dad238e1`. Find the site again by its name before you edit.
- Run the Python tests with `python build.py py-tests`, and `python build.py --no-browser` once at the end.
- Do not commit.

## Not in scope

- A report that reads the column. First collect rows. The weekly retro can then count the failed steps.
- The build-cost table of `AGENTS.md`. Section 1 of [ticket 05](05-task-text.md) holds it.
- The other rows of N16: no change (D28, D29).
