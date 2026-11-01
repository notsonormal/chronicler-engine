# Measure what docs-only gate runs cost

Type: research (AFK)
Status: open
Blocked by: —

## Question

How often does the full gate run for a change that touches only docs or scripts, and what does it cost? Which gate steps can such a change skip safely?

## Context

- **Covers** C-21 (checks scoped to the change). See `assets/candidates.md`, theme D, and check K-39.
- **Facts (known).** The full gate runs the fixed tuple `_GATE_CHECKS` of 11 checks (`build.py:1191-1203`), about 145 s warm (`AGENTS.md`). `build.py` has no logic that reads the changed files. Hex filters its workflows by path (`assets/research-06-branch-to-installed-app.md`, row 3).
- **Do.**
  1. From `logs/build_history.txt`, count the full-gate runs, both `python build.py` and `--no-browser`. The `tree` column holds the short HEAD, plus a content digest when the tree is dirty (`tree_identity` in `scripts/build_history.py`). The digest does not name the changed files, so find the kind of change in another way. Option (a): from the session logs, take the files that a session edited before each `python build.py` call. Option (b): for each commit, take the gate runs whose HEAD is its parent, and use the file list of the commit. State which method you used and what it misses.
  2. Add up the time those runs took.
  3. For each gate step, list the kinds of file that can break it. Check whether any Rust code or test reads files in `docs/`, for example the spec validator, the guardrails or `include_str!`.
  4. Propose a skip list for each kind of change, and give the risk of each skip.
- **Not in scope.** Do not edit `build.py`.
- **What already exists.** `scripts/build_history.py` names the columns: `timestamp | duration_s | exit_code | args | tree | tests`. Each run also writes a `logs/build_*.log` with the result of each step.
- **Failure modes.** Each checkout keeps its own `logs/`, so this file does not hold the runs made in worktrees. State that undercount. A run that failed early took less time than a full run, so count failed runs apart.
- **Output.** Write `assets/research-14-gate-scope.md`. The `## Answer` gives the gist, the recommendation and at most 2 questions for ticket 16.
- **When done.** Add `## Answer` and set `Status: resolved`.
